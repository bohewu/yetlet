//! Opt-in read-only experiment: azure_cli AZ_EXE VAULT SECRET V1_VERSION.
//! After each ready marker, an external operator rotates/retires the version
//! and sends `rotated`/`retired` on stdin. Azure CLI authentication is external.

use std::{io::Write, path::PathBuf, sync::Arc, time::Duration};
use tokio::{process::Command, time::timeout};
use yetlet::secrets::{ExposeSecret, SecretError, SecretFuture, SecretProvider, SecretString};

struct AzureCliSecrets {
    executable: PathBuf,
    vault: String,
    version: Option<String>,
}

impl SecretProvider for AzureCliSecrets {
    fn get<'a>(&'a self, name: &'a str) -> SecretFuture<'a> {
        Box::pin(async move {
            let mut command = Command::new(&self.executable);
            command.args([
                "keyvault",
                "secret",
                "show",
                "--vault-name",
                &self.vault,
                "--name",
                name,
            ]);
            if let Some(version) = &self.version {
                command.args(["--version", version]);
            }
            command
                .args(["--query", "value", "--output", "json", "--only-show-errors"])
                .kill_on_drop(true);
            let output = timeout(Duration::from_secs(30), command.output())
                .await
                .map_err(|_| SecretError::Unavailable)?
                .map_err(|_| SecretError::Unavailable)?;
            decode(output.status.success(), &output.stdout, &output.stderr)
        })
    }
}

fn decode(
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<Option<SecretString>, SecretError> {
    if success {
        return serde_json::from_slice::<String>(stdout)
            .map(|value| Some(SecretString::from(value)))
            .map_err(|_| SecretError::Unavailable);
    }
    let contains = |marker: &[u8]| stderr.windows(marker.len()).any(|part| part == marker);
    if contains(b"(Forbidden)") || contains(b"(Unauthorized)") {
        Err(SecretError::AccessDenied)
    } else if contains(b"(SecretNotFound)") {
        Ok(None)
    } else {
        Err(SecretError::Unavailable)
    }
}

async fn required(provider: &dyn SecretProvider, name: &str) -> Result<SecretString, &'static str> {
    provider
        .get(name)
        .await
        .map_err(|_| "Azure secret lookup failed")?
        .ok_or("Azure secret was missing")
}

async fn checkpoint(marker: &str, expected: &'static str) -> Result<(), &'static str> {
    println!("{marker}");
    std::io::stdout().flush().map_err(|_| "Output failed")?;
    tokio::task::spawn_blocking(move || {
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .map_err(|_| "Checkpoint input failed")?;
        if line.trim_end_matches(['\r', '\n']) != expected {
            return Err("Unexpected checkpoint input");
        }
        Ok(())
    })
    .await
    .map_err(|_| "Checkpoint input failed")?
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let mut args = std::env::args_os().skip(1);
    let usage = "Usage: azure_cli AZ_EXE VAULT SECRET V1_VERSION";
    let executable = PathBuf::from(args.next().ok_or(usage)?);
    let vault = args.next().ok_or(usage)?.into_string().map_err(|_| usage)?;
    let name = args.next().ok_or(usage)?.into_string().map_err(|_| usage)?;
    let version = args.next().ok_or(usage)?.into_string().map_err(|_| usage)?;
    if args.next().is_some() {
        return Err(usage);
    }
    let latest: Arc<dyn SecretProvider> = Arc::new(AzureCliSecrets {
        executable: executable.clone(),
        vault: vault.clone(),
        version: None,
    });
    let pinned: Arc<dyn SecretProvider> = Arc::new(AzureCliSecrets {
        executable,
        vault,
        version: Some(version),
    });
    let retained = required(latest.as_ref(), &name).await?;
    let old = required(pinned.as_ref(), &name).await?;
    if retained.expose_secret().is_empty() || retained.expose_secret() != old.expose_secret() {
        return Err("Initial version check failed");
    }
    if latest
        .get(&format!("{name}-missing"))
        .await
        .map_err(|_| "Missing secret lookup failed")?
        .is_some()
    {
        return Err("Missing secret check failed");
    }
    checkpoint("AZURE_V1_READY", "rotated").await?;
    let current = required(latest.as_ref(), &name).await?;
    if current.expose_secret() == retained.expose_secret() {
        return Err("Rotated version check failed");
    }
    checkpoint("AZURE_V2_READY", "retired").await?;
    if !matches!(pinned.get(&name).await, Err(SecretError::AccessDenied)) {
        return Err("Retired version check failed");
    }
    let still_current = required(latest.as_ref(), &name).await?;
    if still_current.expose_secret() != current.expose_secret()
        || retained.expose_secret() != old.expose_secret()
    {
        return Err("Retained version check failed");
    }
    println!("AZURE_ROTATION_PASS");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_preserves_empty_and_whitespace_strings_exactly() {
        for expected in ["", " \t\nvalue\r\n "] {
            let json = serde_json::to_vec(expected).unwrap();
            let value = decode(true, &json, b"").unwrap().unwrap();
            assert_eq!(value.expose_secret(), expected);
        }
    }

    #[test]
    fn backend_failures_are_classified_without_diagnostics() {
        assert!(
            decode(false, b"private-value", b"(SecretNotFound) private-name")
                .unwrap()
                .is_none()
        );
        for marker in ["(Forbidden)", "(Unauthorized)"] {
            let diagnostic = format!("{marker} private-name private-value");
            let error = decode(false, b"private-value", diagnostic.as_bytes()).unwrap_err();
            assert_eq!(error, SecretError::AccessDenied);
            assert_eq!(error.to_string(), "secret access denied");
            assert_eq!(format!("{error:?}"), "AccessDenied");
            assert!(std::error::Error::source(&error).is_none());
        }
    }

    #[test]
    fn unknown_exit_and_malformed_output_are_unavailable() {
        for (success, stdout, stderr) in [
            (
                false,
                b"private-value".as_slice(),
                b"private-diagnostic".as_slice(),
            ),
            (true, b"private-value".as_slice(), b"".as_slice()),
            (true, b"null".as_slice(), b"".as_slice()),
        ] {
            let error = decode(success, stdout, stderr).unwrap_err();
            assert_eq!(error, SecretError::Unavailable);
            assert_eq!(error.to_string(), "secret provider unavailable");
            assert_eq!(format!("{error:?}"), "Unavailable");
            assert!(std::error::Error::source(&error).is_none());
        }
    }
}
