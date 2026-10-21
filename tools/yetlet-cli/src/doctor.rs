//! Environment-only diagnostics. Probe output never becomes a diagnostic message.
use crate::platform::process::{ProbeError, probe};

#[derive(Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub origin: &'static str,
    pub capability: &'static str,
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub struct Report(pub Vec<Diagnostic>);

impl Report {
    pub fn ok(&self) -> bool {
        [
            "DOCTOR_RUSTC_VERSION",
            "DOCTOR_CARGO_VERSION",
            "DOCTOR_NEW_PLATFORM",
        ]
        .iter()
        .all(|code| self.0.iter().any(|d| d.code == *code && d.status == "pass"))
            && !self.0.iter().any(|d| d.status == "fail")
    }

    pub fn exit_code(&self) -> u8 {
        if self.0.iter().any(|d| d.code == "DOCTOR_CLEANUP") {
            125
        } else {
            u8::from(!self.ok())
        }
    }

    pub fn render(&self, json: bool) -> String {
        if json {
            serde_json::json!({
                "schema_version": 1,
                "cli_version": env!("CARGO_PKG_VERSION"),
                "scope": "environment",
                "completed": self.exit_code() != 125,
                "ok": self.ok(),
                "diagnostics": self.0.iter().map(|d| serde_json::json!({
                    "code": d.code, "origin": d.origin, "capability": d.capability,
                    "status": d.status, "message": d.message
                })).collect::<Vec<_>>()
            })
            .to_string()
        } else {
            self.0
                .iter()
                .map(|d| format!("{} {} {}: {}", d.code, d.status, d.capability, d.message))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

pub fn check() -> Report {
    let mut diagnostics = Vec::new();
    for tool in ["rustc", "cargo"] {
        match probe(tool) {
            Ok(output) => diagnostics.push(version(tool, &output)),
            Err(error) => {
                let message = match error {
                    ProbeError::Unavailable => {
                        "local executable unavailable or unsafe; use a protected Rust/Cargo 1.99 installation"
                    }
                    ProbeError::Failed => "local version probe failed; no installation attempted",
                    ProbeError::Timeout => "local version probe exceeded its 3-second deadline",
                    ProbeError::OutputLimit => {
                        "local version probe exceeded its 64-KiB output bound"
                    }
                    ProbeError::Cleanup => {
                        "local version probe cleanup could not be verified within 2 seconds"
                    }
                };
                diagnostics.push(diagnostic(tool, "fail", message));
                if matches!(error, ProbeError::Cleanup) {
                    diagnostics.push(Diagnostic {
                        code: "DOCTOR_CLEANUP",
                        origin: "doctor.probe",
                        capability: "probe-cleanup",
                        status: "fail",
                        message: "owned version probe cleanup failed; environment check incomplete"
                            .into(),
                    });
                    break;
                }
            }
        }
    }
    diagnostics.push(Diagnostic {
        code: "DOCTOR_NEW_PLATFORM",
        origin: "doctor.compiled",
        capability: "new-platform",
        status: if crate::platform::fs::NEW_SUPPORTED {
            "pass"
        } else {
            "unsupported"
        },
        message: crate::platform::fs::NEW_DIAGNOSTIC.into(),
    });
    for (code, capability) in [
        ("DOCTOR_FRAMEWORK", "framework"),
        ("DOCTOR_APP_READINESS", "app-settings-readiness"),
        ("DOCTOR_ASSETS", "assets"),
        ("DOCTOR_BUILD_DEV", "build-run-execution"),
        ("DOCTOR_CONSOLE", "console"),
        ("DOCTOR_GENERATOR", "generator"),
    ] {
        diagnostics.push(Diagnostic {
            code,
            origin: "doctor.scope",
            capability,
            status: "not-checked",
            message: "outside this environment-only report; no readiness claim".into(),
        });
    }
    Report(diagnostics)
}

fn diagnostic(tool: &'static str, status: &'static str, message: &str) -> Diagnostic {
    Diagnostic {
        code: if tool == "rustc" {
            "DOCTOR_RUSTC_VERSION"
        } else {
            "DOCTOR_CARGO_VERSION"
        },
        origin: "doctor.probe",
        capability: tool,
        status,
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
            if (major, minor) >= (1, 99) {
                "pass"
            } else {
                "fail"
            },
            &format!(
                "{tool} {major}.{minor}.{patch}; requires stable 1.99 or newer (other output redacted)"
            ),
        ),
        None => diagnostic(
            tool,
            "fail",
            "unrecognized stable version identity (output redacted); requires 1.99 or newer",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_requires_all_environment_checks_and_never_counts_unchecked_as_pass() {
        let mut report = Report(vec![
            version("rustc", b"rustc 1.99.0"),
            version("cargo", b"cargo 1.99.0"),
        ]);
        assert!(!report.ok());
        report.0.push(Diagnostic {
            code: "DOCTOR_NEW_PLATFORM",
            origin: "doctor.compiled",
            capability: "new-platform",
            status: "pass",
            message: "supported".into(),
        });
        assert!(report.ok());
        let value: serde_json::Value = serde_json::from_str(&report.render(true)).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["scope"], "environment");
        assert_eq!(value["completed"], true);
        for diagnostic in value["diagnostics"].as_array().unwrap() {
            for field in ["code", "origin", "capability", "status", "message"] {
                assert!(diagnostic[field].is_string());
            }
        }
        report.0[0].status = "unsupported";
        assert!(!report.ok());
        report.0.push(Diagnostic {
            code: "DOCTOR_CLEANUP",
            origin: "doctor.probe",
            capability: "probe-cleanup",
            status: "fail",
            message: "cleanup failed".into(),
        });
        assert_eq!(report.exit_code(), 125);
        let value: serde_json::Value = serde_json::from_str(&report.render(true)).unwrap();
        assert_eq!(value["completed"], false);
        assert_eq!(value["ok"], false);
    }
}
