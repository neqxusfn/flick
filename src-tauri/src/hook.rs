//! Hooks run only on this process's message-loop thread. No allocation, locking,
//! IPC, disk access, or SendInput in either callback; only atomics and try_send.
use crossbeam_channel::Sender;
use std::{
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, OnceLock,
    },
    thread,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::WindowsAndMessaging::*,
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Down(u32),
    Up(u32),
    Capture(u32),
    Toggle,
    Calibrate,
    Test,
}
pub struct Shared {
    pub enabled: AtomicBool,
    pub trigger: AtomicU32,
    pub recording: AtomicBool,
    pub calibration_armed: AtomicBool,
    pub thread_id: AtomicU32,
    pub healthy: AtomicBool,
    sender: Sender<Event>,
    held: [AtomicBool; 256],
}
static SHARED: OnceLock<Arc<Shared>> = OnceLock::new();
impl Shared {
    pub fn new(sender: Sender<Event>) -> Arc<Self> {
        Arc::new(Self {
            enabled: AtomicBool::new(false),
            trigger: AtomicU32::new(5),
            recording: AtomicBool::new(false),
            calibration_armed: AtomicBool::new(false),
            thread_id: AtomicU32::new(0),
            healthy: AtomicBool::new(false),
            sender,
            held: std::array::from_fn(|_| AtomicBool::new(false)),
        })
    }
    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Release);
        let id = self.thread_id.load(Ordering::Acquire);
        if id != 0 {
            unsafe {
                PostThreadMessageW(id, WM_QUIT, 0, 0);
            }
        }
    }
}
fn route(key: u32, down: bool) -> bool {
    let Some(s) = SHARED.get() else {
        return false;
    };
    route_on(s, key, down)
}
fn route_on(s: &Shared, key: u32, down: bool) -> bool {
    let Some(held) = s.held.get(key as usize) else {
        return false;
    };
    if !down {
        if held.swap(false, Ordering::AcqRel) {
            let _ = s.sender.try_send(Event::Up(key));
            return true;
        }
        return false;
    }
    if held.load(Ordering::Acquire) {
        return true;
    } // suppress repeat, even after rebinding
    let event = if s.recording.load(Ordering::Acquire) {
        Event::Capture(key)
    } else if key == 0x77 {
        Event::Toggle
    } else if key == 0x78 && s.calibration_armed.load(Ordering::Acquire) {
        Event::Calibrate
    } else if s.enabled.load(Ordering::Acquire) && key == s.trigger.load(Ordering::Acquire) {
        Event::Down(key)
    } else {
        return false;
    };
    if s.sender.try_send(event).is_ok() {
        held.store(true, Ordering::Release);
        if matches!(event, Event::Capture(_)) {
            s.recording.store(false, Ordering::Release);
        }
        true
    } else {
        false
    } // fail open if the coordinator cannot keep up
}

unsafe extern "system" fn mouse(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = &*(l as *const MSLLHOOKSTRUCT);
        if info.flags & LLMHF_INJECTED == 0 {
            let event = match w as u32 {
                WM_XBUTTONDOWN | WM_XBUTTONUP => Some((
                    if info.mouseData >> 16 == 1 { 5 } else { 6 },
                    w as u32 == WM_XBUTTONDOWN,
                )),
                WM_MBUTTONDOWN | WM_MBUTTONUP => Some((4, w as u32 == WM_MBUTTONDOWN)),
                _ => None,
            };
            if let Some((key, down)) = event {
                if route(key, down) {
                    return 1;
                }
            }
        }
    }
    CallNextHookEx(null_mut(), code, w, l)
}
unsafe extern "system" fn keyboard(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = &*(l as *const KBDLLHOOKSTRUCT);
        if info.flags & LLKHF_INJECTED == 0
            && route(info.vkCode, matches!(w as u32, WM_KEYDOWN | WM_SYSKEYDOWN))
        {
            return 1;
        }
    }
    CallNextHookEx(null_mut(), code, w, l)
}
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_INPUT {
        crate::calibration::on_input(l);
    }
    DefWindowProcW(hwnd, msg, w, l) // WM_INPUT foreground cleanup is required
}
pub fn start(shared: Arc<Shared>) -> Result<(), String> {
    SHARED
        .set(shared.clone())
        .map_err(|_| "Input is already running.")?;
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    thread::Builder::new()
        .name("flick-hooks".into())
        .spawn(move || unsafe {
            let instance = GetModuleHandleW(null());
            let class: Vec<u16> = "FlickRawInput\0".encode_utf16().collect();
            let wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                ..zeroed()
            };
            RegisterClassW(&wc);
            let hwnd = CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                null_mut(),
                instance,
                null(),
            );
            let mh = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse), instance, 0);
            let kh = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), instance, 0);
            let raw_ok = !hwnd.is_null() && crate::calibration::register(hwnd);
            if mh.is_null() || kh.is_null() || !raw_ok {
                if !mh.is_null() {
                    UnhookWindowsHookEx(mh);
                }
                if !kh.is_null() {
                    UnhookWindowsHookEx(kh);
                }
                if !hwnd.is_null() {
                    DestroyWindow(hwnd);
                }
                let _ = ready_tx.send(Err(
                    "Windows could not listen for your trigger. Restart Flick.".to_string(),
                ));
                return;
            }
            shared
                .thread_id
                .store(GetCurrentThreadId(), Ordering::Release);
            shared.healthy.store(true, Ordering::Release);
            let _ = ready_tx.send(Ok(()));
            let mut msg: MSG = zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            shared.healthy.store(false, Ordering::Release);
            shared.enabled.store(false, Ordering::Release);
            UnhookWindowsHookEx(mh);
            UnhookWindowsHookEx(kh);
            DestroyWindow(hwnd);
        })
        .map_err(|_| "The input listener could not start.".to_string())?;
    ready_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| "The input listener did not respond.".to_string())?
}
pub fn key_name(key: u32) -> String {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetKeyNameTextW, MapVirtualKeyW, MAPVK_VK_TO_VSC,
    };
    match key {
        4 => return "Middle click".into(),
        5 => return "Mouse 4".into(),
        6 => return "Mouse 5".into(),
        _ => {}
    }
    let mut buffer = [0u16; 64];
    let scan = unsafe { MapVirtualKeyW(key, MAPVK_VK_TO_VSC) };
    let extended = if [
        0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2d, 0x2e, 0xa3, 0xa5, 0x5b, 0x5c, 0x6f,
    ]
    .contains(&key)
    {
        1 << 24
    } else {
        0
    };
    let len = unsafe {
        GetKeyNameTextW(
            ((scan << 16) | extended) as i32,
            buffer.as_mut_ptr(),
            size_of::<[u16; 64]>() as i32 / 2,
        )
    };
    if len > 0 {
        String::from_utf16_lossy(&buffer[..len as usize])
    } else {
        format!("Key {key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::bounded;
    #[test]
    fn suppresses_repeats_and_pairs_release_after_rebind() {
        let (tx, rx) = bounded(16);
        let s = Shared::new(tx);
        s.enabled.store(true, Ordering::Release);
        assert!(route_on(&s, 5, true));
        assert!(matches!(rx.recv().unwrap(), Event::Down(5)));
        assert!(route_on(&s, 5, true));
        assert!(rx.is_empty());
        s.trigger.store(6, Ordering::Release);
        s.enabled.store(false, Ordering::Release);
        assert!(route_on(&s, 5, false));
        assert!(matches!(rx.recv().unwrap(), Event::Up(5)));
        assert!(!route_on(&s, 5, true));
        assert!(!route_on(&s, 5, false));
    }
    #[test]
    fn capture_takes_one_key_and_full_queue_fails_open() {
        let (tx, rx) = bounded(1);
        let s = Shared::new(tx);
        s.recording.store(true, Ordering::Release);
        assert!(route_on(&s, 65, true));
        assert!(!s.recording.load(Ordering::Acquire));
        assert!(!route_on(&s, 0x77, true));
        assert!(matches!(rx.recv().unwrap(), Event::Capture(65)));
        assert!(route_on(&s, 65, false));
    }
}
