use std::process::ExitCode;
use yetlet_cli::{Cli, HELP, app, doctor, parse, scaffold};

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
        Ok(Cli::New {
            name,
            framework,
            dir,
        }) => match scaffold::create(&name, &framework, &dir) {
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
