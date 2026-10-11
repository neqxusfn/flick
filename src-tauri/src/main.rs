#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod calibration;
mod engine;
mod flick;
mod hook;
mod math;
mod platform;
mod profiles;
mod tray;
use engine::{Engine, Snapshot};
use profiles::Settings;
use std::sync::Arc;
use tauri::{Manager, State};

#[tauri::command]
fn get_state(engine: State<Arc<Engine>>) -> Snapshot {
    engine.snapshot()
}
#[tauri::command]
fn save_settings(engine: State<Arc<Engine>>, settings: Settings) -> Result<(), String> {
    engine.save(settings)
}
#[tauri::command]
fn set_enabled(engine: State<Arc<Engine>>, enabled: bool) {
    engine.set_enabled(enabled);
}
#[tauri::command]
fn record_trigger(engine: State<Arc<Engine>>) {
    engine.record();
}
#[tauri::command]
fn cancel_capture(engine: State<Arc<Engine>>) {
    engine.cancel_capture();
    engine.publish();
}
#[tauri::command]
fn calibrate(engine: State<Arc<Engine>>, cancel: bool) {
    if cancel {
        engine.cancel_calibration();
    } else {
        engine.arm_calibration();
    }
    engine.publish();
}
#[tauri::command]
fn test_flick(engine: State<Arc<Engine>>) -> Result<(), String> {
    if !engine.snapshot().enabled {
        return Err("Enable Flick before testing a turn.".into());
    }
    engine
        .events
        .try_send(hook::Event::Test)
        .map_err(|_| "Flick is busy. Try again.".into())
}
#[tauri::command]
fn clear_error(engine: State<Arc<Engine>>) {
    *engine.error.lock().unwrap() = None;
    engine.publish();
}
#[tauri::command]
fn run_as_admin(app: tauri::AppHandle) -> Result<(), String> {
    platform::relaunch_elevated()?;
    app.exit(0);
    Ok(())
}

fn main() {
    platform::wait_for_previous();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            tray::show(app)
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--minimized"])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_settings,
            set_enabled,
            record_trigger,
            cancel_capture,
            calibrate,
            test_flick,
            clear_error,
            run_as_admin
        ])
        .setup(|app| {
            let (settings, error) = match profiles::load() {
                Ok(s) => (s, None),
                Err(e) => (Settings::default(), Some(e)),
            };
            let minimized =
                settings.start_minimized || std::env::args().any(|a| a == "--minimized");
            let engine = Engine::start(app.handle().clone(), settings, error);
            app.manage(engine.clone());
            tray::setup(app.handle())?;
            engine.publish();
            if !minimized {
                tray::show(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("Flick could not start")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                if let Some(engine) = app.try_state::<Arc<Engine>>() {
                    engine.input.stop();
                }
            }
        });
}
