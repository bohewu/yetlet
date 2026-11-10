use std::process::ExitCode;
use yetlet_cli::{Cli, VERSION, app, compose, docker, doctor, inspect, parse, usage};

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args
        .first()
        .is_some_and(|arg| arg == "__yetlet_style_worker")
    {
        return match args.as_slice() {
            [_, operation] => match operation
                .to_str()
                .ok_or_else(|| "invalid private style stage".to_owned())
                .and_then(app::internal_style_worker)
            {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            },
            _ => ExitCode::FAILURE,
        };
    }
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
        Ok(command @ Cli::New { .. }) => match command.create_new() {
            Ok(()) => {
                println!(
                    "Created project with exact SDK and Cargo.lock. Follow its README for build prerequisites."
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("CLI_NEW: {}", error.message);
                if error.code >= 128 {
                    app::Completion::Cancelled(error.code - 128).finish();
                }
                ExitCode::from(error.code as u8)
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
