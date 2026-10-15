use std::{
    io,
    os::windows::{io::AsRawHandle, process::CommandExt},
    process::{Child, Command},
    sync::atomic::Ordering,
    thread,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_BROKEN_PIPE, HANDLE},
    System::{
        Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler},
        JobObjects::*,
        Pipes::PeekNamedPipe,
        Threading::CREATE_SUSPENDED,
    },
};

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtResumeProcess(process: HANDLE) -> i32;
}

pub trait Pipe: AsRawHandle {}
impl<T: AsRawHandle> Pipe for T {}
pub fn configure(_: &impl Pipe) -> io::Result<()> {
    Ok(())
}
pub fn available(pipe: &impl Pipe) -> io::Result<usize> {
    let mut bytes = 0;
    let result = unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut bytes,
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        let error = io::Error::last_os_error();
        // Read of a broken pipe returns EOF; request one byte to observe it.
        if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
            return Ok(1);
        }
        return Err(error);
    }
    Ok(bytes as usize)
}

pub struct Scope {
    job: HANDLE,
}
impl Scope {
    pub fn prepare(command: &mut Command) -> io::Result<Self> {
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let scope = Self { job };
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        command.creation_flags(CREATE_SUSPENDED);
        Ok(scope)
    }
    pub fn attach(&mut self, child: &Child) -> io::Result<()> {
        if unsafe { AssignProcessToJobObject(self.job, child.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        // std closes the initial thread handle; resume only this suspended process
        // through its retained handle, never by enumerating or reopening a PID.
        if unsafe { NtResumeProcess(child.as_raw_handle()) } < 0 {
            return Err(io::Error::other("resume owned process failed"));
        }
        Ok(())
    }
    pub fn exited(&self, child: &mut Child) -> io::Result<bool> {
        child.try_wait().map(|s| s.is_some())
    }
    pub fn terminate(&mut self) -> io::Result<()> {
        if unsafe { TerminateJobObject(self.job, 125) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    pub fn request_stop(&mut self, _: bool) -> io::Result<()> {
        self.terminate()
    }
    pub fn finish(&self, deadline: Instant) -> io::Result<()> {
        loop {
            let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
            if unsafe {
                QueryInformationJobObject(
                    self.job,
                    JobObjectBasicAccountingInformation,
                    (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    std::mem::size_of_val(&info) as u32,
                    std::ptr::null_mut(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            if info.ActiveProcesses == 0 {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(super::POLL);
        }
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.job);
        }
    }
}

unsafe extern "system" fn cancel(kind: u32) -> i32 {
    if kind == CTRL_C_EVENT || kind == CTRL_BREAK_EVENT {
        super::CANCELLED.store(true, Ordering::Relaxed);
        1
    } else {
        0
    }
}
pub fn install_cancellation() -> io::Result<()> {
    if unsafe { SetConsoleCtrlHandler(Some(cancel), 1) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
