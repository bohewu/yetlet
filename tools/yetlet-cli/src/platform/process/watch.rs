//! Watchexec is only an event source; native adapters own every Rust generation.
use crate::platform::fs::{self, App};
use std::{
    io::{self, BufRead, BufReader, Read},
    path::{Component, Path, PathBuf},
    process::Command,
    sync::mpsc::{self, Receiver, TryRecvError},
};

pub(super) fn command(app: &App, frontend_root: &str) -> Command {
    let mut command = Command::new("watchexec");
    command.args([
        "--only-emit-events",
        "--emit-events-to=json-stdio",
        "--postpone",
        "--watch",
        ".",
        "--project-origin",
        ".",
        "--no-discover-ignore",
        "--no-follow-symlinks",
        "--debounce",
        "200ms",
        "--filter",
        "**/*.rs",
        "--filter",
        "Cargo.toml",
        "--filter",
        "Cargo.lock",
    ]);
    for ignore in ["**/target/**", "**/node_modules/**", "**/dist/**"] {
        command.args(["--ignore", ignore]);
    }
    command.arg("--ignore").arg(format!(
        "/{}/**",
        glob_literal(frontend_root.trim_end_matches('/'))
    ));
    if let Some(target) = std::env::var_os("CARGO_TARGET_DIR")
        && let Some(ignore) = target_ignore(fs::app_root(app), Path::new(&target))
    {
        command.arg("--ignore").arg(ignore);
    }
    for (key, _) in std::env::vars_os() {
        let upper = key.to_string_lossy().to_ascii_uppercase();
        if upper.starts_with("APP_")
            || matches!(upper.as_str(), "YETLET_DEV_SECRETS" | "YETLET_CONFIG_ROOT")
        {
            command.env_remove(key);
        }
    }
    // No shell, package command, dependency installation or Cargo child here.
    command
}

fn target_ignore(app_root: &Path, target: &Path) -> Option<String> {
    // Cargo may not have created the output directory yet. Normalize lexically
    // so ordinary relative components never become literal glob components.
    let mut normalized = PathBuf::new();
    for component in app_root.join(target).components() {
        match component {
            Component::CurDir => (),
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    let relative = normalized.strip_prefix(app_root).ok()?;
    if relative.as_os_str().is_empty() {
        return None;
    }
    Some(format!(
        "/{}/**",
        glob_literal(&relative.to_string_lossy().replace('\\', "/"))
    ))
}

fn glob_literal(path: &str) -> String {
    let mut escaped = String::new();
    for c in path.chars() {
        if matches!(c, '*' | '?' | '[' | ']' | '{' | '}' | '!') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

pub(crate) struct Events {
    receiver: Receiver<Result<(), ()>>,
    ended: bool,
}

impl Events {
    pub(crate) fn new(reader: impl Read + Send + 'static) -> Self {
        // A burst needs at most one pending restart, never an unbounded queue.
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            loop {
                let mut line = Vec::new();
                match reader.by_ref().take(65_537).read_until(b'\n', &mut line) {
                    Ok(0) => break,
                    Ok(_) if line.len() <= 65_536 && line.last() == Some(&b'\n') => (),
                    _ => {
                        let _ = sender.try_send(Err(()));
                        break;
                    }
                }
                let event: serde_json::Value = match serde_json::from_slice(&line) {
                    Ok(event) => event,
                    Err(_) => {
                        let _ = sender.try_send(Err(()));
                        break;
                    }
                };
                let Some(tags) = event.get("tags").and_then(serde_json::Value::as_array) else {
                    let _ = sender.try_send(Err(()));
                    break;
                };
                if tags.iter().any(|tag| {
                    tag.get("kind").and_then(serde_json::Value::as_str) == Some("path")
                        && tag
                            .get("absolute")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|path| !path.is_empty())
                }) && sender
                    .try_send(Ok(()))
                    .is_err_and(|error| matches!(error, mpsc::TrySendError::Disconnected(_)))
                {
                    break;
                }
            }
        });
        Self {
            receiver,
            ended: false,
        }
    }

    pub(crate) fn changed(&mut self) -> io::Result<bool> {
        let failed = || io::Error::other("Watchexec event stream ended or is invalid");
        if self.ended {
            return Err(failed());
        }
        let mut changed = false;
        loop {
            match self.receiver.try_recv() {
                Ok(Ok(())) => changed = true,
                Ok(Err(())) => return Err(failed()),
                Err(TryRecvError::Empty) => return Ok(changed),
                Err(TryRecvError::Disconnected) => {
                    self.ended = true;
                    return if changed { Ok(true) } else { Err(failed()) };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, thread, time::Duration};

    #[test]
    fn cargo_target_ignore_normalizes_relative_components_without_filesystem_reads() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for target in ["./build-output", "nested/../build-output"] {
            assert_eq!(
                target_ignore(root, Path::new(target)).as_deref(),
                Some("/build-output/**")
            );
        }
        let reentered = Path::new("..")
            .join(root.file_name().unwrap())
            .join("build-output");
        assert_eq!(
            target_ignore(root, &reentered).as_deref(),
            Some("/build-output/**")
        );
        assert_eq!(
            target_ignore(root, &root.join("./nested/../build-output")).as_deref(),
            Some("/build-output/**")
        );
        for target in [".", "nested/..", "../outside-output"] {
            assert_eq!(target_ignore(root, Path::new(target)), None);
        }
    }

    #[test]
    fn event_stream_coalesces_paths_and_reports_end() {
        let input = b"{\"tags\":[]}\n{\"tags\":[{\"kind\":\"path\",\"absolute\":\"/app/src/main.rs\"}]}\n{\"tags\":[{\"kind\":\"path\",\"absolute\":\"/app/Cargo.toml\"}]}\n";
        let mut events = Events::new(Cursor::new(input));
        let mut observed = false;
        for _ in 0..100 {
            match events.changed() {
                Ok(changed) => observed |= changed,
                Err(_) => {
                    assert!(observed);
                    return;
                }
            }
            thread::sleep(Duration::from_millis(2));
        }
        panic!("event EOF was not observed");
    }

    #[test]
    fn invalid_or_oversized_events_fail_closed() {
        for input in [b"not json\n".to_vec(), b"{}\n".to_vec(), vec![b'x'; 65_537]] {
            let mut events = Events::new(Cursor::new(input));
            assert!((0..100).any(|_| {
                thread::sleep(Duration::from_millis(2));
                events.changed().is_err()
            }));
        }
    }
}
