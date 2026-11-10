//! Public sparse-index transport and atomic complete-set cache. No Cargo cache parsing.
use crate::sdk_version::{self as sdk, Request, ResolvedSdk};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub(crate) const LIMIT: usize = 8 * 1024 * 1024;
const CACHE_LIMIT: u64 = 4 * LIMIT as u64;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub name: String,
    pub body: String,
    pub sha256: String,
    pub validated_at: u64,
    pub etag: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    schema: u32,
    registry: String,
    records: Vec<Record>,
}

pub(crate) enum Reply {
    Modified(String, Option<String>),
    NotModified,
}
pub(crate) trait Registry {
    fn fetch(
        &mut self,
        package: &str,
        etag: Option<&str>,
        remaining: Duration,
    ) -> Result<Reply, String>;
}
pub(crate) struct CratesIo;
impl Registry for CratesIo {
    fn fetch(
        &mut self,
        package: &str,
        etag: Option<&str>,
        remaining: Duration,
    ) -> Result<Reply, String> {
        let suffix = match package {
            "yetlet" => "ye/tl/yetlet",
            "yetlet-browser" => "ye/tl/yetlet-browser",
            _ => return Err("SDK_METADATA_INVALID: unsupported package".into()),
        };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(remaining))
            .max_redirects(0)
            .build()
            .into();
        let mut request = agent.get(format!("{}{suffix}", sdk::REGISTRY)).header(
            "User-Agent",
            concat!("yetlet-cli/", env!("CARGO_PKG_VERSION")),
        );
        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        let mut response = request.call().map_err(|_| format!("SDK_REGISTRY_UNAVAILABLE: {package}; refresh failed; retry explicitly with --offline if cache is prepared"))?;
        if response.status() == 304 {
            return Ok(Reply::NotModified);
        }
        if response.status() != 200 {
            return Err(format!(
                "SDK_REGISTRY_UNAVAILABLE: {package}; unexpected HTTP status"
            ));
        }
        let etag = response
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .filter(|v| v.len() <= 1024)
            .map(str::to_owned);
        let body = response
            .body_mut()
            .with_config()
            .limit(LIMIT as u64)
            .read_to_string()
            .map_err(|_| {
                format!("SDK_REGISTRY_UNAVAILABLE: {package}; bounded metadata read failed")
            })?;
        Ok(Reply::Modified(body, etag))
    }
}

pub(crate) fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| "SDK_METADATA_INVALID: system clock precedes epoch".into())
}

pub(crate) fn home() -> Result<PathBuf, String> {
    let path = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                .map(|p| PathBuf::from(p).join(".cargo"))
        })
        .ok_or("SDK_OFFLINE_CACHE_MISS: Cargo home unavailable")?;
    crate::framework::absolute(&path)
        .map(|p| p.join("yetlet/sdk-index-v1"))
        .map_err(|_| "SDK_METADATA_INVALID: unsafe cache path".into())
}

fn validate(cache: &Cache, names: &[&str], now: u64) -> Result<(), String> {
    if cache.schema != 1 || cache.registry != sdk::REGISTRY || cache.records.len() != names.len() {
        return Err("SDK_METADATA_INVALID: incomplete cache set or schema".into());
    }
    for name in names {
        let matches: Vec<_> = cache.records.iter().filter(|r| r.name == *name).collect();
        if matches.len() != 1 {
            return Err("SDK_METADATA_INVALID: duplicate/missing cached package".into());
        }
        let record = matches[0];
        if record.validated_at > now
            || sdk::digest(record.body.as_bytes()) != record.sha256
            || record
                .etag
                .as_ref()
                .is_some_and(|e| e.len() > 1024 || e.contains(['\r', '\n']))
        {
            return Err(format!(
                "SDK_METADATA_INVALID: {name}: cache digest/time/validator rejected"
            ));
        }
        sdk::index(name, &record.body)?;
    }
    Ok(())
}

fn physical(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("SDK_METADATA_INVALID: cache links are forbidden".into());
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err("SDK_METADATA_INVALID: cache path inaccessible".into()),
        }
    }
    Ok(())
}

fn load(path: &Path, names: &[&str], now: u64) -> Result<Option<Cache>, String> {
    physical(path)?;
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("SDK_METADATA_INVALID: cache unreadable".into()),
    };
    if !meta.is_file() || meta.len() > CACHE_LIMIT {
        return Err("SDK_METADATA_INVALID: cache file rejected".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|f| f.take(CACHE_LIMIT + 1).read_to_end(&mut bytes))
        .map_err(|_| "SDK_METADATA_INVALID: cache read failed")?;
    if bytes.len() as u64 > CACHE_LIMIT {
        return Err("SDK_METADATA_INVALID: cache exceeds limit".into());
    }
    let cache: Cache =
        serde_json::from_slice(&bytes).map_err(|_| "SDK_METADATA_INVALID: cache JSON rejected")?;
    validate(&cache, names, now)?;
    Ok(Some(cache))
}

fn save(path: &Path, cache: &Cache) -> Result<(), String> {
    use std::io::Write;
    physical(path)?;
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent)
        .map_err(|_| "SDK_METADATA_INVALID: cannot create cache directory")?;
    physical(path)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|_| "SDK_METADATA_INVALID: cannot stage metadata cache")?;
    serde_json::to_writer(temp.as_file_mut(), cache)
        .map_err(|_| "SDK_METADATA_INVALID: cannot serialize metadata cache")?;
    temp.flush()
        .and_then(|()| temp.as_file().sync_all())
        .map_err(|_| "SDK_METADATA_INVALID: cannot sync metadata cache")?;
    temp.persist(path)
        .map_err(|_| "SDK_METADATA_INVALID: cannot publish complete metadata cache")?;
    Ok(())
}

pub(crate) fn resolve(
    request: Request<'_>,
    registry: &mut impl Registry,
    root: &Path,
    now: u64,
) -> Result<ResolvedSdk, String> {
    let result = (|| {
        if let Some(version) = request.exact {
            sdk::exact(version).map_err(str::to_owned)?;
        }
        let names: &[&str] = if request.template.browser() {
            &["yetlet", "yetlet-browser"]
        } else {
            &["yetlet"]
        };
        let path = root.join(if request.template.browser() {
            "crates-io-browser.json"
        } else {
            "crates-io-native.json"
        });
        let previous = load(&path, names, now)?;
        let cache = if request.offline {
            previous.ok_or("SDK_OFFLINE_CACHE_MISS: required metadata set absent; prepare with an online new first")?
        } else {
            let deadline = Instant::now() + Duration::from_secs(30);
            let mut records = Vec::new();
            for name in names {
                let prior = previous
                    .as_ref()
                    .and_then(|p| p.records.iter().find(|r| r.name == *name));
                let remaining = deadline
                    .checked_duration_since(Instant::now())
                    .ok_or("SDK_REGISTRY_UNAVAILABLE: total metadata deadline")?;
                let (body, etag) =
                    match registry.fetch(name, prior.and_then(|r| r.etag.as_deref()), remaining)? {
                        Reply::Modified(body, etag) => (body, etag),
                        Reply::NotModified => {
                            let prior =
                                prior.ok_or("SDK_METADATA_INVALID: 304 without accepted cache")?;
                            (prior.body.clone(), prior.etag.clone())
                        }
                    };
                sdk::index(name, &body)?;
                records.push(Record {
                    name: (*name).into(),
                    sha256: sdk::digest(body.as_bytes()),
                    body,
                    etag,
                    validated_at: now,
                });
            }
            let cache = Cache {
                schema: 1,
                registry: sdk::REGISTRY.into(),
                records,
            };
            validate(&cache, names, now)?;
            save(&path, &cache)?;
            cache
        };
        let policy = serde_json::from_str(sdk::POLICY)
            .map_err(|_| "SDK_METADATA_INVALID: compiled compatibility policy")?;
        sdk::select(
            request,
            &cache.records,
            if request.offline {
                "offline-cache"
            } else {
                "registry-refresh"
            },
            &policy,
        )
    })();
    result.map_err(|e: String| {
        format!(
            "{e}; mode={} template={} request={}",
            if request.offline { "offline" } else { "online" },
            request.template.selector(),
            request.exact.unwrap_or("latest-semver")
        )
    })
}
