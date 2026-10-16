//! Two private, statically selected CLI OS seams.
pub(crate) mod fs;
pub(crate) mod process;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Completion;
    use std::{io, path::Path};

    #[test]
    fn adapter_entrypoints_have_static_internal_contracts() {
        type CreateProject = fn(&Path, &str, &[(&str, String)]) -> Result<(), String>;
        let _: CreateProject = fs::create_project;
        let _: fn(&Path) -> Result<fs::App, String> = fs::open_app;
        let _: fn(&Path) -> io::Result<Vec<u8>> = fs::read_framework;
        let _: fn(&fs::App, bool, bool) -> io::Result<Completion> = process::run_cargo;
        let _: fn(Completion) -> ! = process::finish;
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    #[test]
    fn unsupported_commands_refuse_before_paths_or_children() {
        let missing = Path::new("missing/../Cargo.toml");
        assert_eq!(
            crate::scaffold::create("demo", missing, missing).unwrap_err(),
            "new is unsupported on this platform: no verified no-replace/reparse implementation; nothing written"
        );
        for (dev, release) in [(false, false), (false, true), (true, false)] {
            let failure = crate::app::run(missing, dev, release).unwrap_err();
            assert_eq!(failure.code, 1);
            assert_eq!(
                failure.message,
                "build/dev are unsupported on this platform: native cwd/process ownership Not verified; no child started"
            );
        }
        let app = fs::App;
        assert_eq!(
            process::run_cargo(&app, false, false).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
