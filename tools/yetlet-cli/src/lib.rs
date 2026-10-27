//! Unpublished local CLI: literal arguments, diagnostics, SSR scaffolding and frozen Cargo.

pub mod app;
mod browser;
pub mod compose;
pub mod docker;
pub mod doctor;
mod framework;
mod image;
pub mod inspect;
mod k8s;
pub mod key;
mod platform;
pub mod scaffold;
mod settings;

#[cfg(test)]
#[path = "../tests/fixtures/compose_app.rs"]
mod cloud_fixture;
#[cfg(test)]
#[path = "../tests/fixtures/docker_framework.rs"]
mod docker_fixture;

use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub const VERSION: &str = concat!(
    "yetlet ",
    env!("CARGO_PKG_VERSION"),
    " (unpublished preview)"
);
pub const HELP: &str = "Yetlet CLI preview (unpublished)\n\nUsage:\n  yetlet --help | -h\n  yetlet --version | -V\n  yetlet new NAME [--parent DIRECTORY] [--template native|browser] [--vite] [--tailwind] [--service] [--settings]\n  yetlet doctor [--json]\n  yetlet inspect [--manifest-path PATH] [--json]\n  yetlet key status [KEY] [--provider NAME] [--manifest-path PATH]\n  yetlet key rotate [KEY] [--provider NAME] [--manifest-path PATH] [--yes] [--from PATH]\n  yetlet run [--manifest-path PATH]\n  yetlet watch [--manifest-path PATH]\n  yetlet build [--manifest-path PATH] [--release]\n  yetlet generate docker [--manifest-path PATH] --output DIRECTORY\n  yetlet generate compose [--manifest-path PATH] --output DIRECTORY --image IMAGE\n  yetlet generate k8s [--manifest-path PATH] --output DIRECTORY --image IMAGE\n\nUse --help after a complete command path for scoped help. Paths remain literal.\nRequires Rust/Cargo 1.99+. No implicit installation, lock refresh or deployment.\nExit: 0 success/help, 1 preflight/report, 2 usage, 125 spawn/cleanup; native child exits/signals are preserved after cleanup.\n";
const NEW_HELP: &str = "Usage: yetlet new NAME [--parent DIRECTORY] [--template native|browser] [--vite] [--tailwind] [--service] [--settings]\n\nNAME may appear anywhere among options. --parent defaults to cwd and must exist.\nNAME: 1-64 lowercase ASCII letters/digits/-/_, starting with a letter; reserved names refused.\n--vite selects the complete frontend scaffold and implies --settings. Plain new keeps app-runtime settings opt-in.\n--tailwind selects standalone Tailwind or the official Vite plugin. --service adds bind/health/shutdown.\n--template native|browser selects a built-in starter (default native), independent of runtime settings.\nThe browser template persists cli.target=browser and includes Vite/public settings.\nBrowser creation refuses --service. Plain creation defaults to native.\nCreates one new tree without overwrite, dependency installation, lock generation or process startup.\n";
const DOCTOR_HELP: &str = "Usage: yetlet doctor [--json]\n\nReports bounded local Rust/Cargo 1.99+ identity and compiled new-filesystem capability.\nEnvironment only; application, framework, settings, frontend and execution readiness are not checked.\n--json emits schema 1. No builds, installs, Node probes or listeners.\n";
const INSPECT_HELP: &str = "Usage: yetlet inspect [--manifest-path PATH] [--json]\n\nExplains public-settings execution plans for build/run/watch without running tools.\nManifest defaults to exactly cwd/Cargo.toml; requires a protected standalone app and Cargo.lock.\n--json emits project-execution-plan schema 1, separate from environment-only doctor.\nNative watch plans select Rust rebuild/restart; browser watch plans select WASM rebuild/full-page reload\nonly when cli.vite.auto_start and cli.vite.auto_build are both true.\nNo writes, secret sources, frontend inspection or readiness verification.\n";
const RUN_HELP: &str = "Usage: yetlet run [--manifest-path PATH]\n\nManifest defaults to exactly cwd/Cargo.toml; no ancestor or workspace search.\nRequires a protected standalone app and existing Cargo.lock; runs frozen Cargo.\nPublic settings select native Rust watch/Vite or cli.target=browser static preview.\nBrowser run starts only the prepared frontend, honoring cli.vite.auto_start. No implicit installs.\nOwned processes must retain their process group/Job; escaped daemons are unsupported.\n";
const WATCH_HELP: &str = "Usage: yetlet watch [--manifest-path PATH]\n\nManifest defaults to exactly cwd/Cargo.toml; requires a protected standalone app and Cargo.lock.\nPublic settings are validated before selecting the target-specific pipeline.\nNative: forces cli.rust.watch for this invocation and requires Watchexec on PATH.\nRust rebuild/restart retains Vite; Vite enabled, auto_start and root settings still apply.\nBrowser: requires both cli.vite.auto_start and cli.vite.auto_build; otherwise skips without children.\nThe frontend owns serialized WASM rebuild/full-page reload; no native watcher or Rust state restoration.\nVite owns TS/CSS HMR. No implicit installs or settings-file changes; cancellation cleans up owned processes.\n";
const BUILD_HELP: &str = "Usage: yetlet build [--manifest-path PATH] [--release]\n\nManifest defaults to exactly cwd/Cargo.toml. Requires a protected standalone app and existing Cargo.lock.\nSettings select native frontend-then-Cargo, or cli.target=browser frontend-only build.\nBrowser build honors cli.vite.auto_build; skipped stages exit 0 without children.\n--release selects the Cargo build profile; it does not change APP_ENVIRONMENT. No implicit installs.\n";
const GENERATE_HELP: &str = "Usage:\n  yetlet generate docker [--manifest-path PATH] --output DIRECTORY\n  yetlet generate compose [--manifest-path PATH] --output DIRECTORY --image IMAGE\n  yetlet generate k8s [--manifest-path PATH] --output DIRECTORY --image IMAGE\n\nWrites configuration only for qualified source/lock representations, without overwrite.\nNo image build/pull, source export, credentials lookup or deployment.\n";
const DOCKER_HELP: &str = "Usage: yetlet generate docker [--manifest-path PATH] --output DIRECTORY\n\nManifest defaults to cwd/Cargo.toml. Output is a complete new directory under an existing protected parent.\nQualified source/lock inspection only; writes configuration without tools or source export.\n";
const COMPOSE_HELP: &str = "Usage: yetlet generate compose [--manifest-path PATH] --output DIRECTORY --image IMAGE\n\nManifest defaults to cwd/Cargo.toml. Uses an explicit separately built Linux/amd64 image.\nIMAGE requires an explicit tag or sha256 digest. No build, pull or deployment.\n";
const K8S_HELP: &str = "Usage: yetlet generate k8s [--manifest-path PATH] --output DIRECTORY --image IMAGE\n\nManifest defaults to cwd/Cargo.toml. Replicas come from cli.generate.k8s.replicas (default 1, integer 1-100).\nPackage names must retain narrower DNS-label syntax; underscores are refused.\nWrites configuration for an explicit separately built Linux/amd64 image; no cluster action.\n";

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Help(&'static str),
    Version,
    Key(key::Command),
    Doctor {
        json: bool,
    },
    Inspect {
        manifest: PathBuf,
        json: bool,
    },
    New {
        name: String,
        dir: PathBuf,
        template: scaffold::Template,
        service: bool,
        runtime_settings: bool,
        tailwind: bool,
        vite: bool,
    },
    Build {
        manifest: PathBuf,
        release: bool,
    },
    Run {
        manifest: PathBuf,
    },
    Watch {
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
            } => {
                let app = platform::fs::open_app(manifest)?;
                let settings = settings::load(&app)?;
                k8s::generate(manifest, output, image, settings.generate.k8s.replicas)
            }
            _ => Err("expected generate k8s command".into()),
        }
    }
}

/// Safe usage selected only from recognized command paths, never arbitrary argv.
pub fn usage(args: &[OsString]) -> &'static str {
    match args.first().and_then(|v| v.to_str()) {
        Some("key") => key::HELP,
        Some("new") => NEW_HELP,
        Some("doctor") => DOCTOR_HELP,
        Some("inspect") => INSPECT_HELP,
        Some("run") => RUN_HELP,
        Some("watch") => WATCH_HELP,
        Some("build") => BUILD_HELP,
        Some("generate") => match args.get(1).and_then(|v| v.to_str()) {
            Some("docker") => DOCKER_HELP,
            Some("compose") => COMPOSE_HELP,
            Some("k8s") => K8S_HELP,
            _ => GENERATE_HELP,
        },
        _ => HELP,
    }
}

pub fn parse(args: &[OsString]) -> Result<Cli, &'static str> {
    if args.is_empty() {
        return Ok(Cli::Help(HELP));
    }
    if args.len() == 1 {
        match args[0].to_str() {
            Some("--help" | "-h") => return Ok(Cli::Help(HELP)),
            Some("--version" | "-V") => return Ok(Cli::Version),
            _ => (),
        }
    }
    let command = args[0].to_str().ok_or("command must be UTF-8")?;
    if command == "key" {
        return key::parse(&args[1..]);
    }
    if command == "dev" {
        return Err("dev was renamed; use yetlet run");
    }
    let (start, generator) = match command {
        "new" | "doctor" | "inspect" | "run" | "watch" | "build" => (1, None),
        "generate" => {
            if args.len() == 2 && matches!(args[1].to_str(), Some("--help" | "-h")) {
                return Ok(Cli::Help(GENERATE_HELP));
            }
            let generator = args
                .get(1)
                .and_then(|v| v.to_str())
                .ok_or("generate requires docker, compose or k8s")?;
            if !matches!(generator, "docker" | "compose" | "k8s") {
                return Err("generate requires docker, compose or k8s");
            }
            (2, Some(generator))
        }
        _ => return Err("unknown command; see --help"),
    };
    if args.len() == start + 1 && matches!(args[start].to_str(), Some("--help" | "-h")) {
        return Ok(Cli::Help(usage(args)));
    }
    let mut name = None;
    let mut parent = None;
    let mut template = None;
    let mut manifest = None;
    let mut output = None;
    let mut image = None;
    let (mut json, mut service, mut runtime_settings, mut tailwind, mut vite, mut release) =
        (false, false, false, false, false, false);
    let mut index = start;
    while index < args.len() {
        let (option, inline) = split_option(&args[index]);
        match option {
            Some("--parent") if command == "new" && parent.is_none() => {
                parent = Some(path_value(args, &mut index, inline)?)
            }
            Some("--template") if command == "new" && template.is_none() => {
                template = Some(match argument_value(args, &mut index, inline)?.to_str() {
                    Some("native") => scaffold::Template::Native,
                    Some("browser") => scaffold::Template::Browser,
                    _ => return Err("--template requires native or browser"),
                });
            }
            Some("--manifest-path")
                if matches!(command, "inspect" | "run" | "watch" | "build" | "generate")
                    && manifest.is_none() =>
            {
                manifest = Some(path_value(args, &mut index, inline)?)
            }
            Some("--output") if generator.is_some() && output.is_none() => {
                output = Some(path_value(args, &mut index, inline)?)
            }
            Some("--image") if matches!(generator, Some("compose" | "k8s")) && image.is_none() => {
                let value = argument_value(args, &mut index, inline)?
                    .to_str()
                    .ok_or("--image requires a literal ASCII image reference")?;
                image::validate(value)?;
                image = Some(value.to_owned());
            }
            Some("--json")
                if matches!(command, "doctor" | "inspect") && !json && inline.is_none() =>
            {
                json = true
            }
            Some("--service") if command == "new" && !service && inline.is_none() => service = true,
            Some("--settings") if command == "new" && !runtime_settings && inline.is_none() => {
                runtime_settings = true
            }
            Some("--tailwind") if command == "new" && !tailwind && inline.is_none() => {
                tailwind = true
            }
            Some("--vite") if command == "new" && !vite && inline.is_none() => vite = true,
            Some("--release") if command == "build" && !release && inline.is_none() => {
                release = true
            }
            Some("--cloud") => return Err("--cloud was removed; use new --service"),
            Some("--with-config") => return Err("--with-config was removed; use new --settings"),
            Some("--with-tailwind") => {
                return Err("--with-tailwind was removed; use new --tailwind");
            }
            Some("--dir") => return Err("--dir was removed; use new --parent DIRECTORY"),
            Some("--assets" | "--with-vite-dev" | "--dev-bridge") => {
                return Err(
                    "old Vite options were removed; use new --vite and public cli.vite settings",
                );
            }
            Some("--framework-path") => {
                return Err(
                    "--framework-path was removed; configure dependency versions or local overrides in Cargo.toml",
                );
            }
            Some("--replicas") => {
                return Err(
                    "--replicas was removed; set cli.generate.k8s.replicas in public settings",
                );
            }
            _ if command == "new"
                && name.is_none()
                && !args[index].as_encoded_bytes().starts_with(b"-") =>
            {
                let value = args[index].to_str().ok_or("NAME must be UTF-8")?;
                scaffold::validate_name(value)?;
                name = Some(value.to_owned());
            }
            _ => {
                return Err(
                    "unknown, duplicate or misplaced argument; help/version must be exclusive",
                );
            }
        }
        index += 1;
    }
    let manifest = manifest.unwrap_or_else(|| PathBuf::from("Cargo.toml"));
    let template = template.unwrap_or_default();
    if template == scaffold::Template::Browser && service {
        return Err("browser template does not select the native --service profile");
    }
    match command {
        "new" => Ok(Cli::New {
            name: name.ok_or("new requires NAME")?,
            dir: parent.unwrap_or_else(|| PathBuf::from(".")),
            template,
            service,
            runtime_settings: runtime_settings || vite,
            tailwind,
            vite,
        }),
        "doctor" => Ok(Cli::Doctor { json }),
        "inspect" => Ok(Cli::Inspect { manifest, json }),
        "run" => Ok(Cli::Run { manifest }),
        "watch" => Ok(Cli::Watch { manifest }),
        "build" => Ok(Cli::Build { manifest, release }),
        _ => {
            let output = output.ok_or("generator requires --output DIRECTORY")?;
            match generator {
                Some("docker") => Ok(Cli::GenerateDocker { manifest, output }),
                Some("compose") => Ok(Cli::GenerateCompose {
                    manifest,
                    output,
                    image: image.ok_or("generate compose requires --image IMAGE")?,
                }),
                _ => Ok(Cli::GenerateK8s {
                    manifest,
                    output,
                    image: image.ok_or("generate k8s requires --image IMAGE")?,
                }),
            }
        }
    }
}

fn split_option(value: &OsStr) -> (Option<&str>, Option<&OsStr>) {
    let bytes = value.as_encoded_bytes();
    if let Some(index) = bytes.iter().position(|&v| v == b'=') {
        // ASCII '=' is a self-synchronizing boundary in OsStr's encoding. The suffix
        // remains borrowed from the same native string, including non-UTF-8 paths.
        let suffix = unsafe { OsStr::from_encoded_bytes_unchecked(&bytes[index + 1..]) };
        (std::str::from_utf8(&bytes[..index]).ok(), Some(suffix))
    } else {
        (value.to_str(), None)
    }
}

fn argument_value<'a>(
    args: &'a [OsString],
    index: &mut usize,
    inline: Option<&'a OsStr>,
) -> Result<&'a OsStr, &'static str> {
    let value = match inline {
        Some(value) => value,
        None => {
            *index += 1;
            args.get(*index)
                .ok_or("option requires a value")?
                .as_os_str()
        }
    };
    if value.is_empty()
        || value.to_str().is_some_and(|v| v.trim().is_empty())
        || value.as_encoded_bytes().starts_with(b"-")
    {
        return Err("option requires a nonblank literal value (use ./ for a leading '-')");
    }
    Ok(value)
}

fn path_value(
    args: &[OsString],
    index: &mut usize,
    inline: Option<&OsStr>,
) -> Result<PathBuf, &'static str> {
    Ok(argument_value(args, index, inline)?.into())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn argv(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(OsString::from).collect()
    }

    #[test]
    fn command_grammar_normalizes_creation_and_literal_values() {
        assert_eq!(parse(&argv(&[])), Ok(Cli::Help(HELP)));
        assert_eq!(parse(&argv(&["-V"])), Ok(Cli::Version));
        for args in [
            vec![
                "new",
                "--parent=/tmp/應 用;$literal",
                "--vite",
                "demo",
                "--tailwind",
                "--service",
                "--settings",
            ],
            vec![
                "new",
                "demo",
                "--parent",
                "/tmp/應 用;$literal",
                "--settings",
                "--service",
                "--tailwind",
                "--vite",
            ],
        ] {
            assert_eq!(
                parse(&argv(&args)),
                Ok(Cli::New {
                    name: "demo".into(),
                    dir: "/tmp/應 用;$literal".into(),
                    template: scaffold::Template::Native,
                    service: true,
                    runtime_settings: true,
                    tailwind: true,
                    vite: true
                })
            );
        }
        assert_eq!(
            parse(&argv(&["new", "demo", "--vite"])),
            Ok(Cli::New {
                name: "demo".into(),
                dir: ".".into(),
                template: scaffold::Template::Native,
                service: false,
                runtime_settings: true,
                tailwind: false,
                vite: true
            })
        );
        assert_eq!(
            parse(&argv(&["new", "demo"])),
            Ok(Cli::New {
                name: "demo".into(),
                dir: ".".into(),
                template: scaffold::Template::Native,
                service: false,
                runtime_settings: false,
                tailwind: false,
                vite: false
            })
        );
        assert_eq!(
            parse(&argv(&["run"])),
            Ok(Cli::Run {
                manifest: "Cargo.toml".into()
            })
        );
        assert_eq!(
            parse(&argv(&["watch"])),
            Ok(Cli::Watch {
                manifest: "Cargo.toml".into()
            })
        );
        for args in [
            vec!["watch", "--manifest-path", "./應 用;$literal/Cargo.toml"],
            vec!["watch", "--manifest-path=./應 用;$literal/Cargo.toml"],
        ] {
            assert_eq!(
                parse(&argv(&args)),
                Ok(Cli::Watch {
                    manifest: "./應 用;$literal/Cargo.toml".into()
                })
            );
        }
        assert_eq!(
            parse(&argv(&[
                "build",
                "--release",
                "--manifest-path=./應 用/Cargo.toml"
            ])),
            Ok(Cli::Build {
                manifest: "./應 用/Cargo.toml".into(),
                release: true
            })
        );
        assert_eq!(
            parse(&argv(&["generate", "docker", "--output=out"])),
            Ok(Cli::GenerateDocker {
                manifest: "Cargo.toml".into(),
                output: "out".into()
            })
        );
        assert_eq!(
            parse(&argv(&[
                "generate",
                "k8s",
                "--image",
                "localhost/app:v1",
                "--output",
                "out"
            ])),
            Ok(Cli::GenerateK8s {
                manifest: "Cargo.toml".into(),
                output: "out".into(),
                image: "localhost/app:v1".into()
            })
        );
    }

    #[test]
    fn template_selection_is_a_literal_creation_option() {
        for args in [
            vec!["new", "tasks", "--template", "browser", "--tailwind"],
            vec!["new", "--template=browser", "--tailwind", "tasks"],
        ] {
            assert_eq!(
                parse(&argv(&args)),
                Ok(Cli::New {
                    name: "tasks".into(),
                    dir: ".".into(),
                    template: scaffold::Template::Browser,
                    service: false,
                    runtime_settings: false,
                    tailwind: true,
                    vite: false,
                })
            );
        }
        assert_eq!(
            parse(&argv(&["new", "tasks", "--template=native"])),
            parse(&argv(&["new", "tasks"]))
        );
        assert!(NEW_HELP.contains("--template native|browser"));
        assert!(!NEW_HELP.contains("APP_CLI__TARGET"));
        for args in [
            vec!["new", "tasks", "--template"],
            vec!["new", "tasks", "--template="],
            vec!["new", "tasks", "--template", "unknown"],
            vec!["new", "tasks", "--template", "Browser"],
            vec!["new", "tasks", "--template=browser", "--template=native"],
            vec!["new", "tasks", "--service", "--template=browser"],
            vec!["run", "--template=browser"],
            vec!["build", "--template=browser"],
        ] {
            assert!(parse(&argv(&args)).is_err(), "{args:?}");
        }
    }

    #[test]
    fn invalid_grammar_and_removed_spellings_fail_before_effects() {
        for args in [
            vec!["new"],
            vec!["new", "demo", "other"],
            vec!["new", "demo", "--vite", "build"],
            vec!["new", "demo", "--vite=true"],
            vec!["new", "demo", "--vite", "--vite"],
            vec!["new", "demo", "--settings", "--settings"],
            vec!["new", "demo", "--parent", " "],
            vec!["new", "demo", "--parent="],
            vec!["run", "--manifest-path="],
            vec!["watch", "--manifest-path="],
            vec!["watch", "--manifest-path", " "],
            vec!["watch", "--release"],
            vec!["watch", "extra"],
            vec!["watch", "--manifest-path=x", "--manifest-path=y"],
            vec!["watch", "--help", "--manifest-path=x"],
            vec!["watch", "--manifest-path=x", "-h"],
            vec!["watch", "--", "anything"],
            vec!["build", "--release=false"],
            vec!["doctor", "--json=true"],
            vec!["doctor", "--json", "--json"],
            vec!["run", "--manifest-path", "Cargo.toml", "--manifest-path=x"],
            vec!["generate", "--output", "out", "docker"],
            vec!["generate", "docker", "--image=x:y", "--output=out"],
            vec![
                "generate",
                "k8s",
                "--output=out",
                "--image=x:y",
                "--replicas=2",
            ],
            vec!["dev"],
            vec!["new", "demo", "--framework-path=x"],
            vec!["doctor", "--framework-path=x"],
            vec!["new", "demo", "--cloud"],
            vec!["new", "demo", "--with-config"],
            vec!["new", "demo", "--with-tailwind"],
            vec!["new", "demo", "--dir=x"],
            vec!["new", "demo", "--assets=vite"],
            vec!["new", "demo", "--with-vite-dev"],
            vec!["run", "--vite"],
            vec!["--help", "run"],
            vec!["new", "demo", "--help"],
            vec!["generate", "docker", "--help", "--output=out"],
            vec!["run", "--", "anything"],
        ] {
            assert!(parse(&argv(&args)).is_err(), "{args:?}");
        }
    }

    #[test]
    fn removed_creation_flags_keep_canonical_migration_diagnostics() {
        for (old, canonical) in [
            ("--cloud", "--service"),
            ("--with-config", "--settings"),
            ("--with-tailwind", "--tailwind"),
        ] {
            assert_eq!(
                parse(&argv(&["new", "demo", old])).unwrap_err(),
                format!("{old} was removed; use new {canonical}")
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn native_non_utf8_paths_survive_both_option_forms() {
        use std::os::unix::ffi::OsStringExt;
        let path = OsString::from_vec(b"./native-\xff/Cargo.toml".to_vec());
        let expected = Ok(Cli::Run {
            manifest: path.clone().into(),
        });
        assert_eq!(
            parse(&["run".into(), "--manifest-path".into(), path]),
            expected
        );
        let inline = OsString::from_vec(b"--manifest-path=./native-\xff/Cargo.toml".to_vec());
        assert_eq!(parse(&["run".into(), inline]), expected);
        let path = OsString::from_vec(b"./native-\xff/Cargo.toml".to_vec());
        let expected = Ok(Cli::Watch {
            manifest: path.clone().into(),
        });
        assert_eq!(
            parse(&["watch".into(), "--manifest-path".into(), path]),
            expected
        );
        let inline = OsString::from_vec(b"--manifest-path=./native-\xff/Cargo.toml".to_vec());
        assert_eq!(parse(&["watch".into(), inline]), expected);
    }

    #[test]
    fn config_manifest_dependencies_and_root_files_are_explicit() {
        for service in [false, true] {
            let framework = Path::new("/tmp/framework space \"quoted");
            let files = scaffold::config_files("demo", framework, service).unwrap();
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
            assert_eq!(manifest["package"].get("metadata").is_some(), service);
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
    fn cloud_only_selects_its_templates_signal_feature_and_contract_marker() {
        for service in [false, true] {
            let files = scaffold::files("demo", Path::new("/tmp/framework"), service).unwrap();
            assert_eq!(
                files.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
                ["Cargo.toml", "README.md", ".gitignore", "src/main.rs"]
            );
            let manifest: toml::Value = files[0].1.parse().unwrap();
            let features = manifest["dependencies"]["tokio"]["features"]
                .as_array()
                .unwrap();
            assert_eq!(
                features.iter().any(|v| v.as_str() == Some("signal")),
                service
            );
            if service {
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
