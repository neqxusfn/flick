use crate::{
    calibration, flick,
    hook::{self, Event},
    math::{self, Remainder},
    platform::{self, Environment},
    profiles::{self, Direction, Mode, Settings},
};
use crossbeam_channel::{bounded, select, tick, Sender};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use tauri_plugin_autostart::ManagerExt;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub settings: Settings,
    pub enabled: bool,
    pub recording: bool,
    pub calibration: String,
    pub error: Option<String>,
    pub environment: Environment,
    pub healthy: bool,
}
pub struct Engine {
    pub settings: Mutex<Settings>,
    pub input: Arc<hook::Shared>,
    pub events: Sender<Event>,
    pub error: Mutex<Option<String>>,
    generation: AtomicU64,
    environment: Mutex<Environment>,
    capture_deadline: Mutex<Option<Instant>>,
    calibration_deadline: Mutex<Option<Instant>>,
    app: AppHandle,
}
#[derive(Clone, Copy)]
struct Job {
    counts: i32,
    duration: u32,
    generation: u64,
}
impl Engine {
    pub fn start(app: AppHandle, settings: Settings, error: Option<String>) -> Arc<Self> {
        let (events, receiver) = bounded(256);
        let input = hook::Shared::new(events.clone());
        input
            .trigger
            .store(settings.active().trigger, Ordering::Release);
        let engine = Arc::new(Self {
            settings: Mutex::new(settings),
            input: input.clone(),
            events,
            error: Mutex::new(error),
            generation: AtomicU64::new(0),
            environment: Mutex::new(platform::environment()),
            capture_deadline: Mutex::new(None),
            calibration_deadline: Mutex::new(None),
            app,
        });
        if let Err(error) = hook::start(input) {
            engine.fail(error);
        }
        let (jobs, work) = bounded::<Job>(2);
        let (finished, completed) = bounded(2);
        let runner = engine.clone();
        thread::Builder::new()
            .name("flick-motion".into())
            .spawn(move || {
                while let Ok(job) = work.recv() {
                    let result = flick::execute(
                        job.counts,
                        job.duration,
                        &runner.input.enabled,
                        &runner.generation,
                        job.generation,
                    );
                    if finished.send(result).is_err() {
                        break;
                    }
                }
            })
            .expect("Could not start the motion worker");
        let coordinator = engine.clone();
        thread::Builder::new().name("flick-coordinator".into()).spawn(move || {
            let heartbeat = tick(Duration::from_millis(100));
            let mut remainder = Remainder::default();
            let mut last_profile = coordinator.settings.lock().unwrap().active().clone();
            let mut outstanding: usize = 0;
            let mut back: Option<(u32, Job)> = None;
            let mut last_foreground = Instant::now();
            loop {
                select! {
                    recv(receiver) -> event => {
                        let Ok(event) = event else { break; };
                        match event {
                            Event::Toggle => { coordinator.cancel_capture(); coordinator.cancel_calibration(); coordinator.set_enabled(!coordinator.input.enabled.load(Ordering::Acquire)); back = None; }
                            Event::Capture(key) => {
                                coordinator.cancel_capture();
                                if key != 0x1b {
                                    let mut settings = coordinator.settings.lock().unwrap().clone();
                                    let id = settings.active_id.clone();
                                    if let Some(p) = settings.profiles.iter_mut().find(|p| p.id == id) { p.trigger = key; p.trigger_label = hook::key_name(key); }
                                    if let Err(e) = coordinator.save(settings) { coordinator.fail(e); }
                                }
                                coordinator.publish();
                            }
                            Event::Calibrate => { coordinator.calibration_key(); back = None; }
                            Event::Down(_) | Event::Test => {
                                if !coordinator.input.enabled.load(Ordering::Acquire) || coordinator.input.recording.load(Ordering::Acquire) || coordinator.input.calibration_armed.load(Ordering::Acquire) { continue; }
                                if outstanding != 0 { if matches!(event, Event::Test) { coordinator.fail("A turn is already in progress.".into()); } continue; }
                                let profile = coordinator.settings.lock().unwrap().active().clone();
                                if matches!(event, Event::Down(key) if key != profile.trigger) { continue; }
                                if last_profile != profile { remainder = Remainder::default(); last_profile = profile.clone(); }
                                if let Ok(exact) = math::exact_counts(&profile) {
                                    let count = remainder.next(exact) * if profile.direction == Direction::Left { -1 } else { 1 };
                                    let job = Job { counts: count, duration: profile.duration, generation: coordinator.generation.load(Ordering::Acquire) };
                                    if jobs.try_send(job).is_ok() {
                                        outstanding += 1;
                                        if matches!(event, Event::Down(_)) && profile.flick_back { back = Some((profile.trigger, Job { counts: -count, ..job })); }
                                    }
                                }
                            }
                            Event::Up(key) => {
                                if let Some((trigger, job)) = back {
                                    if key == trigger {
                                        back = None;
                                        if coordinator.input.enabled.load(Ordering::Acquire) && job.generation == coordinator.generation.load(Ordering::Acquire) && jobs.try_send(job).is_ok() { outstanding += 1; }
                                    }
                                }
                            }
                        }
                    }
                    recv(completed) -> result => {
                        outstanding = outstanding.saturating_sub(1);
                        if let Ok(Err(error)) = result { coordinator.set_enabled(false); coordinator.fail(error); back = None; }
                    }
                    recv(heartbeat) -> _ => {
                        let capture_expired = coordinator.capture_deadline.lock().unwrap().is_some_and(|t| Instant::now() >= t);
                        let calibration_expired = coordinator.calibration_deadline.lock().unwrap().is_some_and(|t| Instant::now() >= t);
                        if capture_expired { coordinator.cancel_capture(); coordinator.publish(); }
                        if calibration_expired { coordinator.cancel_calibration(); coordinator.fail("Calibration timed out. Try again when you are in game.".into()); }
                        if last_foreground.elapsed() >= Duration::from_millis(500) {
                            last_foreground = Instant::now(); coordinator.check_foreground();
                        }
                    }
                }
            }
        }).expect("Could not start the input coordinator");
        engine
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            settings: self.settings.lock().unwrap().clone(),
            enabled: self.input.enabled.load(Ordering::Acquire),
            recording: self.input.recording.load(Ordering::Acquire),
            healthy: self.input.healthy.load(Ordering::Acquire),
            calibration: if calibration::measuring() {
                "measuring"
            } else if self.input.calibration_armed.load(Ordering::Acquire) {
                "armed"
            } else {
                "idle"
            }
            .into(),
            error: self.error.lock().unwrap().clone(),
            environment: self.environment.lock().unwrap().clone(),
        }
    }
    pub fn publish(&self) {
        let snapshot = self.snapshot();
        let _ = self.app.emit("flick-state", &snapshot);
        crate::tray::refresh(&self.app, &snapshot);
    }
    pub fn fail(&self, error: String) {
        *self.error.lock().unwrap() = Some(error);
        self.publish();
    }
    pub fn set_enabled(&self, enabled: bool) {
        if !enabled {
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        if enabled && !self.settings.lock().unwrap().risk_acknowledged {
            self.fail(
                "Read and acknowledge the account-risk warning before enabling Flick.".into(),
            );
            return;
        }
        if enabled && !self.input.healthy.load(Ordering::Acquire) {
            self.fail("The input listener is unavailable. Restart Flick.".into());
            return;
        }
        if enabled && self.input.calibration_armed.load(Ordering::Acquire) {
            self.cancel_calibration();
        }
        self.input.enabled.store(enabled, Ordering::Release);
        self.publish();
    }
    pub fn save(&self, settings: Settings) -> Result<(), String> {
        settings.validate()?;
        let mut current = self.settings.lock().unwrap();
        if current.risk_acknowledged && !settings.risk_acknowledged {
            return Err(
                "The account-risk acknowledgment cannot be removed while Flick is running.".into(),
            );
        }
        let autostart = self.app.autolaunch();
        let changed_start = settings.start_with_windows != current.start_with_windows;
        if changed_start {
            if settings.start_with_windows {
                autostart.enable()
            } else {
                autostart.disable()
            }
            .map_err(|_| "Windows could not change your startup preference.".to_string())?;
        }
        if let Err(error) = profiles::save(&settings) {
            if changed_start {
                let _ = if current.start_with_windows {
                    autostart.enable()
                } else {
                    autostart.disable()
                };
            }
            return Err(error);
        }
        self.input
            .trigger
            .store(settings.active().trigger, Ordering::Release);
        *current = settings;
        drop(current);
        self.publish();
        Ok(())
    }
    pub fn record(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.cancel_calibration();
        *self.capture_deadline.lock().unwrap() = Some(Instant::now() + Duration::from_secs(15));
        self.input.recording.store(true, Ordering::Release);
        self.publish();
    }
    pub fn cancel_capture(&self) {
        self.input.recording.store(false, Ordering::Release);
        *self.capture_deadline.lock().unwrap() = None;
    }
    pub fn arm_calibration(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.cancel_capture();
        self.input.enabled.store(false, Ordering::Release);
        self.input.calibration_armed.store(true, Ordering::Release);
        *self.calibration_deadline.lock().unwrap() =
            Some(Instant::now() + Duration::from_secs(120));
        self.publish();
    }
    pub fn cancel_calibration(&self) {
        calibration::stop();
        self.input.calibration_armed.store(false, Ordering::Release);
        *self.calibration_deadline.lock().unwrap() = None;
    }
    fn calibration_key(&self) {
        if !self.input.calibration_armed.load(Ordering::Acquire) {
            return;
        }
        if !calibration::measuring() {
            calibration::start();
            self.publish();
            return;
        }
        let counts = calibration::stop();
        self.cancel_calibration();
        if !(1..=1_000_000).contains(&counts) {
            self.fail("No usable turn was measured. Move your mouse and try again.".into());
            return;
        }
        let mut settings = self.settings.lock().unwrap().clone();
        let id = settings.active_id.clone();
        if let Some(p) = settings.profiles.iter_mut().find(|p| p.id == id) {
            p.mode = Mode::Raw;
            p.raw_counts = counts as f64;
        }
        if let Err(error) = self.save(settings) {
            self.fail(error);
        }
        self.publish();
    }
    fn check_foreground(&self) {
        let (process, elevated) = platform::foreground();
        let mut environment = self.environment.lock().unwrap();
        let changed = environment.foreground_elevated != elevated;
        environment.foreground_elevated = elevated;
        drop(environment);
        let mut settings = self.settings.lock().unwrap();
        let next = if settings.auto_switch
            && !self.input.calibration_armed.load(Ordering::Acquire)
            && !self.input.recording.load(Ordering::Acquire)
        {
            settings
                .profiles
                .iter()
                .find(|p| !p.process.is_empty() && p.process.eq_ignore_ascii_case(&process))
                .map(|p| p.id.clone())
        } else {
            None
        };
        let switched = next.is_some_and(|id| {
            if id == settings.active_id {
                return false;
            }
            settings.active_id = id;
            self.input
                .trigger
                .store(settings.active().trigger, Ordering::Release);
            true
        });
        drop(settings);
        if changed || switched {
            self.publish();
        }
    }
}
