//! Flat unsigned local checksums. Descriptor-relative traversal deliberately fails closed.
use crate::tooling::{Result, fail};
use serde::{
    Deserialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fmt, path::Path};
pub const MANIFEST: &str = "RELEASE-CHECKSUMS.json";
pub const NOTICE: &str = "unsigned local verification; not trusted provenance";
pub const MAX_FILES: usize = 128;
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
pub fn safe_name(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b.len() <= 128
        && b[0].is_ascii_alphanumeric()
        && b.iter()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(c))
        && s != MANIFEST
}
// serde_json::Value otherwise silently accepts duplicate keys, including nested entries.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("unique JSON")
            }
            fn visit_map<M: MapAccess<'de>>(
                self,
                mut m: M,
            ) -> std::result::Result<Unique, M::Error> {
                let mut out = serde_json::Map::new();
                while let Some((k, v)) = m.next_entry::<String, Unique>()? {
                    if out.insert(k, v.0).is_some() {
                        return Err(de::Error::custom("duplicate JSON key"));
                    }
                }
                Ok(Unique(Value::Object(out)))
            }
            fn visit_seq<S: SeqAccess<'de>>(
                self,
                mut s: S,
            ) -> std::result::Result<Unique, S::Error> {
                let mut a = vec![];
                while let Some(v) = s.next_element::<Unique>()? {
                    a.push(v.0)
                }
                Ok(Unique(json!(a)))
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
        }
        d.deserialize_any(V)
    }
}
pub fn parse_manifest(raw: &[u8]) -> Result<Vec<Value>> {
    if raw.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(fail("manifest size limit"));
    }
    let Unique(v) = serde_json::from_slice(raw).map_err(|_| fail("invalid manifest JSON"))?;
    fn keys(v: &Value, expected: &[&str]) -> bool {
        v.as_object().is_some_and(|o| {
            o.len() == expected.len() && expected.iter().all(|s| o.contains_key(*s))
        })
    }
    if !keys(&v, &["schema_version", "notice", "artifacts"])
        || v["schema_version"].as_u64() != Some(1)
        || v["notice"] != NOTICE
    {
        return Err(fail("invalid manifest identity"));
    }
    let entries = v["artifacts"]
        .as_array()
        .ok_or_else(|| fail("invalid manifest entries"))?;
    if entries.is_empty() || entries.len() > MAX_FILES {
        return Err(fail("artifact count limit"));
    }
    let mut names = BTreeSet::new();
    let mut total = 0;
    for e in entries {
        let name = e["path"]
            .as_str()
            .ok_or_else(|| fail("artifact path type"))?;
        let size = e["size"]
            .as_u64()
            .ok_or_else(|| fail("artifact size type"))?;
        let hash = e["sha256"]
            .as_str()
            .ok_or_else(|| fail("artifact hash type"))?;
        if size > MAX_FILE_BYTES {
            return Err(fail("invalid artifact entry"));
        }
        total += size; // Each size and the preceding total are already bounded.
        if !keys(e, &["path", "size", "sha256"])
            || !safe_name(name)
            || !names.insert(name)
            || size > MAX_FILE_BYTES
            || total > MAX_TOTAL_BYTES
            || hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(fail("invalid artifact entry"));
        }
    }
    let mut entries = entries.clone();
    entries.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(entries)
}
#[cfg(unix)]
mod posix {
    use super::*;
    use std::{
        ffi::{CStr, CString},
        fs::File,
        io::{Read, Write},
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::MetadataExt,
        },
        path::Component,
    };
    fn c(s: &std::ffi::OsStr) -> Result<CString> {
        use std::os::unix::ffi::OsStrExt;
        CString::new(s.as_bytes()).map_err(|_| fail("invalid path"))
    }
    pub fn directory(path: &Path) -> Result<File> {
        if path.as_os_str().is_empty()
            || path.components().any(|p| matches!(p, Component::ParentDir))
        {
            return Err(fail("unsafe directory"));
        }
        let absolute = std::env::current_dir()?.join(path);
        let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
        // Each opened ancestor remains a directory descriptor; no canonicalize race.
        let fd = unsafe { libc::open(c(Path::new("/").as_os_str())?.as_ptr(), flags) };
        if fd < 0 {
            return Err(fail("directory open"));
        }
        let mut dir = unsafe { File::from_raw_fd(fd) };
        for part in absolute.components() {
            if let Component::Normal(name) = part {
                let next = unsafe { libc::openat(dir.as_raw_fd(), c(name)?.as_ptr(), flags) };
                if next < 0 {
                    return Err(fail("directory no-follow"));
                }
                dir = unsafe { File::from_raw_fd(next) };
            }
        }
        Ok(dir)
    }
    fn fingerprint(m: &std::fs::Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
        (
            m.dev(),
            m.ino(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        )
    }
    fn stat(dir: &File, name: &str) -> Result<libc::stat> {
        let mut st = std::mem::MaybeUninit::uninit();
        if unsafe {
            libc::fstatat(
                dir.as_raw_fd(),
                c(name.as_ref())?.as_ptr(),
                st.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(fail("file stat"));
        }
        Ok(unsafe { st.assume_init() })
    }
    pub fn read(dir: &File, name: &str, limit: u64) -> Result<Vec<u8>> {
        read_with(dir, name, limit, || {}, || {})
    }
    fn read_with(
        dir: &File,
        name: &str,
        limit: u64,
        before_open: impl FnOnce(),
        after_open: impl FnOnce(),
    ) -> Result<Vec<u8>> {
        let before = stat(dir, name)?;
        if before.st_mode & libc::S_IFMT != libc::S_IFREG
            || before.st_size < 0
            || before.st_size as u64 > limit
        {
            return Err(fail("nonregular or oversized file"));
        }
        before_open();
        let fd = unsafe {
            libc::openat(
                dir.as_raw_fd(),
                c(name.as_ref())?.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(fail("file no-follow"));
        }
        let mut file = unsafe { File::from_raw_fd(fd) };
        let m = file.metadata()?;
        #[cfg(target_os = "linux")]
        let times_match = m.mtime() == before.st_mtime
            && m.mtime_nsec() == before.st_mtime_nsec
            && m.ctime() == before.st_ctime
            && m.ctime_nsec() == before.st_ctime_nsec;
        #[cfg(not(target_os = "linux"))]
        let times_match = false; // This platform's stat timestamp layout is not qualified.
        if m.dev() != before.st_dev
            || m.ino() != before.st_ino
            || m.len() != before.st_size as u64
            || !times_match
        {
            return Err(fail("file changed while opening"));
        }
        after_open();
        let mut bytes = vec![];
        (&mut file).take(limit + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != m.len()
            || bytes.len() as u64 > limit
            || fingerprint(&m) != fingerprint(&file.metadata()?)
        {
            return Err(fail("file changed while reading"));
        }
        Ok(bytes)
    }
    pub fn artifacts(dir: &File) -> Result<Vec<Value>> {
        // fdopendir owns only its duplicate; all later opens use the original descriptor.
        let fd = unsafe { libc::dup(dir.as_raw_fd()) };
        if fd < 0 {
            return Err(fail("directory duplicate"));
        }
        let stream = unsafe { libc::fdopendir(fd) };
        if stream.is_null() {
            unsafe { libc::close(fd) };
            return Err(fail("directory scan"));
        }
        let mut names = BTreeSet::new();
        let result = (|| {
            loop {
                let e = unsafe { libc::readdir(stream) };
                if e.is_null() {
                    break;
                }
                let name = unsafe { CStr::from_ptr((*e).d_name.as_ptr()) }
                    .to_str()
                    .map_err(|_| fail("artifact name encoding"))?;
                if [".", "..", MANIFEST].contains(&name) {
                    continue;
                }
                if !safe_name(name) {
                    return Err(fail("unsafe artifact name"));
                }
                names.insert(name.to_owned());
                if names.len() > MAX_FILES {
                    return Err(fail("artifact count limit"));
                }
            }
            if names.is_empty() {
                return Err(fail("empty artifact input"));
            }
            let mut total = 0;
            let mut entries = vec![];
            for name in names {
                let bytes = read(dir, &name, MAX_FILE_BYTES)?;
                total += bytes.len() as u64;
                if total > MAX_TOTAL_BYTES {
                    return Err(fail("total size limit"));
                }
                entries.push(
                    json!({"path":name,"size":bytes.len(),"sha256":crate::tooling::sha(&bytes)}),
                );
            }
            Ok(entries)
        })();
        unsafe { libc::closedir(stream) };
        result
    }
    pub fn create(dir: &File, bytes: &[u8]) -> Result<()> {
        let name = c(MANIFEST.as_ref())?;
        let fd = unsafe {
            libc::openat(
                dir.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(fail("manifest already exists or cannot create"));
        }
        let mut file = unsafe { File::from_raw_fd(fd) };
        if file.write_all(bytes).and_then(|()| file.flush()).is_err() {
            unsafe { libc::unlinkat(dir.as_raw_fd(), name.as_ptr(), 0) };
            return Err(fail("manifest write"));
        }
        Ok(())
    }
    #[cfg(test)]
    mod race_tests {
        use super::*;
        #[test]
        fn replacement_between_stat_and_open_is_rejected() {
            let t = tempfile::tempdir().unwrap();
            let p = t.path().join("artifact");
            std::fs::write(&p, b"old").unwrap();
            let dir = directory(t.path()).unwrap();
            assert!(
                read_with(
                    &dir,
                    "artifact",
                    10,
                    || {
                        std::fs::rename(&p, t.path().join("saved")).unwrap();
                        std::fs::write(&p, b"new").unwrap();
                    },
                    || {}
                )
                .is_err()
            );
        }
        #[test]
        fn mutation_after_open_is_rejected() {
            let t = tempfile::tempdir().unwrap();
            let p = t.path().join("artifact");
            std::fs::write(&p, b"old").unwrap();
            let dir = directory(t.path()).unwrap();
            assert!(
                read_with(
                    &dir,
                    "artifact",
                    10,
                    || {},
                    || {
                        std::fs::write(&p, b"changed").unwrap();
                    }
                )
                .is_err()
            );
        }
        #[test]
        fn symlink_replacement_between_stat_and_open_is_rejected() {
            let t = tempfile::tempdir().unwrap();
            let p = t.path().join("artifact");
            std::fs::write(&p, b"old").unwrap();
            let dir = directory(t.path()).unwrap();
            assert!(
                read_with(
                    &dir,
                    "artifact",
                    10,
                    || {
                        std::fs::remove_file(&p).unwrap();
                        std::os::unix::fs::symlink("outside", &p).unwrap();
                    },
                    || {}
                )
                .is_err()
            );
        }
    }
}
pub fn run(command: &str, directory: &Path) -> Result<()> {
    let result = (|| {
        #[cfg(unix)]
        {
            let dir = posix::directory(directory)?;
            match command {
                "verify" => {
                    let expected =
                        parse_manifest(&posix::read(&dir, MANIFEST, MAX_MANIFEST_BYTES)?)?;
                    if expected != posix::artifacts(&dir)? {
                        return Err(fail("artifact mismatch"));
                    }
                }
                "create" => {
                    let entries = posix::artifacts(&dir)?;
                    let mut value = json!({"artifacts":entries,"notice":NOTICE,"schema_version":1});
                    sort_json(&mut value);
                    let bytes = crate::tooling::json_bytes(&value);
                    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
                        return Err(fail("manifest size limit"));
                    }
                    posix::create(&dir, &bytes)?;
                }
                _ => return Err(fail("unknown checksum operation")),
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = (command, directory);
            Err(fail("requires POSIX no-follow support"))
        }
    })();
    result.map_err(|_| fail("release checksum preflight failed"))?;
    println!("release checksum {command}: PASS ({NOTICE})");
    Ok(())
}

fn sort_json(v: &mut Value) {
    match v {
        Value::Object(o) => {
            o.sort_keys();
            for v in o.values_mut() {
                sort_json(v)
            }
        }
        Value::Array(a) => {
            for v in a {
                sort_json(v)
            }
        }
        _ => {}
    }
}
