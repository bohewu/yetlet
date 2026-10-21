//! Native owned Cargo Job; console cancellation is scoped to its child group.
use crate::{
    app::Completion,
    platform::fs::{App, Executable},
};
use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs::File,
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
        DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
        SetHandleInformation, WAIT_OBJECT_0, WAIT_TIMEOUT,
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
        Pipes::CreatePipe,
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

pub(super) fn win(ok: i32) -> io::Result<()> {
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
pub(super) fn terminated(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut value = wide(value)?;
    value.push(0);
    Ok(value)
}

pub(crate) fn pin_cwd(app: &App, command: &mut Command) -> io::Result<()> {
    if !matches!(
        command.get_program().to_str(),
        Some("node" | "watchexec" | "watchexec.exe")
    ) && !matches!(command.get_args().next(), Some(arg) if arg == "build" || arg == "run")
    {
        return Err(io::ErrorKind::Unsupported.into());
    }
    app.verify()?;
    command.current_dir(&app.cwd);
    Ok(())
}

pub(super) fn executable(program: &OsStr) -> io::Result<Executable> {
    // Only absolute PATH entries are eligible. Never ask Windows to search the
    // selected application directory or invoke cmd/bat/PowerShell.
    if matches!(
        program.to_str(),
        Some("cargo" | "rustc" | "node" | "watchexec" | "watchexec.exe")
    ) {
        for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
            if !directory.is_absolute() {
                continue;
            }
            let name = if program == "watchexec.exe" {
                "watchexec.exe".to_owned()
            } else {
                format!("{}.exe", program.to_string_lossy())
            };
            let path = directory.join(name);
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
pub(super) fn argument(output: &mut Vec<u16>, value: &OsStr) -> io::Result<()> {
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

pub(super) fn environment(command: &Command) -> io::Result<Vec<u16>> {
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

pub(super) fn stdio(which: u32) -> io::Result<OwnedHandle> {
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

fn null_input() -> io::Result<OwnedHandle> {
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    let raw = unsafe {
        CreateFileW(
            terminated(OsStr::new("NUL"))?.as_ptr(),
            windows_sys::Win32::Foundation::GENERIC_READ,
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
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}

pub(super) struct Attributes(pub(super) Vec<usize>);
impl Attributes {
    pub(super) fn new(handles: &[HANDLE]) -> io::Result<Self> {
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

pub(super) struct Job(pub(super) OwnedHandle);
impl Job {
    pub(super) fn new() -> io::Result<Self> {
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
    pub(super) fn active(&self) -> io::Result<u32> {
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
    pub(super) fn cleanup(&self, code: u32) -> io::Result<()> {
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
    pid: u32,
    _program: Executable,
    _script: Option<Executable>,
    stdout: Option<File>,
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

pub(crate) fn npm_command(dev: bool) -> io::Result<Command> {
    let node = executable(OsStr::new("node"))?;
    let script = node
        .path
        .parent()
        .ok_or(io::ErrorKind::NotFound)?
        .join("node_modules/npm/bin/npm-cli.js");
    let checked = Executable::open(&script)?;
    checked.verify()?;
    let mut command = Command::new("node");
    command
        .arg(script)
        .args(["run", if dev { "dev" } else { "build" }]);
    Ok(command)
}

pub(crate) fn run_pair(
    app: &App,
    rust: &mut Command,
    frontend: &App,
    command: &mut Command,
) -> io::Result<Completion> {
    // Install before starting either child; headless/raw-console limits remain explicit.
    let mut console = Console::install()?;
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], 5173));
    drop(std::net::TcpListener::bind(address)?);
    let mut front = spawn_owned(frontend, command)?;
    let mut back: Option<Child> = None;
    let result: io::Result<bool> = (|| {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if CONTROL.load(AtomicOrdering::SeqCst) != 0 || exited(&front)? {
                return Ok(true);
            }
            if std::net::TcpStream::connect_timeout(&address, Duration::from_millis(20)).is_ok() {
                break;
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            std::thread::sleep(POLL);
        }
        back = Some(spawn_owned(app, rust)?);
        loop {
            if CONTROL.load(AtomicOrdering::SeqCst) != 0 {
                return Ok(false);
            }
            if exited(&front)? {
                return Ok(true);
            }
            if exited(back.as_ref().unwrap())? {
                return Ok(false);
            }
            std::thread::sleep(POLL);
        }
    })();
    let cancelled = CONTROL.load(AtomicOrdering::SeqCst) != 0;
    let front_signal = break_signal(&front);
    let back_signal = back.as_ref().map_or(Ok(()), break_signal);
    let deadline = Instant::now() + GRACE;
    while Instant::now() < deadline {
        std::thread::sleep(POLL);
    }
    let front_cleanup = front.cleanup(if cancelled { CONTROL_EXIT } else { 125 });
    let back_cleanup = back.as_mut().map_or(Ok(()), |child| {
        child.cleanup(if cancelled { CONTROL_EXIT } else { 125 })
    });
    let restore = console.restore();
    front_cleanup?;
    back_cleanup?;
    restore?;
    front_signal?;
    back_signal?;
    let frontend_exited = result?;
    if cancelled {
        return Ok(Completion::Exited(ExitStatus::from_raw(CONTROL_EXIT)));
    }
    let child = if frontend_exited {
        &front
    } else {
        back.as_ref().ok_or(io::ErrorKind::Other)?
    };
    let mut code = 0;
    win(unsafe { GetExitCodeProcess(child.process.as_raw_handle(), &mut code) })?;
    let status = ExitStatus::from_raw(code);
    Ok(if frontend_exited {
        Completion::FrontendExited(status)
    } else {
        Completion::Exited(status)
    })
}

/// Watchexec emits source notifications only; each Cargo generation has its own
/// Job, while Vite and the watcher retain their separate Jobs for the session.
pub(crate) fn run_watch(
    app: &App,
    rust: &mut Command,
    watcher: &mut Command,
    frontend: Option<(&App, &mut Command)>,
) -> io::Result<Completion> {
    let mut console = Console::install()?;
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], 5173));
    if frontend.is_some() {
        drop(std::net::TcpListener::bind(address)?);
    }
    let mut watch = spawn_with_output(app, watcher, true)?;
    let mut events = super::watch::Events::new(watch.stdout.take().unwrap());
    let mut front = None;
    let mut back = None;
    let result = (|| {
        if let Some((frontend, command)) = frontend {
            front = Some(spawn_owned(frontend, command)?);
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if let Some(completion) = watch_session_end(&watch, front.as_ref())? {
                    return Ok(completion);
                }
                if std::net::TcpStream::connect_timeout(&address, Duration::from_millis(20)).is_ok()
                {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(io::ErrorKind::TimedOut.into());
                }
                std::thread::sleep(POLL);
            }
        }
        // Initial compilation is independent of --postpone's event stream.
        back = Some(spawn_owned(app, rust)?);
        loop {
            if let Some(completion) = watch_session_end(&watch, front.as_ref())? {
                return Ok(completion);
            }
            if let Some(child) = back.as_mut()
                && exited(child)?
            {
                let status = exit_status(child)?;
                stop_generation(child)?;
                back = None;
                eprintln!(
                    "CLI_CHILD_EXIT stage=rust: child ended with {status}; waiting for source changes"
                );
            }
            let changed = match events.changed() {
                Ok(changed) => changed,
                Err(error) => {
                    if let Some(completion) = watch_session_end(&watch, front.as_ref())? {
                        return Ok(completion);
                    }
                    return Err(error);
                }
            };
            if changed {
                if let Some(mut child) = back.take() {
                    stop_generation(&mut child)?;
                }
                // Source restarts never set CONTROL or signal the frontend.
                if let Some(completion) = watch_session_end(&watch, front.as_ref())? {
                    return Ok(completion);
                }
                eprintln!("CLI_RUST_RESTART: source changed; starting Cargo");
                back = Some(spawn_owned(app, rust)?);
            }
            std::thread::sleep(POLL);
        }
    })();
    // Signal all live owned groups before the shared grace period; regardless
    // of any signal failure, terminate/reap every Job and restore the handler.
    let cancelled = CONTROL.load(AtomicOrdering::SeqCst) != 0;
    let watch_signal = break_signal(&watch);
    let front_signal = front.as_ref().map_or(Ok(()), break_signal);
    let back_signal = back.as_ref().map_or(Ok(()), break_signal);
    let deadline = Instant::now() + GRACE;
    while Instant::now() < deadline {
        std::thread::sleep(POLL);
    }
    let code = if cancelled { CONTROL_EXIT } else { 125 };
    let watch_cleanup = watch.cleanup(code);
    let front_cleanup = front.as_mut().map_or(Ok(()), |child| child.cleanup(code));
    let back_cleanup = back.as_mut().map_or(Ok(()), |child| child.cleanup(code));
    let restore = console.restore();
    watch_cleanup?;
    front_cleanup?;
    back_cleanup?;
    restore?;
    watch_signal?;
    front_signal?;
    back_signal?;
    result
}

fn exit_status(child: &Child) -> io::Result<ExitStatus> {
    let mut code = 0;
    win(unsafe { GetExitCodeProcess(child.process.as_raw_handle(), &mut code) })?;
    Ok(ExitStatus::from_raw(code))
}

fn watch_session_end(watch: &Child, front: Option<&Child>) -> io::Result<Option<Completion>> {
    if CONTROL.load(AtomicOrdering::SeqCst) != 0 {
        return Ok(Some(Completion::Exited(ExitStatus::from_raw(CONTROL_EXIT))));
    }
    if let Some(front) = front
        && exited(front)?
    {
        return Ok(Some(Completion::FrontendExited(exit_status(front)?)));
    }
    if exited(watch)? {
        let status = exit_status(watch)?;
        eprintln!("CLI_CHILD_EXIT stage=watcher: child ended with {status}");
        return Ok(Some(Completion::WatcherExited(status)));
    }
    Ok(None)
}

fn stop_generation(child: &mut Child) -> io::Result<()> {
    let signal = break_signal(child);
    let deadline = Instant::now() + GRACE;
    while child.job.active()? != 0 && Instant::now() < deadline {
        std::thread::sleep(POLL);
    }
    child.cleanup(125)?;
    signal
}

fn break_signal(child: &Child) -> io::Result<()> {
    if child.job.active()? != 0 {
        win(unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child.pid) })?;
    }
    Ok(())
}

fn exited(child: &Child) -> io::Result<bool> {
    match unsafe { WaitForSingleObject(child.process.as_raw_handle(), 0) } {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        _ => Err(io::Error::last_os_error()),
    }
}

pub(crate) fn run(app: &App, command: &mut Command) -> io::Result<Completion> {
    let dev = command.get_args().next() == Some(OsStr::new("run"));
    run_owned(app, command, dev)
}

fn run_owned(app: &App, command: &mut Command, dev: bool) -> io::Result<Completion> {
    let mut console = if dev { Some(Console::install()?) } else { None };
    let mut child = spawn_owned(app, command)?;
    let result = (|| {
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
            win(unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child.pid) })?;
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

fn spawn_owned(app: &App, command: &mut Command) -> io::Result<Child> {
    spawn_with_output(app, command, false)
}

fn spawn_with_output(app: &App, command: &mut Command, capture: bool) -> io::Result<Child> {
    let program = executable(command.get_program()).inspect_err(|error| {
        // Fixed policy guidance and numeric OS evidence only: never print the
        // selected path, command arguments, environment or arbitrary error text.
        eprintln!(
            "CLI_PROCESS stage=executable: trusted executable selection failed (kind={:?}, native_os_error={:?}); use an absolute, non-reparse PATH entry whose executable has a trusted owner and no untrusted write access",
            error.kind(),
            error.raw_os_error()
        );
    })?;
    let script = if command.get_program() == "node" {
        Some(Executable::open(std::path::Path::new(
            command
                .get_args()
                .next()
                .ok_or(io::ErrorKind::InvalidInput)?,
        ))?)
    } else {
        None
    };
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
    let (stdout, output) = if capture {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        let (mut read, mut write) = (null_mut(), null_mut());
        win(unsafe { CreatePipe(&mut read, &mut write, &attributes, 0) })?;
        let read = unsafe { OwnedHandle::from_raw_handle(read) };
        let write = unsafe { OwnedHandle::from_raw_handle(write) };
        // Only the writer is inherited, through the same explicit allowlist as
        // normal stdio. The read end belongs exclusively to our event reader.
        win(unsafe { SetHandleInformation(read.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) })?;
        (Some(File::from(read)), write)
    } else {
        (None, stdio(STD_OUTPUT_HANDLE)?)
    };
    let stdio = [
        if capture || command.get_program() == "node" {
            null_input()?
        } else {
            stdio(STD_INPUT_HANDLE)?
        },
        output,
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
    let child = Child {
        process: unsafe { OwnedHandle::from_raw_handle(info.hProcess) },
        thread: unsafe { OwnedHandle::from_raw_handle(info.hThread) },
        job,
        done: false,
        pid: info.dwProcessId,
        _program: program,
        _script: script,
        stdout,
    };
    (|| {
        // SAFETY: exact suspended process handle; failure never resumes the child.
        if let Err(error) = win(unsafe {
            AssignProcessToJobObject(child.job.0.as_raw_handle(), child.process.as_raw_handle())
        }) {
            {
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
        child._program.verify()?;
        if let Some(script) = &child._script {
            script.verify()?;
        }
        // SAFETY: owned primary thread suspended exactly once by CreateProcessW.
        if unsafe { ResumeThread(child.thread.as_raw_handle()) } != 1 {
            return Err(io::Error::other("cannot resume the owned primary thread"));
        }
        Ok(())
    })()?;
    Ok(child)
}

pub(crate) fn signal(_: &ExitStatus) -> Option<i32> {
    None
}
pub(crate) fn reraise(_: i32) {}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
