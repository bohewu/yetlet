//! Read-only public-settings intentions; no process or frontend readiness checks.
use crate::{platform::fs, settings};
use serde::Serialize;
use std::{fmt::Write, path::Path};

const LIMITATIONS: &str = "Settings-derived intentions only. Tools, assets, TLS files, secrets, runtime startup and readiness are not verified. Browser watch uses full reload without Rust state restoration.";

#[derive(Serialize)]
struct Stage {
    selected: bool,
    reason: &'static str,
}

#[derive(Serialize)]
struct Plan {
    supported: bool,
    rust: Stage,
    rust_watch: Stage,
    vite: Stage,
}

fn plan(settings: &settings::Settings, command: &str) -> Plan {
    let browser = settings.target == settings::Target::Browser;
    let build = command == "build";
    let browser_watch = browser && command == "watch";
    let vite = if browser_watch {
        Stage {
            selected: settings.vite.auto_start && settings.vite.auto_build,
            reason: if !settings.vite.auto_start {
                "cli.vite.auto_start is false; browser watch is skipped"
            } else if !settings.vite.auto_build {
                "cli.vite.auto_build is false; browser watch is skipped"
            } else {
                "auto_start and auto_build select npm run dev for browser WASM rebuild/full reload"
            },
        }
    } else if !settings.vite.enabled {
        Stage {
            selected: false,
            reason: "cli.vite.enabled is false",
        }
    } else if build {
        Stage {
            selected: settings.vite.auto_build,
            reason: if settings.vite.auto_build {
                "cli.vite.enabled and auto_build select npm run build"
            } else {
                "cli.vite.auto_build is false"
            },
        }
    } else {
        Stage {
            selected: settings.vite.auto_start,
            reason: if !settings.vite.auto_start {
                "cli.vite.auto_start is false"
            } else if browser {
                "cli.vite.enabled and auto_start select npm run dev for static preview"
            } else {
                "cli.vite.enabled and auto_start select npm run dev alongside Rust"
            },
        }
    };
    Plan {
        supported: true,
        rust: Stage {
            selected: !browser,
            reason: if browser {
                "browser target has no native Cargo stage; frontend scripts own WASM builds"
            } else if build {
                "native frozen Cargo build follows any selected frontend build success; --release selects the Cargo profile"
            } else {
                "native frozen Cargo run; selected frontend startup precedes Rust"
            },
        },
        rust_watch: Stage {
            selected: if browser_watch {
                settings.vite.auto_start && settings.vite.auto_build
            } else {
                !browser && !build && (command == "watch" || settings.rust.watch)
            },
            reason: if browser_watch {
                if settings.vite.auto_start && settings.vite.auto_build {
                    "Vite watches app Rust inputs; serialized frozen WASM builds publish complete generations then fully reload"
                } else {
                    "browser watch requires both auto_start and auto_build"
                }
            } else if browser {
                "browser target has no native Rust watch"
            } else if build {
                "build never watches Rust"
            } else if command == "watch" {
                "watch forces Rust watch after public settings validation"
            } else if settings.rust.watch {
                "cli.rust.watch is true"
            } else {
                "cli.rust.watch is false"
            },
        },
        vite,
    }
}

/// Explain the selected public plan without executing it or accessing secrets.
pub fn explain(manifest: &Path, json: bool) -> Result<String, String> {
    let app = fs::open_app(manifest)?;
    let (settings, environment) = settings::load_selected(&app, std::env::vars_os())?;
    let target = if settings.target == settings::Target::Browser {
        "browser"
    } else {
        "native"
    };
    let commands = ["build", "run", "watch"].map(|command| (command, plan(&settings, command)));
    if json {
        let report = serde_json::json!({
            "schema_version": 1,
            "scope": "project-execution-plan",
            "environment": environment,
            "target": target,
            "rust_watch": settings.rust.watch,
            "vite": {
                "enabled": settings.vite.enabled,
                "root": settings.vite.root,
                "auto_build": settings.vite.auto_build,
                "auto_start": settings.vite.auto_start,
            },
            "commands": commands.iter().map(|(name, plan)| ((*name).to_owned(), serde_json::json!(plan))).collect::<serde_json::Map<_, _>>(),
            "limitations": LIMITATIONS,
        });
        // Keep machine output valid JSON and safe even when printed to a terminal.
        // DEL and non-ASCII controls/formatting characters use JSON UTF-16 escapes too.
        let mut output = String::new();
        for ch in report.to_string().chars() {
            if ch.is_ascii() && ch != '\u{7f}' {
                output.push(ch);
            } else {
                for unit in ch.encode_utf16(&mut [0; 2]) {
                    write!(output, "\\u{unit:04x}").unwrap();
                }
            }
        }
        Ok(output)
    } else {
        let mut output = format!(
            "Project execution plan (settings only)\nEnvironment: {environment}\nTarget: {target}\nConfigured Rust watch: {}\nVite: enabled={}, root=\"{}\", auto_build={}, auto_start={}\n",
            settings.rust.watch,
            settings.vite.enabled,
            settings.vite.root.escape_default(),
            settings.vite.auto_build,
            settings.vite.auto_start,
        );
        for (command, plan) in commands {
            writeln!(output, "{command}: supported={}", plan.supported).unwrap();
            for (name, stage) in [
                ("Rust", plan.rust),
                ("Rust watch", plan.rust_watch),
                ("Vite", plan.vite),
            ] {
                writeln!(
                    output,
                    "  {name}: {} ({})",
                    if stage.selected {
                        "selected"
                    } else {
                        "skipped"
                    },
                    stage.reason
                )
                .unwrap();
            }
        }
        output.push_str(LIMITATIONS);
        Ok(output)
    }
}
