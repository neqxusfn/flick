use crate::math;
use std::{
    mem::size_of,
    ptr::null,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0},
    System::Threading::{
        CreateWaitableTimerExW, SetWaitableTimer, WaitForSingleObject, TIMER_ALL_ACCESS,
    },
    UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_MOVE, MOUSEEVENTF_MOVE_NOCOALESCE,
        MOUSEINPUT,
    },
};

struct Timer(HANDLE);
impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn execute(
    counts: i32,
    duration: u32,
    enabled: &AtomicBool,
    generation: &AtomicU64,
    expected_generation: u64,
) -> Result<(), String> {
    let n = if duration == 0 { 1 } else { duration + 1 };
    let parts = math::steps(counts, n);
    // CREATE_WAITABLE_TIMER_HIGH_RESOLUTION; fall back for older Windows builds.
    let handle = unsafe { CreateWaitableTimerExW(null(), null(), 2, TIMER_ALL_ACCESS) };
    let handle = if handle.is_null() {
        unsafe { CreateWaitableTimerExW(null(), null(), 0, TIMER_ALL_ACCESS) }
    } else {
        handle
    };
    if handle.is_null() {
        return Err("Windows could not start the precision timer.".into());
    }
    let timer = Timer(handle);
    let start = Instant::now();
    for (index, dx) in parts.into_iter().enumerate() {
        if !enabled.load(Ordering::Acquire)
            || generation.load(Ordering::Acquire) != expected_generation
        {
            break;
        }
        let target = start
            + Duration::from_micros(
                u64::from(duration) * 1000 * index as u64 / u64::from((n - 1).max(1)),
            );
        if let Some(remaining) = target.checked_duration_since(Instant::now()) {
            let due = -(remaining.as_nanos().div_ceil(100) as i64).max(1);
            if unsafe { SetWaitableTimer(timer.0, &due, 0, None, null(), 0) } == 0
                || unsafe { WaitForSingleObject(timer.0, 1000) } != WAIT_OBJECT_0
            {
                return Err("The precision timer was interrupted. Try again.".into());
            }
        }
        if !enabled.load(Ordering::Acquire)
            || generation.load(Ordering::Acquire) != expected_generation
        {
            break;
        }
        if dx == 0 {
            continue;
        }
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_MOVE_NOCOALESCE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        if unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) } != 1 {
            return Err(
                "Windows blocked the turn. The game may need Flick to run as administrator.".into(),
            );
        }
    }
    Ok(())
}
