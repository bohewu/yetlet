//! Application SDK policy and selection, independent of the CLI's build SDK.
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const REGISTRY: &str = "https://index.crates.io/";
pub(crate) const CARGO_REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
pub(crate) const POLICY: &str = include_str!("../resources/sdk-compatibility.json");

pub(crate) fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

pub fn exact(value: &str) -> Result<Version, &'static str> {
    let version =
        Version::parse(value).map_err(|_| "--sdk-version requires canonical exact SemVer")?;
    if value.len() > 128
        || !value.is_ascii()
        || !version.build.is_empty()
        || version.to_string() != value
    {
        return Err("--sdk-version requires canonical exact SemVer without build metadata");
    }
    Ok(version)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Request<'a> {
    pub exact: Option<&'a str>,
    pub template: crate::scaffold::Template,
    pub settings: bool,
    pub offline: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Package {
    pub name: String,
    pub checksum: String,
    pub metadata_sha256: String,
    pub validated_at: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ResolvedSdk {
    pub version: String,
    pub packages: Vec<Package>,
    pub registry: &'static str,
    pub source: &'static str,
    pub policy: String,
    pub policy_sha256: String,
    pub template: &'static str,
    pub requested: Option<String>,
    pub prerelease: bool,
    #[serde(skip)]
    pub(crate) m26_startup: bool,
}

impl ResolvedSdk {
    pub(crate) fn notice(&self, now: u64) -> String {
        let mut value = format!(
            "SDK {} from {}; template={}; policy={}",
            self.version, self.source, self.template, self.policy
        );
        if self.prerelease {
            value.push_str("; Experimental prerelease");
        }
        if self.source == "offline-cache" {
            value.push_str(
                "; latest known to this cache; current registry and yank status not checked",
            );
        }
        for package in &self.packages {
            value.push_str(&format!(
                "; {} validated_at={} age={}s",
                package.name,
                package.validated_at,
                now.saturating_sub(package.validated_at)
            ));
        }
        value
    }
}

pub(crate) struct Policy {
    pub revision: String,
    pub selection: String,
    scaffold_capabilities: ScaffoldCapabilities,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyDocument {
    revision: String,
    selection: String,
    scaffold_capabilities: ScaffoldCapabilities,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScaffoldCapabilities {
    m26_startup: Vec<StartupCapability>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartupCapability {
    sdk_version: String,
    yetlet_sha256: String,
}

impl Policy {
    pub(crate) fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > 16_384 {
            return Err(
                "SDK_METADATA_INVALID: compiled compatibility policy exceeds byte limit".into(),
            );
        }
        let document: PolicyDocument = serde_json::from_str(raw)
            .map_err(|e| format!("SDK_METADATA_INVALID: compiled compatibility policy: {e}"))?;
        // Serde structs also accept positional sequences. Require JSON objects,
        // while the typed parse above detects duplicate and unknown raw keys.
        let shape: serde_json::Value = serde_json::from_str(raw)
            .map_err(|e| format!("SDK_METADATA_INVALID: compiled compatibility policy: {e}"))?;
        if !shape.is_object()
            || !shape["scaffold_capabilities"].is_object()
            || !shape["scaffold_capabilities"]["m26_startup"]
                .as_array()
                .is_some_and(|rows| rows.iter().all(serde_json::Value::is_object))
        {
            return Err("SDK_METADATA_INVALID: compatibility policy requires JSON objects".into());
        }
        if document.revision != "sdk-latest-v2" || document.selection != "latest-semver" {
            return Err("SDK_METADATA_INVALID: unsupported compiled SDK selection policy".into());
        }
        let mut versions = BTreeSet::new();
        for row in &document.scaffold_capabilities.m26_startup {
            if exact(&row.sdk_version).is_err()
                || !versions.insert(&row.sdk_version)
                || row.yetlet_sha256.len() != 64
                || !row
                    .yetlet_sha256
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            {
                return Err("SDK_METADATA_INVALID: m26_startup version, duplicate version or checksum rejected".into());
            }
        }
        Ok(Self {
            revision: document.revision,
            selection: document.selection,
            scaffold_capabilities: document.scaffold_capabilities,
            sha256: digest(raw.as_bytes()),
        })
    }
}

#[derive(Clone, Deserialize)]
pub(crate) struct IndexVersion {
    pub name: String,
    pub vers: String,
    pub cksum: String,
    pub yanked: bool,
    pub features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub features2: BTreeMap<String, Vec<String>>,
    pub deps: Vec<Dependency>,
    #[serde(default = "index_schema")]
    pub v: u32,
}
fn index_schema() -> u32 {
    1
}

#[derive(Clone, Deserialize)]
pub(crate) struct Dependency {
    pub name: String,
    pub req: String,
    pub package: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

pub(crate) fn index(name: &str, raw: &str) -> Result<Vec<IndexVersion>, String> {
    if raw.is_empty() || raw.len() > crate::sdk_registry::LIMIT {
        return Err(format!(
            "SDK_METADATA_INVALID: {name}: empty or oversized index"
        ));
    }
    let mut seen = BTreeSet::new();
    raw.lines().map(|line| {
        let row: IndexVersion = serde_json::from_str(line)
            .map_err(|_| format!("SDK_METADATA_INVALID: {name}: invalid index row"))?;
        let mut version = Version::parse(&row.vers)
            .map_err(|_| format!("SDK_METADATA_INVALID: {name}: invalid SemVer"))?;
        version.build = semver::BuildMetadata::EMPTY;
        if row.name != name || !matches!(row.v, 1 | 2) || !seen.insert(version)
            || row.cksum.len() != 64 || !row.cksum.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            || row.deps.iter().any(|d| VersionReq::parse(&d.req).is_err()) {
            return Err(format!("SDK_METADATA_INVALID: {name}: identity, schema, duplicate version or checksum rejected"));
        }
        Ok(row)
    }).collect()
}

impl crate::scaffold::Template {
    pub(crate) fn selector(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Browser => "browser",
            Self::NativeStyle => "native-style",
            Self::BrowserStyle => "browser-style",
        }
    }
    pub(crate) fn browser(self) -> bool {
        matches!(self, Self::Browser | Self::BrowserStyle)
    }
}

fn feature(row: &IndexVersion, name: &str, seen: &mut BTreeSet<String>) -> bool {
    if !seen.insert(name.to_owned()) {
        return true;
    }
    let values = row.features2.get(name).or_else(|| row.features.get(name));
    let Some(values) = values else {
        return row.deps.iter().any(|d| d.name == name && d.optional);
    };
    values.iter().all(|value| {
        if let Some(dep) = value.strip_prefix("dep:") {
            row.deps.iter().any(|d| d.name == dep && d.optional)
        } else if let Some((dep, _)) = value.split_once('/') {
            row.deps.iter().any(|d| d.name == dep.trim_end_matches('?'))
        } else {
            feature(row, value, seen)
        }
    })
}

// Validate the selected version, never use these checks to choose an older SDK.
// Cargo owns toolchain and transitive dependency feasibility; the CLI build SDK
// and its Rust minimum do not bound application SDK selection.
fn compatible(request: Request<'_>, version: &str, rows: &[&IndexVersion]) -> Result<(), String> {
    for row in rows {
        let required: &[&str] = if row.name == "yetlet-browser" {
            &["dom", "dom-keyed"]
        } else if request.template.browser() {
            &["reactive"]
        } else if request.settings {
            &["server-axum", "config"]
        } else {
            &["server-axum"]
        };
        for feature_name in required {
            if !feature(row, feature_name, &mut BTreeSet::new()) {
                return Err(format!(
                    "{} required feature {feature_name} is missing or invalid",
                    row.name
                ));
            }
        }
        if row.name == "yetlet-browser"
            && !row.deps.iter().any(|d| {
                d.package.as_deref().unwrap_or(&d.name) == "yetlet"
                    && VersionReq::parse(&d.req)
                        .is_ok_and(|r| r.matches(&Version::parse(version).unwrap()))
            })
        {
            return Err("yetlet-browser dependency does not admit the same yetlet version".into());
        }
    }
    Ok(())
}

pub(crate) fn select(
    request: Request<'_>,
    records: &[crate::sdk_registry::Record],
    source: &'static str,
    policy: &Policy,
) -> Result<ResolvedSdk, String> {
    if policy.revision != "sdk-latest-v2" || policy.selection != "latest-semver" {
        return Err("SDK_METADATA_INVALID: unsupported compiled SDK selection policy".into());
    }
    let core = records
        .iter()
        .find(|r| r.name == "yetlet")
        .ok_or("SDK_METADATA_INVALID: missing yetlet metadata")?;
    let core_rows = index("yetlet", &core.body)?;
    let companion = records.iter().find(|r| r.name == "yetlet-browser");
    let browser_rows = companion
        .map(|r| index("yetlet-browser", &r.body))
        .transpose()?;
    let available = |version: &str| -> Result<Vec<&IndexVersion>, String> {
        let row = core_rows.iter().find(|r| r.vers == version).ok_or_else(|| format!("SDK_VERSION_NOT_FOUND: {version} in {source}; refresh online or select a known version"))?;
        let mut rows = vec![row];
        if request.template.browser() {
            rows.push(
                browser_rows
                    .as_ref()
                    .and_then(|rows| rows.iter().find(|r| r.vers == version))
                    .ok_or_else(|| {
                        format!("SDK_PACKAGE_UNAVAILABLE: yetlet-browser {version} in {source}")
                    })?,
            );
        }
        if rows.iter().any(|r| r.yanked) {
            return Err(format!(
                "SDK_VERSION_YANKED: {version} in {source}; choose another SDK"
            ));
        }
        Ok(rows)
    };
    let version = match request.exact {
        Some(v) => {
            exact(v).map_err(str::to_owned)?;
            available(v)?;
            v.to_owned()
        }
        None => core_rows
            .iter()
            .filter(|r| available(&r.vers).is_ok())
            .filter_map(|r| exact(&r.vers).ok())
            .max()
            .ok_or_else(|| {
                format!(
                    "SDK_NO_COMPATIBLE_VERSION: template={} policy={}; no non-yanked SDK with all required packages in {source}",
                    request.template.selector(),
                    policy.revision
                )
            })?
            .to_string(),
    };
    let rows = available(&version)?;
    let capability = policy
        .scaffold_capabilities
        .m26_startup
        .iter()
        .find(|row| row.sdk_version == version);
    if let Some(capability) = capability
        && capability.yetlet_sha256 != rows[0].cksum
    {
        return Err(format!(
            "SDK_INCOMPATIBLE: {version} template={} policy={} policy_sha256={}: template-capability m26_startup core checksum contradicts selected package identity",
            request.template.selector(),
            policy.revision,
            policy.sha256
        ));
    }
    compatible(request, &version, &rows).map_err(|reason| {
        format!(
            "SDK_INCOMPATIBLE: {version} template={} policy={}: {reason}",
            request.template.selector(),
            policy.revision
        )
    })?;
    let packages = rows
        .iter()
        .map(|row| {
            let record = records.iter().find(|r| r.name == row.name).unwrap();
            Package {
                name: row.name.clone(),
                checksum: row.cksum.clone(),
                metadata_sha256: digest(record.body.as_bytes()),
                validated_at: record.validated_at,
            }
        })
        .collect();
    Ok(ResolvedSdk {
        prerelease: !Version::parse(&version).unwrap().pre.is_empty(),
        version,
        packages,
        registry: REGISTRY,
        source,
        policy: policy.revision.clone(),
        policy_sha256: policy.sha256.clone(),
        template: request.template.selector(),
        requested: request.exact.map(str::to_owned),
        m26_startup: capability.is_some(),
    })
}

#[cfg(test)]
mod tests;
