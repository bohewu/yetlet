use super::*;
use std::{fs, os::windows::fs::OpenOptionsExt};
use windows_sys::{
    Wdk::Storage::FileSystem::{REPARSE_DATA_BUFFER, REPARSE_DATA_BUFFER_0_1},
    Win32::{
        Foundation::GENERIC_WRITE,
        Security::{
            Authorization::{GetSecurityInfo, SE_FILE_OBJECT},
            DACL_SECURITY_INFORMATION, GetSecurityDescriptorControl, SE_DACL_PROTECTED,
        },
        System::{
            IO::DeviceIoControl, Ioctl::FSCTL_SET_REPARSE_POINT,
            SystemServices::IO_REPARSE_TAG_MOUNT_POINT,
        },
    },
};

fn entries() -> Vec<(&'static str, String)> {
    vec![
        ("README.md", "UTF-8: 雪 😀\n".into()),
        ("src/main.rs", "fn main() {}\n".into()),
    ]
}
fn names(path: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(path)
        .unwrap()
        .map(|p| p.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}
// A real mount-point reparse object; no shell, admin symlink privilege or skipped test.
fn junction(path: &Path, target: &Path) {
    fs::create_dir(path).unwrap();
    let target = format!("\\??\\{}", target.display());
    let substitute: Vec<_> = target.encode_utf16().chain([0]).collect();
    let path_offset = offset_of!(REPARSE_DATA_BUFFER, Anonymous)
        + offset_of!(REPARSE_DATA_BUFFER_0_1, PathBuffer);
    let length = path_offset + substitute.len() * 2 + 2; // empty terminated print name
    let mut data = vec![
        0_usize;
        length
            .max(size_of::<REPARSE_DATA_BUFFER>())
            .div_ceil(size_of::<usize>())
    ];
    let data_ptr = data.as_mut_ptr().cast::<REPARSE_DATA_BUFFER>();
    // SAFETY: aligned SDK struct plus allocated tail, all offsets/lengths in bytes.
    unsafe {
        (*data_ptr).ReparseTag = IO_REPARSE_TAG_MOUNT_POINT;
        (*data_ptr).ReparseDataLength = (length - 8) as u16;
        let mount = &mut (*data_ptr).Anonymous.MountPointReparseBuffer;
        mount.SubstituteNameLength = ((substitute.len() - 1) * 2) as u16;
        mount.PrintNameOffset = (substitute.len() * 2) as u16;
        std::ptr::copy_nonoverlapping(
            substitute.as_ptr(),
            mount.PathBuffer.as_mut_ptr(),
            substitute.len(),
        );
    }
    let file = fs::OpenOptions::new()
        .access_mode(GENERIC_WRITE)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .unwrap();
    let mut returned = 0;
    // SAFETY: real directory handle and correctly sized reparse input buffer.
    win(unsafe {
        DeviceIoControl(
            handle(&file),
            FSCTL_SET_REPARSE_POINT,
            data_ptr.cast(),
            length as u32,
            null_mut(),
            0,
            &mut returned,
            null_mut(),
        )
    })
    .unwrap();
}

fn frontend_entries() -> Vec<(&'static str, String)> {
    vec![
        ("base.toml", "[cli]\n".into()),
        ("src/main.rs", "owned source".into()),
        ("frontend/package.json", "owned frontend".into()),
        ("frontend/.env.example", "VITE_PUBLIC_VALUE=example".into()),
    ]
}

#[test]
fn frontend_and_src_are_one_no_replace_publication_with_owned_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let files = frontend_entries();
    create_project(root.path(), "app", &files).unwrap();
    for (path, bytes) in &files {
        assert_eq!(
            fs::read(root.path().join("app").join(path)).unwrap(),
            bytes.as_bytes()
        );
    }
    assert!(create_project(root.path(), "app", &files).is_err());
    assert!(
        create(root.path(), "rollback", &files, || Err(io::Error::other(
            "injected failure"
        )))
        .is_err()
    );
    let mut duplicate = files.clone();
    duplicate.push(("frontend/package.json", "replacement".into()));
    assert!(create_project(root.path(), "partial", &duplicate).is_err());
    assert_eq!(names(root.path()), ["app"]);
    assert!(
        create(root.path(), "race", &files, || {
            junction(&root.path().join("race"), &root.path().join("app"));
            Ok(())
        })
        .is_err()
    );
    assert_eq!(
        fs::read(root.path().join("race/frontend/package.json")).unwrap(),
        b"owned frontend"
    );
    assert_eq!(names(root.path()), ["app", "race"]);
    for path in [
        "frontend/../escape",
        "frontend/nested/app.ts",
        "frontend/unlisted",
        "other/package.json",
    ] {
        assert!(create_project(root.path(), "bad", &[(path, "bad".into())]).is_err());
        assert!(!root.path().join("bad").exists());
    }
    for files in [
        vec![
            ("src/main.rs", "".into()),
            ("cargo-overlay/Cargo.toml", "".into()),
        ],
        vec![
            ("frontend/package.json", "".into()),
            ("cargo-overlay/Cargo.toml", "".into()),
        ],
    ] {
        assert!(create_project(root.path(), "mixed", &files).is_err());
    }
}

#[test]
fn protected_frontend_root_is_project_relative_pinned_and_rejects_junctions() {
    use crate::platform::fs::{app_root, frontend_directory};
    let root = tempfile::tempdir().unwrap();
    create_project(root.path(), "app", &frontend_entries()).unwrap();
    let path = root.path().join("app");
    fs::create_dir_all(path.join("ui/client")).unwrap();
    junction(&path.join("linked"), &path.join("frontend"));
    junction(&path.join("ui/linked"), &path.join("frontend"));
    let app = open_directory(&path).unwrap();
    let frontend = frontend_directory(&app, Path::new("frontend/")).unwrap();
    assert_eq!(app_root(&frontend), path.join("frontend"));
    assert_eq!(
        read_app_file(&frontend, "package.json").unwrap(),
        b"owned frontend"
    );
    assert_eq!(
        app_root(&frontend_directory(&app, Path::new("ui/client")).unwrap()),
        path.join("ui/client")
    );
    assert!(read_app_file(&app, "frontend/package.json").is_err());
    assert!(app_directory_entries(&app, "frontend").is_err());
    for directory in [
        "",
        ".",
        "..",
        "../frontend",
        "ui/../frontend",
        "C:/frontend",
        "C:frontend",
        "\\\\server\\share",
        "linked",
        "ui/linked",
        "missing",
        "ui/client:stream",
    ] {
        assert!(
            frontend_directory(&app, Path::new(directory)).is_err(),
            "{directory}"
        );
    }
    assert!(fs::rename(path.join("frontend"), path.join("moved")).is_err());
    frontend.verify().unwrap();
}

#[test]
fn explicit_public_settings_root_retains_non_reparse_reads_without_manifest_or_lock() {
    let root = tempfile::tempdir().unwrap();
    create_project(root.path(), "settings", &[("base.toml", "[cli]\n".into())]).unwrap();
    let path = root.path().join("settings");
    let settings = open_directory(&path).unwrap();
    assert_eq!(read_app_file(&settings, "base.toml").unwrap(), b"[cli]\n");
    assert!(open_directory(Path::new("relative")).is_err());
    assert!(open_directory(&path.join("..")).is_err());
    assert!(read_app_file(&settings, "private.toml").is_err());
    junction(&path.join("unsafe.toml"), &path);
    assert!(read_app_file(&settings, "unsafe.toml").is_err());
    fs::write(path.join("base.toml"), vec![0; 1024 * 1024 + 1]).unwrap();
    assert!(
        read_app_file(&settings, "base.toml")
            .unwrap_err()
            .contains("size limit")
    );
}

#[test]
fn unicode_bytes_and_private_stage_publish_on_ntfs() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("space 雪 😀");
    fs::create_dir(&parent).unwrap();
    create_project(&parent, "demo", &entries()).unwrap();
    assert_eq!(
        fs::read(parent.join("demo/README.md")).unwrap(),
        "UTF-8: 雪 😀\n".as_bytes()
    );
    assert_eq!(names(&parent), ["demo"]);
    let anchor = Anchor::open(&parent.join("demo")).unwrap();
    anchor.verify().unwrap();
    let acl_file = fs::OpenOptions::new()
        .access_mode(READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(parent.join("demo"))
        .unwrap();
    let mut sd = null_mut();
    // SAFETY: live directory handle and valid output pointer; result is LocalAlloc-owned.
    assert_eq!(
        unsafe {
            GetSecurityInfo(
                handle(&acl_file),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                &mut sd,
            )
        },
        0
    );
    let _owner = Local(sd);
    let mut control = 0;
    let mut revision = 0;
    // SAFETY: returned initialized security descriptor and matching outputs.
    win(unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision) }).unwrap();
    assert_ne!(control & SE_DACL_PROTECTED, 0);
}

#[test]
fn docker_overlay_unicode_output_rejects_junction_races_and_foreign_writable_parent() {
    let entries = vec![
        ("Dockerfile", "owned configuration\n".into()),
        ("cargo-overlay/Cargo.toml", "owned overlay\n".into()),
    ];
    let root = tempfile::tempdir().unwrap();
    // Construct the fixture parent atomically with the native owner-only DACL.
    // Inherited TempDir ACLs can be group-writable, which the generator must reject.
    let anchor = Anchor::open(root.path()).unwrap();
    let sd = private_security().unwrap();
    let owned = relative(
        anchor.directory(),
        "private-test-output".as_ref(),
        true,
        true,
        Some(&sd),
    )
    .unwrap();
    drop(owned);
    drop(anchor);
    let parent = root.path().join("private-test-output");
    // The NtCreateFile traversal handle intentionally lacks READ_CONTROL.
    // Reopen with the adapter's security access before checking the DACL.
    let parent_anchor = Anchor::open(&parent).unwrap();
    let security = parent_anchor.security_directory().unwrap();
    protected(&security, true).unwrap();
    drop(security);
    drop(parent_anchor);
    create_project(parent.as_path(), "container space 雪", &entries).unwrap();
    assert_eq!(
        fs::read(
            parent
                .as_path()
                .join("container space 雪/cargo-overlay/Cargo.toml")
        )
        .unwrap(),
        b"owned overlay\n"
    );
    assert!(create_project(parent.as_path(), "container space 雪", &entries).is_err());
    assert!(
        create(parent.as_path(), "race", &entries, || {
            fs::write(parent.as_path().join("race"), "foreign")
        })
        .is_err()
    );
    assert_eq!(fs::read(parent.as_path().join("race")).unwrap(), b"foreign");
    junction(
        &parent.as_path().join("alias"),
        &parent.as_path().join("container space 雪"),
    );
    assert!(create_project(&parent.as_path().join("alias"), "out", &entries).is_err());
    let anchor = Anchor::open(parent.as_path()).unwrap();
    let private = private_security().unwrap();
    let hostile = foreign_writable_security(&private);
    let directory = relative(
        anchor.directory(),
        "unsafe".as_ref(),
        true,
        true,
        Some(&hostile),
    )
    .unwrap();
    drop(directory);
    drop(anchor);
    assert!(create_project(&parent.as_path().join("unsafe"), "out", &entries).is_err());
    assert!(names(&parent.as_path().join("unsafe")).is_empty());
}

#[test]
fn k8s_root_files_require_protected_parent_and_keep_no_replace_junction_and_rollback() {
    let entries = vec![
        ("deployment.yaml", "{}\n".into()),
        ("service.yaml", "{}\n".into()),
        ("README.md", "owned instructions\n".into()),
    ];
    let root = tempfile::tempdir().unwrap();
    let anchor = Anchor::open(root.path()).unwrap();
    let sd = private_security().unwrap();
    drop(
        relative(
            anchor.directory(),
            "private-k8s".as_ref(),
            true,
            true,
            Some(&sd),
        )
        .unwrap(),
    );
    let parent = root.path().join("private-k8s");
    create_project(&parent, "k8s space 雪", &entries).unwrap();
    assert_eq!(names(&parent.join("k8s space 雪")).len(), 3);
    assert!(create_project(&parent, "k8s space 雪", &entries).is_err());
    assert!(
        create(&parent, "race", &entries, || fs::write(
            parent.join("race"),
            "foreign"
        ))
        .is_err()
    );
    assert_eq!(fs::read(parent.join("race")).unwrap(), b"foreign");
    junction(&parent.join("alias"), &parent.join("k8s space 雪"));
    assert!(create_project(&parent.join("alias"), "out", &entries).is_err());
    let private_anchor = Anchor::open(&parent).unwrap();
    let hostile = foreign_writable_security(&sd);
    drop(
        relative(
            private_anchor.directory(),
            "unsafe".as_ref(),
            true,
            true,
            Some(&hostile),
        )
        .unwrap(),
    );
    assert!(create_project(&parent.join("unsafe"), "out", &entries).is_err());
    assert!(names(&parent.join("unsafe")).is_empty());
    assert!(
        create(&parent, "rollback", &entries, || Err(io::Error::other(
            "injected failure"
        )))
        .is_err()
    );
    assert!(!parent.join("rollback").exists());
    assert!(
        names(&parent)
            .iter()
            .all(|name| !name.starts_with(".yetlet-"))
    );
}

#[test]
fn compose_root_files_keep_owner_acl_junction_race_and_rollback_guards() {
    let entries = vec![
        ("compose.yaml", "owned YAML\n".into()),
        ("README.md", "owned README\n".into()),
    ];
    let root = tempfile::tempdir().unwrap();
    let anchor = Anchor::open(root.path()).unwrap();
    let sd = private_security().unwrap();
    drop(
        relative(
            anchor.directory(),
            "private-output".as_ref(),
            true,
            true,
            Some(&sd),
        )
        .unwrap(),
    );
    let parent = root.path().join("private-output");
    create_project(&parent, "compose space 雪", &entries).unwrap();
    assert_eq!(
        fs::read(parent.join("compose space 雪/compose.yaml")).unwrap(),
        b"owned YAML\n"
    );
    assert_eq!(names(&parent.join("compose space 雪")).len(), 2);
    assert!(create_project(&parent, "compose space 雪", &entries).is_err());
    assert!(
        create(&parent, "race", &entries, || fs::write(
            parent.join("race"),
            "foreign"
        ))
        .is_err()
    );
    assert_eq!(fs::read(parent.join("race")).unwrap(), b"foreign");
    junction(&parent.join("alias"), &parent.join("compose space 雪"));
    assert!(create_project(&parent.join("alias"), "out", &entries).is_err());
    let private_anchor = Anchor::open(&parent).unwrap();
    let hostile = foreign_writable_security(&sd);
    drop(
        relative(
            private_anchor.directory(),
            "unsafe".as_ref(),
            true,
            true,
            Some(&hostile),
        )
        .unwrap(),
    );
    assert!(create_project(&parent.join("unsafe"), "out", &entries).is_err());
    assert!(names(&parent.join("unsafe")).is_empty());
    assert!(
        create(&parent, "rollback", &entries, || Err(io::Error::other(
            "injected failure"
        )))
        .is_err()
    );
    assert!(!parent.join("rollback").exists());
    assert!(
        names(&parent)
            .iter()
            .all(|name| !name.starts_with(".yetlet-"))
    );
    for path in [
        "../escape",
        "tree/compose.yaml",
        "tree/../escape",
        "tree\\compose.yaml",
    ] {
        assert!(create_project(&parent, "bad", &[(path, "bad".into())]).is_err());
        assert!(!parent.join("bad").exists());
    }
}

#[test]
fn existing_directory_file_and_locked_target_are_never_replaced() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("directory")).unwrap();
    fs::write(root.path().join("directory/sentinel"), "foreign").unwrap();
    fs::write(root.path().join("file"), "foreign bytes").unwrap();
    fs::write(root.path().join("locked"), "locked bytes").unwrap();
    let locked = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(root.path().join("locked"))
        .unwrap();
    for name in ["directory", "file", "locked"] {
        assert!(create_project(root.path(), name, &entries()).is_err());
    }
    drop(locked);
    assert_eq!(
        fs::read(root.path().join("directory/sentinel")).unwrap(),
        b"foreign"
    );
    assert_eq!(
        fs::read(root.path().join("file")).unwrap(),
        b"foreign bytes"
    );
    assert_eq!(
        fs::read(root.path().join("locked")).unwrap(),
        b"locked bytes"
    );
    assert_eq!(names(root.path()), ["directory", "file", "locked"]);
}

#[test]
fn partial_write_failure_rolls_back_only_owned_stage() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("sentinel"), "foreign").unwrap();
    let mut files = entries();
    files.push(("../escape", "must not escape".into()));
    assert!(create_project(root.path(), "demo", &files).is_err());
    assert_eq!(names(root.path()), ["sentinel"]);
    assert_eq!(fs::read(root.path().join("sentinel")).unwrap(), b"foreign");
}

#[test]
fn late_destination_wins_native_nonreplacing_rename() {
    for directory in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("demo");
        let result = create(root.path(), "demo", &entries(), || {
            if directory {
                fs::create_dir(&target)?;
                fs::write(target.join("sentinel"), "late")?;
            } else {
                fs::write(&target, "late")?;
            }
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(names(root.path()), ["demo"]);
        assert_eq!(
            fs::read(if directory {
                target.join("sentinel")
            } else {
                target
            })
            .unwrap(),
            b"late"
        );
    }
}

#[test]
fn junction_parent_framework_file_and_destination_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("sentinel"), "outside").unwrap();
    let alias = root.path().join("alias");
    junction(&alias, outside.path());
    assert!(create_project(&alias, "demo", &entries()).is_err());
    assert!(read_framework(&alias.join("sentinel")).is_err());
    assert!(create_project(root.path(), "alias", &entries()).is_err());
    // Also exercise a reparse FILE selection where a regular file is expected.
    assert!(read_framework(&alias).is_err());
    assert_eq!(names(outside.path()), ["sentinel"]);
    assert_eq!(
        fs::read(outside.path().join("sentinel")).unwrap(),
        b"outside"
    );
    assert_eq!(names(root.path()), ["alias"]);
    fs::remove_dir(alias).unwrap();
}

#[test]
fn held_parent_chain_denies_delete_and_stage_substitution() {
    let root = tempfile::tempdir().unwrap();
    let outer = root.path().join("outer");
    let parent = outer.join("parent");
    fs::create_dir_all(&parent).unwrap();
    create(&parent, "demo", &entries(), || {
        assert!(fs::rename(&outer, root.path().join("moved")).is_err());
        assert!(fs::rename(&parent, outer.join("moved-parent")).is_err());
        let name = names(&parent).pop().unwrap();
        assert!(name.starts_with(".yetlet-"));
        assert!(fs::rename(parent.join(name), parent.join("foreign-stage")).is_err());
        let opener = fs::OpenOptions::new()
            .access_mode(DELETE)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(&parent);
        assert!(opener.is_err());
        Ok(())
    })
    .unwrap();
    assert_eq!(names(&parent), ["demo"]);
}

#[test]
fn preexisting_delete_sharer_refuses_before_staging() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("parent");
    fs::create_dir(&parent).unwrap();
    let opener = fs::OpenOptions::new()
        .access_mode(DELETE)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(&parent)
        .unwrap();
    assert!(create_project(&parent, "demo", &entries()).is_err());
    assert!(names(&parent).is_empty());
    drop(opener);
}

#[test]
fn rollback_retains_foreign_file_with_different_identity() {
    let root = tempfile::tempdir().unwrap();
    let parent = Anchor::open(root.path()).unwrap();
    let sd = private_security().unwrap();
    let mut stage = Stage::make(&parent, &sd).unwrap();
    stage
        .populate(&[("README.md", "original".into())], &sd)
        .unwrap();
    let path = root.path().join(&stage.root.as_ref().unwrap().name);
    stage.files[0].1.file.take();
    fs::rename(path.join("README.md"), path.join("saved-original")).unwrap();
    fs::write(path.join("README.md"), "foreign replacement").unwrap();
    assert!(stage.rollback().is_err());
    assert_eq!(
        fs::read(path.join("README.md")).unwrap(),
        b"foreign replacement"
    );
    assert_eq!(fs::read(path.join("saved-original")).unwrap(), b"original");
    drop(stage);
    drop(parent);
}

#[test]
fn ambiguous_windows_paths_and_device_components_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    for path in [
        root.path().join("../escape"),
        root.path().join("trailing."),
        root.path().join("trailing "),
        root.path().join("nul.txt"),
        root.path().join("stream:ads"),
    ] {
        assert!(create_project(&path, "demo", &entries()).is_err());
    }
    for path in [r"\\server\share", r"\\?\C:\", r"\\.\C:\"] {
        assert!(Anchor::open(Path::new(path)).is_err());
    }
    assert!(names(root.path()).is_empty());
}

#[test]
fn invalid_app_paths_refuse_before_child_start() {
    for (dev, release) in [(false, false), (false, true), (true, false)] {
        let failure =
            crate::app::run(Path::new("missing/../Cargo.toml"), dev, release).unwrap_err();
        assert_eq!(failure.code, 1);
        assert!(failure.message.contains("manifest path"));
    }
}

fn app_fixture(root: &Path) -> PathBuf {
    create_project(
        root,
        "app",
        &[
            (
                "Cargo.toml",
                "[package]\nname='fixture'\nversion='0.1.0'\n[workspace]\n".into(),
            ),
            (
                "Cargo.lock",
                "version=4\n[[package]]\nname='fixture'\nversion='0.1.0'\n".into(),
            ),
            ("src/main.rs", "fn main() {}\n".into()),
        ],
    )
    .unwrap();
    root.join("app/Cargo.toml")
}

#[test]
fn standalone_unicode_app_holds_cwd_manifest_and_lock_until_drop() {
    let root = tempfile::Builder::new()
        .prefix("yt088c 雪 😀 ")
        .tempdir()
        .unwrap();
    let manifest = app_fixture(root.path());
    let app = crate::platform::fs::open_app(&manifest).unwrap();
    app.verify().unwrap();
    assert!(fs::rename(root.path(), root.path().with_extension("moved")).is_err());
    assert!(fs::rename(manifest.parent().unwrap(), root.path().join("moved-app")).is_err());
    assert!(fs::rename(&manifest, manifest.with_extension("saved")).is_err());
    assert!(fs::write(&manifest, "foreign").is_err());
    assert!(fs::write(manifest.with_file_name("Cargo.lock"), "foreign").is_err());
    // No pinned handle prevents Cargo creating its ordinary output directory.
    fs::create_dir(manifest.parent().unwrap().join("target")).unwrap();
    drop(app);
    fs::rename(&manifest, manifest.with_extension("saved")).unwrap();
}

#[test]
fn app_preflight_rejects_missing_lock_reparse_files_and_missing_parent() {
    let root = tempfile::tempdir().unwrap();
    let manifest = app_fixture(root.path());
    let lock = manifest.with_file_name("Cargo.lock");
    fs::remove_file(&lock).unwrap();
    assert!(crate::platform::fs::open_app(&manifest).is_err());
    assert!(!lock.exists());
    let outside = tempfile::tempdir().unwrap();
    junction(&lock, outside.path());
    assert!(crate::platform::fs::open_app(&manifest).is_err());
    fs::remove_dir(&lock).unwrap();
    fs::write(
        &lock,
        "version=4\n[[package]]\nname='fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    let alias = root.path().join("alias");
    junction(&alias, manifest.parent().unwrap());
    assert!(crate::platform::fs::open_app(&alias.join("Cargo.toml")).is_err());
    fs::remove_dir(alias).unwrap();
    fs::remove_file(&manifest).unwrap();
    junction(&manifest, outside.path());
    assert!(crate::platform::fs::open_app(&manifest).is_err());
    fs::remove_dir(&manifest).unwrap();
    assert!(crate::platform::fs::open_app(&root.path().join("missing/Cargo.toml")).is_err());
    assert!(names(outside.path()).is_empty());
}

#[test]
fn windows_app_reuses_standalone_manifest_and_bounded_lock_parser() {
    let root = tempfile::tempdir().unwrap();
    let manifest = app_fixture(root.path());
    for text in [
        "[workspace]\n",
        "[package]\n",
        "[package]\nworkspace='..'\n[workspace]\n",
        "[package]\n[workspace]\nmembers=['other']\n",
        "invalid TOML",
    ] {
        fs::write(&manifest, text).unwrap();
        assert!(crate::platform::fs::open_app(&manifest).is_err());
    }
    fs::write(
        &manifest,
        "[package]\nname='fixture'\nversion='0.1.0'\n[workspace]\n",
    )
    .unwrap();
    for text in ["", "broken TOML", "version=4\n", "version=4\npackage=[]\n"] {
        fs::write(manifest.with_file_name("Cargo.lock"), text).unwrap();
        assert!(crate::platform::fs::open_app(&manifest).is_err());
    }
    fs::write(
        manifest.with_file_name("Cargo.lock"),
        vec![b' '; 1024 * 1024 + 1],
    )
    .unwrap();
    assert!(
        crate::platform::fs::open_app(&manifest)
            .err()
            .unwrap()
            .contains("size limit")
    );
}

fn foreign_writable_security(private: &Local) -> Local {
    use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
    let mut text = null_mut();
    let mut length = 0;
    // SAFETY: live descriptor; the SDK allocates a terminated SDDL string.
    win(unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            private.0,
            SDDL_REVISION_1,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut text,
            &mut length,
        )
    })
    .unwrap();
    let _text = Local(text.cast());
    // SAFETY: SDK-reported WCHAR buffer length; stop at its first terminator.
    let buffer = unsafe { std::slice::from_raw_parts(text, length as usize) };
    let end = buffer.iter().position(|unit| *unit == 0).unwrap();
    let sddl = String::from_utf16(&buffer[..end]).unwrap();
    // Keep the explicit current-user owner/full-control ACE. Add a real,
    // non-inheritable Everyone write grant only on the selected fixture object.
    let sddl: Vec<_> = format!("{sddl}(A;;FW;;;WD)")
        .encode_utf16()
        .chain([0])
        .collect();
    let mut sd = null_mut();
    // SAFETY: terminated SDDL; the resulting descriptor is LocalAlloc-owned.
    win(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut sd,
            null_mut(),
        )
    })
    .unwrap();
    Local(sd)
}

#[test]
fn writable_foreign_acl_fails_closed() {
    let source = tempfile::tempdir().unwrap();
    let manifest = app_fixture(source.path());
    assert!(crate::platform::fs::open_app(&manifest).is_ok());
    let files = [
        ("Cargo.toml", fs::read(&manifest).unwrap()),
        (
            "Cargo.lock",
            fs::read(manifest.with_file_name("Cargo.lock")).unwrap(),
        ),
    ];
    let private = private_security().unwrap();
    let hostile = foreign_writable_security(&private);
    for selected in ["Cargo.toml", "Cargo.lock", "app"] {
        let root = tempfile::tempdir().unwrap();
        let parent = Anchor::open(root.path()).unwrap();
        let security = |name| Some(if selected == name { &hostile } else { &private });
        // Set the DACL at FILE_CREATE. No WRITE_OWNER, post-create DACL mutation,
        // inherited child propagation or alternate-user privileges are needed.
        let directory = relative(
            parent.directory(),
            "app".as_ref(),
            true,
            true,
            security("app"),
        )
        .unwrap();
        for (name, bytes) in &files {
            let mut file =
                relative(&directory, name.as_ref(), false, true, security(name)).unwrap();
            file.write_all(bytes).unwrap();
        }
        drop(directory);
        drop(parent);
        let failure = crate::platform::fs::open_app(&root.path().join("app/Cargo.toml"))
            .err()
            .expect("a real foreign-writable DACL must be rejected");
        assert!(
            failure.contains("untrusted writable ACL"),
            "{selected}: {failure}"
        );
        assert!(
            failure.contains(if selected == "app" {
                "app cwd"
            } else {
                selected
            }),
            "{failure}"
        );
    }
}

#[test]
fn foreign_owner_is_rejected_even_if_privileged_writers_are_trusted() {
    // Do not change filesystem ownership or require administrator privileges.
    // Exercise the same native SID comparison used by protected(file, true).
    let current = private_security().unwrap();
    let sddl: Vec<_> = "O:SYD:P(A;;FA;;;SY)".encode_utf16().chain([0]).collect();
    let mut sd = null_mut();
    win(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut sd,
            null_mut(),
        )
    })
    .unwrap();
    let _sd = Local(sd);
    let mut owner = null_mut();
    let mut user = null_mut();
    let mut defaulted = 0;
    win(unsafe { GetSecurityDescriptorOwner(sd, &mut owner, &mut defaulted) }).unwrap();
    win(unsafe { GetSecurityDescriptorOwner(current.0, &mut user, &mut defaulted) }).unwrap();
    // A SYSTEM-runner would not have a different SID; use Everyone instead.
    let text = if unsafe { EqualSid(owner, user) } != 0 {
        "O:WDD:P(A;;FA;;;WD)"
    } else {
        "O:SYD:P(A;;FA;;;SY)"
    };
    let text: Vec<_> = text.encode_utf16().chain([0]).collect();
    let mut other = null_mut();
    win(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text.as_ptr(),
            SDDL_REVISION_1,
            &mut other,
            null_mut(),
        )
    })
    .unwrap();
    let _other = Local(other);
    win(unsafe { GetSecurityDescriptorOwner(other, &mut owner, &mut defaulted) }).unwrap();
    assert!(!trusted_sid(owner, user, true));
    assert!(trusted_sid(user, user, true));
}

#[test]
fn preexisting_delete_or_write_opener_refuses_app_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let manifest = app_fixture(root.path());
    for path in [manifest.clone(), manifest.with_file_name("Cargo.lock")] {
        for access in [DELETE, FILE_WRITE_DATA] {
            let opener = fs::OpenOptions::new()
                .access_mode(access)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                .open(&path)
                .unwrap();
            assert!(crate::platform::fs::open_app(&manifest).is_err());
            drop(opener);
        }
    }
}
