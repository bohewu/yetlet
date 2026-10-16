//! Native owned Cargo Job; console cancellation is scoped to its child group.
use crate::{
    app::Completion,
    platform::fs::{App, Executable},
};
use std::{
    cmp::Ordering,
    ffi::OsStr,
    io,
    mem::{size_of, size_of_val},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::ExitStatusExt,
    },
    process::{Command, ExitStatus},
    ptr::{null, null_mut},
    sync::atomic::{AtomicBool, AtomicU32, Ordering as AtomicOrdering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
        WAIT_TIMEOUT,
    },
    Globalization::CompareStringOrdinal,
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
    System::{
        Console::{
            CTRL_BREAK_EVENT, CTRL_C_EVENT, ENABLE_PROCESSED_INPUT, GenerateConsoleCtrlEvent,
            GetConsoleMode, GetConsoleProcessList, GetStdHandle, STD_ERROR_HANDLE,
            STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetConsoleCtrlHandler,
        },
        JobObjects::*,
        Threading::*,
    },
};

const CLEANUP: Duration = Duration::from_secs(5);
const GRACE: Duration = Duration::from_millis(750);
const POLL: Duration = Duration::from_millis(10);
const CONTROL_EXIT: u32 = 0xc000013a; // STATUS_CONTROL_C_EXIT, not a POSIX signal.
static CONSOLE_OWNED: AtomicBool = AtomicBool::new(false);
static CONTROL: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn control(event: u32) -> i32 {
    if matches!(event, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        // No allocation, locks, IO or Win32 calls on the OS handler thread.
        let _ =
            CONTROL.compare_exchange(0, event + 1, AtomicOrdering::SeqCst, AtomicOrdering::SeqCst);
        1
    } else {
        0
    }
}

struct Console(bool);
impl Console {
    fn install() -> io::Result<Self> {
        let mut pid = 0;
        // This also works when standard streams are redirected but a console
        // remains attached. Never attach/detach or change console modes here.
        if unsafe { GetConsoleProcessList(&mut pid, 1) } == 0 {
            eprintln!("Windows dev requires an attached console; no Cargo child was started");
            return Err(io::ErrorKind::Unsupported.into());
        }
        let name = terminated(OsStr::new("CONIN$"))?;
        let raw = unsafe {
            CreateFileW(
                name.as_ptr(),
                windows_sys::Win32::Foundation::GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null(),
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let input = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut mode = 0;
        win(unsafe { GetConsoleMode(input.as_raw_handle(), &mut mode) })?;
        if mode & ENABLE_PROCESSED_INPUT == 0 {
            eprintln!(
                "Windows dev requires processed console Ctrl-C input; no Cargo child was started"
            );
            return Err(io::ErrorKind::Unsupported.into());
        }
        CONSOLE_OWNED
            .compare_exchange(false, true, AtomicOrdering::SeqCst, AtomicOrdering::SeqCst)
            .map_err(|_| io::Error::other("console supervision is already active"))?;
        CONTROL.store(0, AtomicOrdering::SeqCst);
        if let Err(error) = win(unsafe { SetConsoleCtrlHandler(Some(control), 1) }) {
            CONSOLE_OWNED.store(false, AtomicOrdering::SeqCst);
            return Err(error);
        }
        Ok(Self(true))
    }
    fn restore(&mut self) -> io::Result<()> {
        if self.0 {
            // Remove only our added handler; preserve caller handlers, modes,
            // stdio and the inherited Ctrl-C-ignore attribute without changing it.
            win(unsafe { SetConsoleCtrlHandler(Some(control), 0) })?;
            self.0 = false;
            CONSOLE_OWNED.store(false, AtomicOrdering::SeqCst);
        }
        Ok(())
    }
}
impl Drop for Console {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn win(ok: i32) -> io::Result<()> {
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err(io::Error::other("NUL in native argument"));
    }
    Ok(value)
}
fn terminated(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut value = wide(value)?;
    value.push(0);
    Ok(value)
}

pub(crate) fn pin_cwd(app: &App, command: &mut Command) -> io::Result<()> {
    if !matches!(command.get_args().next(), Some(arg) if arg == "build" || arg == "run") {
        return Err(io::ErrorKind::Unsupported.into());
    }
    app.verify()?;
    command.current_dir(&app.cwd);
    Ok(())
}

fn executable(program: &OsStr) -> io::Result<Executable> {
    // Only absolute PATH entries are eligible. Never ask Windows to search the
    // selected application directory or invoke cmd/bat/PowerShell.
    if program == "cargo" {
        for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
            if !directory.is_absolute() {
                continue;
            }
            let path = directory.join("cargo.exe");
            // This probe only skips missing PATH entries. Authorization and all
            // executable reads use the pinned, no-reparse handle open below.
            if path.try_exists()? {
                return Executable::open(&path);
            }
        }
        return Err(io::ErrorKind::NotFound.into());
    }
    // Direct absolute native fixture executables are an internal test seam.
    #[cfg(test)]
    if std::path::Path::new(program).is_absolute() {
        return Executable::open(std::path::Path::new(program));
    }
    Err(io::ErrorKind::Unsupported.into())
}

// Windows CRT argv quoting. No shell is involved, including for Unicode/spaces.
fn argument(output: &mut Vec<u16>, value: &OsStr) -> io::Result<()> {
    output.push(b'"' as u16);
    let mut slashes = 0;
    for unit in wide(value)? {
        if unit == b'\\' as u16 {
            slashes += 1;
            continue;
        }
        output.extend(std::iter::repeat_n(
            b'\\' as u16,
            slashes * if unit == b'"' as u16 { 2 } else { 1 },
        ));
        if unit == b'"' as u16 {
            output.push(b'\\' as u16);
        }
        output.push(unit);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
    output.push(b'"' as u16);
    Ok(())
}

fn ordinal(a: &[u16], b: &[u16]) -> Ordering {
    // SAFETY: live UTF-16 buffers, bounded lengths, comparison has no side effects.
    match unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), b.len() as i32, 1) }
    {
        1 => Ordering::Less,
        3 => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

fn environment(command: &Command) -> io::Result<Vec<u16>> {
    let mut vars: Vec<(Vec<u16>, Vec<u16>)> = std::env::vars_os()
        .map(|(key, value)| Ok((wide(&key)?, wide(&value)?)))
        .collect::<io::Result<_>>()?;
    for (key, value) in command.get_envs() {
        let key = wide(key)?;
        vars.retain(|(existing, _)| ordinal(existing, &key) != Ordering::Equal);
        if let Some(value) = value {
            vars.push((key, wide(value)?));
        }
    }
    vars.sort_by(|(a, _), (b, _)| ordinal(a, b));
    let mut block = Vec::new();
    for (key, value) in vars {
        block.extend(key);
        block.push(b'=' as u16);
        block.extend(value);
        block.push(0);
    }
    block.push(0);
    if block.len() == 1 {
        block.push(0);
    }
    Ok(block)
}

fn stdio(which: u32) -> io::Result<OwnedHandle> {
    // SAFETY: obtain a borrowed standard handle without changing it.
    let original = unsafe { GetStdHandle(which) };
    let raw = if original.is_null() || original == INVALID_HANDLE_VALUE {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        // SAFETY: the literal NUL device is used only for absent headless stdio.
        let raw = unsafe {
            CreateFileW(
                terminated(OsStr::new("NUL"))?.as_ptr(),
                if which == STD_INPUT_HANDLE {
                    windows_sys::Win32::Foundation::GENERIC_READ
                } else {
                    windows_sys::Win32::Foundation::GENERIC_WRITE
                },
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                &attributes,
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        raw
    } else {
        let mut raw = null_mut();
        // SAFETY: duplicate into this process; only the duplicate is inheritable.
        win(unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                original,
                GetCurrentProcess(),
                &mut raw,
                0,
                1,
                DUPLICATE_SAME_ACCESS,
            )
        })?;
        raw
    };
    // SAFETY: the native API transferred ownership of one live handle.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}

struct Attributes(Vec<usize>);
impl Attributes {
    fn new(handles: &[HANDLE]) -> io::Result<Self> {
        let mut bytes = 0;
        // SAFETY: standard size query; no list is initialized on this call.
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut buffer = vec![0_usize; bytes.div_ceil(size_of::<usize>())];
        // SAFETY: aligned writable buffer of the queried size.
        win(unsafe {
            InitializeProcThreadAttributeList(buffer.as_mut_ptr().cast(), 1, 0, &mut bytes)
        })?;
        let mut list = Self(buffer);
        // SAFETY: initialized list; handles remain live through CreateProcessW.
        win(unsafe {
            UpdateProcThreadAttribute(
                list.0.as_mut_ptr().cast(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                std::mem::size_of_val(handles),
                null_mut(),
                null(),
            )
        })?;
        Ok(list)
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: initialized list is destroyed before releasing its backing buffer.
        unsafe {
            DeleteProcThreadAttributeList(self.0.as_mut_ptr().cast());
        }
    }
}

struct Job(OwnedHandle);
impl Job {
    fn new() -> io::Result<Self> {
        // SAFETY: unnamed, non-inheritable job. No unrelated job can be opened.
        let raw = unsafe { CreateJobObjectW(null(), null()) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful API call returned an owned job handle.
        let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: SDK class/record/size match. No BREAKAWAY flags are enabled.
        win(unsafe {
            SetInformationJobObject(
                job.0.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&limits) as u32,
            )
        })?;
        Ok(job)
    }
    fn active(&self) -> io::Result<u32> {
        let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        // SAFETY: matching SDK output class/record/size and live job.
        win(unsafe {
            QueryInformationJobObject(
                self.0.as_raw_handle(),
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of_val(&info) as u32,
                null_mut(),
            )
        })?;
        Ok(info.ActiveProcesses)
    }
    fn cleanup(&self, code: u32) -> io::Result<()> {
        // SAFETY: only the private owned job is terminated, never a PID/group search.
        win(unsafe { TerminateJobObject(self.0.as_raw_handle(), code) })?;
        let deadline = Instant::now() + CLEANUP;
        while self.active()? != 0 {
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}

struct Child {
    process: OwnedHandle,
    thread: OwnedHandle,
    job: Job,
    done: bool,
}
impl Child {
    fn cleanup(&mut self, code: u32) -> io::Result<()> {
        // TerminateProcess is also required when assignment failed: the suspended
        // child may not yet belong to our Job. Its HANDLE prevents PID reuse.
        if unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } == WAIT_TIMEOUT {
            win(unsafe { TerminateProcess(self.process.as_raw_handle(), code) })?;
        }
        self.job.cleanup(code)?;
        match unsafe {
            WaitForSingleObject(self.process.as_raw_handle(), CLEANUP.as_millis() as u32)
        } {
            WAIT_OBJECT_0 => {
                self.done = true;
                Ok(())
            }
            WAIT_TIMEOUT => Err(io::ErrorKind::TimedOut.into()),
            _ => Err(io::Error::last_os_error()),
        }
    }
}
impl Drop for Child {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.cleanup(125);
        }
        // Closing the non-inherited last job handle is a final fail-closed kill.
    }
}

pub(crate) fn run(app: &App, command: &mut Command) -> io::Result<Completion> {
    let dev = command.get_args().next() == Some(OsStr::new("run"));
    run_owned(app, command, dev)
}

fn run_owned(app: &App, command: &mut Command, dev: bool) -> io::Result<Completion> {
    let mut console = if dev { Some(Console::install()?) } else { None };
    let program = executable(command.get_program())?;
    let application = terminated(program.path.as_os_str())?;
    let cwd = terminated(app.cwd.as_os_str())?;
    let mut argv = Vec::new();
    argument(&mut argv, program.path.as_os_str())?;
    for arg in command.get_args() {
        argv.push(b' ' as u16);
        argument(&mut argv, arg)?;
    }
    argv.push(0);
    if argv.len() > 32767 {
        return Err(io::Error::other("native command line is too long"));
    }
    let env = environment(command)?;
    let stdio = [
        stdio(STD_INPUT_HANDLE)?,
        stdio(STD_OUTPUT_HANDLE)?,
        stdio(STD_ERROR_HANDLE)?,
    ];
    let handles = stdio.each_ref().map(|h| h.as_raw_handle());
    let mut attributes = Attributes::new(&handles)?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    [
        startup.StartupInfo.hStdInput,
        startup.StartupInfo.hStdOutput,
        startup.StartupInfo.hStdError,
    ] = handles;
    startup.lpAttributeList = attributes.0.as_mut_ptr().cast();
    let job = Job::new()?;
    program.verify()?;
    app.verify()?;
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: explicit vetted exe/cwd, writable argv, terminated Unicode env,
    // live inheritable stdio allowlist. Both returned handles are adopted below.
    // The initial thread cannot execute before Job assignment and second verify.
    win(unsafe {
        CreateProcessW(
            application.as_ptr(),
            argv.as_mut_ptr(),
            null(),
            null(),
            1,
            CREATE_SUSPENDED
                | CREATE_UNICODE_ENVIRONMENT
                | EXTENDED_STARTUPINFO_PRESENT
                | CREATE_NEW_PROCESS_GROUP,
            env.as_ptr().cast(),
            cwd.as_ptr(),
            &startup.StartupInfo,
            &mut info,
        )
    })?;
    let mut child = Child {
        process: unsafe { OwnedHandle::from_raw_handle(info.hProcess) },
        thread: unsafe { OwnedHandle::from_raw_handle(info.hThread) },
        job,
        done: false,
    };
    let result = (|| {
        // SAFETY: exact suspended process handle; failure never resumes the child.
        if let Err(error) = win(unsafe {
            AssignProcessToJobObject(child.job.0.as_raw_handle(), child.process.as_raw_handle())
        }) {
            if dev {
                eprintln!(
                    "Windows dev cannot establish its private Job (nested Job may be incompatible); suspended child will be reaped"
                );
            }
            return Err(error);
        }
        let mut member = 0;
        win(unsafe {
            IsProcessInJob(
                child.process.as_raw_handle(),
                child.job.0.as_raw_handle(),
                &mut member,
            )
        })?;
        if member == 0 {
            return Err(io::Error::other("child Job membership is unverified"));
        }
        app.verify()?;
        program.verify()?;
        // SAFETY: owned primary thread suspended exactly once by CreateProcessW.
        if unsafe { ResumeThread(child.thread.as_raw_handle()) } != 1 {
            return Err(io::Error::other("cannot resume the owned primary thread"));
        }
        let cancelled = if dev {
            loop {
                if CONTROL.load(AtomicOrdering::SeqCst) != 0 {
                    break true;
                }
                match unsafe { WaitForSingleObject(child.process.as_raw_handle(), 10) } {
                    WAIT_OBJECT_0 => break false,
                    WAIT_TIMEOUT => (),
                    _ => return Err(io::Error::last_os_error()),
                }
            }
        } else {
            if unsafe { WaitForSingleObject(child.process.as_raw_handle(), INFINITE) }
                != WAIT_OBJECT_0
            {
                return Err(io::Error::last_os_error());
            }
            false
        };
        if dev && child.job.active()? != 0 {
            // info.dwProcessId is the group we created, with its process HANDLE
            // held through the last signal. Never generate CTRL_C_EVENT or use
            // group zero. New consoles/groups may miss BREAK but stay in our Job.
            win(unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, info.dwProcessId) })?;
            let deadline = Instant::now() + GRACE;
            while child.job.active()? != 0 && Instant::now() < deadline {
                std::thread::sleep(POLL);
            }
        }
        if dev {
            child.cleanup(if cancelled { CONTROL_EXIT } else { 125 })?;
        }
        let mut code = 0;
        win(unsafe { GetExitCodeProcess(child.process.as_raw_handle(), &mut code) })?;
        Ok(Completion::Exited(ExitStatus::from_raw(code)))
    })();
    // Even a successful Cargo exit must not leave owned descendants behind.
    let cleanup = if child.done {
        Ok(())
    } else {
        child.cleanup(125)
    };
    let restore = console.as_mut().map_or(Ok(()), Console::restore);
    cleanup?;
    restore?;
    result
}

pub(crate) fn signal(_: &ExitStatus) -> Option<i32> {
    None
}
pub(crate) fn reraise(_: i32) {}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
