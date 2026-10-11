use std::{
    mem::{size_of, zeroed},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM},
    UI::Input::*,
};
static MEASURING: AtomicBool = AtomicBool::new(false);
static COUNTS: Mutex<i64> = Mutex::new(0);
pub fn start() {
    let mut counts = COUNTS.lock().unwrap();
    *counts = 0;
    MEASURING.store(true, Ordering::Release);
}
pub fn stop() -> i64 {
    let counts = COUNTS.lock().unwrap();
    MEASURING.store(false, Ordering::Release);
    counts.abs()
}
pub fn measuring() -> bool {
    MEASURING.load(Ordering::Acquire)
}
pub unsafe fn register(hwnd: HWND) -> bool {
    let device = RAWINPUTDEVICE {
        usUsagePage: 1,
        usUsage: 2,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    };
    RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32) != 0
}
pub unsafe fn on_input(l: LPARAM) {
    if !measuring() {
        return;
    }
    let mut raw: RAWINPUT = zeroed();
    let mut size = size_of::<RAWINPUT>() as u32;
    let read = GetRawInputData(
        l as HRAWINPUT,
        RID_INPUT,
        (&mut raw as *mut RAWINPUT).cast(),
        &mut size,
        size_of::<RAWINPUTHEADER>() as u32,
    );
    if read == u32::MAX || raw.header.dwType != RIM_TYPEMOUSE || raw.header.hDevice.is_null() {
        return;
    }
    let mouse = raw.data.mouse;
    // Absolute/tablet packets are deliberately excluded. Counts are signed net X,
    // so small corrections while manually aiming do not inflate the measurement.
    if mouse.usFlags & 1 == 0 {
        let mut counts = COUNTS.lock().unwrap();
        // Serialize the final packet against stop; never change a completed capture.
        if measuring() {
            *counts += i64::from(mouse.lLastX);
        }
    }
}
