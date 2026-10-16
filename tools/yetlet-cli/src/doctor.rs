use std::{
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug)]
pub struct Diagnostic {
    pub capability: &'static str,
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub struct Report(pub Vec<Diagnostic>);

impl Report {
    pub fn ok(&self) -> bool {
        self.0.iter().all(|d| d.status != "fail")
    }

    pub fn render(&self, json: bool) -> String {
        if json {
            serde_json::json!({"ok": self.ok(), "diagnostics": self.0.iter().map(|d|
                serde_json::json!({"capability": d.capability, "status": d.status, "message": d.message})
            ).collect::<Vec<_>>()}).to_string()
        } else {
            self.0
                .iter()
                .map(|d| format!("{} {}: {}", d.status, d.capability, d.message))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

pub fn check(framework: Option<&Path>) -> Report {
    let mut diagnostics = ["rustc", "cargo"]
        .into_iter()
        .map(|tool| {
            // Disable rustup's missing-toolchain auto-install even when PATH selects
            // its proxies. Never surface raw stdout/stderr or OS error paths.
            let output = Command::new(tool)
                .arg("--version")
                .env("RUSTUP_AUTO_INSTALL", "0")
                .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
                .env("CARGO_NET_OFFLINE", "true")
                .stdin(Stdio::null())
                .output();
            match output {
                Ok(output) if output.status.success() => version(tool, &output.stdout),
                Ok(_) => diagnostic(
                    tool,
                    false,
                    "local --version probe failed; no installation attempted",
                ),
                Err(_) => diagnostic(
                    tool,
                    false,
                    "executable unavailable; provision Rust/Cargo 1.99 separately",
                ),
            }
        })
        .collect::<Vec<_>>();
    diagnostics.push(match framework {
        Some(path) => match crate::framework::validate(path) {
            Ok(_) => diagnostic(
                "framework",
                true,
                "selected local workspace and required crate sources found (paths redacted)",
            ),
            Err(message) => diagnostic("framework", false, message),
        },
        None => Diagnostic {
            capability: "framework",
            status: "not-selected",
            message: "optional; select --framework-path PATH to check local sources".into(),
        },
    });
    diagnostics.push(diagnostic(
        "new-platform",
        crate::platform::fs::NEW_SUPPORTED,
        crate::platform::fs::NEW_DIAGNOSTIC,
    ));
    Report(diagnostics)
}

fn diagnostic(capability: &'static str, ok: bool, message: &str) -> Diagnostic {
    Diagnostic {
        capability,
        status: if ok { "pass" } else { "fail" },
        message: message.into(),
    }
}

pub fn version(tool: &'static str, bytes: &[u8]) -> Diagnostic {
    let parsed = std::str::from_utf8(bytes).ok().and_then(|s| {
        let mut parts = s.split_whitespace();
        if parts.next()? != tool {
            return None;
        }
        let numbers = parts
            .next()?
            .split('.')
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        if numbers.len() != 3 {
            return None;
        }
        Some((numbers[0], numbers[1], numbers[2]))
    });
    match parsed {
        Some((major, minor, patch)) => diagnostic(
            tool,
            (major, minor) >= (1, 99),
            &format!(
                "{tool} {major}.{minor}.{patch}; requires stable 1.99 or newer (other output redacted)"
            ),
        ),
        None => diagnostic(
            tool,
            false,
            "unrecognized stable version identity (output redacted); requires 1.99 or newer",
        ),
    }
}
