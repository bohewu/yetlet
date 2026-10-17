//! Unpublished local CLI: literal arguments, diagnostics, SSR scaffolding and frozen Cargo.

pub mod app;
pub mod compose;
pub mod docker;
pub mod doctor;
mod framework;
mod image;
mod k8s;
mod platform;
pub mod scaffold;

#[cfg(test)]
#[path = "../tests/fixtures/compose_app.rs"]
mod cloud_fixture;
#[cfg(test)]
#[path = "../tests/fixtures/docker_framework.rs"]
mod docker_fixture;

use std::{ffi::OsString, path::PathBuf};

pub const HELP: &str = "Yetlet CLI preview (unpublished, Linux/Windows; YT-088)\n\nUsage:\n  yetlet [--help]\n  yetlet doctor [--framework-path PATH] [--json]\n  yetlet new NAME --framework-path PATH [--dir DIRECTORY] [--cloud] [--with-config] [--with-tailwind] [--assets vite] [--with-vite-dev]\n  yetlet build --manifest-path PATH [--release]\n  yetlet dev --manifest-path PATH\n  yetlet generate docker --manifest-path APP/Cargo.toml --output DIRECTORY\n  yetlet generate compose --manifest-path APP/Cargo.toml --output DIRECTORY --image REGISTRY/REPOSITORY:TAG\n  yetlet generate k8s --manifest-path APP/Cargo.toml --output DIRECTORY --image REGISTRY/REPOSITORY:TAG [--replicas COUNT]\n\nnew --assets vite adds explicit frontend npm ci/typecheck/build steps, never a runtime Node process.\nnew --assets vite --with-vite-dev adds a separately started loopback-only npm dev profile; see its README.\nWith --assets vite, --with-tailwind selects only the official Tailwind Vite plugin.\nWithout --assets, --with-tailwind adds the established offline standalone Tailwind CSS 4.3.3 step.\nnew --with-config opts into typed app-root configuration; see its README.\nnew --cloud opts into app-owned bind/health/graceful shutdown; see its README.\nnew creates DIRECTORY/NAME; DIRECTORY defaults to the current directory and must\nalready exist. Linux uses protected, symlink-safe paths; Windows requires protected local fixed NTFS paths without reparse traversal. No '..' or overwrite.\nNAME: 1-64 lowercase ASCII letters, digits, '-' or '_', starting with a letter;\nRust keywords and reserved device/Cargo names are rejected. Paths may contain\nspaces and Unicode. No shell expansion is performed.\n\nRequires Rust/Cargo 1.99 or newer. doctor only runs local --version probes;\nnew and generate docker/compose/k8s run no subprocess. Kubernetes writes deployment.yaml, service.yaml and README.md for a separately built Linux/amd64 image; --replicas is ASCII decimal 1-100 (default 1). Its package name must be unchanged DNS label syntax (1-63 letters/digits/hyphens, letter prefix, letter/digit suffix); underscores are refused. No cluster action is performed. Compose references an explicit separately built Linux/amd64 image; it never builds or pulls an image. Docker generation writes configuration only; prepare two separate sanitized named source contexts as instructed in its README. Neither command installs, builds or downloads anything.\nA new app has no Cargo.lock: explicitly generate its own lock offline using the\nlocal cache, then cargo check --frozen (see the generated README).\nbuild/dev require a standalone [package] + [workspace] Cargo.toml and its own\nexisting Cargo.lock. App cwd/manifest/lock must be protected against untrusted writes by native\npermissions/ACLs; Windows build/dev require an owner-protected cargo.exe.\nLinux and Windows are supported within local OS/console limits; other OSes fail closed.\nThey run one foreground cargo build/run --frozen, without network fallback.\ndev has no watcher/browser; Ctrl-C cleans its owned process group/Job.\nWindows dev requires an attached console with processed Ctrl-C input. Applications\nmust retain the inherited process group; daemonization is unsupported.\nExit: 0 success/help, 1 preflight, 2 usage, 125 spawn/cleanup failure; otherwise\nthe native Cargo exit or signal is preserved, including cancellation.\nPublication and global installation remain outside this preview.\n";

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
        cloud: bool,
        with_config: bool,
        with_tailwind: bool,
    },
    /// Separate variant preserves the established no-assets parse results.
    NewVite {
        name: String,
        framework: PathBuf,
        dir: PathBuf,
        cloud: bool,
        with_config: bool,
        with_tailwind: bool,
    },
    /// Explicit dev tooling opt-in; established New/NewVite results stay unchanged.
    NewViteDev {
        name: String,
        framework: PathBuf,
        dir: PathBuf,
        cloud: bool,
        with_config: bool,
        with_tailwind: bool,
    },
    Build {
        manifest: PathBuf,
        release: bool,
    },
    Dev {
        manifest: PathBuf,
    },
    GenerateDocker {
        manifest: PathBuf,
        output: PathBuf,
    },
    GenerateCompose {
        manifest: PathBuf,
        output: PathBuf,
        image: String,
    },
    GenerateK8s {
        manifest: PathBuf,
        output: PathBuf,
        image: String,
        replicas: u32,
    },
}

impl Cli {
    /// Execute only the parsed Kubernetes CLI command. No public generator API.
    pub fn generate_k8s(&self) -> Result<(), String> {
        match self {
            Self::GenerateK8s {
                manifest,
                output,
                image,
                replicas,
            } => k8s::generate(manifest, output, image, *replicas),
            _ => Err("expected generate k8s command".into()),
        }
    }
}

pub fn parse(args: &[OsString]) -> Result<Cli, &'static str> {
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        return Ok(Cli::Help);
    }
    let command = args[0].to_str().ok_or("command must be UTF-8")?;
    if command == "generate" {
        let generator = args.get(1).and_then(|v| v.to_str());
        if !matches!(generator, Some("docker" | "compose" | "k8s")) {
            return Err("generate requires the docker, compose or k8s command");
        }
        let mut manifest = None;
        let mut output = None;
        let mut image = None;
        let mut replicas = None;
        let mut index = 2;
        while index < args.len() {
            match args[index].to_str() {
                Some("--manifest-path") if manifest.is_none() => {
                    index += 1;
                    manifest = Some(value(args, index)?);
                }
                Some("--output") if output.is_none() => {
                    index += 1;
                    output = Some(value(args, index)?);
                }
                Some("--image")
                    if matches!(generator, Some("compose" | "k8s")) && image.is_none() =>
                {
                    index += 1;
                    let literal = args
                        .get(index)
                        .and_then(|v| v.to_str())
                        .ok_or("--image requires a literal ASCII image reference")?;
                    image::validate(literal)?;
                    image = Some(literal.to_owned());
                }
                Some("--replicas") if generator == Some("k8s") && replicas.is_none() => {
                    index += 1;
                    replicas = Some(k8s::parse_replicas(
                        args.get(index)
                            .and_then(|v| v.to_str())
                            .ok_or("--replicas requires an ASCII decimal integer 1-100")?,
                    )?);
                }
                _ => return Err("unknown, duplicate or misplaced generate option"),
            }
            index += 1;
        }
        if generator == Some("k8s") {
            let manifest = manifest.ok_or("generate k8s requires --manifest-path PATH")?;
            let output = output.ok_or("generate k8s requires --output DIRECTORY")?;
            if [manifest.as_os_str(), output.as_os_str()]
                .iter()
                .any(|v| v.to_str().is_some_and(|s| s.trim().is_empty()))
            {
                return Err("generate k8s requires nonblank paths");
            }
            return Ok(Cli::GenerateK8s {
                manifest,
                output,
                image: image.ok_or(
                    "generate k8s requires --image REGISTRY/REPOSITORY:TAG or @sha256:DIGEST",
                )?,
                replicas: replicas.unwrap_or(1),
            });
        }
        if generator == Some("compose") {
            let manifest = manifest.ok_or("generate compose requires --manifest-path PATH")?;
            let output = output.ok_or("generate compose requires --output DIRECTORY")?;
            // Compose rejects blank paths without changing historical commands.
            if [manifest.as_os_str(), output.as_os_str()]
                .iter()
                .any(|v| v.to_str().is_some_and(|s| s.trim().is_empty()))
            {
                return Err("generate compose requires nonblank paths");
            }
            return Ok(Cli::GenerateCompose {
                manifest,
                output,
                image: image.ok_or(
                    "generate compose requires --image REGISTRY/REPOSITORY:TAG or @sha256:DIGEST",
                )?,
            });
        }
        return Ok(Cli::GenerateDocker {
            manifest: manifest.ok_or("generate docker requires --manifest-path PATH")?,
            output: output.ok_or("generate docker requires --output DIRECTORY")?,
        });
    }
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
    let mut cloud = false;
    let mut with_config = false;
    let mut with_tailwind = false;
    let mut vite = false;
    let mut vite_dev = false;
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
            Some("--cloud") if command == "new" && !cloud => cloud = true,
            Some("--with-config") if command == "new" && !with_config => with_config = true,
            Some("--with-tailwind") if command == "new" && !with_tailwind => with_tailwind = true,
            Some("--with-vite-dev") if command == "new" && !vite_dev => vite_dev = true,
            Some("--assets") if command == "new" && !vite => {
                index += 1;
                if args.get(index).and_then(|v| v.to_str()) != Some("vite") {
                    return Err("--assets requires exactly vite");
                }
                vite = true;
            }
            _ => return Err("unknown, duplicate or misplaced option"),
        }
        index += 1;
    }
    if vite_dev && !vite {
        return Err("--with-vite-dev requires --assets vite");
    }
    match name {
        Some(name) if vite_dev => Ok(Cli::NewViteDev {
            name,
            framework: framework.ok_or("new requires --framework-path PATH")?,
            dir: dir.unwrap_or_else(|| PathBuf::from(".")),
            cloud,
            with_config,
            with_tailwind,
        }),
        Some(name) if vite => Ok(Cli::NewVite {
            name,
            framework: framework.ok_or("new requires --framework-path PATH")?,
            dir: dir.unwrap_or_else(|| PathBuf::from(".")),
            cloud,
            with_config,
            with_tailwind,
        }),
        Some(name) => Ok(Cli::New {
            name,
            framework: framework.ok_or("new requires --framework-path PATH")?,
            dir: dir.unwrap_or_else(|| PathBuf::from(".")),
            cloud,
            with_config,
            with_tailwind,
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
                cloud: false,
                with_config: false,
                with_tailwind: false,
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
    fn configuration_is_explicit_unique_and_new_only() {
        for cloud in [false, true] {
            let mut args = argv(&[
                "new",
                "demo",
                "--with-config",
                "--framework-path",
                "/tmp/framework",
            ]);
            if cloud {
                args.push("--cloud".into());
            }
            assert_eq!(
                parse(&args),
                Ok(Cli::New {
                    name: "demo".into(),
                    framework: "/tmp/framework".into(),
                    dir: ".".into(),
                    cloud,
                    with_config: true,
                    with_tailwind: false,
                })
            );
            args.push("--with-config".into());
            assert!(parse(&args).is_err());
        }
        for args in [
            vec!["--with-config"],
            vec!["new", "--with-config", "demo", "--framework-path", "path"],
            vec!["new", "demo", "--framework-path", "--with-config"],
            vec![
                "new",
                "demo",
                "--framework-path",
                "path",
                "--with-config=true",
            ],
            vec!["doctor", "--with-config"],
            vec!["build", "--manifest-path", "Cargo.toml", "--with-config"],
            vec!["dev", "--manifest-path", "Cargo.toml", "--with-config"],
            vec!["generate", "docker", "--with-config"],
            vec!["generate", "compose", "--with-config"],
            vec!["generate", "k8s", "--with-config"],
        ] {
            assert!(parse(&argv(&args)).is_err(), "{args:?}");
        }
    }

    #[test]
    fn config_manifest_dependencies_and_root_files_are_explicit() {
        for cloud in [false, true] {
            let framework = Path::new("/tmp/framework space \"quoted");
            let files = scaffold::config_files("demo", framework, cloud).unwrap();
            assert_eq!(
                files.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
                [
                    "Cargo.toml",
                    "README.md",
                    ".gitignore",
                    "src/main.rs",
                    "src/config.rs",
                    "base.toml",
                    "development.toml",
                    "staging.toml",
                    "production.toml",
                    ".env.example",
                ]
            );
            let manifest: toml::Value = files[0].1.parse().unwrap();
            assert_eq!(
                manifest["dependencies"]["yetlet-config"]["path"].as_str(),
                framework.join("crates/yetlet-config").to_str()
            );
            assert_eq!(
                manifest["dependencies"]["serde"]["features"][0].as_str(),
                Some("derive")
            );
            assert_eq!(
                manifest["dependencies"]["secrecy"]["features"][0].as_str(),
                Some("serde")
            );
            assert_eq!(manifest["package"]["publish"].as_bool(), Some(false));
            assert_eq!(manifest["package"].get("metadata").is_some(), cloud);
            assert_eq!(manifest["dependencies"].as_table().unwrap().len(), 7);
            for (_, contents) in &files[5..9] {
                contents.parse::<toml::Value>().unwrap();
            }
            assert!(!files.iter().any(|(p, _)| *p == "Cargo.lock"));
            assert!(files[1].1.contains("generate docker"));
            assert!(files[2].1.ends_with("!.env.example\n"));
            syn::parse_file(&files[3].1).unwrap();
            syn::parse_file(&files[4].1).unwrap();
        }
    }

    #[test]
    fn cloud_is_an_explicit_unique_new_option_after_name() {
        for options in [
            vec![
                "new",
                "demo",
                "--cloud",
                "--framework-path",
                "/tmp/framework",
            ],
            vec![
                "new",
                "demo",
                "--framework-path",
                "/tmp/framework",
                "--cloud",
            ],
        ] {
            assert_eq!(
                parse(&argv(&options)),
                Ok(Cli::New {
                    name: "demo".into(),
                    framework: "/tmp/framework".into(),
                    dir: ".".into(),
                    cloud: true,
                    with_config: false,
                    with_tailwind: false,
                })
            );
        }
        for invalid in [
            vec!["--cloud"],
            vec!["--cloud", "new", "demo"],
            vec![
                "new",
                "--cloud",
                "demo",
                "--framework-path",
                "/tmp/framework",
            ],
            vec![
                "new",
                "demo",
                "--cloud",
                "--cloud",
                "--framework-path",
                "/tmp/framework",
            ],
            vec!["new", "demo", "--framework-path", "--cloud"],
            vec![
                "new",
                "demo",
                "--cloud=true",
                "--framework-path",
                "/tmp/framework",
            ],
            vec!["doctor", "--cloud"],
            vec!["build", "--manifest-path", "Cargo.toml", "--cloud"],
            vec!["dev", "--manifest-path", "Cargo.toml", "--cloud"],
        ] {
            assert!(parse(&argv(&invalid)).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn cloud_only_selects_its_templates_signal_feature_and_contract_marker() {
        for cloud in [false, true] {
            let files = scaffold::files("demo", Path::new("/tmp/framework"), cloud).unwrap();
            assert_eq!(
                files.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
                ["Cargo.toml", "README.md", ".gitignore", "src/main.rs"]
            );
            let manifest: toml::Value = files[0].1.parse().unwrap();
            let features = manifest["dependencies"]["tokio"]["features"]
                .as_array()
                .unwrap();
            assert_eq!(features.iter().any(|v| v.as_str() == Some("signal")), cloud);
            if cloud {
                assert_eq!(
                    manifest["package"]["metadata"]["yetlet"]["cloud-profile"].as_str(),
                    Some("native-ssr-v1")
                );
                assert_eq!(files[3].1, include_str!("../templates/cloud/main.rs"));
                assert_eq!(
                    files[1].1,
                    include_str!("../templates/cloud/README.md").replace("@NAME@", "demo")
                );
            } else {
                assert!(manifest["package"].get("metadata").is_none());
                assert_eq!(files[3].1, include_str!("../templates/main.rs"));
                assert_eq!(
                    files[1].1,
                    include_str!("../templates/README.md").replace("@NAME@", "demo")
                );
            }
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
        let files = scaffold::files("demo_app", framework, false).unwrap();
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
