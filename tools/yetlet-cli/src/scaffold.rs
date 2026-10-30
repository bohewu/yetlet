use std::path::Path;

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    const RESERVED: &[&str] = &[
        "as",
        "async",
        "await",
        "break",
        "const",
        "continue",
        "crate",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
        "abstract",
        "become",
        "box",
        "do",
        "final",
        "gen",
        "macro",
        "override",
        "priv",
        "try",
        "typeof",
        "unsized",
        "virtual",
        "yield",
        "con",
        "prn",
        "aux",
        "nul",
        "test",
        "build",
        "deps",
        "examples",
        "incremental",
        "yetlet",
        "yetlet-browser",
        // Cargo package IDs must not collide with our path dependencies at 0.1.0.
        "yetlet-core",
        "yetlet-html",
        "yetlet-server-axum",
    ];
    if name.len() > 64
        || !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        || RESERVED.contains(&name)
        || ["com", "lpt"].iter().any(|prefix| {
            name.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
        })
    {
        return Err("invalid or reserved project NAME (see --help)");
    }
    Ok(())
}

// Keep the default scaffold profile independent of the opt-in service profile.
pub fn create(name: &str, framework: &Path, dir: &Path) -> Result<(), String> {
    create_with_profile(name, framework, dir, false)
}

/// Create the opt-in native service profile with bind, health and shutdown hooks.
pub fn create_service(name: &str, framework: &Path, dir: &Path) -> Result<(), String> {
    create_with_profile(name, framework, dir, true)
}

fn create_with_profile(
    name: &str,
    framework: &Path,
    dir: &Path,
    service: bool,
) -> Result<(), String> {
    validate_name(name).map_err(str::to_owned)?;
    crate::platform::fs::ensure_create_supported()?;
    let framework = crate::framework::validate(framework).map_err(str::to_owned)?;
    let entries = files(name, &framework, service)?;
    crate::platform::fs::create_project(dir, name, &entries)
}

pub(crate) fn files(
    name: &str,
    framework: &Path,
    service: bool,
) -> Result<Vec<(&'static str, String)>, String> {
    let quoted = |path: &Path| -> Result<String, String> {
        Ok(toml::Value::String(
            path.to_str()
                .ok_or("framework path must be UTF-8 for Cargo.toml")?
                .into(),
        )
        .to_string())
    };
    let framework_package = quoted(&framework.join("crates/yetlet"))?;
    let signal = if service { ", \"signal\"" } else { "" };
    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.99\"\ndescription = \"Native Yetlet SSR application preview\"\nlicense = \"MIT OR Apache-2.0\"\npublish = false\n\n[workspace]\nresolver = \"3\"\n\n[dependencies]\naxum = \"0.8\"\ntokio = {{ version = \"1\", features = [\"macros\", \"net\", \"rt-multi-thread\"{signal}] }}\nyetlet = {{ path = {framework_package}, default-features = false, features = [\"server-axum\"] }}\n"
    );
    let (manifest, readme, main) = if service {
        (
            format!("{manifest}\n[package.metadata.yetlet]\ncloud-profile = \"native-ssr-v1\"\n"),
            include_str!("../templates/cloud/README.md"),
            include_str!("../templates/cloud/main.rs"),
        )
    } else {
        (
            manifest,
            include_str!("../templates/README.md"),
            include_str!("../templates/main.rs"),
        )
    };
    Ok(vec![
        ("Cargo.toml", manifest),
        ("README.md", readme.replace("@NAME@", name)),
        (".gitignore", "/target/\n.env\n.env.*\n".into()),
        ("src/main.rs", main.into()),
    ])
}

/// Explicit application-owned configuration; existing scaffold APIs stay unchanged.
pub fn create_config(
    name: &str,
    framework: &Path,
    dir: &Path,
    service: bool,
) -> Result<(), String> {
    validate_name(name).map_err(str::to_owned)?;
    crate::platform::fs::ensure_create_supported()?;
    let framework = crate::framework::validate(framework).map_err(str::to_owned)?;
    crate::framework::validate_config(&framework).map_err(str::to_owned)?;
    let entries = config_files(name, &framework, service)?;
    crate::platform::fs::create_project(dir, name, &entries)
}

pub(crate) fn config_files(
    name: &str,
    framework: &Path,
    service: bool,
) -> Result<Vec<(&'static str, String)>, String> {
    let mut entries = files(name, framework, service)?;
    let dependencies = "serde = { version = \"=1.0.229\", features = [\"derive\"] }\nsecrecy = { version = \"=0.10.3\", features = [\"serde\"] }\n";
    // Insert inside [dependencies], before the existing service metadata table.
    entries[0].1 = entries[0].1.replacen(
        "[dependencies]\n",
        &format!("[dependencies]\n{dependencies}"),
        1,
    );
    entries[0].1 = entries[0].1.replace(
        "features = [\"server-axum\"]",
        "features = [\"server-axum\", \"config\"]",
    );
    entries[1].1 = include_str!("../templates/config/README.md").replace("@NAME@", name);
    entries[2].1.push_str("!.env.example\n");
    entries[3].1 = if service {
        include_str!("../templates/config/cloud.rs").into()
    } else {
        include_str!("../templates/config/main.rs").into()
    };
    entries.extend([
        ("src/config.rs", include_str!("../templates/config/config.rs").into()),
        ("base.toml", include_str!("../templates/config/base.toml").into()),
        ("development.toml", "[service]\npublic_message = \"Development configuration\"\n".into()),
        ("staging.toml", "[service]\npublic_message = \"Staging configuration\"\n".into()),
        ("production.toml", "[service]\npublic_message = \"Production configuration\"\n".into()),
        (".env.example", "# Synthetic empty declaration only; supply your own value in an explicit private file.\nAPP_SERVICE__TOKEN=\n".into()),
    ]);
    Ok(entries)
}

/// Opt-in build-time CSS. This writes sources only and never invokes a tool.
pub fn create_tailwind(
    name: &str,
    framework: &Path,
    dir: &Path,
    service: bool,
    runtime_settings: bool,
) -> Result<(), String> {
    validate_name(name).map_err(str::to_owned)?;
    crate::platform::fs::ensure_create_supported()?;
    let framework = crate::framework::validate(framework).map_err(str::to_owned)?;
    if runtime_settings {
        crate::framework::validate_config(&framework).map_err(str::to_owned)?;
    }
    let entries = tailwind_files(name, &framework, service, runtime_settings)?;
    crate::platform::fs::create_project(dir, name, &entries)
}

pub(crate) fn tailwind_files(
    name: &str,
    framework: &Path,
    service: bool,
    runtime_settings: bool,
) -> Result<Vec<(&'static str, String)>, String> {
    let mut entries = if runtime_settings {
        config_files(name, framework, service)?
    } else {
        files(name, framework, service)?
    };
    entries[0].1 = entries[0].1.replacen(
        "[dependencies]\n",
        "[dependencies]\nsha2 = \"=0.10.9\"\n",
        1,
    );
    entries[0]
        .1
        .push_str("\n[dev-dependencies]\ntower = { version = \"0.5\", features = [\"util\"] }\n");
    // All replacements apply only to built-in, opt-in copies. The established
    // no-Tailwind templates and public scaffold entry points stay byte-identical.
    let main = &mut entries[3].1;
    let start = main.find("async fn home(").ok_or("missing built-in home")?;
    let end = if service {
        main.find("async fn live()")
    } else {
        main.find("#[tokio::main]")
    }
    .ok_or("missing built-in entry point")?;
    let (arguments, message) = if runtime_settings {
        (
            "    State(settings): State<Arc<config::Settings>>,\n",
            "&settings.service.public_message",
        )
    } else {
        ("", "\"Hello <Yetlet> & Rust!\"")
    };
    let handlers = include_str!("../templates/tailwind/handlers.rs")
        .replace("@ARGUMENTS@", arguments)
        .replace("@MESSAGE@", message);
    let (handlers, tests) = handlers
        .split_once("#[cfg(test)]")
        .ok_or("missing built-in tests")?;
    main.replace_range(start..end, handlers);
    *main = format!("mod tailwind;\n\n{main}");
    *main = main
        .replace("use yetlet::server_axum::{App, Page};", "use yetlet::server_axum::App;")
        .replace(".page(\"/\", \"Yetlet\", home)", ".get(\"/\", home)")
        .replace(
            "async fn main() -> Result<(), Box<dyn std::error::Error>> {",
            "async fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let stylesheet = tailwind::Stylesheet::load()?;",
        );
    if service {
        let original = if runtime_settings {
            "router(settings, readiness)"
        } else {
            "router(readiness)"
        };
        *main = main.replace(
            &format!("axum::serve(listener, {original})"),
            &format!("let app = tailwind::install({original}, stylesheet);\n    axum::serve(listener, app)"),
        );
    } else {
        *main = main.replace(
            "axum::serve(listener, app)",
            "axum::serve(listener, tailwind::install(app, stylesheet))",
        );
    }
    *main = main.replace(
        "let port: u16 = std::env::var(\"YETLET_PORT\").unwrap_or_else(|_| \"3000\".into()).parse()?;",
        "let port: u16 = std::env::var(\"YETLET_PORT\")\n        .unwrap_or_else(|_| \"3000\".into())\n        .parse()?;",
    );
    main.push_str("\n#[cfg(test)]");
    main.push_str(tests.trim_end());
    main.push('\n');
    // Plain SSR did not previously import Extension/Arc. Use qualified paths in
    // the shared handlers so the original profile imports remain sufficient.
    entries[1].1 = format!(
        "{}\n\n{}",
        include_str!("../templates/tailwind/README.md").replace("@NAME@", name),
        entries[1].1,
    );
    entries[2].1.push_str("/dist/\n");
    entries.extend([
        (
            "tailwind.css",
            include_str!("../templates/tailwind/input.css").into(),
        ),
        (
            "src/tailwind.rs",
            include_str!("../templates/tailwind/tailwind.rs").into(),
        ),
    ]);
    Ok(entries)
}

/// Opt-in Vite production sources only; no tool execution or dependency install.
pub fn create_vite(
    name: &str,
    framework: &Path,
    dir: &Path,
    service: bool,
    runtime_settings: bool,
    tailwind: bool,
) -> Result<(), String> {
    validate_name(name).map_err(str::to_owned)?;
    crate::platform::fs::ensure_create_supported()?;
    let framework = crate::framework::validate(framework).map_err(str::to_owned)?;
    if runtime_settings {
        crate::framework::validate_config(&framework).map_err(str::to_owned)?;
    }
    let entries = vite_files(name, &framework, service, runtime_settings, tailwind)?;
    crate::platform::fs::create_project(dir, name, &entries)
}

/// App-owned, foreground Vite dev tooling. Rust and production sources are identical.
pub fn create_vite_dev(
    name: &str,
    framework: &Path,
    dir: &Path,
    service: bool,
    runtime_settings: bool,
    tailwind: bool,
) -> Result<(), String> {
    validate_name(name).map_err(str::to_owned)?;
    crate::platform::fs::ensure_create_supported()?;
    let framework = crate::framework::validate(framework).map_err(str::to_owned)?;
    if runtime_settings {
        crate::framework::validate_config(&framework).map_err(str::to_owned)?;
    }
    let mut entries = vite_files(name, &framework, service, runtime_settings, tailwind)?;
    for (path, content) in &mut entries {
        match *path {
            "README.md" => {
                *content = format!(
                    "{}\n\n{content}",
                    include_str!("../templates/vite/DEV-README.md").replace("@NAME@", name)
                );
            }
            "package.json" => {
                *content = content.replace(
                    "\"build\": \"vite build\"",
                    "\"build\": \"vite build\",\n    \"dev\": \"vite --config vite.dev.config.mjs\"",
                );
            }
            _ => {}
        }
    }
    entries.extend([
        (
            "vite.dev.config.mjs",
            include_str!("../templates/vite/vite.dev.config.mjs").into(),
        ),
        (
            "vite.dev.bridge.mjs",
            include_str!("../templates/vite/vite.dev.bridge.mjs").into(),
        ),
    ]);
    crate::platform::fs::create_project(dir, name, &entries)
}

pub(crate) fn vite_files(
    name: &str,
    framework: &Path,
    service: bool,
    runtime_settings: bool,
    tailwind: bool,
) -> Result<Vec<(&'static str, String)>, String> {
    // Never pass through the standalone Tailwind generator: Vite owns CSS here.
    let mut entries = if runtime_settings {
        config_files(name, framework, service)?
    } else {
        files(name, framework, service)?
    };
    let serde = if runtime_settings {
        ""
    } else {
        "serde = { version = \"=1.0.229\", features = [\"derive\"] }\n"
    };
    entries[0].1 = entries[0].1.replacen(
        "[dependencies]\n",
        &format!("[dependencies]\n{serde}serde_json = \"=1.0.151\"\n"),
        1,
    );
    entries[0].1.push_str(
        "\n[dev-dependencies]\ntower = { version = \"0.5\", features = [\"util\"] }\ntempfile = \"=3.27.0\"\n",
    );
    let main = &mut entries[3].1;
    let start = main.find("async fn home(").ok_or("missing built-in home")?;
    let end = main
        .find(if service {
            "async fn live()"
        } else {
            "#[tokio::main]"
        })
        .ok_or("missing built-in entry point")?;
    let handlers = include_str!("../templates/vite/handlers.rs")
        .replace(
            "@SETTINGS_FIELD@",
            if runtime_settings {
                "    settings: config::Settings,\n"
            } else {
                ""
            },
        )
        .replace(
            "@MESSAGE@",
            if runtime_settings {
                "&state.settings.service.public_message"
            } else {
                "\"Hello <Yetlet> & Rust!\""
            },
        )
        .replace(
            "@PROOF_ID@",
            if tailwind {
                "tailwind-proof"
            } else {
                "plain-proof"
            },
        );
    main.replace_range(start..end, &format!("{handlers}\n"));
    *main = format!("mod vite;\n\n{main}")
        .replace("use yetlet::server_axum::{App, Page};", "use yetlet::server_axum::App;")
        .replace("use axum::extract::{Path, State};", "use axum::extract::Path;")
        .replace("extract::{Path, State},", "extract::Path,")
        .replace("use std::sync::Arc;\n", "")
        .replace("use axum::{\n    Extension,\n    extract::Path,\n    http::StatusCode,\n};", "use axum::{Extension, extract::Path, http::StatusCode};")
        .replace(".page(\"/\", \"Yetlet\", home)", ".get(\"/\", home)")
        .replace(".with_state(Arc::new(settings))", ".with_state(state)")
        .replace(
            "async fn main() -> Result<(), Box<dyn std::error::Error>> {",
            "async fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let assets = vite::Snapshot::load_with_env()?;",
        );
    if !runtime_settings {
        *main = main.replace(
            "        .into_router()",
            "        .with_state(state)\n        .into_router()",
        );
    }
    let state = format!(
        "    let state = std::sync::Arc::new(AppState {{\n        assets: assets.clone(),\n{}    }});\n",
        if runtime_settings {
            "        settings,\n"
        } else {
            ""
        },
    );
    if service {
        *main = main
            .replace(
                "fn router(settings: config::Settings, readiness: Arc<AtomicBool>)",
                "fn router(state: Arc<AppState>, readiness: Arc<AtomicBool>)",
            )
            .replace(
                "fn router(readiness: Arc<AtomicBool>)",
                "fn router(state: Arc<AppState>, readiness: Arc<AtomicBool>)",
            )
            .replace(
                "    let readiness = Arc::new(AtomicBool::new(true));",
                &format!("{state}    let readiness = Arc::new(AtomicBool::new(true));"),
            )
            .replace("router(settings, readiness)", "router(state, readiness)")
            .replace(
                "axum::serve(listener, router(readiness))",
                "axum::serve(listener, router(state, readiness))",
            )
            .replace(
                "axum::serve(listener, router(state, readiness))",
                "axum::serve(\n        listener,\n        router(state, readiness).merge(vite::router(assets)),\n    )",
            )
            .replace(
                "        .with_graceful_shutdown(shutdown)\n        .await?;",
                "    .with_graceful_shutdown(shutdown)\n    .await?;",
            );
    } else {
        *main = main
            .replace(
                "    let app: axum::Router",
                &format!("{state}    let app: axum::Router"),
            )
            .replace(
                "axum::serve(listener, app)",
                "axum::serve(listener, app.merge(vite::router(assets)))",
            );
    }
    // Format only this opt-in copy; established template bytes are untouched.
    *main = main.replace(
        "let port: u16 = std::env::var(\"YETLET_PORT\").unwrap_or_else(|_| \"3000\".into()).parse()?;",
        "let port: u16 = std::env::var(\"YETLET_PORT\")\n        .unwrap_or_else(|_| \"3000\".into())\n        .parse()?;",
    );
    entries[1].1 = entries[1]
        .1
        .replace(
            "No JavaScript or WASM is loaded.",
            "The selected Vite entry enhances native SSR; no WASM is loaded.",
        )
        .replace(
            "No JS, DI container, secret discovery,",
            "No DI container, secret discovery,",
        )
        .replace("No JS/WASM, Controller, DI,", "No WASM, Controller, DI,");
    entries[1].1 = format!(
        "{}\n\n{}",
        include_str!("../templates/vite/README.md").replace("@NAME@", name),
        entries[1].1
    );
    entries[2].1.push_str("/dist/\n/node_modules/\n");
    let config = include_str!("../templates/vite/vite.config.mjs")
        .replace(
            "@PLUGIN_IMPORT@",
            if tailwind {
                "import tailwindcss from '@tailwindcss/vite';\n"
            } else {
                ""
            },
        )
        .replace("@PLUGIN@", if tailwind { "tailwindcss()" } else { "" });
    entries.extend([
        (
            "src/vite.rs",
            include_str!("../templates/vite/vite.rs").into(),
        ),
        ("vite.config.mjs", config),
        (
            "tsconfig.json",
            include_str!("../templates/vite/tsconfig.json").into(),
        ),
        (
            "package.json",
            if tailwind {
                include_str!("../templates/vite/tailwind-package.json")
            } else {
                include_str!("../templates/vite/plain-package.json")
            }
            .into(),
        ),
        (
            "package-lock.json",
            if tailwind {
                include_str!("../templates/vite/tailwind-package-lock.json")
            } else {
                include_str!("../templates/vite/plain-package-lock.json")
            }
            .into(),
        ),
        ("app.ts", include_str!("../templates/vite/app.ts").into()),
        (
            "app.css",
            if tailwind {
                include_str!("../templates/vite/tailwind.css")
            } else {
                include_str!("../templates/vite/plain.css")
            }
            .into(),
        ),
        (
            "companion.ts",
            include_str!("../templates/vite/companion.ts").into(),
        ),
        (
            "shared.ts",
            include_str!("../templates/vite/shared.ts").into(),
        ),
        (
            "shared.css",
            include_str!("../templates/vite/shared.css").into(),
        ),
        ("lazy.ts", include_str!("../templates/vite/lazy.ts").into()),
        (
            "lazy.css",
            include_str!("../templates/vite/lazy.css").into(),
        ),
        (
            "mark.svg",
            include_str!("../templates/vite/mark.svg").into(),
        ),
    ]);
    Ok(entries)
}

/// Release-facing creation: file publication only, with no source checkout lookup.
/// The declarations remain unpublished until the release qualification gates pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Template {
    #[default]
    Native,
    Browser,
}

/// Publish the explicitly selected built-in template, independent of runtime settings.
pub fn create_selected(
    name: &str,
    dir: &Path,
    template: Template,
    service: bool,
    runtime_settings: bool,
    tailwind: bool,
    vite: bool,
) -> Result<(), String> {
    validate_name(name).map_err(str::to_owned)?;
    crate::platform::fs::ensure_create_supported()?;
    let entries = if template == Template::Browser {
        if service {
            return Err("browser target does not select the native --service profile".into());
        }
        crate::browser::files(name, tailwind)?
    } else {
        selected_files(name, service, runtime_settings, tailwind, vite)?
    };
    crate::platform::fs::create_project(dir, name, &entries)
}

pub(crate) fn selected_files(
    name: &str,
    service: bool,
    runtime_settings: bool,
    tailwind: bool,
    vite: bool,
) -> Result<Vec<(&'static str, String)>, String> {
    // Vite needs the same typed runtime settings as the CLI. Plain SSR stays opt-in.
    let runtime_settings = runtime_settings || vite;
    // These paths only seed the established template assembly; no filesystem lookup.
    let framework = Path::new("__yetlet_release_template__");
    let mut entries = if vite {
        vite_files(name, framework, service, true, tailwind)?
    } else if tailwind {
        tailwind_files(name, framework, service, runtime_settings)?
    } else if runtime_settings {
        config_files(name, framework, service)?
    } else {
        files(name, framework, service)?
    };
    let mut manifest: toml::Value = entries[0]
        .1
        .parse()
        .map_err(|_| "invalid built-in manifest")?;
    let dependency = manifest["dependencies"]["yetlet"]
        .as_table_mut()
        .ok_or("invalid built-in framework dependency")?;
    dependency.remove("path");
    dependency.insert(
        "version".into(),
        toml::Value::String(format!("={}", crate::framework::package_version())),
    );
    entries[0].1 = toml::to_string(&manifest).map_err(|_| "cannot render generated manifest")?;
    let cli = format!(
        "\n[cli.rust]\nwatch = false\n\n[cli.vite]\nenabled = {vite}\nroot = \"frontend/\"\nauto_start = {vite}\nauto_build = {vite}\n\n[cli.generate.k8s]\nreplicas = 1\n"
    );
    if runtime_settings {
        for (path, content) in &mut entries {
            if *path == "src/config.rs" {
                *content = content.replace(
                    "pub struct Settings {",
                    "pub struct Settings {\n    pub cli: Cli,",
                );
                content.push_str(include_str!("../templates/config/cli.rs"));
                *content = content.replace(
                    "        if self.service.token",
                    "        self.cli.validate()?;\n        if self.service.token",
                );
            }
            if *path == "base.toml" {
                content.push_str(&cli);
            }
        }
    } else {
        entries.push(("base.toml", cli.clone()));
        for path in ["development.toml", "staging.toml", "production.toml"] {
            entries.push((path, "# Public CLI settings inherit base.toml.\n".into()));
        }
    }
    for (path, content) in &mut entries {
        if matches!(
            *path,
            "development.toml" | "staging.toml" | "production.toml"
        ) {
            content.push_str(
                "\n# Override declared [cli.rust], [cli.vite] or [cli.generate.k8s] values here.\n",
            );
        }
    }
    if vite {
        for (path, content) in &mut entries {
            *path = match *path {
                "package.json" => "frontend/package.json",
                "package-lock.json" => "frontend/package-lock.json",
                "vite.config.mjs" => "frontend/vite.config.mjs",
                "tsconfig.json" => "frontend/tsconfig.json",
                "app.ts" => "frontend/app.ts",
                "app.css" => "frontend/app.css",
                "companion.ts" => "frontend/companion.ts",
                "shared.ts" => "frontend/shared.ts",
                "shared.css" => "frontend/shared.css",
                "lazy.ts" => "frontend/lazy.ts",
                "lazy.css" => "frontend/lazy.css",
                "mark.svg" => "frontend/mark.svg",
                other => other,
            };
            match *path {
                "frontend/package.json" => *content = content.replace(
                    "\"build\": \"vite build\"", "\"build\": \"vite build\",\n    \"dev\": \"vite --config vite.dev.config.mjs\""
                ),
                "frontend/vite.config.mjs" => {
                    *content = content
                        .replace("No dotenv discovery", "Frontend-root dotenv only")
                        .replace("envDir: false", "envDir: root");
                    if tailwind {
                        *content = format!("{}{}", include_str!("../templates/vite/selected-tailwind-source.mjs"), content)
                            .replace("plugins: [tailwindcss()]", "plugins: [rustSource, tailwindcss()]");
                    }
                    *content = format!("{}{}", include_str!("../templates/vite/selected-dotenv.mjs"), content)
                        .replace("plugins: [", "plugins: [frontendDotenv, ");
                }
                "frontend/app.css" if tailwind => *content = content.replace("./src/main.rs", "../src/main.rs"),
                "src/main.rs" => *content = content
                    .replace("    let assets = vite::Snapshot::load_with_env()?;\n", "")
                    .replace("    let address = settings.validate(", "    let assets = vite::Snapshot::load_selected(&settings.cli.vite.root)?;\n    let address = settings.validate("),
                "src/vite.rs" => {
                    let start = content.find("    pub(crate) fn load_with_env()").ok_or("missing snapshot loader")?;
                    let end = content[start..].find("        if snapshot.images()").ok_or("missing snapshot validation")? + start;
                    content.replace_range(start..end, include_str!("../templates/vite/selected-loader.rs"));
                    *content = content.replace("let root = Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"dist\");", "let root = Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(crate::config::load().expect(\"app settings required\").cli.vite.root).join(\"dist\");");
                }
                _ => {}
            }
        }
        entries.extend([
            ("frontend/.env.example", "# Frontend values are public when imported. Never store backend credentials here.\nVITE_PUBLIC_MESSAGE=Yetlet\n".into()),
            ("frontend/vite.dev.config.mjs", include_str!("../templates/vite/vite.dev.config.mjs").into()),
            ("frontend/vite.dev.bridge.mjs", include_str!("../templates/vite/vite.dev.bridge.mjs").into()),
        ]);
        entries[2]
            .1
            .push_str("/frontend/dist/\n/frontend/node_modules/\n!frontend/.env.example\n");
    }
    entries[1].1 = include_str!("../templates/SELECTED-README.md")
        .replace("@NAME@", name)
        .replace("@VERSION@", env!("CARGO_PKG_VERSION"))
        .replace("@SDK_VERSION@", &crate::framework::package_version());
    if runtime_settings {
        entries[1]
            .1
            .push_str(include_str!("../templates/config/SELECTED-README.md"));
    }
    if vite {
        entries[1]
            .1
            .push_str(include_str!("../templates/vite/SELECTED-README.md"));
    } else if tailwind {
        entries[1]
            .1
            .push_str(include_str!("../templates/tailwind/SELECTED-README.md"));
    }
    if runtime_settings {
        entries[0].1 = entries[0].1.replacen("[dependencies]\n", "[dependencies]\naxum-server = { version = \"=0.8.0\", default-features = false, features = [\"tls-rustls-no-provider\"], optional = true }\nrustls = { version = \"0.23\", default-features = false, features = [\"ring\", \"std\", \"tls12\"], optional = true }\n", 1);
        entries[0]
            .1
            .push_str("\n[features]\nyetlet-dev-tls = [\"dep:axum-server\", \"dep:rustls\"]\n");
        entries[2].1.push_str("/.dev/\n");
        for (path, content) in &mut entries {
            match *path {
                "base.toml" => {
                    *content = content.replacen("[service]", "[server.tls]\nenabled = false\ncertificate = \"\"\nprivate_key = \"\"\n\n[service]", 1);
                }
                "src/config.rs" => {
                    *content = content.replacen(
                        "    port: u16,",
                        "    port: u16,\n    #[serde(default)]\n    pub tls: Tls,",
                        1,
                    );
                    content.push_str(&include_str!("../templates/config/tls.rs").replace(
                        "@FRONTEND_ROOT@",
                        "self.cli.vite.root.trim_end_matches('/')",
                    ));
                }
                "src/main.rs" => {
                    // Resolve/parse TLS before any listener exists or Settings moves into State.
                    let address_end = if service {
                        "let address = settings.validate(true)?;"
                    } else {
                        "let address = settings.validate(false)?;"
                    };
                    *content = content.replace(address_end, &format!("{address_end}\n    let https = settings.server.tls.enabled;\n    let tls = settings.tls().await?;"));
                    *content = content.replace("println!(\"Listening on http://{}\", listener.local_addr()?);", "println!(\"Listening on {}://{}\", if https { \"https\" } else { \"http\" }, listener.local_addr()?);");
                    let start = content
                        .find("axum::serve(")
                        .ok_or("missing selected server")?;
                    let end = start
                        + content[start..]
                            .find(".await?;")
                            .ok_or("missing selected serve await")?;
                    let expression = content[start..end]
                        .split(".with_graceful_shutdown")
                        .next()
                        .unwrap()
                        .trim_end();
                    let close = expression
                        .rfind(')')
                        .ok_or("missing selected serve close")?;
                    let shutdown = if service {
                        "shutdown"
                    } else {
                        "std::future::pending::<()>()"
                    };
                    let expression = format!(
                        "{}, {shutdown}).await?;",
                        expression[..close].trim_end().trim_end_matches(',')
                    )
                    .replacen("axum::serve(", "config::serve(", 1)
                    .replacen("listener,", "listener, tls,", 1);
                    content.replace_range(start..end + ".await?;".len(), &expression);
                }
                _ => {}
            }
        }
    }

    Ok(entries)
}
