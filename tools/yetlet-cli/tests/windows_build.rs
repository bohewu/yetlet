#![cfg(windows)]
use std::{
    fs, io,
    mem::size_of,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Path, PathBuf},
    process::{Command, Output},
    ptr::null_mut,
};
use windows_sys::Win32::{
    Foundation::{GENERIC_WRITE, INVALID_HANDLE_VALUE, LocalFree},
    Security::{
        ACCESS_ALLOWED_ACE,
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            GetSecurityInfo, SDDL_REVISION_1, SE_FILE_OBJECT,
        },
        DACL_SECURITY_INFORMATION, EqualSid, GetAce, GetSecurityDescriptorControl,
        GetSecurityDescriptorOwner, INHERIT_ONLY_ACE, OWNER_SECURITY_INFORMATION, PSID,
        SE_DACL_PROTECTED, SECURITY_ATTRIBUTES,
    },
    Storage::FileSystem::{
        CREATE_NEW, CreateDirectoryW, CreateFileW, FILE_ALL_ACCESS, FILE_ATTRIBUTE_NORMAL,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ,
        FILE_SHARE_WRITE, READ_CONTROL,
    },
    System::SystemServices::ACCESS_ALLOWED_ACE_TYPE,
};

struct Local(*mut core::ffi::c_void);
impl Drop for Local {
    fn drop(&mut self) {
        // SAFETY: each SDK LocalAlloc result is adopted exactly once.
        unsafe { LocalFree(self.0) };
    }
}

fn security(file: &fs::File) -> Local {
    let mut descriptor = null_mut();
    // SAFETY: live READ_CONTROL handle and valid descriptor output pointer.
    assert_eq!(
        unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                &mut descriptor,
            )
        },
        0,
        "fixture security query failed"
    );
    Local(descriptor)
}

fn assert_owner_only(file: &fs::File, expected_owner: PSID) {
    use windows_sys::Win32::Security::GetSecurityDescriptorDacl;
    let descriptor = security(file);
    let mut owner = null_mut();
    let mut dacl = null_mut();
    let (mut defaulted, mut present, mut control, mut revision) = (0, 0, 0, 0);
    // SAFETY: SDK-owned descriptor and initialized outputs; all borrowed pointers
    // remain live until after the ACL/SID assertions.
    unsafe {
        assert_ne!(
            GetSecurityDescriptorOwner(descriptor.0, &mut owner, &mut defaulted),
            0
        );
        assert!(!owner.is_null());
        assert_ne!(EqualSid(owner, expected_owner), 0, "fixture owner changed");
        assert_ne!(
            GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision),
            0
        );
        assert_ne!(
            control & SE_DACL_PROTECTED,
            0,
            "fixture DACL is not protected"
        );
        assert_ne!(
            GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut dacl, &mut defaulted),
            0
        );
        assert_ne!(present, 0);
        assert!(!dacl.is_null());
        assert_eq!((*dacl).AceCount, 1, "fixture must grant only its owner");
        let mut ace = null_mut();
        assert_ne!(GetAce(dacl, 0, &mut ace), 0);
        let allowed = &*ace.cast::<ACCESS_ALLOWED_ACE>();
        assert_eq!(allowed.Header.AceType as u32, ACCESS_ALLOWED_ACE_TYPE);
        assert_eq!(allowed.Header.AceFlags as u32 & INHERIT_ONLY_ACE, 0);
        assert_eq!(allowed.Mask, FILE_ALL_ACCESS);
        assert_ne!(
            EqualSid(
                core::ptr::addr_of!(allowed.SidStart).cast_mut().cast(),
                owner
            ),
            0
        );
    }
}

fn directory(path: &Path) -> fs::File {
    fs::OpenOptions::new()
        .access_mode(READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .unwrap()
}

fn private_cargo(app: &Path) {
    // Accepted new creates this app with an explicit current-token owner and
    // protected owner-only DACL. Check that baseline before using its owner SID.
    let app_handle = directory(app);
    let app_security = security(&app_handle);
    let mut owner = null_mut();
    let mut defaulted = 0;
    // SAFETY: owner SID borrows the live app security descriptor.
    assert_ne!(
        unsafe { GetSecurityDescriptorOwner(app_security.0, &mut owner, &mut defaulted) },
        0
    );
    assert!(!owner.is_null());
    assert_owner_only(&app_handle, owner);
    let mut text = null_mut();
    // SAFETY: valid owner SID and LocalAlloc string output.
    assert_ne!(unsafe { ConvertSidToStringSidW(owner, &mut text) }, 0);
    let _text = Local(text.cast());
    let mut length = 0;
    // SAFETY: SDK returns a terminated SID string.
    while unsafe { *text.add(length) } != 0 {
        length += 1;
    }
    let sid = String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) }).unwrap();
    let sddl: Vec<_> = format!("O:{sid}D:P(A;OICI;FA;;;{sid})")
        .encode_utf16()
        .chain([0])
        .collect();
    let mut descriptor = null_mut();
    // SAFETY: terminated SDDL and LocalAlloc descriptor output.
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                null_mut(),
            )
        },
        0
    );
    let descriptor = Local(descriptor);
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let private = app.join(".yt088c-private-cargo");
    let path: Vec<_> = private.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: private app parent, create-new directory, explicit protected DACL
    // at creation. Never repair inherited permissions after publication.
    assert_ne!(
        unsafe { CreateDirectoryW(path.as_ptr(), &attributes) },
        0,
        "private Cargo directory creation failed: {}",
        io::Error::last_os_error()
    );
    let parent = directory(&private);
    assert_owner_only(&parent, owner);
    let destination = private.join("cargo.exe");
    let path: Vec<_> = destination.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: create-new file with explicit owner/DACL and non-inherited handle.
    let raw = unsafe {
        CreateFileW(
            path.as_ptr(),
            GENERIC_WRITE | READ_CONTROL,
            FILE_SHARE_READ,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };
    assert_ne!(
        raw,
        INVALID_HANDLE_VALUE,
        "private Cargo file creation failed: {}",
        io::Error::last_os_error()
    );
    // SAFETY: successful CreateFileW transfers the owned handle.
    let mut destination = unsafe { fs::File::from_raw_handle(raw) };
    assert_owner_only(&destination, owner);
    let cargo = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .filter(|directory| directory.is_absolute())
        .map(|directory| directory.join("cargo.exe"))
        .find(|candidate| candidate.is_file())
        .expect("native cargo proxy should be in absolute PATH");
    let mut source = fs::File::open(cargo).unwrap();
    let bytes = io::copy(&mut source, &mut destination).unwrap();
    assert_eq!(bytes, source.metadata().unwrap().len());
    assert_owner_only(&destination, owner);
    // All writer handles close before executable validation and CreateProcess.
}

fn cli(path: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
    command.args(["build", "--manifest-path"]).arg(path);
    // This integration owns its output directory; an outer native Runner may
    // use a shared CARGO_TARGET_DIR for compiling this test executable.
    if let Some(parent) = path.parent() {
        command.env("CARGO_TARGET_DIR", parent.join("target"));
    }
    // A positive native build must execute a vetted Cargo proxy. Windows
    // Runner user homes may grant group write access to the rustup proxy,
    // which the CLI correctly refuses; our fixture copies that proxy under
    // the app's protected DACL instead of weakening executable validation.
    if let Some(parent) = path.parent() {
        let private = parent.join(".yt088c-private-cargo");
        if private.join("cargo.exe").is_file() {
            let mut paths = vec![private];
            paths.extend(std::env::split_paths(
                &std::env::var_os("PATH").unwrap_or_default(),
            ));
            command.env("PATH", std::env::join_paths(paths).unwrap());
        }
    }
    command
}
fn code(output: Output, expected: i32) -> Output {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn fixture(root: &Path) -> PathBuf {
    // Reuse accepted new's explicit owner/protected DACL. The runner's temp
    // root may inherit group-writable grants and cannot itself be the app cwd.
    let framework = root.join("framework");
    fs::create_dir(&framework).unwrap();
    fs::write(
        framework.join("Cargo.toml"),
        "[workspace]\n[workspace.package]\nrust-version='1.99'\n",
    )
    .unwrap();
    for name in ["yetlet-core", "yetlet-server-axum"] {
        let dir = framework.join("crates").join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname='{name}'\n"),
        )
        .unwrap();
        fs::write(dir.join("src/lib.rs"), "// fixture only\n").unwrap();
    }
    yetlet_cli::scaffold::create("app", &framework, root).unwrap();
    let path = root.join("app");
    fs::write(
        path.join("Cargo.toml"),
        "[package]\nname='yt088c_fixture'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    fs::write(path.join("src/main.rs"), "fn main() {}\n").unwrap();
    private_cargo(&path);
    path
}

#[test]
fn native_cli_build_debug_release_and_lock_cache_errors_are_frozen() {
    // MinGW's GNU linker can fail to open a Unicode executable output path;
    // keep the GNU *link output* ASCII while the Windows scaffold/path tests
    // independently retain Unicode, spaces and reparse-path coverage.
    // MSVC continues this build journey inside the original Unicode path.
    #[cfg(target_env = "gnu")]
    let prefix = "yt088c-build-";
    #[cfg(not(target_env = "gnu"))]
    let prefix = "yt088c 空 間;$literal-😀";
    let root = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let app = fixture(root.path());
    let manifest = app.join("Cargo.toml");
    let lock = app.join("Cargo.lock");
    code(cli(&manifest).output().unwrap(), 1);
    assert!(!lock.exists());
    // Explicit fixture lock. The CLI never runs a resolver/lock generator.
    fs::write(
        &lock,
        "version=4\n[[package]]\nname='yt088c_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    let initial = fs::read(&lock).unwrap();
    code(cli(&manifest).output().unwrap(), 0);
    code(cli(&manifest).arg("--release").output().unwrap(), 0);
    assert!(app.join("target/debug/yt088c_fixture.exe").is_file());
    assert!(app.join("target/release/yt088c_fixture.exe").is_file());
    assert_eq!(fs::read(&lock).unwrap(), initial);
    // This local dependency needs no download/cache; only a lock update is absent.
    fs::create_dir_all(app.join("dependency/src")).unwrap();
    fs::write(
        app.join("dependency/Cargo.toml"),
        "[package]\nname='local_dep'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::write(app.join("dependency/src/lib.rs"), "// local\n").unwrap();
    let original = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        format!("{original}\n[dependencies]\nlocal_dep={{path='dependency'}}\n"),
    )
    .unwrap();
    let output = code(cli(&manifest).output().unwrap(), 101);
    assert!(String::from_utf8_lossy(&output.stderr).contains("lock"));
    assert_eq!(fs::read(&lock).unwrap(), initial);
    // Separate explicit offline fixture setup, never CLI bootstrap. The pinned
    // serde_json source is already required by this isolated CLI workspace.
    fs::write(
        &manifest,
        format!("{original}\n[dependencies]\nserde_json='=1.0.149'\n"),
    )
    .unwrap();
    code(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(&manifest)
            .current_dir(&app)
            .env("RUSTUP_AUTO_INSTALL", "0")
            .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .unwrap(),
        0,
    );
    let cached_lock = fs::read(&lock).unwrap();
    let cache = tempfile::tempdir().unwrap();
    let output = code(
        cli(&manifest)
            .env("CARGO_HOME", cache.path())
            .output()
            .unwrap(),
        101,
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("offline"));
    assert_eq!(fs::read(&lock).unwrap(), cached_lock);
    assert!(!cache.path().join("registry/cache").exists());
}

#[test]
fn preflight_and_executable_selection_never_launch_app_local_cargo() {
    let root = tempfile::tempdir().unwrap();
    let app = fixture(root.path());
    let manifest = app.join("Cargo.toml");
    let empty = app.join("empty-path");
    fs::create_dir(&empty).unwrap();
    for path in [
        &manifest,
        &app.join("missing/Cargo.toml"),
        &app.join("../Cargo.toml"),
        app.as_path(),
    ] {
        code(
            cli(path)
                .env("PATH", &empty)
                .env("CARGO_HOME", app.join("no-cache"))
                .output()
                .unwrap(),
            1,
        );
    }
    assert!(!app.join("Cargo.lock").exists());
    fs::write(
        app.join("Cargo.lock"),
        "version=4\n[[package]]\nname='yt088c_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    // A runnable app-local executable must never be Windows' implicit first hit.
    fs::copy(std::env::current_exe().unwrap(), app.join("cargo.exe")).unwrap();
    code(
        cli(&manifest)
            .env("PATH", &empty)
            .env("CARGO_HOME", app.join("no-cache"))
            .output()
            .unwrap(),
        125,
    );
    assert!(!app.join("no-cache").exists());
    assert!(!app.join("target").exists());
}

// YT-088d: reuse the accepted protected Cargo/app fixture without changing any
// build/new/ACL assertion. Each console journey runs in a fresh owned console.
fn dev_cli(manifest: &Path) -> Command {
    let build = cli(manifest);
    let mut dev = Command::new(build.get_program());
    dev.args(["dev", "--manifest-path"]).arg(manifest);
    dev.envs(build.get_envs().map(|(key, value)| (key, value.unwrap())));
    dev
}

use std::{
    os::windows::{io::OwnedHandle, process::CommandExt},
    sync::atomic::{AtomicU32, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::WAIT_OBJECT_0,
    System::{Console::*, JobObjects::*, Threading::*},
};

struct ConsoleJob(OwnedHandle);
impl ConsoleJob {
    fn new() -> Self {
        let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        assert!(!raw.is_null(), "cannot create fixture Job");
        let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        assert_ne!(
            unsafe {
                SetInformationJobObject(
                    job.0.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            },
            0
        );
        job
    }
    fn assign(&self, child: &mut std::process::Child) {
        if unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), child.as_raw_handle()) } == 0 {
            let error = io::Error::last_os_error();
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("cannot own console fixture (nested Job): {error}");
        }
    }
    fn reap(&self) {
        assert_ne!(
            unsafe { TerminateJobObject(self.0.as_raw_handle(), 125) },
            0
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
            assert_ne!(
                unsafe {
                    QueryInformationJobObject(
                        self.0.as_raw_handle(),
                        JobObjectBasicAccountingInformation,
                        (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                        size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                        null_mut(),
                    )
                },
                0
            );
            if info.ActiveProcesses == 0 {
                break;
            }
            assert!(Instant::now() < deadline, "fixture Job not empty");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn wait_marker(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !path.is_file() {
        assert!(
            Instant::now() < deadline,
            "missing native fixture marker: {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn wait_child(child: &mut std::process::Child) -> std::process::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "native console child timed out; proof incomplete"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn console_fixture(case: &str) {
    // ASCII GNU link output, as in the accepted 088c build integration. MSVC
    // preserves its Unicode path coverage and both use a real native console.
    #[cfg(target_env = "gnu")]
    let prefix = "yt088d-console-";
    #[cfg(not(target_env = "gnu"))]
    let prefix = "yt088d-console 雪 😀-";
    let root = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let job = ConsoleJob::new();
    let mut helper = Command::new(std::env::current_exe().unwrap());
    helper
        .args(["--exact", "native_dev_console_fixture", "--nocapture"])
        .env("YT088D_CASE", case)
        .env("YT088D_ROOT", root.path())
        .stdout(fs::File::create(root.path().join("console.stdout")).unwrap())
        .stderr(fs::File::create(root.path().join("console.stderr")).unwrap())
        .creation_flags(CREATE_NEW_CONSOLE);
    let mut helper = helper
        .spawn()
        .expect("true native console required; no skip permitted");
    job.assign(&mut helper);
    // The helper waits here before it can create any descendants.
    fs::write(root.path().join("go"), "go").unwrap();
    let status = wait_child(&mut helper);
    job.reap();
    assert!(
        status.success(),
        "console {case} failed {status}\n{}\n{}",
        fs::read_to_string(root.path().join("console.stdout")).unwrap(),
        fs::read_to_string(root.path().join("console.stderr")).unwrap()
    );
}

#[test]
fn native_cli_dev_real_cargo_exit_lock_and_offline_policy() {
    console_fixture("frozen");
}
#[test]
fn native_cli_dev_console_ctrl_c_break_resistant_tree_and_isolation() {
    for case in ["ctrl-c", "ctrl-break"] {
        console_fixture(case);
    }
}

static FIXTURE_EVENT: AtomicU32 = AtomicU32::new(0);
unsafe extern "system" fn fixture_control(event: u32) -> i32 {
    FIXTURE_EVENT.store(event + 1, Ordering::SeqCst);
    1
}

const DEV_APP: &str = r#"
use std::{sync::atomic::{AtomicU32, Ordering}, time::Duration};
static EVENT: AtomicU32 = AtomicU32::new(0);
#[link(name="kernel32")]
unsafe extern "system" { fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32)->i32>, add:i32)->i32; }
unsafe extern "system" fn control(event:u32)->i32 { EVENT.store(event+1,Ordering::SeqCst); 1 }
fn main() {
    assert_eq!(std::env::var("CARGO_NET_OFFLINE").unwrap(),"true");
    assert_eq!(std::env::var("RUSTUP_AUTO_INSTALL").unwrap(),"0");
    assert_eq!(std::env::var("RUSTUP_SKIP_UPDATE_CHECK").unwrap(),"1");
    assert!(std::path::Path::new("Cargo.toml").is_file());
    assert!(std::path::Path::new("Cargo.lock").is_file());
    if std::env::var("YT088D_BEHAVIOR").unwrap()=="exit" { std::process::exit(23); }
    assert_ne!(unsafe { SetConsoleCtrlHandler(Some(control),1) },0);
    let depth:usize=std::env::args().nth(1).unwrap_or("0".into()).parse().unwrap();
    let _child=if depth<2 { Some(std::process::Command::new(std::env::current_exe().unwrap()).arg((depth+1).to_string()).spawn().unwrap()) } else { None };
    std::fs::write(format!("pid-{depth}"),std::process::id().to_string()).unwrap();
    loop {
        if EVENT.load(Ordering::SeqCst)!=0 { std::fs::write(format!("break-{depth}"),"observed").unwrap(); }
        std::thread::sleep(Duration::from_millis(10));
    }
}
"#;

fn dev_frozen(app: &Path) {
    let manifest = app.join("Cargo.toml");
    let lock = app.join("Cargo.lock");
    fs::write(app.join("src/main.rs"), DEV_APP).unwrap();
    code(
        dev_cli(&manifest)
            .env("YT088D_BEHAVIOR", "exit")
            .output()
            .unwrap(),
        1,
    );
    assert!(!lock.exists());
    assert!(!app.join("target").exists());
    fs::write(
        &lock,
        "version=4\n[[package]]\nname='yt088c_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    let initial = fs::read(&lock).unwrap();
    code(
        dev_cli(&manifest)
            .env("YT088D_BEHAVIOR", "exit")
            .output()
            .unwrap(),
        23,
    );
    assert_eq!(fs::read(&lock).unwrap(), initial);
    let original = fs::read_to_string(&manifest).unwrap();
    fs::create_dir_all(app.join("dependency/src")).unwrap();
    fs::write(
        app.join("dependency/Cargo.toml"),
        "[package]\nname='local_dep'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::write(app.join("dependency/src/lib.rs"), "// local\n").unwrap();
    fs::write(
        &manifest,
        format!("{original}\n[dependencies]\nlocal_dep={{path='dependency'}}\n"),
    )
    .unwrap();
    let output = code(dev_cli(&manifest).output().unwrap(), 101);
    assert!(String::from_utf8_lossy(&output.stderr).contains("lock"));
    assert_eq!(fs::read(&lock).unwrap(), initial);
    fs::write(
        &manifest,
        format!("{original}\n[dependencies]\nserde_json='=1.0.149'\n"),
    )
    .unwrap();
    code(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(&manifest)
            .env("RUSTUP_AUTO_INSTALL", "0")
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .unwrap(),
        0,
    );
    let cached_lock = fs::read(&lock).unwrap();
    let cache = tempfile::tempdir().unwrap();
    let output = code(
        dev_cli(&manifest)
            .env("CARGO_HOME", cache.path())
            .output()
            .unwrap(),
        101,
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("offline"));
    assert_eq!(fs::read(&lock).unwrap(), cached_lock);
    assert!(!cache.path().join("registry/cache").exists());
    // Restore this fixture's original manifest/lock before the headless negative.
    fs::write(&manifest, original).unwrap();
    fs::write(&lock, initial).unwrap();
    let output = code(
        dev_cli(&manifest)
            .creation_flags(DETACHED_PROCESS)
            .output()
            .unwrap(),
        125,
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires an attached console"));
}

fn dev_cancel(app: &Path, case: &str) {
    let manifest = app.join("Cargo.toml");
    fs::write(app.join("src/main.rs"), DEV_APP).unwrap();
    fs::write(
        app.join("Cargo.lock"),
        "version=4\n[[package]]\nname='yt088c_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    let lock = fs::read(app.join("Cargo.lock")).unwrap();
    let sentinel_root = app.join("sentinel");
    fs::create_dir(&sentinel_root).unwrap();
    let mut sentinel = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "native_dev_console_fixture", "--nocapture"])
        .env("YT088D_CASE", "sentinel")
        .env("YT088D_ROOT", &sentinel_root)
        .creation_flags(CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .unwrap();
    wait_marker(&sentinel_root.join("ready"));
    let mut command = dev_cli(&manifest);
    command.env("YT088D_BEHAVIOR", "resist");
    if case == "ctrl-break" {
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
    let mut child = command.spawn().unwrap();
    let mut owned = Vec::new();
    for depth in 0..3 {
        let path = app.join(format!("pid-{depth}"));
        wait_marker(&path);
        let pid = fs::read_to_string(path).unwrap().parse().unwrap();
        let raw = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                pid,
            )
        };
        assert!(!raw.is_null(), "cannot pin owned app descendant");
        owned.push(unsafe { OwnedHandle::from_raw_handle(raw) });
    }
    // Only this test's CREATE_NEW_CONSOLE contains the C broadcast. Product
    // code never broadcasts C; BREAK is targeted to the CLI's own group.
    let start = Instant::now();
    let (event, group) = if case == "ctrl-c" {
        (CTRL_C_EVENT, 0)
    } else {
        (CTRL_BREAK_EVENT, child.id())
    };
    assert_ne!(unsafe { GenerateConsoleCtrlEvent(event, group) }, 0);
    let status = wait_child(&mut child);
    assert_eq!(status.code().unwrap() as u32, 0xc000013a);
    assert!(
        start.elapsed() < Duration::from_secs(12),
        "cleanup exceeded graceful/hard bounds"
    );
    for (depth, process) in owned.iter().enumerate() {
        assert_eq!(
            unsafe { WaitForSingleObject(process.as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
        assert!(
            app.join(format!("break-{depth}")).is_file(),
            "resistant descendant did not observe real BREAK"
        );
    }
    assert!(
        sentinel.try_wait().unwrap().is_none(),
        "unrelated group terminated"
    );
    assert!(
        !sentinel_root.join("event").exists(),
        "unrelated group received a forwarded event"
    );
    assert_eq!(fs::read(app.join("Cargo.lock")).unwrap(), lock);
    sentinel.kill().unwrap();
    sentinel.wait().unwrap();
}

#[test]
fn native_dev_console_fixture() {
    let Ok(case) = std::env::var("YT088D_CASE") else {
        return;
    };
    let root = PathBuf::from(std::env::var_os("YT088D_ROOT").unwrap());
    if case == "sentinel" {
        assert_ne!(
            unsafe { SetConsoleCtrlHandler(Some(fixture_control), 1) },
            0
        );
        fs::write(root.join("ready"), "ready").unwrap();
        loop {
            if FIXTURE_EVENT.load(Ordering::SeqCst) != 0 {
                fs::write(root.join("event"), "event").unwrap();
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    wait_marker(&root.join("go"));
    let mut pid = 0;
    assert!(
        unsafe { GetConsoleProcessList(&mut pid, 1) } > 0,
        "no true console; test incomplete"
    );
    assert_ne!(
        unsafe { SetConsoleCtrlHandler(Some(fixture_control), 1) },
        0
    );
    let app = fixture(&root);
    match case.as_str() {
        "frozen" => dev_frozen(&app),
        "ctrl-c" | "ctrl-break" => dev_cancel(&app, &case),
        _ => panic!("unknown console fixture"),
    }
    assert_ne!(
        unsafe { SetConsoleCtrlHandler(Some(fixture_control), 0) },
        0
    );
}
