use std::process::ExitCode;
use yetlet_cli::{Cli, HELP, app, compose, docker, doctor, parse, scaffold};

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match parse(&args) {
        Ok(Cli::Help) => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(Cli::Doctor { framework, json }) => {
            let report = doctor::check(framework.as_deref());
            println!("{}", report.render(json));
            ExitCode::from(u8::from(!report.ok()))
        }
        Ok(Cli::Build { manifest, release }) => application(&manifest, false, release),
        Ok(Cli::Dev { manifest }) => application(&manifest, true, false),
        Ok(command @ Cli::GenerateK8s { .. }) => match command.generate_k8s() {
            Ok(()) => {
                println!(
                    "Created Kubernetes configuration. Read its README before operator-selected validation or deployment."
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("generate k8s failed: {error}");
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
                eprintln!("generate compose failed: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(Cli::GenerateDocker { manifest, output }) => {
            match docker::generate(&manifest, &output) {
                Ok(()) => {
                    println!(
                        "Created Docker configuration. Read its README before preparing the two sanitized source exports."
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("generate docker failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(Cli::NewViteDev {
            name,
            framework,
            dir,
            cloud,
            with_config,
            with_tailwind,
        }) => match scaffold::create_vite_dev(
            &name,
            &framework,
            &dir,
            cloud,
            with_config,
            with_tailwind,
        ) {
            Ok(()) => {
                println!(
                    "Created {name}. Read its README for the explicit two-terminal dev profile."
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("new failed: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(Cli::NewVite {
            name,
            framework,
            dir,
            cloud,
            with_config,
            with_tailwind,
        }) => {
            match scaffold::create_vite(&name, &framework, &dir, cloud, with_config, with_tailwind)
            {
                Ok(()) => {
                    println!(
                        "Created {name}. Follow its README for explicit frontend and Cargo builds."
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("new failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(Cli::New {
            name,
            framework,
            dir,
            cloud,
            with_config,
            with_tailwind,
        }) => match if with_tailwind {
            scaffold::create_tailwind(&name, &framework, &dir, cloud, with_config)
        } else if with_config {
            scaffold::create_config(&name, &framework, &dir, cloud)
        } else if cloud {
            scaffold::create_cloud(&name, &framework, &dir)
        } else {
            scaffold::create(&name, &framework, &dir)
        } {
            Ok(()) => {
                println!(
                    "Created {name}. Follow its README for the explicit offline lockfile step."
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("new failed: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{error}\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn application(manifest: &std::path::Path, dev: bool, release: bool) -> ExitCode {
    match app::run(manifest, dev, release) {
        Ok(completion) => completion.finish(),
        Err(error) => {
            eprintln!(
                "{} failed: {}",
                if dev { "dev" } else { "build" },
                error.message
            );
            ExitCode::from(error.code as u8)
        }
    }
}
