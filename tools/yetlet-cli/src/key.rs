//! Explicit local key administration. No signing command or implicit provider fallback.
use crate::{
    Cli, framework,
    platform::fs::{
        self,
        key::{Directory, Snapshot},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    io::{self, BufRead, IsTerminal, Write},
    path::{Path, PathBuf},
};
use yetlet_signing::{KeyRef, KeyState, LocalSigner, PublicMaterial, generate_self_signed};

pub const HELP: &str = "Usage:\n  yetlet key status [KEY] [--provider NAME] [--manifest-path PATH]\n  yetlet key rotate [KEY] [--provider NAME] [--manifest-path PATH] [--yes] [--from PATH]\n\nNames select configured aliases; omitted names require explicit signing defaults.\nStatus is public-only and read-only. Rotate updates the selected profile TOML after confirmation.\n--yes approves the displayed exact plan for automation; otherwise stdin/stdout must be a terminal.\n--from imports a replacement encrypted PFX using the configured password_env.\nWithout --from, only mode=self_signed_test can generate a fresh 30-day test certificate.\nRotation retains old material; it does not revoke or delete keys. No cloud adapters or signing command.\n";
const INVALID: &str = "invalid signing configuration or unsafe settings/material path";
const MATERIAL: &str = "invalid signing material, password, trust pin or duplicate public key";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub rotate: bool,
    pub key: Option<String>,
    pub provider: Option<String>,
    pub manifest: PathBuf,
    pub yes: bool,
    pub from: Option<PathBuf>,
}
fn alias(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.starts_with(|c: char| c.is_ascii_lowercase())
        && !s.ends_with('_')
        && !s.contains("__")
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
pub(crate) fn parse(args: &[OsString]) -> Result<Cli, &'static str> {
    if args.len() == 1 && matches!(args[0].to_str(), Some("--help" | "-h")) {
        return Ok(Cli::Help(HELP));
    }
    let rotate = match args.first().and_then(|a| a.to_str()) {
        Some("status") => false,
        Some("rotate") => true,
        _ => return Err("key requires status or rotate"),
    };
    if args.len() == 2 && matches!(args[1].to_str(), Some("--help" | "-h")) {
        return Ok(Cli::Help(HELP));
    }
    let mut c = Command {
        rotate,
        key: None,
        provider: None,
        manifest: "Cargo.toml".into(),
        yes: false,
        from: None,
    };
    let mut manifest = false;
    let mut i = 1;
    while i < args.len() {
        let (opt, inline) = crate::split_option(&args[i]);
        match opt {
            Some("--provider") if c.provider.is_none() => {
                let v = crate::argument_value(args, &mut i, inline)?
                    .to_str()
                    .ok_or("provider alias must be ASCII snake_case")?;
                if !alias(v) {
                    return Err("provider alias must be ASCII snake_case");
                }
                c.provider = Some(v.into());
            }
            Some("--manifest-path") if !manifest => {
                c.manifest = crate::path_value(args, &mut i, inline)?;
                manifest = true;
            }
            Some("--from") if rotate && c.from.is_none() => {
                c.from = Some(crate::path_value(args, &mut i, inline)?);
            }
            Some("--yes") if rotate && !c.yes && inline.is_none() => c.yes = true,
            Some(v) if !v.starts_with('-') && inline.is_none() && c.key.is_none() => {
                if !alias(v) {
                    return Err("key alias must be ASCII snake_case");
                }
                c.key = Some(v.into());
            }
            _ => {
                return Err("unknown, duplicate or misplaced key argument; help must be exclusive");
            }
        }
        i += 1;
    }
    Ok(Cli::Key(c))
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SigningSettings {
    pub default_provider: String,
    pub default_key: String,
    pub providers: BTreeMap<String, ProviderSettings>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderSettings {
    pub kind: String,
    pub root: String,
    pub mode: String,
    pub password_env: String,
    pub keys: BTreeMap<String, KeySettings>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeySettings {
    pub active_version: String,
    pub versions: Vec<VersionSettings>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionSettings {
    pub version: String,
    pub certificate: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    pub certificate_pin: String,
    pub state: KeyState,
}

struct Loaded {
    project: PathBuf,
    settings_root: PathBuf,
    profile_path: PathBuf,
    base: Snapshot,
    profile: Snapshot,
    settings: SigningSettings,
    provider: String,
    key: String,
}
fn safe_relative(path: &str) -> bool {
    crate::settings::valid_frontend_root(path) && !path.ends_with('/')
}
fn pin(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(MATERIAL.into());
    }
    let mut result = [0; 32];
    for (i, v) in result.iter_mut().enumerate() {
        *v = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).map_err(|_| MATERIAL)?;
    }
    Ok(result)
}
fn hex(value: &[u8]) -> String {
    value.iter().map(|b| format!("{b:02x}")).collect()
}
fn read_material(root: &Path, relative: &str, private: bool) -> Result<Vec<u8>, String> {
    if !safe_relative(relative) {
        return Err(INVALID.into());
    }
    let path = root.join(relative);
    let dir = Directory::open(path.parent().ok_or(INVALID)?, false).map_err(|_| INVALID)?;
    dir.read(
        path.file_name().and_then(|s| s.to_str()).ok_or(INVALID)?,
        private,
    )
    .map_err(|_| INVALID.into())
}
impl Loaded {
    fn load(c: &Command) -> Result<Self, String> {
        let app = fs::open_app(&c.manifest)?;
        let project = fs::app_root(&app).to_owned();
        let environment = std::env::vars_os().collect::<Vec<_>>();
        let settings_root = match std::env::var_os("YETLET_CONFIG_ROOT") {
            Some(v) => {
                let p = PathBuf::from(v);
                if !p.is_absolute() {
                    return Err(INVALID.into());
                }
                framework::absolute(&p).map_err(|_| INVALID)?
            }
            None => project.clone(),
        };
        let overlay = match std::env::var("APP_ENVIRONMENT") {
            Err(std::env::VarError::NotPresent) => "production.toml",
            Ok(v) if v == "Production" => "production.toml",
            Ok(v) if v == "Development" => "development.toml",
            Ok(v) if v == "Staging" => "staging.toml",
            _ => return Err(INVALID.into()),
        };
        let mut filtered = Vec::new();
        for (k, v) in environment {
            let upper = k.to_string_lossy().to_ascii_uppercase();
            if upper == "APP_SIGNING" || upper.starts_with("APP_SIGNING_") {
                if c.rotate {
                    return Err("rotation refuses APP_SIGNING overrides; remove them so the saved settings will be effective".into());
                }
                filtered.push((k, v));
            } else if upper == "APP_ENVIRONMENT" {
                filtered.push((k, v));
            }
        }
        let profile_path = settings_root.join(overlay);
        let base = Snapshot::open(&settings_root.join("base.toml"), false).map_err(|_| INVALID)?;
        let profile = Snapshot::open(&profile_path, c.rotate)
            .map_err(|_| "selected settings profile is unsafe, read-only or unwritable")?;
        let merged: serde_json::Value = yetlet_config::Loader::new(&settings_root, "APP")
            .defaults(String::from_utf8(base.bytes.clone()).map_err(|_| INVALID)?)
            .load_from_env(filtered)
            .map_err(|_| INVALID)?;
        base.ensure().map_err(|_| INVALID)?;
        profile.ensure().map_err(|_| INVALID)?;
        let settings: SigningSettings =
            serde_json::from_value(merged.get("signing").ok_or(INVALID)?.clone())
                .map_err(|_| INVALID)?;
        let provider = c
            .provider
            .clone()
            .unwrap_or_else(|| settings.default_provider.clone());
        let key = c
            .key
            .clone()
            .unwrap_or_else(|| settings.default_key.clone());
        if !alias(&provider) || !alias(&key) {
            return Err(
                "explicit signing default_provider/default_key or named selection is required"
                    .into(),
            );
        }
        for (pn, p) in &settings.providers {
            if !alias(pn) || !safe_relative(&p.root) {
                return Err(INVALID.into());
            }
            if p.kind != "local" {
                return Err(
                    "unsupported configured signing provider; no fallback is available".into(),
                );
            }
            if !matches!(p.mode.as_str(), "self_signed_test" | "imported") {
                return Err("unsupported local certificate mode".into());
            }
            for (kn, k) in &p.keys {
                if !alias(kn) {
                    return Err(INVALID.into());
                }
                let mut versions = BTreeSet::new();
                for v in &k.versions {
                    if v.version.is_empty()
                        || v.version.len() > 128
                        || v.version.chars().any(char::is_control)
                        || !versions.insert(&v.version)
                        || !safe_relative(&v.certificate)
                        || v.private_key.as_ref().is_some_and(|p| !safe_relative(p))
                    {
                        return Err(INVALID.into());
                    }
                    pin(&v.certificate_pin)?;
                }
                if (k.active_version.is_empty() && !k.versions.is_empty())
                    || (!k.active_version.is_empty()
                        && !k
                            .versions
                            .iter()
                            .any(|v| v.version == k.active_version && v.state == KeyState::Active))
                {
                    return Err(INVALID.into());
                }
            }
        }
        settings
            .providers
            .get(&provider)
            .and_then(|p| p.keys.get(&key))
            .ok_or("unknown configured provider or key")?;
        Ok(Self {
            project,
            settings_root,
            profile_path,
            base,
            profile,
            settings,
            provider,
            key,
        })
    }
    fn selected(&self) -> (&ProviderSettings, &KeySettings) {
        let p = &self.settings.providers[&self.provider];
        (p, &p.keys[&self.key])
    }
    fn public_history(&self) -> Result<Vec<PublicMaterial>, String> {
        let mut all: Vec<PublicMaterial> = Vec::new();
        for p in self.settings.providers.values() {
            let root = self.project.join(&p.root);
            for k in p.keys.values() {
                for v in &k.versions {
                    let pem = read_material(&root, &v.certificate, false)?;
                    let material =
                        PublicMaterial::from_certificate_pem(&pem).map_err(|_| MATERIAL)?;
                    if material.certificate_pin() != Some(pin(&v.certificate_pin)?)
                        || all.iter().any(|prior| prior.same_public_key(&material))
                    {
                        return Err(MATERIAL.into());
                    }
                    all.push(material);
                }
            }
        }
        Ok(all)
    }
    fn ensure(&self) -> Result<(), String> {
        self.base
            .ensure()
            .and_then(|()| self.profile.ensure())
            .map_err(|_| {
                "settings changed since the displayed plan; no activation performed".into()
            })
    }
}

pub fn execute(command: &Command) -> Result<(), String> {
    let loaded = Loaded::load(command)?;
    let history = loaded.public_history()?;
    if !command.rotate {
        return status(&loaded);
    }
    rotate(&loaded, command, &history)
}
fn status(loaded: &Loaded) -> Result<(), String> {
    let (provider, key) = loaded.selected();
    println!(
        "Provider: {} (local)\nKey: {}\nActive version: {}\nSettings file: {}",
        loaded.provider,
        loaded.key,
        if key.active_version.is_empty() {
            "(none)"
        } else {
            &key.active_version
        },
        loaded.profile_path.display()
    );
    for version in &key.versions {
        let pem = read_material(
            &loaded.project.join(&provider.root),
            &version.certificate,
            false,
        )?;
        let public = PublicMaterial::from_certificate_pem(&pem).map_err(|_| MATERIAL)?;
        let (start, end) = public.certificate_validity().ok_or(MATERIAL)?;
        println!(
            "Version: {} ({:?})\n  Certificate SHA-256: {}\n  Certificate not before: {}\n  Certificate not after: {}\n  Policy expiry: none (certificate validity is separate)",
            version.version,
            version.state,
            hex(&public.certificate_pin().ok_or(MATERIAL)?),
            start,
            end
        );
    }
    loaded.ensure()
}
fn password(name: &str) -> Result<String, String> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        || name.starts_with("APP_")
    {
        return Err(
            "password_env must explicitly name a separate uppercase environment variable".into(),
        );
    }
    let p =
        std::env::var(name).map_err(|_| "configured password environment source is unavailable")?;
    if p.is_empty() || p.len() > 4096 || p.contains('\0') {
        return Err("configured password environment source is invalid".into());
    }
    Ok(p)
}
fn confirmed(
    yes: bool,
    terminal: bool,
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<(), String> {
    if yes {
        return Ok(());
    }
    if !terminal {
        return Err("rotation needs an interactive terminal confirmation or explicit --yes; nothing changed".into());
    }
    write!(
        output,
        "Approve this exact rotation and settings update? Type yes: "
    )
    .and_then(|()| output.flush())
    .map_err(|_| "cannot display confirmation")?;
    let mut answer = String::new();
    io::Read::take(input, 64)
        .read_line(&mut answer)
        .map_err(|_| "cannot read confirmation")?;
    if answer.trim() != "yes" {
        return Err("rotation declined; nothing changed".into());
    }
    Ok(())
}
fn rotate(loaded: &Loaded, command: &Command, history: &[PublicMaterial]) -> Result<(), String> {
    let (provider, key) = loaded.selected();
    if command.from.is_none() && provider.mode != "self_signed_test" {
        return Err("imported/CA-issued certificates require explicit --from replacement PFX; renewal is not available".into());
    }
    let secret = password(&provider.password_env)?;
    let key_dir_path = loaded.project.join(&provider.root).join(&loaded.key);
    let key_dir = Directory::open(&key_dir_path, false)
        .map_err(|_| "key directory must already exist with owned, protected write access")?;
    let settings_dir = Directory::open(&loaded.settings_root, false).map_err(|_| INVALID)?;
    key_dir
        .writable()
        .and_then(|()| settings_dir.writable())
        .map_err(|_| "key/settings directory is unwritable")?;
    let mut number = 1_u64;
    while key
        .versions
        .iter()
        .any(|v| v.version == format!("v{number}"))
    {
        number = number.checked_add(1).ok_or(INVALID)?;
    }
    let version = format!("v{number}");
    key_dir.absent(&version).map_err(|_|"new version destination already exists or is unsafe; retained material must be inspected")?;
    let reference = KeyRef {
        provider: loaded.provider.clone(),
        key: loaded.key.clone(),
        version: version.clone(),
    };
    let imported = if let Some(path) = &command.from {
        let path = framework::absolute(path).map_err(|_| INVALID)?;
        let parent = Directory::open(path.parent().ok_or(INVALID)?, false).map_err(|_| INVALID)?;
        let source = parent
            .read(
                path.file_name().and_then(|s| s.to_str()).ok_or(INVALID)?,
                true,
            )
            .map_err(|_| "replacement PFX must be a protected private regular file")?;
        Some(LocalSigner::from_pkcs12(reference.clone(), &source, &secret).map_err(|_| MATERIAL)?)
    } else {
        None
    };
    if let Some(signer) = &imported {
        let public = signer.public_material();
        if history.iter().any(|p| p.same_public_key(&public)) {
            return Err(MATERIAL.into());
        }
    }
    println!(
        "Rotation plan\n  Provider: {}\n  Key: {}\n  Current version: {}\n  New version: {}\n  Settings file: {}\n  Material: {}",
        loaded.provider,
        loaded.key,
        if key.active_version.is_empty() {
            "(none)"
        } else {
            &key.active_version
        },
        version,
        loaded.profile_path.display(),
        if imported.is_some() {
            "explicit replacement encrypted PFX"
        } else {
            "fresh self-signed TEST certificate (30 days)"
        }
    );
    if let Some(signer) = &imported {
        println!(
            "  Replacement certificate SHA-256: {}",
            hex(&signer.public_material().certificate_pin().ok_or(MATERIAL)?)
        );
    }
    confirmed(
        command.yes,
        io::stdin().is_terminal() && io::stdout().is_terminal(),
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
    )?;
    loaded.ensure()?;
    let _lock = settings_dir
        .lock()
        .map_err(|_| "cannot acquire exclusive settings rotation lock; nothing activated")?;
    loaded.ensure()?;
    key_dir
        .absent(&version)
        .map_err(|_| "new version appeared since plan; nothing activated")?;
    let signer = match imported {
        Some(s) => s,
        None => {
            let generated = generate_self_signed(&loaded.key, &secret, 30).map_err(|_| MATERIAL)?;
            LocalSigner::from_pem(
                reference,
                &generated.encrypted_private_key_pem,
                &secret,
                &generated.certificate_pem,
            )
            .map_err(|_| MATERIAL)?
        }
    };
    let public = signer.public_material();
    if history.iter().any(|p| p.same_public_key(&public)) {
        return Err(MATERIAL.into());
    }
    let private = signer
        .encrypted_private_key_pem(&secret)
        .map_err(|_| MATERIAL)?;
    let certificate = public.certificate_pem().map_err(|_| MATERIAL)?;
    let next = VersionSettings {
        version: version.clone(),
        certificate: format!("{}/{version}/certificate.pem", loaded.key),
        private_key: Some(format!("{}/{version}/private.pem", loaded.key)),
        certificate_pin: hex(&public.certificate_pin().ok_or(MATERIAL)?),
        state: KeyState::Active,
    };
    let updated = updated_profile(loaded, &next)?;
    // All validation precedes publication; settings activation is the last mutation.
    loaded.ensure()?;
    key_dir
        .publish(
            &version,
            &[
                (
                    "private.pem",
                    String::from_utf8(private).map_err(|_| MATERIAL)?,
                ),
                (
                    "certificate.pem",
                    String::from_utf8(certificate).map_err(|_| MATERIAL)?,
                ),
            ],
        )
        .map_err(|_| "cannot publish new private version; active settings unchanged")?;
    loaded.ensure().and_then(|()|loaded.profile.replace(updated.as_bytes()).map_err(|_|"settings activation failed or durability is uncertain; inspect selected settings; retained new version is never deleted".into()))?;
    println!(
        "Activated provider {} key {} version {} in {}. Previous public/private versions retained.",
        loaded.provider,
        loaded.key,
        version,
        loaded.profile_path.display()
    );
    Ok(())
}
fn updated_profile(loaded: &Loaded, next: &VersionSettings) -> Result<String, String> {
    let text = std::str::from_utf8(&loaded.profile.bytes).map_err(|_| INVALID)?;
    let mut document = text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|_| INVALID)?;
    let (_, key) = loaded.selected();
    let mut target = document.as_item_mut();
    for part in [
        "signing",
        "providers",
        &loaded.provider,
        "keys",
        &loaded.key,
    ] {
        target = &mut target[part];
        if target.is_none() {
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            *target = toml_edit::Item::Table(table);
        } else if let Some(inline) = target.as_inline_table() {
            // Expand only the selected key path. Copy formatted keys/values
            // rather than into_table(), which normalizes away decorations.
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            *table.decor_mut() = inline.decor().clone();
            for (name, value) in inline.iter() {
                table.insert_formatted(
                    inline.key(name).ok_or(INVALID)?,
                    toml_edit::Item::Value(value.clone()),
                );
            }
            *target = toml_edit::Item::Table(table);
        }
    }
    // An overlay replaces the complete array. If it has no array, copy the
    // declared base representation, including retained operator annotations.
    if target.get("versions").is_none() {
        let base = std::str::from_utf8(&loaded.base.bytes)
            .map_err(|_| INVALID)?
            .parse::<toml_edit::DocumentMut>()
            .map_err(|_| INVALID)?;
        target["versions"] =
            base["signing"]["providers"][&loaded.provider]["keys"][&loaded.key]["versions"].clone();
    }
    let mut new_record = toml_edit::InlineTable::new();
    new_record.insert("version", next.version.clone().into());
    new_record.insert("certificate", next.certificate.clone().into());
    if let Some(private) = &next.private_key {
        new_record.insert("private_key", private.clone().into());
    }
    new_record.insert("certificate_pin", next.certificate_pin.clone().into());
    new_record.insert("state", "active".into());
    if let Some(array) = target["versions"].as_array_mut() {
        for entry in array.iter_mut() {
            let table = entry.as_inline_table_mut().ok_or(INVALID)?;
            if table.get("version").and_then(toml_edit::Value::as_str)
                == Some(key.active_version.as_str())
            {
                replace_annotated_string(table.get_mut("state").ok_or(INVALID)?, "retired");
            }
        }
        let mut new_value = toml_edit::Value::InlineTable(new_record);
        if array
            .trailing()
            .as_str()
            .is_some_and(|text| text.contains('#'))
        {
            // The old trailing comment belongs after the previous last record,
            // before the newly appended record rather than after it.
            new_value.decor_mut().set_prefix(array.trailing().clone());
            array.set_trailing("\n");
            array.push_formatted(new_value);
        } else {
            array.push(new_value);
        }
    } else if let Some(tables) = target["versions"].as_array_of_tables_mut() {
        for table in tables.iter_mut() {
            if table.get("version").and_then(toml_edit::Item::as_str)
                == Some(key.active_version.as_str())
            {
                replace_annotated_string(
                    table
                        .get_mut("state")
                        .and_then(toml_edit::Item::as_value_mut)
                        .ok_or(INVALID)?,
                    "retired",
                );
            }
        }
        tables.push(new_record.into_table());
    } else {
        return Err(INVALID.into());
    }
    if let Some(value) = target
        .get_mut("active_version")
        .and_then(toml_edit::Item::as_value_mut)
    {
        replace_annotated_string(value, &next.version);
    } else {
        target["active_version"] = toml_edit::value(&next.version);
    }
    let result = document.to_string();
    if result.len() > 65536 {
        return Err("updated settings would exceed the shared loader size limit".into());
    }
    Ok(result)
}

// Preserve field and value comments while changing only the selected string.
fn replace_annotated_string(value: &mut toml_edit::Value, text: &str) {
    let decor = value.decor().clone();
    *value = toml_edit::Value::from(text);
    *value.decor_mut() = decor;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confirmation_is_explicit_and_bounded() {
        for (yes, tty, input, ok) in [
            (false, false, "yes\n", false),
            (false, true, "no\n", false),
            (false, true, "yes\n", true),
            (true, false, "", true),
        ] {
            assert_eq!(
                confirmed(yes, tty, &mut io::Cursor::new(input), &mut Vec::new()).is_ok(),
                ok
            );
        }
    }
    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn settings_transaction_checks_stale_plan_lock_and_replacement() {
        let temp = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        fs::create_project(
            temp.path(),
            "transaction",
            &[("production.toml", "old".into())],
        )
        .unwrap();
        let root = temp.path().join("transaction");
        let selected = root.join("production.toml");
        let snapshot = Snapshot::open(&selected, true).unwrap();
        let directory = Directory::open(&root, false).unwrap();
        let lock = directory.lock().unwrap();
        assert!(directory.lock().is_err());
        drop(lock);
        let _again = directory.lock().unwrap();
        snapshot.replace(b"new").unwrap();
        assert_eq!(std::fs::read(&selected).unwrap(), b"new");
        assert!(snapshot.replace(b"stale").is_err());
        assert_eq!(std::fs::read(&selected).unwrap(), b"new");
        let snapshot = Snapshot::open(&selected, true).unwrap();
        // Occupied staging name forces an atomic-update failure; foreign data remains.
        let temporary = root.join(format!(".yetlet-key-settings-{}", std::process::id()));
        std::fs::write(&temporary, b"foreign").unwrap();
        assert!(snapshot.replace(b"not activated").is_err());
        assert_eq!(std::fs::read(&selected).unwrap(), b"new");
        assert_eq!(std::fs::read(&temporary).unwrap(), b"foreign");
    }

    #[test]
    fn parser_is_exclusive_and_literal() {
        for args in [
            vec![
                "rotate",
                "documents",
                "--provider=local_dev",
                "--yes",
                "--from=./pfx",
            ],
            vec!["status"],
            vec!["status", "--manifest-path=./space app/Cargo.toml"],
        ] {
            assert!(parse(&args.iter().map(OsString::from).collect::<Vec<_>>()).is_ok());
        }
        for args in [
            vec!["sign"],
            vec!["status", "--yes"],
            vec!["rotate", "--yes=false"],
            vec!["rotate", "--yes", "--yes"],
            vec!["status", "a", "b"],
            vec!["status", "--provider=local-dev"],
            vec!["rotate", "--help", "a"],
        ] {
            assert!(parse(&args.iter().map(OsString::from).collect::<Vec<_>>()).is_err());
        }
    }
}
