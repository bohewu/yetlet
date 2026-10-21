//! Fixed environment-only probes. Captured bytes never belong in diagnostics.
use std::time::{Duration, Instant};

const DEADLINE: Duration = Duration::from_secs(3);
const CLEANUP: Duration = Duration::from_secs(2);
const LIMIT: usize = 64 * 1024;
const POLL: Duration = Duration::from_millis(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProbeError {
    Unavailable,
    Failed,
    Timeout,
    OutputLimit,
    Cleanup,
}

pub(crate) fn probe(tool: &'static str) -> Result<Vec<u8>, ProbeError> {
    if !matches!(tool, "rustc" | "cargo") {
        return Err(ProbeError::Unavailable);
    }
    native::probe(tool)
}

struct Capture {
    stdout: Vec<u8>,
    total: usize,
}
impl Capture {
    fn new() -> Self {
        Self {
            stdout: Vec::new(),
            total: 0,
        }
    }
    fn append(&mut self, bytes: &[u8], stdout: bool) -> Result<(), ProbeError> {
        self.total += bytes.len();
        if self.total > LIMIT {
            return Err(ProbeError::OutputLimit);
        }
        if stdout {
            self.stdout.extend_from_slice(bytes);
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod native {
    use super::*;
    use std::{
        ffi::CString,
        fs::{File, OpenOptions},
        io::{self, Read},
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::{
                ffi::OsStrExt,
                fs::{MetadataExt, OpenOptionsExt},
                process::CommandExt,
            },
        },
        path::{Component, Path},
        process::{Command, Stdio},
    };

    fn trusted(file: &File, directory: bool) -> io::Result<()> {
        let meta = file.metadata()?;
        if (meta.uid() != 0 && meta.uid() != unsafe { libc::geteuid() })
            || meta.mode() & 0o022 != 0
            || if directory {
                !meta.is_dir()
            } else {
                !meta.is_file() || meta.mode() & 0o111 == 0
            }
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(())
    }

    fn pinned(path: &Path, executable: bool) -> io::Result<File> {
        let path = path.canonicalize()?;
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open("/")?;
        trusted(&file, true)?;
        let parts: Vec<_> = path
            .components()
            .filter_map(|part| match part {
                Component::Normal(name) => Some(name),
                _ => None,
            })
            .collect();
        for (index, name) in parts.iter().enumerate() {
            let last = index + 1 == parts.len();
            let name = CString::new(name.as_bytes())?;
            let fd = unsafe {
                libc::openat(
                    file.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY
                        | libc::O_CLOEXEC
                        | libc::O_NOFOLLOW
                        | libc::O_NONBLOCK
                        | if last { 0 } else { libc::O_DIRECTORY },
                )
            };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            file = unsafe { File::from_raw_fd(fd) };
            if last && !executable {
                let meta = file.metadata()?;
                if !meta.is_file()
                    || (meta.uid() != 0 && meta.uid() != unsafe { libc::geteuid() })
                    || meta.mode() & 0o022 != 0
                {
                    return Err(io::ErrorKind::PermissionDenied.into());
                }
            } else {
                trusted(&file, !last)?;
            }
        }
        Ok(file)
    }

    fn executable(tool: &str) -> Result<File, ProbeError> {
        for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
            if !directory.is_absolute() {
                continue;
            }
            let path = match directory.join(tool).canonicalize() {
                Ok(path) => path,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(_) => return Err(ProbeError::Unavailable),
            };
            if path.file_name() != Some(std::ffi::OsStr::new("rustup")) {
                return pinned(&path, true).map_err(|_| ProbeError::Unavailable);
            }
            // Recognized rustup proxies are never executed. Select only an
            // already installed default/explicit local toolchain, independent
            // of the invocation project's rust-toolchain file and overrides.
            let home = std::env::var_os("RUSTUP_HOME")
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME")
                        .map(|home| std::path::PathBuf::from(home).join(".rustup"))
                })
                .filter(|home| home.is_absolute())
                .ok_or(ProbeError::Unavailable)?;
            let selected = match std::env::var("RUSTUP_TOOLCHAIN") {
                Ok(selected) => selected,
                Err(std::env::VarError::NotUnicode(_)) => return Err(ProbeError::Unavailable),
                Err(std::env::VarError::NotPresent) => {
                    let file = pinned(&home.join("settings.toml"), false)
                        .map_err(|_| ProbeError::Unavailable)?;
                    let mut bytes = Vec::new();
                    file.take((LIMIT + 1) as u64)
                        .read_to_end(&mut bytes)
                        .map_err(|_| ProbeError::Unavailable)?;
                    if bytes.len() > LIMIT {
                        return Err(ProbeError::Unavailable);
                    }
                    let text = std::str::from_utf8(&bytes).map_err(|_| ProbeError::Unavailable)?;
                    let settings: toml::Value =
                        toml::from_str(text).map_err(|_| ProbeError::Unavailable)?;
                    settings
                        .get("default_toolchain")
                        .and_then(toml::Value::as_str)
                        .ok_or(ProbeError::Unavailable)?
                        .to_owned()
                }
            };
            if selected.is_empty()
                || !selected
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
                || matches!(selected.as_str(), "." | "..")
            {
                return Err(ProbeError::Unavailable);
            }
            return pinned(
                &home
                    .join("toolchains")
                    .join(selected)
                    .join("bin")
                    .join(tool),
                true,
            )
            .map_err(|_| ProbeError::Unavailable);
        }
        Err(ProbeError::Unavailable)
    }

    pub(super) fn probe(tool: &str) -> Result<Vec<u8>, ProbeError> {
        let file = executable(tool)?;
        let mut command = Command::new(format!("/proc/self/fd/{}", file.as_raw_fd()));
        command
            .arg0(tool)
            .arg("--version")
            .current_dir(Path::new("/"))
            .env("RUSTUP_AUTO_INSTALL", "0")
            .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
            .env("CARGO_NET_OFFLINE", "true");
        run(&mut command, DEADLINE)
    }

    fn nonblocking(fd: i32) -> io::Result<()> {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    fn drain(
        pipe: &mut impl Read,
        eof: &mut bool,
        capture: &mut Capture,
        stdout: bool,
    ) -> Result<(), ProbeError> {
        if *eof {
            return Ok(());
        }
        // One bounded read per stream per turn: an infinite writer cannot hide
        // the other pipe, the deadline or the combined output limit.
        let mut bytes = [0; 4096];
        match pipe.read(&mut bytes) {
            Ok(0) => {
                *eof = true;
                Ok(())
            }
            Ok(size) => capture.append(&bytes[..size], stdout),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                Ok(())
            }
            Err(_) => Err(ProbeError::Failed),
        }
    }
    fn exited(pid: i32) -> io::Result<bool> {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        if unsafe {
            libc::waitid(
                libc::P_PID,
                pid as u32,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        } < 0
        {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                return Ok(false);
            }
            return Err(error);
        }
        Ok(unsafe { info.si_pid() } != 0)
    }

    pub(super) fn run(command: &mut Command, budget: Duration) -> Result<Vec<u8>, ProbeError> {
        if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } < 0 {
            return Err(ProbeError::Unavailable);
        }
        command
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let deadline = Instant::now() + budget;
        let mut child = command.spawn().map_err(|_| ProbeError::Unavailable)?;
        let pid = child.id() as i32;
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let mut out_eof = false;
        let mut err_eof = false;
        let mut capture = Capture::new();
        let pipes_ready =
            nonblocking(stdout.as_raw_fd()).is_ok() && nonblocking(stderr.as_raw_fd()).is_ok();
        if !pipes_ready {
            out_eof = true;
            err_eof = true;
        }
        let mut result = (|| {
            if !pipes_ready {
                return Err(ProbeError::Failed);
            }
            loop {
                drain(&mut stdout, &mut out_eof, &mut capture, true)?;
                drain(&mut stderr, &mut err_eof, &mut capture, false)?;
                if exited(pid).map_err(|_| ProbeError::Failed)? {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    return Err(ProbeError::Timeout);
                }
                std::thread::sleep(POLL);
            }
        })();
        // The unreaped leader pins the group identity through the final kill.
        let cleanup_deadline = Instant::now() + CLEANUP;
        if unsafe { libc::kill(-pid, libc::SIGKILL) } < 0
            && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
        {
            return Err(ProbeError::Cleanup);
        }
        let mut status = None;
        loop {
            // After a capture failure drain/discard without allocating. Closing
            // pipes alone is not proof that their owned writers were reaped.
            for (pipe, eof, out) in [
                (&mut stdout as &mut dyn Read, &mut out_eof, true),
                (&mut stderr as &mut dyn Read, &mut err_eof, false),
            ] {
                if *eof {
                    continue;
                }
                let mut bytes = [0; 4096];
                match pipe.read(&mut bytes) {
                    Ok(0) => *eof = true,
                    Ok(size) => {
                        if result.is_ok() {
                            result = capture.append(&bytes[..size], out);
                        }
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                        ) => {}
                    Err(_) => {
                        result = Err(ProbeError::Failed);
                        *eof = true;
                    }
                }
            }
            if status.is_none() {
                status = child.try_wait().map_err(|_| ProbeError::Cleanup)?;
            }
            if status.is_some() {
                loop {
                    let reaped =
                        unsafe { libc::waitpid(-pid, std::ptr::null_mut(), libc::WNOHANG) };
                    if reaped > 0 {
                        continue;
                    }
                    if reaped < 0 {
                        let error = io::Error::last_os_error();
                        if error.kind() == io::ErrorKind::Interrupted {
                            continue;
                        }
                        if error.raw_os_error() != Some(libc::ECHILD) {
                            return Err(ProbeError::Cleanup);
                        }
                    }
                    break;
                }
                if out_eof
                    && err_eof
                    && unsafe { libc::kill(-pid, 0) } < 0
                    && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    break;
                }
            }
            if Instant::now() >= cleanup_deadline {
                return Err(ProbeError::Cleanup);
            }
            std::thread::sleep(POLL);
        }
        result?;
        if !status.unwrap().success() {
            return Err(ProbeError::Failed);
        }
        Ok(capture.stdout)
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod native {
    use super::*;
    pub(super) fn probe(_: &str) -> Result<Vec<u8>, ProbeError> {
        Err(ProbeError::Unavailable)
    }
}

#[cfg(windows)]
mod native {
    use super::super::windows::{self as owned, Attributes, Job};
    use super::*;
    use std::{
        ffi::OsStr,
        mem::size_of,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::Command,
        ptr::{null, null_mut},
    };
    use windows_sys::Win32::{
        Foundation::{
            ERROR_BROKEN_PIPE, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT,
        },
        Security::SECURITY_ATTRIBUTES,
        Storage::FileSystem::{
            CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadFile,
        },
        System::{
            JobObjects::*,
            Pipes::{CreatePipe, PeekNamedPipe},
            Threading::*,
        },
    };

    fn pipe() -> Result<(OwnedHandle, OwnedHandle), ProbeError> {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        let (mut read, mut write) = (null_mut(), null_mut());
        owned::win(unsafe { CreatePipe(&mut read, &mut write, &attributes, 0) })
            .map_err(|_| ProbeError::Unavailable)?;
        let read = unsafe { OwnedHandle::from_raw_handle(read) };
        let write = unsafe { OwnedHandle::from_raw_handle(write) };
        // Only writer ends are passed in the explicit native handle allowlist.
        Ok((read, write))
    }
    fn drain(
        pipe: &OwnedHandle,
        eof: &mut bool,
        capture: &mut Capture,
        stdout: bool,
    ) -> Result<(), ProbeError> {
        if *eof {
            return Ok(());
        }
        let mut available = 0;
        if unsafe {
            PeekNamedPipe(
                pipe.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        } == 0
        {
            if std::io::Error::last_os_error().raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                *eof = true;
                return Ok(());
            }
            return Err(ProbeError::Failed);
        }
        if available == 0 {
            return Ok(());
        }
        let mut bytes = [0u8; 4096];
        let mut size = 0;
        owned::win(unsafe {
            ReadFile(
                pipe.as_raw_handle(),
                bytes.as_mut_ptr(),
                available.min(bytes.len() as u32),
                &mut size,
                null_mut(),
            )
        })
        .map_err(|_| ProbeError::Failed)?;
        capture.append(&bytes[..size as usize], stdout)
    }

    pub(super) fn probe(tool: &str) -> Result<Vec<u8>, ProbeError> {
        let program = owned::executable(OsStr::new(tool)).map_err(|_| ProbeError::Unavailable)?;
        let mut command = Command::new(&program.path);
        command
            .arg("--version")
            .env("RUSTUP_AUTO_INSTALL", "0")
            .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
            .env("CARGO_NET_OFFLINE", "true");
        let application =
            owned::terminated(program.path.as_os_str()).map_err(|_| ProbeError::Unavailable)?;
        // Avoid project-local toolchain selection and an application-directory cwd.
        let cwd = owned::terminated(
            program
                .path
                .parent()
                .ok_or(ProbeError::Unavailable)?
                .as_os_str(),
        )
        .map_err(|_| ProbeError::Unavailable)?;
        let mut argv = Vec::new();
        owned::argument(&mut argv, program.path.as_os_str())
            .map_err(|_| ProbeError::Unavailable)?;
        argv.push(b' ' as u16);
        owned::argument(&mut argv, OsStr::new("--version")).map_err(|_| ProbeError::Unavailable)?;
        argv.push(0);
        let env = owned::environment(&command).map_err(|_| ProbeError::Unavailable)?;
        let (stdout, out_write) = pipe()?;
        let (stderr, err_write) = pipe()?;
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        let nul = owned::terminated(OsStr::new("NUL")).map_err(|_| ProbeError::Unavailable)?;
        let raw = unsafe {
            CreateFileW(
                nul.as_ptr(),
                windows_sys::Win32::Foundation::GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                &attributes,
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(ProbeError::Unavailable);
        }
        let input = unsafe { OwnedHandle::from_raw_handle(raw) };
        let handles: [HANDLE; 3] = [
            input.as_raw_handle(),
            out_write.as_raw_handle(),
            err_write.as_raw_handle(),
        ];
        let mut attributes = Attributes::new(&handles).map_err(|_| ProbeError::Unavailable)?;
        let mut startup = STARTUPINFOEXW::default();
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        [
            startup.StartupInfo.hStdInput,
            startup.StartupInfo.hStdOutput,
            startup.StartupInfo.hStdError,
        ] = handles;
        startup.lpAttributeList = attributes.0.as_mut_ptr().cast();
        let job = Job::new().map_err(|_| ProbeError::Unavailable)?;
        program.verify().map_err(|_| ProbeError::Unavailable)?;
        let deadline = Instant::now() + DEADLINE;
        let mut info = PROCESS_INFORMATION::default();
        owned::win(unsafe {
            CreateProcessW(
                application.as_ptr(),
                argv.as_mut_ptr(),
                null(),
                null(),
                1,
                CREATE_SUSPENDED
                    | CREATE_UNICODE_ENVIRONMENT
                    | EXTENDED_STARTUPINFO_PRESENT
                    | CREATE_NO_WINDOW,
                env.as_ptr().cast(),
                cwd.as_ptr(),
                &startup.StartupInfo,
                &mut info,
            )
        })
        .map_err(|_| ProbeError::Unavailable)?;
        let process = unsafe { OwnedHandle::from_raw_handle(info.hProcess) };
        let thread = unsafe { OwnedHandle::from_raw_handle(info.hThread) };
        drop(out_write);
        drop(err_write);
        drop(input);
        let mut capture = Capture::new();
        let (mut out_eof, mut err_eof) = (false, false);
        let mut result = (|| {
            owned::win(unsafe {
                AssignProcessToJobObject(job.0.as_raw_handle(), process.as_raw_handle())
            })
            .map_err(|_| ProbeError::Unavailable)?;
            program.verify().map_err(|_| ProbeError::Unavailable)?;
            if unsafe { ResumeThread(thread.as_raw_handle()) } != 1 {
                return Err(ProbeError::Failed);
            }
            loop {
                drain(&stdout, &mut out_eof, &mut capture, true)?;
                drain(&stderr, &mut err_eof, &mut capture, false)?;
                match unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } {
                    WAIT_OBJECT_0 => return Ok(()),
                    WAIT_TIMEOUT => (),
                    _ => return Err(ProbeError::Failed),
                }
                if Instant::now() >= deadline {
                    return Err(ProbeError::Timeout);
                }
                std::thread::sleep(POLL);
            }
        })();
        let cleanup_deadline = Instant::now() + CLEANUP;
        // Also terminate the suspended process if Job assignment failed.
        if unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } == WAIT_TIMEOUT {
            owned::win(unsafe { TerminateProcess(process.as_raw_handle(), 125) })
                .map_err(|_| ProbeError::Cleanup)?;
        }
        owned::win(unsafe { TerminateJobObject(job.0.as_raw_handle(), 125) })
            .map_err(|_| ProbeError::Cleanup)?;
        loop {
            if result.is_ok()
                && let Err(error) = drain(&stdout, &mut out_eof, &mut capture, true)
                    .and_then(|_| drain(&stderr, &mut err_eof, &mut capture, false))
            {
                result = Err(error);
            }
            if job.active().map_err(|_| ProbeError::Cleanup)? == 0
                && unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } == WAIT_OBJECT_0
            {
                // Once all owned writers exit, bounded pipes can be drained to EOF.
                if result.is_err() || (out_eof && err_eof) {
                    break;
                }
            }
            if Instant::now() >= cleanup_deadline {
                return Err(ProbeError::Cleanup);
            }
            std::thread::sleep(POLL);
        }
        result?;
        let mut code = 0;
        owned::win(unsafe { GetExitCodeProcess(process.as_raw_handle(), &mut code) })
            .map_err(|_| ProbeError::Failed)?;
        if code != 0 {
            return Err(ProbeError::Failed);
        }
        Ok(capture.stdout)
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{io::Write, process::Command};

    #[test]
    fn fixture() {
        let Ok(mode) = std::env::var("YETLET_PROBE_FIXTURE") else {
            return;
        };
        match mode.as_str() {
            "ok" => {
                println!("rustc 1.99.0 (fixture)");
                eprintln!("bounded stderr");
            }
            "fail" => std::process::exit(23),
            "overflow" => {
                std::io::stdout().write_all(&vec![b'a'; LIMIT / 2]).unwrap();
                std::io::stderr().write_all(&vec![b'b'; LIMIT / 2]).unwrap();
            }
            "descendant" => unsafe {
                let pid = libc::fork();
                if pid == 0 {
                    libc::signal(libc::SIGTERM, libc::SIG_IGN);
                    loop {
                        libc::pause();
                    }
                }
                println!("{pid}");
                std::io::stdout().flush().unwrap();
            },
            "timeout" => loop {
                std::thread::sleep(Duration::from_secs(1));
            },
            _ => panic!("unknown fixture"),
        }
    }
    fn run(mode: &str, duration: Duration) -> Result<Vec<u8>, ProbeError> {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "platform::process::probe::tests::fixture",
                "--nocapture",
            ])
            .env("YETLET_PROBE_FIXTURE", mode);
        native::run(&mut command, duration)
    }
    #[test]
    fn successful_capture_and_failed_exit() {
        let output = run("ok", DEADLINE).unwrap();
        assert!(output.windows(12).any(|bytes| bytes == b"rustc 1.99.0"));
        assert!(!output.windows(14).any(|bytes| bytes == b"bounded stderr"));
        assert_eq!(run("fail", DEADLINE), Err(ProbeError::Failed));
    }
    #[test]
    fn combined_output_is_bounded() {
        assert_eq!(run("overflow", DEADLINE), Err(ProbeError::OutputLimit));
    }
    #[test]
    fn deadline_and_resistant_descendant_cleanup() {
        let start = Instant::now();
        assert_eq!(
            run("timeout", Duration::from_millis(75)),
            Err(ProbeError::Timeout)
        );
        assert!(start.elapsed() < Duration::from_secs(3));
        let output = run("descendant", DEADLINE).unwrap();
        let text = String::from_utf8(output).unwrap();
        let pid: i32 = text.lines().find_map(|line| line.parse().ok()).unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
    #[test]
    fn fixed_tool_allowlist() {
        assert_eq!(probe("sh"), Err(ProbeError::Unavailable));
    }
}
