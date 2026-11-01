use std::process::ExitCode;
use yetlet_cli::{Cli, VERSION, app, compose, docker, doctor, inspect, parse, scaffold, usage};

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match parse(&args) {
        Ok(Cli::Help(help)) => {
            print!("{help}");
            ExitCode::SUCCESS
        }
        Ok(Cli::Key(command)) => match yetlet_cli::key::execute(&command) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("CLI_KEY: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(Cli::Version) => {
            println!("{VERSION}");
            ExitCode::SUCCESS
        }
        Ok(Cli::Doctor { json }) => {
            let report = doctor::check();
            println!("{}", report.render(json));
            ExitCode::from(report.exit_code())
        }
        Ok(Cli::Inspect { manifest, json }) => match inspect::explain(&manifest, json) {
            Ok(report) => {
                println!("{report}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("CLI_PROJECT inspect: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(Cli::Build { manifest, release }) => application(&manifest, false, release, false),
        Ok(command @ Cli::BuildStatic { .. }) => match command.build_static() {
            Ok(completion) => completion.finish(),
            Err(error) => {
                eprintln!("CLI_STATIC: {}", error.message);
                ExitCode::from(error.code as u8)
            }
        },
        Ok(Cli::Run { manifest }) => application(&manifest, true, false, false),
        Ok(Cli::Watch { manifest }) => application(&manifest, true, false, true),
        Ok(command @ Cli::GenerateK8s { .. }) => match command.generate_k8s() {
            Ok(()) => {
                println!(
                    "Created Kubernetes configuration. Read its README before operator-selected validation or deployment."
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("CLI_SOURCE_UNSUPPORTED generate k8s: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(Cli::GenerateCompose {
            manifest,
            output,
            image,
        }) => match compose::generate(&manifest, &output, &image) {
            Ok(()) => {
                println!(
                    "Created Compose configuration. Read its README before using your separately built local image."
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("CLI_SOURCE_UNSUPPORTED generate compose: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(Cli::GenerateDocker { manifest, output }) => {
            match docker::generate(&manifest, &output) {
                Ok(()) => {
                    println!(
                        "Created Docker configuration. Read its README before preparing the sanitized source exports."
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("CLI_SOURCE_UNSUPPORTED generate docker: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(Cli::New {
            name,
            dir,
            template,
            service,
            runtime_settings,
            tailwind,
            vite,
        }) => match scaffold::create_selected(
            &name,
            &dir,
            template,
            service,
            runtime_settings,
            tailwind,
            vite,
        ) {
            Ok(()) => {
                println!(
                    "Created {name}. Follow its README for explicit dependency and lockfile preparation."
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("CLI_PUBLISH new: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("CLI_USAGE parser: {error}\n\n{}", usage(&args));
            ExitCode::from(2)
        }
    }
}

fn application(manifest: &std::path::Path, run: bool, release: bool, watch: bool) -> ExitCode {
    let result = if watch {
        app::watch(manifest)
    } else {
        app::run(manifest, run, release)
    };
    match result {
        Ok(completion) => completion.finish(),
        Err(error) => {
            let code = if error.code == 125 {
                "CLI_SPAWN"
            } else {
                "CLI_PROJECT"
            };
            eprintln!(
                "{code} {}: {}",
                if watch {
                    "watch"
                } else if run {
                    "run"
                } else {
                    "build"
                },
                error.message
            );
            ExitCode::from(error.code as u8)
        }
    }
}
