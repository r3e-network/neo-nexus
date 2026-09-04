//! Targeted console shutdown. Console attachment is process-wide, so changing
//! it inside the web server would affect unrelated threads and console handlers.
//! A short-lived copy of the CLI attaches when the node uses another console.

use std::{
    io,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const HELPER_ARGUMENT: &str = "--internal-console-break";
const CTRL_BREAK_EVENT: u32 = 1;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[link(name = "kernel32")]
extern "system" {
    fn GenerateConsoleCtrlEvent(event: u32, group: u32) -> i32;
    fn GetConsoleProcessList(processes: *mut u32, count: u32) -> u32;
    fn FreeConsole() -> i32;
    fn AttachConsole(pid: u32) -> i32;
    fn GetStdHandle(kind: u32) -> *mut std::ffi::c_void;
    fn GetFileType(handle: *mut std::ffi::c_void) -> u32;
    fn SetHandleInformation(handle: *mut std::ffi::c_void, mask: u32, flags: u32) -> i32;
}

pub(super) fn prevent_standard_pipe_inheritance() -> io::Result<()> {
    // std::process redirects the node to its log, but Windows can still inherit
    // the CLI's original capture-pipe handles as *additional* handles. That
    // keeps `neo-nexus --node-start` output open until the node exits. Explicit
    // Stdio::inherit continues to work because Command duplicates that handle.
    for kind in [-10_i32, -11, -12] {
        let handle = unsafe { GetStdHandle(kind as u32) };
        if unsafe { GetFileType(handle) } == 3 && unsafe { SetHandleInformation(handle, 1, 0) } == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

fn shares_console(pid: u32) -> bool {
    let mut processes = vec![0; 64];
    let mut count =
        unsafe { GetConsoleProcessList(processes.as_mut_ptr(), processes.len() as u32) };
    if count as usize > processes.len() {
        processes.resize(count as usize, 0);
        count = unsafe { GetConsoleProcessList(processes.as_mut_ptr(), processes.len() as u32) };
    }
    count > 0 && processes[..(count as usize).min(processes.len())].contains(&pid)
}

fn send_break(pid: u32) -> io::Result<()> {
    // Zero would broadcast to every process sharing the console. Never permit
    // it, even through a manually invoked internal helper.
    if pid == 0 || pid == std::process::id() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid target process group",
        ));
    }
    if unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(super) fn request_break(pid: u32) -> io::Result<()> {
    if shares_console(pid) {
        return send_break(pid);
    }
    // No shell and no user-supplied command text. CREATE_NO_WINDOW hides the
    // helper; it attaches to the existing target console without creating one.
    let mut helper = Command::new(std::env::current_exe()?)
        .args([HELPER_ARGUMENT, &pid.to_string()])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = helper.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(io::Error::other("target console did not accept CTRL_BREAK"))
            };
        }
        if Instant::now() >= deadline {
            let _ = helper.kill();
            let _ = helper.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "console helper timed out",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

/// Reserved CLI dispatch, called before the normal manager opens a workspace.
/// Returns `None` for every ordinary invocation.
#[doc(hidden)]
pub fn console_break_helper_from_args() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    if args.next()?.to_str()? != HELPER_ARGUMENT {
        return None;
    }
    let pid = args
        .next()
        .and_then(|arg| arg.to_str().and_then(|text| text.parse::<u32>().ok()));
    let Some(pid) = pid.filter(|pid| *pid != 0 && *pid != std::process::id()) else {
        return Some(2);
    };
    if args.next().is_some() {
        return Some(2);
    }
    // Only this disposable helper detaches. The parent keeps its console and
    // handlers, including when multiple nodes stop concurrently.
    unsafe {
        FreeConsole();
    }
    if unsafe { AttachConsole(pid) } == 0 {
        return Some(1);
    }
    let result = if shares_console(pid) {
        send_break(pid)
    } else {
        Err(io::Error::other("target left its console"))
    };
    unsafe {
        FreeConsole();
    }
    Some(if result.is_ok() { 0 } else { 1 })
}
