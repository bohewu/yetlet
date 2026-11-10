//! Existing source-export/template tests exercise file assembly, not SDK discovery.
//! Resolver and successful-new locking are covered by sdk_version tests.
use std::{ffi::OsString, path::Path};

pub fn create(root: &Path, name: &str, template: &str, options: &[&str]) -> Result<(), String> {
    let mut args = vec![
        OsString::from("new"),
        name.into(),
        "--parent".into(),
        root.into(),
        "--template".into(),
        template.into(),
    ];
    args.extend(options.iter().map(OsString::from));
    match yetlet_cli::parse(&args).map_err(str::to_owned)? {
        yetlet_cli::Cli::New {
            name,
            dir,
            template,
            service,
            runtime_settings,
            tailwind,
            vite,
            ..
        } => yetlet_cli::scaffold::create_selected(
            &name,
            &dir,
            template,
            service,
            runtime_settings,
            tailwind,
            vite,
        ),
        _ => unreachable!(),
    }
}
