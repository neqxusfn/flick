use serde::Serialize;
use std::{
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
    System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::{
        Shell::ShellExecuteW,
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, SystemParametersInfoW, SPI_GETMOUSE,
            SW_SHOWNORMAL,
        },
    },
};

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub acceleration: bool,
    pub elevated: bool,
    pub foreground_elevated: bool,
}
fn elevated(process: HANDLE) -> bool {
    unsafe {
        let mut token = null_mut();
        if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation: TOKEN_ELEVATION = zeroed();
        let mut size = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}
pub fn foreground() -> (String, bool) {
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), &mut pid);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return (String::new(), false);
        }
        let is_elevated = elevated(process);
        let mut name = [0u16; 1024];
        let mut size = name.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, name.as_mut_ptr(), &mut size);
        CloseHandle(process);
        let path = if ok != 0 {
            String::from_utf16_lossy(&name[..size as usize])
        } else {
            String::new()
        };
        (
            path.rsplit('\\').next().unwrap_or("").to_lowercase(),
            is_elevated,
        )
    }
}
pub fn environment() -> Environment {
    let mut mouse = [0i32; 3];
    unsafe {
        SystemParametersInfoW(SPI_GETMOUSE, 0, mouse.as_mut_ptr().cast(), 0);
    }
    Environment {
        acceleration: mouse[2] != 0,
        elevated: elevated(unsafe { GetCurrentProcess() }),
        foreground_elevated: foreground().1,
    }
}
pub fn relaunch_elevated() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    let exe = std::env::current_exe().map_err(|_| "Flick could not find its app file.")?;
    let path: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    // The new elevated instance waits for this PID to exit before taking the lock.
    let args: Vec<u16> = format!("--wait-for={}\0", std::process::id())
        .encode_utf16()
        .collect();
    let result = unsafe {
        ShellExecuteW(
            null_mut(),
            verb.as_ptr(),
            path.as_ptr(),
            args.as_ptr(),
            null(),
            SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        Err("Administrator access was cancelled or unavailable.".into())
    } else {
        Ok(())
    }
}
pub fn wait_for_previous() {
    use windows_sys::Win32::System::Threading::{WaitForSingleObject, PROCESS_SYNCHRONIZE};
    if let Some(pid) = std::env::args().find_map(|s| {
        s.strip_prefix("--wait-for=")
            .and_then(|p| p.parse::<u32>().ok())
    }) {
        unsafe {
            let process = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
            if !process.is_null() {
                WaitForSingleObject(process, 10000);
                CloseHandle(process);
            }
        }
    }
}
