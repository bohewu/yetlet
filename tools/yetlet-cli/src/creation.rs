//! New-project orchestration: resolver result -> exact scaffold -> Cargo lock.
use crate::{
    app::Failure,
    platform::{fs, process},
    scaffold::{self, Template},
    sdk_registry,
    sdk_version::{self as sdk, Request, ResolvedSdk},
};
use std::{path::Path, process::Command};

#[derive(Clone, Copy)]
pub(crate) struct Options {
    pub template: Template,
    pub service: bool,
    pub settings: bool,
    pub tailwind: bool,
    pub vite: bool,
}

pub(crate) fn execute(
    name: &str,
    parent: &Path,
    options: Options,
    exact: Option<&str>,
    offline: bool,
) -> Result<ResolvedSdk, Failure> {
    let offline =
        offline || std::env::var("CARGO_NET_OFFLINE").is_ok_and(|v| v.eq_ignore_ascii_case("true"));
    let request = Request {
        exact,
        template: options.template,
        settings: options.settings || options.vite,
        offline,
    };
    let result = (|| {
        let root = sdk_registry::home()?;
        let now = sdk_registry::now()?;
        create(
            name,
            parent,
            options,
            request,
            &mut sdk_registry::CratesIo,
            &root,
            now,
            || {
                let report = crate::doctor::check();
                if report.ok() {
                    Ok(())
                } else {
                    Err(format!(
                        "CLI_INFRASTRUCTURE: installed tool preflight failed (exit={}): {}",
                        report.exit_code(),
                        report.render(false)
                    ))
                }
            },
            generate_lock,
        )
    })();
    result.map_err(|message: String| Failure {
        code: if message.contains("CLI_INFRASTRUCTURE") {
            125
        } else if message.contains("CLI_CANCELLED") {
            128 + process::cancellation_signal()
        } else {
            1
        },
        message,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create(
    name: &str,
    parent: &Path,
    options: Options,
    request: Request<'_>,
    registry: &mut impl sdk_registry::Registry,
    cache: &Path,
    now: u64,
    tools: impl FnOnce() -> Result<(), String>,
    lock: impl FnMut(&Path, bool) -> Result<(), String>,
) -> Result<ResolvedSdk, String> {
    create_with_policy(
        name,
        parent,
        options,
        request,
        registry,
        cache,
        now,
        tools,
        lock,
        sdk::POLICY,
    )
}

// Production and fixture policies take the same resolution/render/staging path.
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_with_policy(
    name: &str,
    parent: &Path,
    options: Options,
    request: Request<'_>,
    registry: &mut impl sdk_registry::Registry,
    cache: &Path,
    now: u64,
    tools: impl FnOnce() -> Result<(), String>,
    mut lock: impl FnMut(&Path, bool) -> Result<(), String>,
    raw_policy: &str,
) -> Result<ResolvedSdk, String> {
    let options = Options {
        settings: options.settings || options.vite,
        ..options
    };
    let request = Request {
        template: options.template,
        settings: options.settings,
        ..request
    };
    scaffold::validate_name(name).map_err(str::to_owned)?;
    if options.template.browser() && options.service {
        return Err("browser template does not select --service".into());
    }
    if let Some(version) = request.exact {
        sdk::exact(version).map_err(str::to_owned)?;
    }
    fs::ensure_create_supported()?;
    let parent = crate::framework::absolute(parent).map_err(|_| "invalid parent path")?;
    fs::preflight_new(&parent, name)?;
    tools()?;
    eprintln!(
        "SDK resolution mode={}",
        if request.offline { "offline" } else { "online" }
    );
    let sdk = sdk_registry::resolve_with_policy(request, registry, cache, now, raw_policy)?;
    eprintln!("{}", sdk.notice(now));
    let (mut entries, startup_profile) = render(name, options, &sdk)?;
    for (path, data) in &mut entries {
        if *path == "README.md" {
            *data = data.replace("\ncargo generate-lockfile\n", "\n")
                .replace("\ncargo generate-lockfile --offline\n", "\n")
                .replace("Creation writes files only: no Cargo lock, package installation, build, download\nor listener.", "Creation resolved the registry SDK and generated this app's Cargo.lock before publication.\nNo package installation, build or listener is started.")
                .replace("No dependency install, lock generation or process starts\nduring creation.", "Creation generates Cargo.lock in private staging; it does not install dependencies or start the app.")
                .replace("normal dependency lock/cache separately as above", "dependency cache separately as above; retain its generated app lock")
                .replace("--template browser --tailwind", &format!("--template browser --tailwind --sdk-version {}",sdk.version));
            data.push_str("\nThe CLI resolved an exact SDK and generated Cargo.lock at creation. Retain this lock for later frozen builds. See .yetlet/sdk-resolution.json for creation provenance.\n");
        }
        if *path == ".gitignore" {
            data.push_str("!/.yetlet/\n/.yetlet/*\n!/.yetlet/sdk-resolution.json\n");
        }
    }
    fs::create_prepared(&parent, name, &entries, |stage| {
        lock(stage, request.offline)
            .map_err(|error| format!("{error}; selected SDK {}", sdk.version))?;
        // Lock setup is not permitted to rewrite template inputs or the nested
        // Style workspace lock. Revalidate all original emitted bytes.
        for (path, data) in &entries {
            let bytes = fs::read_framework(&stage.join(path))
                .map_err(|_| "SDK_LOCK_MISMATCH: template input unavailable")?;
            if bytes != data.as_bytes() {
                return Err(format!(
                    "SDK_LOCK_MISMATCH: Cargo changed template input {path}"
                ));
            }
        }
        let bytes = fs::read_framework(&stage.join("Cargo.lock"))
            .map_err(|_| "SDK_LOCK_FAILED: app lock missing or unsafe")?;
        validate_lock(&bytes, &sdk)?;
        let receipt = serde_json::json!({"schema":1, "startup_profile":startup_profile.as_str(), "cli_version":env!("CARGO_PKG_VERSION"), "sdk":sdk, "cargo_lock_sha256":sdk::digest(&bytes)});
        Ok(vec![(
            ".yetlet/sdk-resolution.json",
            serde_json::to_string_pretty(&receipt).unwrap() + "\n",
        )])
    })?;
    Ok(sdk)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupProfile {
    LegacyAxum,
    M26AppServe,
    ConfiguredAxum,
    ServiceAxum,
    BrowserWasm,
}

impl StartupProfile {
    fn select(options: Options, m26_startup: bool) -> Self {
        if options.template.browser() {
            Self::BrowserWasm
        } else if options.service {
            Self::ServiceAxum
        } else if options.settings || options.vite {
            Self::ConfiguredAxum
        } else if m26_startup {
            Self::M26AppServe
        } else {
            Self::LegacyAxum
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::LegacyAxum => "legacy-axum",
            Self::M26AppServe => "m26-app-serve",
            Self::ConfiguredAxum => "configured-axum",
            Self::ServiceAxum => "service-axum",
            Self::BrowserWasm => "browser-wasm",
        }
    }
}

type RenderedFiles = Vec<(&'static str, String)>;

fn render(
    name: &str,
    options: Options,
    sdk: &ResolvedSdk,
) -> Result<(RenderedFiles, StartupProfile), String> {
    let profile = StartupProfile::select(options, sdk.m26_startup);
    let result = (|| {
        let mut entries = if options.template.browser() {
            crate::browser::files_version(name, options.tailwind, &sdk.version)?
        } else {
            scaffold::selected_files_version(
                name,
                options.service,
                options.settings,
                options.tailwind,
                options.vite,
                &sdk.version,
            )?
        };
        let styles = matches!(
            options.template,
            Template::NativeStyle | Template::BrowserStyle
        );
        if styles {
            crate::styles::scaffold(
                &mut entries,
                options.template.browser(),
                options.tailwind,
                options.vite || options.template.browser(),
            )?;
        }
        if profile == StartupProfile::M26AppServe {
            scaffold::m26_startup(&mut entries, styles, options.tailwind)?;
        }
        Ok((entries, profile))
    })();
    result.map_err(|reason: String| {
        if profile == StartupProfile::M26AppServe {
            format!("SDK_INCOMPATIBLE: {} template={} policy={} policy_sha256={}: template-capability {reason}",
                sdk.version, options.template.selector(), sdk.policy, sdk.policy_sha256)
        } else {
            reason
        }
    })
}

pub(crate) fn generate_lock(stage: &Path, offline: bool) -> Result<(), String> {
    let app = fs::open_directory(stage).map_err(|e| format!("CLI_INFRASTRUCTURE: {e}"))?;
    let mut command = Command::new("cargo");
    command
        .args(["generate-lockfile", "--manifest-path"])
        .arg(stage.join("Cargo.toml"))
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
        .env("CARGO_TERM_COLOR", "never");
    if offline {
        command.arg("--offline").env("CARGO_NET_OFFLINE", "true");
    }
    let output = process::capture_cargo(&app, &mut command).map_err(|e| {
        if e.kind() == std::io::ErrorKind::Interrupted { format!("CLI_CANCELLED: {e}") }
        else if e.kind() == std::io::ErrorKind::TimedOut || e.to_string().starts_with("Cargo lock process failed:") {
            format!("SDK_LOCK_FAILED: {e}; prepare the Cargo dependency cache separately for offline use")
        } else { format!("CLI_INFRASTRUCTURE: Cargo spawn/output/cleanup failed: {e}") }
    })?;
    if !output.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output));
    }
    Ok(())
}

pub(crate) fn validate_lock(bytes: &[u8], sdk: &ResolvedSdk) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "SDK_LOCK_MISMATCH: non-UTF8 Cargo lock")?;
    let lock: toml::Value =
        toml::from_str(text).map_err(|_| "SDK_LOCK_MISMATCH: invalid Cargo lock")?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("SDK_LOCK_MISMATCH: no lock packages")?;
    for selected in &sdk.packages {
        let entries: Vec<_> = packages
            .iter()
            .filter(|p| p.get("name").and_then(toml::Value::as_str) == Some(&selected.name))
            .collect();
        if entries.len() != 1
            || entries[0].get("version").and_then(toml::Value::as_str) != Some(&sdk.version)
            || entries[0].get("source").and_then(toml::Value::as_str) != Some(sdk::CARGO_REGISTRY)
            || entries[0].get("checksum").and_then(toml::Value::as_str) != Some(&selected.checksum)
        {
            return Err(format!(
                "SDK_LOCK_MISMATCH: {} version/source/checksum differs from selected registry metadata",
                selected.name
            ));
        }
    }
    for package in packages {
        if package
            .get("source")
            .and_then(toml::Value::as_str)
            .is_some_and(|s| s.starts_with("registry+"))
            && package
                .get("checksum")
                .and_then(toml::Value::as_str)
                .is_none()
        {
            return Err("SDK_LOCK_MISMATCH: missing registry checksum".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const VERSION: &str = "0.9.0-alpha.7";

    fn temporary(parent: Option<&Path>) -> tempfile::TempDir {
        let root = match parent {
            Some(parent) => tempfile::tempdir_in(parent),
            None => tempfile::tempdir(),
        }
        .unwrap();
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        root
    }

    struct Registry;
    impl sdk_registry::Registry for Registry {
        fn fetch(
            &mut self,
            name: &str,
            _: Option<&str>,
            _: Duration,
        ) -> Result<sdk_registry::Reply, String> {
            let row = serde_json::json!({
                "name":name, "vers":VERSION, "cksum":"f".repeat(64), "yanked":false,
                "features":{"server-axum":[],"config":[],"reactive":[],"dom":[],"dom-keyed":[]},
                "deps":if name == "yetlet-browser" {
                    vec![serde_json::json!({"name":"yetlet","req":format!("={VERSION}")})]
                } else { vec![] }, "v":2
            });
            Ok(sdk_registry::Reply::Modified(row.to_string() + "\n", None))
        }
    }

    fn policy(checksum: Option<&str>) -> String {
        serde_json::json!({
            "revision":"sdk-latest-v2", "selection":"latest-semver",
            "scaffold_capabilities":{"m26_startup":checksum.map(|checksum|
                vec![serde_json::json!({"sdk_version":VERSION,"yetlet_sha256":checksum})]
            ).unwrap_or_default()}
        })
        .to_string()
    }

    fn options(template: Template) -> Options {
        Options {
            template,
            service: false,
            settings: false,
            tailwind: false,
            vite: false,
        }
    }

    fn request(options: Options) -> Request<'static> {
        Request {
            exact: Some(VERSION),
            template: options.template,
            settings: options.settings,
            offline: false,
        }
    }

    fn file<'a>(entries: &'a [(&str, String)], path: &str) -> &'a str {
        &entries.iter().find(|entry| entry.0 == path).unwrap().1
    }

    #[test]
    fn m26_simple_shapes_preserve_assets_and_match_manifest_and_readme() {
        for template in [Template::Native, Template::NativeStyle] {
            for tailwind in [false, true] {
                let options = Options {
                    tailwind,
                    ..options(template)
                };
                let cache = tempfile::tempdir().unwrap();
                let sdk = sdk_registry::resolve_with_policy(
                    request(options),
                    &mut Registry,
                    cache.path(),
                    100,
                    &policy(Some(&"f".repeat(64))),
                )
                .unwrap();
                let (entries, profile) = render("demo", options, &sdk).unwrap();
                assert_eq!(profile, StartupProfile::M26AppServe);
                let main = file(&entries, "src/main.rs");
                let runtime = main.split("#[cfg(test)]").next().unwrap();
                assert!(runtime.contains("use yetlet::server_axum::prelude::*;"));
                for absent in ["YETLET_PORT", "TcpListener", "axum::serve", "println!"] {
                    assert!(!runtime.contains(absent), "{absent}: {runtime}");
                }
                let manifest: toml::Value = file(&entries, "Cargo.toml").parse().unwrap();
                assert_eq!(
                    manifest["dependencies"]["yetlet"]["version"].as_str(),
                    Some("=0.9.0-alpha.7")
                );
                assert_eq!(
                    manifest["dependencies"]["yetlet"]["features"]
                        .as_array()
                        .unwrap(),
                    &vec![toml::Value::String("server-axum".into())]
                );
                let tokio = manifest["dependencies"]["tokio"]["features"]
                    .as_array()
                    .unwrap();
                assert_eq!(
                    tokio,
                    &vec![
                        toml::Value::String("macros".into()),
                        toml::Value::String("rt-multi-thread".into())
                    ]
                );
                let styles = template == Template::NativeStyle;
                assert_eq!(
                    manifest["dependencies"].get("axum").is_some(),
                    styles || tailwind
                );
                if styles || tailwind {
                    let asset = if tailwind { "tailwind" } else { "styles_asset" };
                    assert_eq!(
                        runtime
                            .matches(&format!("{asset}::install(app, stylesheet)"))
                            .count(),
                        1
                    );
                    assert_eq!(
                        runtime
                            .matches(&format!("{asset}::Stylesheet::load()?;"))
                            .count(),
                        1
                    );
                    assert!(runtime.contains("App::from_router(app).serve().await?;"));
                    assert!(
                        runtime.find(".into_router()").unwrap()
                            < runtime.find("::install(app").unwrap()
                    );
                } else {
                    assert!(runtime.contains("App::new()\n        .page(\"/\", \"Yetlet\", home)"));
                    assert!(runtime.contains(".serve()\n        .await?;"));
                    assert!(!runtime.contains("use axum::"));
                    assert!(!runtime.contains("into_router"));
                }
                if styles {
                    assert_eq!(
                        file(&entries, "stylegen/Cargo.lock"),
                        include_str!("../templates/styles/Cargo.lock")
                    );
                }
                if styles && tailwind {
                    let module = file(&entries, "src/tailwind.rs");
                    assert_eq!(
                        module
                            .matches(
                                "crate::styles_asset::install(router, stylesheet.generated.clone())"
                            )
                            .count(),
                        1
                    );
                    assert!(module.contains("self.path, self.generated.path"));
                }
                let readme = file(&entries, "README.md");
                for expected in [
                    "http://127.0.0.1:7878",
                    ".serve_on(\"127.0.0.1:8080\")",
                    "SocketAddr",
                    "ignores `YETLET_PORT`",
                    "listener.local_addr()",
                    "into_router()",
                ] {
                    assert!(readme.contains(expected), "{expected}");
                }
            }
        }
    }

    #[test]
    fn profiles_keep_legacy_browser_and_advanced_files_identical() {
        let cases = [
            (options(Template::Native), StartupProfile::LegacyAxum),
            (
                Options {
                    tailwind: true,
                    ..options(Template::NativeStyle)
                },
                StartupProfile::LegacyAxum,
            ),
            (
                Options {
                    settings: true,
                    ..options(Template::Native)
                },
                StartupProfile::ConfiguredAxum,
            ),
            (
                Options {
                    vite: true,
                    tailwind: true,
                    ..options(Template::NativeStyle)
                },
                StartupProfile::ConfiguredAxum,
            ),
            (
                Options {
                    service: true,
                    ..options(Template::Native)
                },
                StartupProfile::ServiceAxum,
            ),
            (
                Options {
                    service: true,
                    settings: true,
                    vite: true,
                    tailwind: true,
                    ..options(Template::NativeStyle)
                },
                StartupProfile::ServiceAxum,
            ),
            (options(Template::Browser), StartupProfile::BrowserWasm),
            (
                Options {
                    tailwind: true,
                    ..options(Template::BrowserStyle)
                },
                StartupProfile::BrowserWasm,
            ),
        ];
        for (options, expected) in cases {
            let cache = tempfile::tempdir().unwrap();
            let mut sdk = sdk_registry::resolve_with_policy(
                request(options),
                &mut Registry,
                cache.path(),
                100,
                &policy(None),
            )
            .unwrap();
            let (entries, profile) = render("demo", options, &sdk).unwrap();
            assert_eq!(profile, expected);
            let mut legacy = if options.template.browser() {
                crate::browser::files_version("demo", options.tailwind, VERSION).unwrap()
            } else {
                scaffold::selected_files_version(
                    "demo",
                    options.service,
                    options.settings,
                    options.tailwind,
                    options.vite,
                    VERSION,
                )
                .unwrap()
            };
            if matches!(
                options.template,
                Template::NativeStyle | Template::BrowserStyle
            ) {
                crate::styles::scaffold(
                    &mut legacy,
                    options.template.browser(),
                    options.tailwind,
                    options.vite || options.template.browser(),
                )
                .unwrap();
            }
            assert_eq!(entries, legacy);
            if expected != StartupProfile::LegacyAxum {
                sdk.m26_startup = true;
                let (capable, profile) = render("demo", options, &sdk).unwrap();
                assert_eq!(profile, expected);
                assert_eq!(capable, legacy);
            }
        }
    }

    #[test]
    fn creation_receipts_bind_successful_profiles_to_fixture_policy_bytes() {
        let retained = std::env::var_os("YETLET_TEST_M26_APPS");
        for template in [
            Template::Native,
            Template::NativeStyle,
            Template::Browser,
            Template::BrowserStyle,
        ] {
            let browser = template.browser();
            let styles = matches!(template, Template::NativeStyle | Template::BrowserStyle);
            for flags in 0..16 {
                let options = Options {
                    template,
                    settings: flags & 1 != 0,
                    service: flags & 2 != 0,
                    tailwind: flags & 4 != 0,
                    vite: flags & 8 != 0,
                };
                if browser && options.service {
                    continue;
                }
                let configured = !browser && (options.settings || options.vite);
                let case = format!("{} flags={flags:04b}", template.selector());
                let name = match (template, flags) {
                    (Template::Native, 0) => "m26-native",
                    (Template::NativeStyle, 0) => "m26-native-style",
                    (Template::Native, 4) => "m26-native-tailwind",
                    (Template::NativeStyle, 4) => "m26-native-style-tailwind",
                    _ => "demo",
                };
                let mut legacy_inventory = None;
                for capable in [false, true] {
                    let raw_policy = policy(capable.then_some(&"f".repeat(64)));
                    let expected = if browser {
                        "browser-wasm"
                    } else if options.service {
                        "service-axum"
                    } else if configured {
                        "configured-axum"
                    } else if capable {
                        "m26-app-serve"
                    } else {
                        "legacy-axum"
                    };
                    let retain = retained
                        .as_deref()
                        .map(Path::new)
                        .filter(|_| expected == "m26-app-serve");
                    let root = temporary(retain);
                    let resolved = create_with_policy(
                        name,
                        root.path(),
                        options,
                        request(options),
                        &mut Registry,
                        &root.path().join("cache"),
                        100,
                        || Ok(()),
                        |stage, _| {
                            let mut lock = "version = 4\n".to_string();
                            let names = if browser {
                                &["yetlet", "yetlet-browser"][..]
                            } else {
                                &["yetlet"][..]
                            };
                            for name in names {
                                lock.push_str(&format!(
                                    "[[package]]\nname = \"{name}\"\nversion = \"{VERSION}\"\nsource = \"{}\"\nchecksum = \"{}\"\n",
                                    sdk::CARGO_REGISTRY,
                                    "f".repeat(64)
                                ));
                            }
                            std::fs::write(stage.join("Cargo.lock"), lock).unwrap();
                            Ok(())
                        },
                        &raw_policy,
                    )
                    .unwrap_or_else(|error| panic!("{case} {expected}: {error}"));
                    let app = root.path().join(name);
                    let receipt_path = ".yetlet/sdk-resolution.json";
                    let receipt: serde_json::Value =
                        serde_json::from_slice(&std::fs::read(app.join(receipt_path)).unwrap())
                            .unwrap();
                    assert_eq!(receipt["schema"], 1);
                    assert_eq!(receipt["startup_profile"], expected, "{case}");
                    assert_eq!(
                        receipt["sdk"]["policy_sha256"],
                        sdk::digest(raw_policy.as_bytes())
                    );
                    assert_eq!(receipt["sdk"]["version"], resolved.version);
                    assert_eq!(receipt["sdk"]["template"], template.selector());
                    assert!(receipt["sdk"].get("m26_startup").is_none());
                    assert_eq!(
                        receipt["cargo_lock_sha256"],
                        sdk::digest(&std::fs::read(app.join("Cargo.lock")).unwrap())
                    );

                    let inventory: Vec<_> = fs::static_output::inventory(&app)
                        .unwrap()
                        .into_iter()
                        .filter(|path| path != receipt_path && !path.ends_with('/'))
                        .map(|path| {
                            let data = std::fs::read(app.join(&path)).unwrap();
                            if path.ends_with(".rs") {
                                syn::parse_file(std::str::from_utf8(&data).unwrap())
                                    .unwrap_or_else(|error| {
                                        panic!("{case} {expected} {path}: {error}")
                                    });
                            }
                            (path, data)
                        })
                        .collect();
                    if browser || configured || options.service {
                        if capable {
                            assert_eq!(legacy_inventory.as_ref(), Some(&inventory), "{case}");
                        } else {
                            legacy_inventory = Some(inventory);
                        }
                    }

                    let read = |path: &str| std::fs::read_to_string(app.join(path)).unwrap();
                    let readme = read("README.md");
                    assert!(
                        readme.contains(VERSION) && readme.contains(receipt_path),
                        "{case}"
                    );
                    let m26 = expected == "m26-app-serve";
                    assert_eq!(readme.contains("127.0.0.1:7878"), m26, "{case}");
                    assert_eq!(readme.contains(".serve_on("), m26, "{case}");
                    assert_eq!(
                        readme.contains("## Application settings"),
                        configured,
                        "{case}"
                    );
                    let manifest: toml::Value = read("Cargo.toml").parse().unwrap();
                    let exact = format!("={VERSION}");
                    if browser {
                        let wasm =
                            &manifest["target"]["cfg(target_arch = \"wasm32\")"]["dependencies"];
                        for package in ["yetlet", "yetlet-browser"] {
                            assert_eq!(wasm[package]["version"].as_str(), Some(exact.as_str()));
                        }
                        assert!(manifest.get("dependencies").is_none(), "{case}");
                        assert!(readme.contains("http://127.0.0.1:5173/"), "{case}");
                    } else {
                        let deps = &manifest["dependencies"];
                        assert_eq!(deps["yetlet"]["version"].as_str(), Some(exact.as_str()));
                        assert!(deps["yetlet"].get("path").is_none(), "{case}");
                        let features = |dep: &str, feature: &str| {
                            deps[dep]["features"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|value| value.as_str() == Some(feature))
                        };
                        assert!(features("yetlet", "server-axum"), "{case}");
                        assert_eq!(features("yetlet", "config"), configured, "{case}");
                        assert_eq!(features("tokio", "net"), !m26, "{case}");
                        assert_eq!(features("tokio", "signal"), options.service, "{case}");
                        assert_eq!(
                            deps.get("axum").is_some(),
                            !m26 || styles || options.tailwind,
                            "{case}"
                        );
                        assert_eq!(manifest.get("features").is_some(), configured, "{case}");

                        let main = read("src/main.rs");
                        let runtime = main.split("#[cfg(test)]").next().unwrap();
                        assert_eq!(runtime.contains("server_axum::prelude"), m26, "{case}");
                        assert_eq!(runtime.contains(".serve()"), m26, "{case}");
                        assert_eq!(runtime.contains("config::serve("), configured, "{case}");
                        assert_eq!(
                            runtime.contains("axum::serve("),
                            !m26 && !configured,
                            "{case}"
                        );
                        assert_eq!(runtime.contains("listener.local_addr()"), !m26, "{case}");
                        assert_eq!(
                            runtime.contains("shutdown_signal("),
                            options.service,
                            "{case}"
                        );
                        assert_eq!(app.join("src/config.rs").exists(), configured, "{case}");
                        if options.vite {
                            assert!(
                                runtime.contains(
                                    "vite::Snapshot::load_selected(&settings.cli.vite.root)"
                                ),
                                "{case}"
                            );
                            assert!(runtime.contains(".merge(vite::router(assets))"), "{case}");
                        } else if styles || options.tailwind {
                            let asset = if options.tailwind {
                                "tailwind"
                            } else {
                                "styles_asset"
                            };
                            let main = main.split("async fn main()").nth(1).unwrap();
                            assert_eq!(
                                main.matches(&format!("{asset}::Stylesheet::load()?"))
                                    .count(),
                                1,
                                "{case}"
                            );
                            assert_eq!(
                                main.split("#[cfg(test)]")
                                    .next()
                                    .unwrap()
                                    .matches(&format!("{asset}::install("))
                                    .count(),
                                1,
                                "{case}"
                            );
                            if styles && options.tailwind {
                                let tailwind = read("src/tailwind.rs");
                                assert!(
                                    tailwind.contains("self.path, self.generated.path"),
                                    "{case}"
                                );
                                assert_eq!(
                                    tailwind.matches("crate::styles_asset::install(").count(),
                                    1,
                                    "{case}"
                                );
                            }
                        }
                    }
                    if styles {
                        assert_eq!(
                            read("stylegen/Cargo.lock"),
                            include_str!("../templates/styles/Cargo.lock"),
                            "{case}"
                        );
                        let style: toml::Value = read("stylegen/Cargo.toml").parse().unwrap();
                        assert!(style.get("workspace").is_some(), "{case}");
                        assert_eq!(style["package"]["name"].as_str(), Some("app-stylegen"));
                        assert!(app.join("src/style_names.rs").is_file(), "{case}");
                    }
                    if browser || options.vite {
                        let package_lock = if options.tailwind {
                            include_str!("../templates/vite/tailwind-package-lock.json")
                        } else {
                            include_str!("../templates/vite/plain-package-lock.json")
                        };
                        assert_eq!(read("frontend/package-lock.json"), package_lock, "{case}");
                        let frontend = read("frontend/app.ts");
                        assert_eq!(
                            frontend.contains("import './style-output/app.generated.css';"),
                            styles,
                            "{case}"
                        );
                        assert!(app.join("frontend/vite.config.mjs").is_file(), "{case}");
                        assert!(app.join("frontend/package.json").is_file(), "{case}");
                    }
                    eprintln!("MATRIX PASS {case} capability={capable} profile={expected}");
                    if let Some(retained) = retain {
                        std::fs::write(retained.join("fixture-policy.json"), &raw_policy).unwrap();
                        std::fs::rename(&app, retained.join(name)).unwrap();
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_policy_or_claim_cannot_stage_or_modify_a_destination() {
        for raw_policy in ["{}".to_owned(), policy(Some(&"e".repeat(64)))] {
            for options in [
                options(Template::Native),
                options(Template::Browser),
                Options {
                    service: true,
                    ..options(Template::Native)
                },
            ] {
                let root = temporary(None);
                let error = create_with_policy(
                    "demo",
                    root.path(),
                    options,
                    request(options),
                    &mut Registry,
                    &root.path().join("cache"),
                    100,
                    || Ok(()),
                    |_, _| panic!("must fail before staging"),
                    &raw_policy,
                )
                .unwrap_err();
                assert!(
                    error.starts_with(if raw_policy == "{}" {
                        "SDK_METADATA_INVALID"
                    } else {
                        "SDK_INCOMPATIBLE"
                    }),
                    "{error}"
                );
                assert!(!root.path().join("demo").exists());
                assert!(!std::fs::read_dir(root.path()).unwrap().any(|entry| {
                    entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".yetlet-")
                }));
                std::fs::create_dir(root.path().join("demo")).unwrap();
                std::fs::write(root.path().join("demo/keep"), "unchanged").unwrap();
                assert!(
                    create_with_policy(
                        "demo",
                        root.path(),
                        options,
                        request(options),
                        &mut Registry,
                        &root.path().join("cache"),
                        100,
                        || panic!("existing destination must precede tools"),
                        |_, _| panic!("existing destination must precede staging"),
                        &raw_policy
                    )
                    .is_err()
                );
                assert_eq!(
                    std::fs::read_to_string(root.path().join("demo/keep")).unwrap(),
                    "unchanged"
                );
                assert_eq!(
                    std::fs::read_dir(root.path().join("demo")).unwrap().count(),
                    1
                );
            }
        }
    }

    #[test]
    fn m26_composition_refuses_missing_or_ambiguous_builtin_anchors() {
        for duplicate in [false, true] {
            let mut entries =
                scaffold::selected_files_version("demo", false, false, false, false, VERSION)
                    .unwrap();
            let main = &mut entries
                .iter_mut()
                .find(|entry| entry.0 == "src/main.rs")
                .unwrap()
                .1;
            if duplicate {
                main.push_str("\nuse axum::extract::Path;\nuse yetlet::view::{Element, View};\nuse yetlet::server_axum::{App, Page};\n");
            } else {
                *main = main.replace(
                    "axum::serve(listener, app)",
                    "unexpected_server(listener, app)",
                );
            }
            assert!(scaffold::m26_startup(&mut entries, false, false).is_err());
        }
    }
}
