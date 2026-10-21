//! Explicit, startup-only native application configuration. See the repository
//! configuration guide for file ownership, precedence and platform limits.
//!
//! ```
//! use yetlet_config::{Environment, Loader};
//! let loader = Loader::new("/explicit/app/settings", "MYAPP");
//! assert_eq!(Environment::default(), Environment::Production);
//! assert_eq!(format!("{loader:?}"), "Loader { .. }");
//! ```

#[cfg(target_arch = "wasm32")]
compile_error!("yetlet-config is native application configuration, not browser state");

#[cfg(feature = "secret-provider")]
mod secrets;
#[cfg(feature = "secret-provider")]
pub use secrets::SecretBinding;

use serde::de::DeserializeOwned;
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fmt,
    fs::{self, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
};
use toml::Value;

/// Per-file byte limit, also applied to in-memory base TOML.
pub const MAX_FILE_BYTES: usize = 65_536;

/// Finite profile names; absent selection is fail-closed Production.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    #[default]
    Production,
}

impl Environment {
    fn parse(value: &str) -> Result<Self, Error> {
        match value {
            "Development" => Ok(Self::Development),
            "Staging" => Ok(Self::Staging),
            "Production" => Ok(Self::Production),
            _ => Err(Error::Environment),
        }
    }

    fn file(self) -> &'static str {
        match self {
            Self::Development => "development.toml",
            Self::Staging => "staging.toml",
            Self::Production => "production.toml",
        }
    }
}

/// Safe error categories. Parser/IO/Serde errors and their source chains are
/// deliberately discarded; no input, key, path or value is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Prefix,
    Environment,
    Input,
    File,
    Toml,
    EnvironmentKey,
    Type,
    Settings,
    #[cfg(feature = "secret-provider")]
    SecretMissing,
    #[cfg(feature = "secret-provider")]
    SecretUnavailable,
    #[cfg(feature = "secret-provider")]
    SecretAccessDenied,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Prefix => "invalid configuration prefix",
            Self::Environment => "invalid configuration environment",
            Self::Input => "configuration input exceeds limits or is invalid Unicode",
            Self::File => "configuration file cannot be read safely",
            Self::Toml => "invalid configuration TOML",
            Self::EnvironmentKey => "unknown, malformed or duplicate configuration key",
            Self::Type => "configuration value has an incompatible type",
            Self::Settings => "configuration does not match application settings",
            #[cfg(feature = "secret-provider")]
            Self::SecretMissing => "required configuration secret is missing",
            #[cfg(feature = "secret-provider")]
            Self::SecretUnavailable => "configuration secret provider is unavailable",
            #[cfg(feature = "secret-provider")]
            Self::SecretAccessDenied => "configuration secret access was denied",
        })
    }
}
impl std::error::Error for Error {}

/// Development-only explicit file format. Dotenv expansion is disabled by
/// rejecting dollar signs; use TOML for literal dollar-containing secrets.
#[derive(Clone, Copy, Debug)]
pub enum SecretFormat {
    Dotenv,
    Toml,
}

/// Startup loader with no implicit file discovery. Debug omits paths and data.
/// Each selected file is required; even NotFound is an error.
pub struct Loader {
    directory: PathBuf,
    prefix: String,
    defaults: Option<String>,
    secrets: Option<(PathBuf, SecretFormat)>,
}

impl fmt::Debug for Loader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Loader { .. }")
    }
}

impl Loader {
    /// Read `base.toml` and the selected finite profile from this directory.
    pub fn new(directory: impl Into<PathBuf>, prefix: impl Into<String>) -> Self {
        Self {
            directory: directory.into(),
            prefix: prefix.into(),
            defaults: None,
            secrets: None,
        }
    }

    /// Use app-owned TOML defaults instead of reading base.toml. Defaults define
    /// the allowed keys/types for subsequent layers. No new keys may be added.
    pub fn defaults(mut self, toml: impl Into<String>) -> Self {
        self.defaults = Some(toml.into());
        self
    }

    /// Enable exactly one explicit development secret file (for example an
    /// app-local `.env.local` or a protected user-private absolute path).
    /// Staging and Production never inspect or open this path.
    pub fn development_secrets(mut self, path: impl Into<PathBuf>, format: SecretFormat) -> Self {
        self.secrets = Some((path.into(), format));
        self
    }

    /// Snapshot real OS variables without mutating the process environment.
    pub fn load<T: DeserializeOwned>(&self) -> Result<T, Error> {
        self.load_from_env(std::env::vars_os())
    }

    /// Load from an explicit environment snapshot, useful for deterministic
    /// tests/embedding. Unrelated variables are ignored; related keys must use
    /// canonical uppercase ASCII and `__` for nesting. Duplicate keys fail.
    pub fn load_from_env<T: DeserializeOwned>(
        &self,
        environment: impl IntoIterator<Item = (OsString, OsString)>,
    ) -> Result<T, Error> {
        if !valid_prefix(&self.prefix) {
            return Err(Error::Prefix);
        }
        let marker = format!("{}_", self.prefix);
        let selector = format!("{marker}ENVIRONMENT");
        let mut profile = Environment::default();
        let mut entries = Vec::new();
        let mut seen = BTreeSet::new();
        for (key, value) in environment {
            // Lossy conversion is used only to recognize the reserved prefix;
            // any related non-Unicode key/value then fails closed.
            if !key
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with(&marker)
            {
                continue;
            }
            let key = key.into_string().map_err(|_| Error::Input)?;
            let value = value.into_string().map_err(|_| Error::Input)?;
            if key.len() > 256 || value.len() > MAX_FILE_BYTES {
                return Err(Error::Input);
            }
            if !seen.insert(key.clone()) {
                return Err(Error::EnvironmentKey);
            }
            if key == selector {
                profile = Environment::parse(&value)?;
            } else {
                entries.push((key, value));
            }
        }
        // Resolve the selector before inspecting any configuration/secret file.
        let base = match &self.defaults {
            Some(text) => text.clone(),
            None => read_file(&self.directory.join("base.toml"), false)?,
        };
        let mut settings = parse_toml(&base)?;
        validate_keys(&settings)?;
        let overlay = parse_toml(&read_file(&self.directory.join(profile.file()), false)?)?;
        merge(&mut settings, overlay)?;
        if profile == Environment::Development
            && let Some((path, format)) = &self.secrets
        {
            let text = read_file(path, true)?;
            match format {
                SecretFormat::Toml => merge(&mut settings, parse_toml(&text)?)?,
                SecretFormat::Dotenv => {
                    // dotenvy otherwise expands from ambient process variables.
                    if text.contains('$') {
                        return Err(Error::EnvironmentKey);
                    }
                    let mut seen = BTreeSet::new();
                    for entry in dotenvy::from_read_iter(text.as_bytes()) {
                        let (key, value) = entry.map_err(|_| Error::EnvironmentKey)?;
                        if !seen.insert(key.clone()) || key == selector {
                            return Err(Error::EnvironmentKey);
                        }
                        override_value(&mut settings, &marker, &key, value)?;
                    }
                }
            }
        }
        for (key, value) in entries {
            override_value(&mut settings, &marker, &key, value)?;
        }
        settings.try_into().map_err(|_| Error::Settings)
    }
}

fn valid_prefix(s: &str) -> bool {
    s.len() <= 64
        && s.starts_with(|c: char| c.is_ascii_uppercase())
        && !s.ends_with('_')
        && !s.contains("__")
        && s.bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
}

fn valid_key(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_lowercase())
        && !s.ends_with('_')
        && !s.contains("__")
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

fn validate_keys(value: &Value) -> Result<(), Error> {
    if let Value::Table(table) = value {
        for (key, value) in table {
            if !valid_key(key) {
                return Err(Error::EnvironmentKey);
            }
            validate_keys(value)?;
        }
    }
    Ok(())
}

fn parse_toml(text: &str) -> Result<Value, Error> {
    if text.len() > MAX_FILE_BYTES {
        return Err(Error::Input);
    }
    let table: toml::Table = toml::from_str(text).map_err(|_| Error::Toml)?;
    if table.contains_key("environment") {
        return Err(Error::EnvironmentKey);
    }
    Ok(Value::Table(table))
}

fn merge(base: &mut Value, overlay: Value) -> Result<(), Error> {
    match (base, overlay) {
        (Value::Table(base), Value::Table(overlay)) => {
            for (key, value) in overlay {
                merge(base.get_mut(&key).ok_or(Error::EnvironmentKey)?, value)?;
            }
        }
        (base, overlay) if std::mem::discriminant(base) == std::mem::discriminant(&overlay) => {
            *base = overlay;
        }
        _ => return Err(Error::Type),
    }
    Ok(())
}

fn override_value(root: &mut Value, marker: &str, key: &str, value: String) -> Result<(), Error> {
    let suffix = key.strip_prefix(marker).ok_or(Error::EnvironmentKey)?;
    if suffix.len() > 256 || suffix != suffix.to_ascii_uppercase() {
        return Err(Error::EnvironmentKey);
    }
    let mut target = root;
    for segment in suffix.split("__") {
        let key = segment.to_ascii_lowercase();
        if !valid_key(&key) {
            return Err(Error::EnvironmentKey);
        }
        target = target
            .as_table_mut()
            .and_then(|table| table.get_mut(&key))
            .ok_or(Error::EnvironmentKey)?;
    }
    *target = match target {
        Value::String(_) => Value::String(value),
        Value::Integer(_) => Value::Integer(value.parse().map_err(|_| Error::Type)?),
        Value::Boolean(_) => Value::Boolean(value.parse().map_err(|_| Error::Type)?),
        Value::Float(_) => {
            let number: f64 = value.parse().map_err(|_| Error::Type)?;
            if !number.is_finite() {
                return Err(Error::Type);
            }
            Value::Float(number)
        }
        _ => return Err(Error::Type),
    };
    Ok(())
}

fn read_file(path: &Path, secret: bool) -> Result<String, Error> {
    let metadata = fs::symlink_metadata(path).map_err(|_| Error::File)?;
    if !metadata.is_file() || (secret && metadata.file_type().is_symlink()) {
        return Err(Error::File);
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Also avoid blocking if an owned path races into a FIFO.
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path).map_err(|_| Error::File)?;
    let metadata = file.metadata().map_err(|_| Error::File)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(Error::File); // Reparse points are not supported.
        }
    }
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES as u64 {
        return Err(Error::File);
    }
    let mut text = String::new();
    file.take(MAX_FILE_BYTES as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|_| Error::File)?;
    if text.len() > MAX_FILE_BYTES {
        return Err(Error::Input);
    }
    Ok(text)
}
