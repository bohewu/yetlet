pub mod board;
pub mod docker;
pub mod local;
pub mod redis;
use crate::tooling::{Result, fail};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
pub fn validate(suite: &str, args: &[OsString]) -> std::result::Result<(), &'static str> {
    match suite {
        "board" if args.is_empty() => Ok(()),
        "board" if args.len() == 2 && args[0] == "--profile" => {
            board::profile(args[1].to_str().ok_or("profile encoding")?)
                .map(|_| ())
                .map_err(|_| "unknown Board profile")
        }
        "local-operations" | "redis-three-app" if args.is_empty() => Ok(()),
        "multi-instance" if args.is_empty() || (args.len() == 2 && args[0] == "--report") => Ok(()),
        _ => Err("unknown qualification or invalid options"),
    }
}
pub fn run(root: &Path, suite: &str, args: &[OsString]) -> Result<()> {
    validate(suite, args).map_err(fail)?;
    match suite {
        "local-operations" => local::run(root),
        "board" => board::run(
            root,
            if args.is_empty() {
                "bounded"
            } else {
                args[1].to_str().unwrap()
            },
        ),
        "multi-instance" => docker::run(root, false, args.get(1).map(PathBuf::from)),
        "redis-three-app" => docker::run(root, true, None),
        _ => Err(fail("unknown qualification")),
    }
}
