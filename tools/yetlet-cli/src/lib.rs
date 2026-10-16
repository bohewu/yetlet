//! Unpublished local CLI: literal arguments, diagnostics, SSR scaffolding and frozen Cargo.

pub mod app;
pub mod doctor;
mod framework;
mod platform;
pub mod scaffold;

use std::{ffi::OsString, path::PathBuf};

pub const HELP: &str = "Yetlet CLI preview (unpublished, Linux/Windows; YT-088)\n\nUsage:\n  yetlet [--help]\n  yetlet doctor [--framework-path PATH] [--json]\n  yetlet new NAME --framework-path PATH [--dir DIRECTORY]\n  yetlet build --manifest-path PATH [--release]\n  yetlet dev --manifest-path PATH\n\nnew creates DIRECTORY/NAME; DIRECTORY defaults to the current directory and must\nalready exist. Linux uses protected, symlink-safe paths; Windows requires protected local fixed NTFS paths without reparse traversal. No '..' or overwrite.\nNAME: 1-64 lowercase ASCII letters, digits, '-' or '_', starting with a letter;\nRust keywords and reserved device/Cargo names are rejected. Paths may contain\nspaces and Unicode. No shell expansion is performed.\n\nRequires Rust/Cargo 1.99 or newer. doctor only runs local --version probes;\nnew runs no subprocess. Neither command installs, builds or downloads anything.\nA new app has no Cargo.lock: explicitly generate its own lock offline using the\nlocal cache, then cargo check --frozen (see the generated README).\nbuild/dev require a standalone [package] + [workspace] Cargo.toml and its own\nexisting Cargo.lock. App cwd/manifest/lock must be protected against untrusted writes by native\npermissions/ACLs; Windows build/dev require an owner-protected cargo.exe.\nLinux and Windows are supported within local OS/console limits; other OSes fail closed.\nThey run one foreground cargo build/run --frozen, without network fallback.\ndev has no watcher/browser; Ctrl-C cleans its owned process group/Job.\nWindows dev requires an attached console with processed Ctrl-C input. Applications\nmust retain the inherited process group; daemonization is unsupported.\nExit: 0 success/help, 1 preflight, 2 usage, 125 spawn/cleanup failure; otherwise\nthe native Cargo exit or signal is preserved, including cancellation.\nPublication and global installation remain outside this preview.\n";

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Help,
    Doctor {
        framework: Option<PathBuf>,
        json: bool,
    },
    New {
        name: String,
        framework: PathBuf,
        dir: PathBuf,
    },
    Build {
        manifest: PathBuf,
        release: bool,
    },
    Dev {
        manifest: PathBuf,
    },
}

pub fn parse(args: &[OsString]) -> Result<Cli, &'static str> {
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        return Ok(Cli::Help);
    }
    let command = args[0].to_str().ok_or("command must be UTF-8")?;
    if matches!(command, "build" | "dev") {
        let mut manifest = None;
        let mut release = false;
        let mut index = 1;
        while index < args.len() {
            match args[index].to_str() {
                Some("--manifest-path") if manifest.is_none() => {
                    index += 1;
                    manifest = Some(value(args, index)?);
                }
                Some("--release") if command == "build" && !release => release = true,
                _ => return Err("unknown, duplicate or misplaced option"),
            }
            index += 1;
        }
        let manifest = manifest.ok_or("build/dev require --manifest-path PATH")?;
        return Ok(if command == "build" {
            Cli::Build { manifest, release }
        } else {
            Cli::Dev { manifest }
        });
    }
    let mut framework = None;
    let mut dir = None;
    let mut json = false;
    let mut index = 1;
    let name = if command == "new" {
        let name = args
            .get(index)
            .and_then(|v| v.to_str())
            .ok_or("new requires NAME")?;
        scaffold::validate_name(name)?;
        index += 1;
        Some(name.to_owned())
    } else if command == "doctor" {
        None
    } else {
        return Err("unknown command (see --help)");
    };
    while index < args.len() {
        match args[index].to_str() {
            Some("--framework-path") if framework.is_none() => {
                index += 1;
                framework = Some(value(args, index)?);
            }
            Some("--dir") if command == "new" && dir.is_none() => {
                index += 1;
                dir = Some(value(args, index)?);
            }
            Some("--json") if command == "doctor" && !json => json = true,
            _ => return Err("unknown, duplicate or misplaced option"),
        }
        index += 1;
    }
    match name {
        Some(name) => Ok(Cli::New {
            name,
            framework: framework.ok_or("new requires --framework-path PATH")?,
            dir: dir.unwrap_or_else(|| PathBuf::from(".")),
        }),
        None => Ok(Cli::Doctor { framework, json }),
    }
}

fn value(args: &[OsString], index: usize) -> Result<PathBuf, &'static str> {
    let value = args.get(index).ok_or("option requires a path")?;
    if value.is_empty() || value.to_string_lossy().starts_with('-') {
        return Err("option requires a nonempty path (use ./ for a leading '-')");
    }
    Ok(value.into())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn argv(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(OsString::from).collect()
    }

    #[test]
    fn commands_and_options_are_literal_and_fail_closed() {
        assert_eq!(parse(&argv(&[])), Ok(Cli::Help));
        assert_eq!(parse(&argv(&["--help"])), Ok(Cli::Help));
        assert_eq!(
            parse(&argv(&[
                "doctor",
                "--framework-path",
                "/tmp/framework",
                "--json"
            ])),
            Ok(Cli::Doctor {
                framework: Some("/tmp/framework".into()),
                json: true
            })
        );
        assert_eq!(
            parse(&argv(&[
                "new",
                "sample_app",
                "--framework-path",
                "/tmp/framework",
                "--dir",
                "/tmp/out"
            ])),
            Ok(Cli::New {
                name: "sample_app".into(),
                framework: "/tmp/framework".into(),
                dir: "/tmp/out".into(),
            })
        );
        for invalid in [
            vec!["build"],
            vec!["dev"],
            vec!["new", "app"],
            vec!["new", "app", "--framework-path"],
            vec!["new", "app", "--framework-path", "/tmp/framework", "--json"],
            vec!["doctor", "--json", "--json"],
            vec!["doctor", "--dir", "/tmp/out"],
            vec!["doctor", "--framework-path", ""],
            vec!["doctor", "--framework-path", "-unsafe"],
            vec!["new", "../escape", "--framework-path", "/tmp/framework"],
        ] {
            assert!(
                parse(&argv(&invalid)).is_err(),
                "unexpectedly accepted {invalid:?}"
            );
        }
    }

    #[test]
    fn build_dev_require_explicit_literal_manifest_and_exact_options() {
        assert_eq!(
            parse(&argv(&[
                "build",
                "--release",
                "--manifest-path",
                "/tmp/應 用;$(literal)/Cargo.toml"
            ])),
            Ok(Cli::Build {
                manifest: "/tmp/應 用;$(literal)/Cargo.toml".into(),
                release: true
            })
        );
        assert_eq!(
            parse(&argv(&["build", "--manifest-path", "./Cargo.toml"])),
            Ok(Cli::Build {
                manifest: "./Cargo.toml".into(),
                release: false
            })
        );
        assert_eq!(
            parse(&argv(&["dev", "--manifest-path", "./Cargo.toml"])),
            Ok(Cli::Dev {
                manifest: "./Cargo.toml".into()
            })
        );
        for invalid in [
            vec!["build", "--release"],
            vec!["dev", "--manifest-path"],
            vec!["dev", "--manifest-path", ""],
            vec!["dev", "--manifest-path", "--release"],
            vec!["dev", "--manifest-path", "Cargo.toml", "--release"],
            vec![
                "build",
                "--manifest-path",
                "Cargo.toml",
                "--release",
                "--release",
            ],
            vec![
                "build",
                "--manifest-path",
                "Cargo.toml",
                "--manifest-path",
                "Cargo.toml",
            ],
            vec!["build", "--manifest-path", "Cargo.toml", "--locked"],
            vec!["dev", "--manifest-path", "Cargo.toml", "--", "app-argument"],
            vec!["dev", "--manifest-path=Cargo.toml"],
            vec!["build", "Cargo.toml"],
            vec!["dev", "--help"],
            vec!["build", "--offline"],
        ] {
            assert!(parse(&argv(&invalid)).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn project_name_restrictions_keep_generated_manifest_safe() {
        for valid in ["demo", "a1", "sample_app", "native-ssr"] {
            assert!(scaffold::validate_name(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "1first",
            "UPPER",
            "with space",
            "../tmp",
            "async",
            "con",
            "com1",
            "lpt9",
            "yetlet-core",
            "yetlet-html",
            "yetlet-server-axum",
        ] {
            assert!(scaffold::validate_name(invalid).is_err(), "{invalid}");
        }
        assert!(scaffold::validate_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn standalone_manifest_escapes_unicode_and_quotes_and_avoids_workspace_inheritance() {
        let framework = Path::new("/tmp/框 架\"local");
        let files = scaffold::files("demo_app", framework).unwrap();
        assert_eq!(files.len(), 4);
        assert!(files.iter().all(|(name, _)| *name != "Cargo.lock"));
        let manifest = files
            .iter()
            .find(|(name, _)| *name == "Cargo.toml")
            .unwrap()
            .1
            .clone();
        let parsed: toml::Value = manifest.parse().unwrap();
        assert_eq!(parsed["package"]["name"].as_str(), Some("demo_app"));
        assert_eq!(parsed["package"]["rust-version"].as_str(), Some("1.99"));
        assert!(parsed["workspace"].is_table());
        assert!(
            parsed["dependencies"]["yetlet-core"]["path"]
                .as_str()
                .unwrap()
                .contains("框 架\"local")
        );
        assert_eq!(
            parsed["dependencies"]["yetlet-core"]["path"]
                .as_str()
                .unwrap(),
            framework.join("crates/yetlet-core").to_str().unwrap()
        );
        let source = &files
            .iter()
            .find(|(name, _)| *name == "src/main.rs")
            .unwrap()
            .1;
        assert!(source.contains("App::new()"));
        assert!(source.contains("Ipv4Addr::LOCALHOST"));
        assert!(!source.contains("<script"));
    }

    #[test]
    fn version_probe_rejects_unknown_or_outdated_output_without_leaking_it() {
        assert_eq!(
            doctor::version("rustc", b"rustc 1.99.0 (hash)").status,
            "pass"
        );
        assert_eq!(
            doctor::version("cargo", b"cargo 1.100.1 (hash)").status,
            "pass"
        );
        assert_eq!(doctor::version("rustc", b"rustc 1.98.9").status, "fail");
        let invalid = doctor::version("rustc", b"secret=/synthetic/nonexistent/credential");
        assert_eq!(invalid.status, "fail");
        assert!(
            !invalid
                .message
                .contains("/synthetic/nonexistent/credential")
        );
    }
}
