//! Repository-only tooling. No framework runtime dependencies.
pub mod measure;
pub mod process;
pub mod qualify;
pub mod release;

use std::{ffi::OsString, path::PathBuf};

pub mod build;
mod build_templates;
pub mod css;
pub mod docs;
pub mod samples;
pub mod smoke;
pub mod tooling;

pub const HELP: &str = "Yetlet repository xtask (unpublished)\n\nUsage: cargo xtask [--help | doctor [--capability foundation] [--json] | build <builder> [options] | check <docs|samples> | smoke <suite> [options]]\n\nBuilders: browser, keyed-bindings, multi-island-dashboard, render-bench [--release];\nscene-island (offline release); form-css-proof, board-assets [--check | --fetch].\nNative smoke (no options): hello-world, rest-api, cart, middleware-lab, forms-validation,\nfile-server, sqlite-crud, sse-notifications, auth-notes, modular-shop.\nBrowser smoke [--profile debug|release]: browser-chrome, browser-firefox,\nkeyed-bindings-chrome, keyed-bindings-firefox, multi-island-dashboard-chrome,\nmulti-island-dashboard-firefox, forms-accessibility-firefox.\nFixed-build browser smoke (no options): form-css-proof-chrome, board-chrome,\nboard-firefox, scene-chrome, scene-firefox, scene-lifecycle-firefox,\nscene-picking-firefox, scene-candidate-firefox, scene-support-firefox.\nBoard requires release native binary. Scene requires debug nested native binary\nand current Rust scene-island assets. Scene selected Firefox uses temporary\ndom.webgpu.enabled=true; Support requests no WebGPU preference. Lifecycle/picking/\ncandidate/support require YT075_FIREFOX_HEADFUL_X11=1. Missing capability is nonzero.\nMeasurements: baseline, render-strategies, sample-dx, scene-dynamic.\nQualification: board [--profile bounded|full|full-sync|sync-12m|staggered-12m],\nlocal-operations, multi-instance [--report PATH], redis-three-app.\nRelease: checksums <create|verify> <directory>; private-rebuild [--cohort original-nine|sixteen] (manual CI only).\nrelease preparation --expected-commit <40-lowercase-hex> (sixteen only, manual CI).\nNo installs or implicit downloads. Doctor is read-only. The alias is root-only.\nExit: 0 success, 1 validation, 2 usage, 124 timeout, 125 infrastructure, 130 cancellation.\n";

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Help,
    Measure {
        suite: String,
        args: Vec<OsString>,
    },
    Qualify {
        suite: String,
        args: Vec<OsString>,
    },
    #[cfg(target_os = "linux")]
    Custody(Box<release::custody::Options>),
    ReleasePreparation {
        expected_commit: String,
    },
    PrivateRebuild {
        cohort: release::private::Cohort,
    },
    Checksums {
        operation: String,
        directory: PathBuf,
    },
    Smoke {
        suite: String,
        args: Vec<OsString>,
    },
    Fixture {
        suite: String,
        base: String,
    },
    Build {
        kind: build::Build,
        options: build::Options,
    },
    Check {
        samples: bool,
    },
    Doctor {
        json: bool,
    },
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Cli, &'static str> {
    let args: Vec<_> = args.into_iter().collect();
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        return Ok(Cli::Help);
    }
    if args
        .first()
        .is_some_and(|s| s == "measure" || s == "qualify")
    {
        let suite = args
            .get(1)
            .and_then(|s| s.to_str())
            .ok_or("missing tool suite")?;
        if args[2..] == ["--help"] {
            return Ok(Cli::Help);
        }
        if args[0] == "measure" {
            measure::Options::parse(suite, &args[2..])?;
            return Ok(Cli::Measure {
                suite: suite.into(),
                args: args[2..].to_vec(),
            });
        }
        qualify::validate(suite, &args[2..])?;
        return Ok(Cli::Qualify {
            suite: suite.into(),
            args: args[2..].to_vec(),
        });
    }
    if args.first().is_some_and(|s| s == "release") {
        #[cfg(target_os = "linux")]
        if args.get(1).is_some_and(|s| s == "custody") {
            return Ok(Cli::Custody(Box::new(release::custody::Options::parse(
                &args[2..],
            )?)));
        }
        if args.get(1).is_some_and(|s| s == "preparation") {
            if args.len() != 4 || args[2] != "--expected-commit" {
                return Err("release preparation --expected-commit <40-lowercase-hex>");
            }
            let expected_commit = args[3].to_str().ok_or("commit encoding")?;
            if !release::preparation::commit_token(expected_commit) {
                return Err("expected commit must be 40 lowercase hexadecimal characters");
            }
            return Ok(Cli::ReleasePreparation {
                expected_commit: expected_commit.into(),
            });
        }
        if args.get(1).is_some_and(|s| s == "private-rebuild") {
            return Ok(Cli::PrivateRebuild {
                cohort: release::private::Cohort::parse(&args[2..])?,
            });
        }
        if args.len() != 4 || args[1] != "checksums" || (args[2] != "create" && args[2] != "verify")
        {
            return Err("release checksums <create|verify> <directory>");
        }
        return Ok(Cli::Checksums {
            operation: args[2].to_str().unwrap().into(),
            directory: args[3].clone().into(),
        });
    }
    if args.first().is_some_and(|s| s == "smoke") {
        let suite = args
            .get(1)
            .and_then(|s| s.to_str())
            .ok_or("missing smoke suite")?;
        if !smoke::native::SUITES.contains(&suite) && !smoke::BROWSER_SUITES.contains(&suite) {
            return Err("unknown smoke suite");
        }
        if args[2..] == ["--help"] || args[2..] == ["-h"] {
            return Ok(Cli::Help);
        }
        if smoke::native::SUITES.contains(&suite)
            || suite == "form-css-proof-chrome"
            || suite.starts_with("board-")
            || suite.starts_with("scene-")
        {
            if args.len() != 2 {
                return Err("this smoke accepts no options");
            }
        } else {
            smoke::profile(&args[2..], smoke::default_profile(suite))
                .map_err(|_| "expected --profile debug|release")?;
        }
        return Ok(Cli::Smoke {
            suite: suite.into(),
            args: args[2..].to_vec(),
        });
    }
    if args.first().is_some_and(|s| s == "fixture") {
        if args[1..] == ["--help"] {
            return Ok(Cli::Help);
        }
        if args.len() != 3 {
            return Err("fixture needs suite and owned base URL");
        }
        let suite = args[1].to_str().ok_or("fixture name must be UTF-8")?;
        if !["redis-cookie-browser", "redis-three-app-browser"].contains(&suite) {
            return Err("unknown fixture");
        }
        let base = args[2].to_str().ok_or("fixture URL must be UTF-8")?;
        smoke::fixtures::validate_base(base)
            .map_err(|_| "fixture URL must be loopback HTTP with explicit port")?;
        return Ok(Cli::Fixture {
            suite: suite.into(),
            base: base.into(),
        });
    }
    if args.first().is_some_and(|s| s == "build") {
        let kind = args
            .get(1)
            .and_then(|s| s.to_str())
            .and_then(build::Build::parse)
            .ok_or("unknown or missing builder")?;
        let mut options = build::Options::default();
        for flag in &args[2..] {
            if flag == "--release"
                && !kind.css()
                && kind != build::Build::SceneIsland
                && !options.release
            {
                options.release = true;
            } else if flag == "--check" && kind.css() && !options.check && !options.fetch {
                options.check = true;
            } else if flag == "--fetch" && kind.css() && !options.fetch && !options.check {
                options.fetch = true;
            } else {
                return Err("unknown, incompatible or repeated builder flag");
            }
        }
        return Ok(Cli::Build { kind, options });
    }
    if args.first().is_some_and(|s| s == "check") {
        if args.len() != 2 {
            return Err("check needs exactly docs or samples");
        }
        return match args[1].to_str() {
            Some("docs") => Ok(Cli::Check { samples: false }),
            Some("samples") => Ok(Cli::Check { samples: true }),
            _ => Err("unknown check"),
        };
    }
    if args.first().is_none_or(|s| s != "doctor") {
        return Err("unknown command or flag");
    }
    let mut json = false;
    let mut capability = false;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--json" && !json {
            json = true;
        } else if args[i] == "--capability" && !capability {
            i += 1;
            if args.get(i).is_none_or(|s| s != "foundation") {
                return Err("supported capability: foundation");
            }
            capability = true;
        } else {
            return Err("unknown or repeated doctor flag");
        }
        i += 1;
    }
    Ok(Cli::Doctor { json })
}

/// Independent of the invocation cwd; user paths remain native and caller-relative.
pub fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a repository parent")
        .to_owned()
}
