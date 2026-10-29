//! Encrypted transport of bounded private evidence. Transport is not builder authentication.
use super::checksums::{self, posix};
use crate::{
    process,
    tooling::{self, Result, fail},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::Duration,
};

pub const RECIPIENT_SHA: &str = "b861a3c7e2abf3caf4c7071c045f6114704db94e8cc39ce010e7fa10711ff506";
fn git_ref() -> String {
    format!("refs/tags/v{}", super::private::package_version())
}
fn workflow() -> String {
    format!("bohewu/yetlet/.github/workflows/release.yml@{}", git_ref())
}
const INVENTORY: &str = "CUSTODY-INVENTORY.json";
pub const CIPHER: &str = "private-evidence.tar.age";
pub const RECEIPT: &str = "transport-receipt.json";
const EXTRAS: [&str; 3] = [
    "openssl-3.5.9.tar.gz",
    "openssl-build.log",
    "openssl-build-result.json",
];
const OPENSSL_SHA: &str = "603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a";
const ARCHIVE_LIMIT: u64 = checksums::MAX_TOTAL_BYTES + 1024 * 1024;
const CIPHER_LIMIT: u64 = ARCHIVE_LIMIT + 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub source: String,
    pub workflow_sha: String,
    pub workflow: String,
    pub git_ref: String,
    pub run: String,
    pub attempt: String,
}
impl Binding {
    fn validate(&self) -> Result<()> {
        let number = |s: &str| {
            !s.is_empty()
                && s.len() <= 20
                && s.bytes().all(|b| b.is_ascii_digit())
                && s.bytes().any(|b| b != b'0')
        };
        if !super::preparation::commit_token(&self.source)
            || self.workflow_sha != self.source
            || self.workflow != workflow()
            || self.git_ref != git_ref()
            || !number(&self.run)
            || !number(&self.attempt)
        {
            return Err(fail("custody binding rejected"));
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Capture {
        private: PathBuf,
        extras: PathBuf,
        exit: i32,
    },
    Verify {
        transport: PathBuf,
        identity: PathBuf,
    },
}
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub operation: Operation,
    pub binding: Binding,
    pub age: PathBuf,
    pub recipient: PathBuf,
    pub output: PathBuf,
}
impl Options {
    pub fn parse(args: &[OsString]) -> std::result::Result<Self, &'static str> {
        let err = "release custody <capture|verify> requires closed named options";
        let operation = args.first().and_then(|a| a.to_str()).ok_or(err)?;
        if !["capture", "verify"].contains(&operation) || args.len() % 2 != 1 {
            return Err(err);
        }
        let mut values = BTreeMap::new();
        for pair in args[1..].as_chunks::<2>().0 {
            let key = pair[0].to_str().ok_or(err)?;
            if values.insert(key, pair[1].clone()).is_some() {
                return Err(err);
            }
        }
        let mut take = |key| values.remove(key).ok_or(err);
        let age = take("--age")?.into();
        let recipient = take("--recipient")?.into();
        let output = take("--output")?.into();
        let source = take("--source")?.into_string().map_err(|_| err)?;
        let run = take("--run")?.into_string().map_err(|_| err)?;
        let attempt = take("--attempt")?.into_string().map_err(|_| err)?;
        let operation = if operation == "capture" {
            Operation::Capture {
                private: take("--private")?.into(),
                extras: take("--extras")?.into(),
                exit: take("--exit")?
                    .to_str()
                    .ok_or(err)?
                    .parse::<i32>()
                    .map_err(|_| err)?,
            }
        } else {
            Operation::Verify {
                transport: take("--transport")?.into(),
                identity: take("--identity")?.into(),
            }
        };
        if !values.is_empty() {
            return Err(err);
        }
        let binding = Binding {
            workflow_sha: source.clone(),
            source,
            workflow: workflow(),
            git_ref: git_ref(),
            run,
            attempt,
        };
        binding.validate().map_err(|_| err)?;
        if matches!(operation, Operation::Capture { exit, .. } if !(0..=255).contains(&exit)) {
            return Err(err);
        }
        Ok(Self {
            operation,
            binding,
            age,
            recipient,
            output,
        })
    }
}
#[derive(Debug)]
struct Entry {
    member: String,
    origin: String,
    name: String,
    size: u64,
    sha256: String,
}
#[derive(Debug)]
struct Inventory {
    schema: u32,
    binding: Binding,
    recipient_sha256: String,
    original_exit: i32,
    status: String,
    complete: bool,
    private_manifest_sha256: Option<String>,
    files: Vec<Entry>,
    missing_extras: Vec<String>,
}
#[derive(Debug)]
struct Receipt {
    schema: u32,
    binding: Binding,
    recipient_sha256: String,
    ciphertext: String,
    size: u64,
    sha256: String,
    inventory_sha256: String,
    original_exit: i32,
    status: String,
    complete: bool,
}
fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| fail("custody exclusive file create failed"))?;
    file.write_all(bytes)
        .map_err(|_| fail("custody file write failed"))?;
    Ok(())
}
fn closed(v: &Value, keys: &[&str]) -> Result<()> {
    if !v
        .as_object()
        .is_some_and(|o| o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k)))
    {
        return Err(fail("custody closed JSON rejected"));
    }
    Ok(())
}
fn parse(bytes: &[u8]) -> Result<Value> {
    let checksums::Unique(v) =
        serde_json::from_slice(bytes).map_err(|_| fail("custody unique JSON rejected"))?;
    Ok(v)
}
fn text(v: &Value, key: &str) -> Result<String> {
    v[key]
        .as_str()
        .map(String::from)
        .ok_or_else(|| fail("custody JSON string rejected"))
}
fn number(v: &Value, key: &str) -> Result<u64> {
    v[key]
        .as_u64()
        .ok_or_else(|| fail("custody JSON integer rejected"))
}
fn binding_value(b: &Binding) -> Value {
    json!({"source":b.source,"workflow_sha":b.workflow_sha,"workflow":b.workflow,"git_ref":b.git_ref,"run":b.run,"attempt":b.attempt})
}
fn binding_parse(v: &Value) -> Result<Binding> {
    closed(
        v,
        &[
            "source",
            "workflow_sha",
            "workflow",
            "git_ref",
            "run",
            "attempt",
        ],
    )?;
    Ok(Binding {
        source: text(v, "source")?,
        workflow_sha: text(v, "workflow_sha")?,
        workflow: text(v, "workflow")?,
        git_ref: text(v, "git_ref")?,
        run: text(v, "run")?,
        attempt: text(v, "attempt")?,
    })
}
fn inventory_value(i: &Inventory) -> Value {
    json!({"schema":i.schema,"binding":binding_value(&i.binding),"recipient_sha256":i.recipient_sha256,"original_exit":i.original_exit,"status":i.status,"complete":i.complete,"private_manifest_sha256":i.private_manifest_sha256,"missing_extras":i.missing_extras,"files":i.files.iter().map(|e| json!({"member":e.member,"origin":e.origin,"name":e.name,"size":e.size,"sha256":e.sha256})).collect::<Vec<_>>()})
}
fn inventory_parse(v: &Value) -> Result<Inventory> {
    closed(
        v,
        &[
            "schema",
            "binding",
            "recipient_sha256",
            "original_exit",
            "status",
            "complete",
            "private_manifest_sha256",
            "missing_extras",
            "files",
        ],
    )?;
    let files = v["files"]
        .as_array()
        .ok_or_else(|| fail("custody files rejected"))?
        .iter()
        .map(|e| {
            closed(e, &["member", "origin", "name", "size", "sha256"])?;
            Ok(Entry {
                member: text(e, "member")?,
                origin: text(e, "origin")?,
                name: text(e, "name")?,
                size: number(e, "size")?,
                sha256: text(e, "sha256")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let missing_extras = v["missing_extras"]
        .as_array()
        .ok_or_else(|| fail("custody extras rejected"))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| fail("custody extra rejected"))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Inventory {
        schema: u32::try_from(number(v, "schema")?).map_err(|_| fail("custody schema rejected"))?,
        binding: binding_parse(&v["binding"])?,
        recipient_sha256: text(v, "recipient_sha256")?,
        original_exit: i32::try_from(number(v, "original_exit")?)
            .map_err(|_| fail("custody exit rejected"))?,
        status: text(v, "status")?,
        complete: v["complete"]
            .as_bool()
            .ok_or_else(|| fail("custody status rejected"))?,
        private_manifest_sha256: if v["private_manifest_sha256"].is_null() {
            None
        } else {
            Some(text(v, "private_manifest_sha256")?)
        },
        files,
        missing_extras,
    })
}
fn receipt_value(r: &Receipt) -> Value {
    json!({"schema":r.schema,"binding":binding_value(&r.binding),"recipient_sha256":r.recipient_sha256,"ciphertext":r.ciphertext,"size":r.size,"sha256":r.sha256,"inventory_sha256":r.inventory_sha256,"original_exit":r.original_exit,"status":r.status,"complete":r.complete})
}
fn receipt_parse(v: &Value) -> Result<Receipt> {
    closed(
        v,
        &[
            "schema",
            "binding",
            "recipient_sha256",
            "ciphertext",
            "size",
            "sha256",
            "inventory_sha256",
            "original_exit",
            "status",
            "complete",
        ],
    )?;
    Ok(Receipt {
        schema: u32::try_from(number(v, "schema")?).map_err(|_| fail("custody schema rejected"))?,
        binding: binding_parse(&v["binding"])?,
        recipient_sha256: text(v, "recipient_sha256")?,
        ciphertext: text(v, "ciphertext")?,
        size: number(v, "size")?,
        sha256: text(v, "sha256")?,
        inventory_sha256: text(v, "inventory_sha256")?,
        original_exit: i32::try_from(number(v, "original_exit")?)
            .map_err(|_| fail("custody exit rejected"))?,
        status: text(v, "status")?,
        complete: v["complete"]
            .as_bool()
            .ok_or_else(|| fail("custody status rejected"))?,
    })
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let dir = posix::directory(
        path.parent()
            .ok_or_else(|| fail("custody parent required"))?,
    )?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| fail("custody filename rejected"))?;
    posix::read(&dir, name, limit)
}
fn recipient(path: &Path) -> Result<()> {
    let bytes = read(path, 128)?;
    let s = std::str::from_utf8(&bytes).map_err(|_| fail("custody recipient rejected"))?;
    if tooling::sha(&bytes) != RECIPIENT_SHA
        || s.lines().count() != 1
        || !s.starts_with("age1")
        || s.trim().len() != 62
    {
        return Err(fail("custody recipient rejected"));
    }
    Ok(())
}
fn tool(program: &Path, args: Vec<OsString>, cwd: &Path, limit: usize) -> Result<Vec<u8>> {
    let report = process::run(
        &process::Spec {
            program: program.into(),
            args,
            cwd: cwd.into(),
            env: vec![
                ("LC_ALL".into(), Some("C".into())),
                ("TAR_OPTIONS".into(), None),
                ("AGEDEBUG".into(), None),
            ],
            timeout: Duration::from_secs(120),
            output_limit: limit,
        },
        &process::CANCELLED,
    );
    if !report.success() || report.stdout.discarded != 0 || report.stderr.discarded != 0 {
        return Err(fail(
            "custody subprocess failed; private diagnostics suppressed",
        ));
    }
    Ok(report.stdout.bytes)
}
fn outside(path: &Path) -> Result<()> {
    if path.starts_with(crate::repository_root()) {
        return Err(fail("custody plaintext must remain outside checkout"));
    }
    Ok(())
}
fn scratch() -> Result<tempfile::TempDir> {
    let base = std::env::temp_dir();
    posix::directory(&base)?;
    outside(&base)?;
    Ok(tempfile::Builder::new()
        .prefix("yetlet-custody-")
        .permissions(fs::Permissions::from_mode(0o700))
        .tempdir_in(base)?)
}
fn publish(stage: &Path, output: &Path) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let a = std::ffi::CString::new(stage.as_os_str().as_bytes())
        .map_err(|_| fail("custody path rejected"))?;
    let b = std::ffi::CString::new(output.as_os_str().as_bytes())
        .map_err(|_| fail("custody path rejected"))?;
    #[cfg(target_os = "linux")]
    if unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            a.as_ptr(),
            libc::AT_FDCWD,
            b.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } == 0
    {
        return Ok(());
    }
    Err(fail("custody atomic publication refused"))
}
fn stage(output: &Path) -> Result<tempfile::TempDir> {
    let parent = output
        .parent()
        .ok_or_else(|| fail("custody output parent required"))?;
    posix::directory(parent)?;
    if !output
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(checksums::safe_name)
    {
        return Err(fail("custody output name rejected"));
    }
    Ok(tempfile::Builder::new()
        .prefix(".custody-")
        .permissions(fs::Permissions::from_mode(0o700))
        .tempdir_in(parent)?)
}
fn names(dir: &fs::File) -> Result<Vec<String>> {
    use std::os::fd::AsRawFd;
    let mut names = BTreeSet::new();
    for e in fs::read_dir(format!("/proc/self/fd/{}", dir.as_raw_fd()))? {
        let name = e?
            .file_name()
            .into_string()
            .map_err(|_| fail("custody filename rejected"))?;
        if name != checksums::MANIFEST && (!checksums::safe_name(&name) || name.len() > 110) {
            return Err(fail("custody filename rejected"));
        }
        names.insert(name);
        if names.len() > checksums::MAX_FILES {
            return Err(fail("custody count limit"));
        }
    }
    Ok(names.into_iter().collect())
}
fn collect(
    dir: &fs::File,
    name: &str,
    origin: &str,
    copied: &Path,
    files: &mut Vec<Entry>,
    total: &mut u64,
) -> Result<()> {
    let bytes = posix::read(
        dir,
        name,
        if name == checksums::MANIFEST {
            checksums::MAX_MANIFEST_BYTES
        } else {
            checksums::MAX_FILE_BYTES
        },
    )?;
    *total += bytes.len() as u64;
    if *total > checksums::MAX_TOTAL_BYTES || files.len() >= checksums::MAX_FILES {
        return Err(fail("custody input limit"));
    }
    let member = format!("{origin}-{name}");
    write(&copied.join(&member), &bytes)?;
    files.push(Entry {
        member,
        origin: origin.into(),
        name: name.into(),
        size: bytes.len() as u64,
        sha256: tooling::sha(&bytes),
    });
    Ok(())
}
fn capture(options: &Options, private: &Path, extras: &Path, exit: i32) -> Result<()> {
    let scratch = scratch()?;
    let copied = scratch.path().join("members");
    fs::DirBuilder::new().mode(0o700).create(&copied)?;
    let mut files = vec![];
    let mut total = 0;
    if fs::symlink_metadata(private).is_ok() {
        let dir = posix::directory(private)?;
        for name in names(&dir)? {
            collect(&dir, &name, "private", &copied, &mut files, &mut total)?;
        }
    } else if exit == 0 {
        return Err(fail("successful private assembly unavailable"));
    }
    if exit == 0 {
        // Verify the immutable captured bytes, rather than reopening a changing input.
        let frozen = scratch.path().join("frozen");
        fs::DirBuilder::new().mode(0o700).create(&frozen)?;
        for entry in &files {
            write(
                &frozen.join(&entry.name),
                &read(&copied.join(&entry.member), checksums::MAX_FILE_BYTES)?,
            )?;
        }
        checksums::run("verify", &frozen)?;
    }
    let extra_dir = posix::directory(extras)?;
    let mut missing_extras = vec![];
    for name in EXTRAS {
        if fs::symlink_metadata(extras.join(name)).is_ok() {
            collect(&extra_dir, name, "extra", &copied, &mut files, &mut total)?;
        } else {
            missing_extras.push(name.into());
        }
    }
    if files.is_empty() {
        return Err(fail("custody NOT SAVED: no evidence available"));
    }
    let manifest = files
        .iter()
        .find(|e| e.origin == "private" && e.name == checksums::MANIFEST)
        .map(|e| e.sha256.clone());
    let complete = exit == 0 && missing_extras.is_empty();
    let inventory = Inventory {
        schema: 1,
        binding: options.binding.clone(),
        recipient_sha256: RECIPIENT_SHA.into(),
        original_exit: exit,
        status: if complete {
            "PREPARATION_SUCCESS"
        } else {
            "FAILED_INCOMPLETE"
        }
        .into(),
        complete,
        private_manifest_sha256: manifest,
        files,
        missing_extras,
    };
    validate_inventory(&inventory)?;
    verify_extras(&inventory, &copied)?;
    let inventory_bytes = tooling::json_bytes(&inventory_value(&inventory));
    if inventory_bytes.len() as u64 > checksums::MAX_MANIFEST_BYTES {
        return Err(fail("custody inventory limit"));
    }
    write(&copied.join(INVENTORY), &inventory_bytes)?;
    let archive = scratch.path().join("evidence.tar");
    write(&archive, &[])?;
    let mut args: Vec<OsString> = vec![
        "--create".into(),
        "--format=ustar".into(),
        "--file".into(),
        archive.as_os_str().into(),
        "--".into(),
        INVENTORY.into(),
    ];
    args.extend(inventory.files.iter().map(|e| OsString::from(&e.member)));
    tool(Path::new("tar"), args, &copied, 4096)?;
    read(&archive, ARCHIVE_LIMIT)?;
    let stage = stage(&options.output)?;
    write(&stage.path().join(CIPHER), &[])?;
    tool(
        &options.age,
        vec![
            "--encrypt".into(),
            "--recipient".into(),
            "age1ry66gggh2nc7hqwfr7jr4hz0hcgz2zvd4ysjqyzqqaat05ucq5rs2lrr6r".into(),
            "--output".into(),
            stage.path().join(CIPHER).into_os_string(),
            archive.into_os_string(),
        ],
        scratch.path(),
        4096,
    )?;
    let ciphertext = read(&stage.path().join(CIPHER), CIPHER_LIMIT)?;
    let receipt = Receipt {
        schema: 1,
        binding: options.binding.clone(),
        recipient_sha256: RECIPIENT_SHA.into(),
        ciphertext: CIPHER.into(),
        size: ciphertext.len() as u64,
        sha256: tooling::sha(&ciphertext),
        inventory_sha256: tooling::sha(&inventory_bytes),
        original_exit: exit,
        status: inventory.status,
        complete,
    };
    write(
        &stage.path().join(RECEIPT),
        &tooling::json_bytes(&receipt_value(&receipt)),
    )?;
    publish(stage.path(), &options.output)
}
use std::os::unix::fs::DirBuilderExt;
fn validate_inventory(i: &Inventory) -> Result<()> {
    i.binding.validate()?;
    if i.schema != 1
        || i.recipient_sha256 != RECIPIENT_SHA
        || !(0..=255).contains(&i.original_exit)
        || i.files.is_empty()
        || i.files.len() > checksums::MAX_FILES
        || i.complete != (i.original_exit == 0 && i.missing_extras.is_empty())
        || i.status
            != if i.complete {
                "PREPARATION_SUCCESS"
            } else {
                "FAILED_INCOMPLETE"
            }
    {
        return Err(fail("custody inventory identity rejected"));
    }
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for e in &i.files {
        if !["private", "extra"].contains(&e.origin.as_str())
            || (e.name != checksums::MANIFEST && !checksums::safe_name(&e.name))
            || e.name.len() > 110
            || e.member != format!("{}-{}", e.origin, e.name)
            || !checksums::safe_name(&e.member)
            || !seen.insert(&e.member)
            || e.size > checksums::MAX_FILE_BYTES
            || (e.name == checksums::MANIFEST && e.size > checksums::MAX_MANIFEST_BYTES)
            || e.sha256.len() != 64
            || !e
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || (e.origin == "extra" && !EXTRAS.contains(&e.name.as_str()))
        {
            return Err(fail("custody inventory entry rejected"));
        }
        total += e.size;
        if total > checksums::MAX_TOTAL_BYTES {
            return Err(fail("custody total limit"));
        }
    }
    let manifest = i
        .files
        .iter()
        .find(|e| e.origin == "private" && e.name == checksums::MANIFEST)
        .map(|e| &e.sha256);
    let missing: Vec<String> = EXTRAS
        .iter()
        .filter(|n| !i.files.iter().any(|e| e.origin == "extra" && e.name == **n))
        .map(|n| (*n).into())
        .collect();
    if manifest != i.private_manifest_sha256.as_ref()
        || missing != i.missing_extras
        || (i.complete && manifest.is_none())
    {
        return Err(fail("custody inventory set rejected"));
    }
    Ok(())
}
fn verify_extras(i: &Inventory, members: &Path) -> Result<()> {
    if i.complete {
        let record: serde_json::Value = parse(&read(
            &members.join("private-BUILD-RECORD.json"),
            checksums::MAX_MANIFEST_BYTES,
        )?)?;
        let source = i
            .files
            .iter()
            .find(|e| e.member == "extra-openssl-3.5.9.tar.gz")
            .ok_or_else(|| fail("custody source missing"))?;
        if source.sha256 != OPENSSL_SHA || record["openssl"]["source_sha256"] != OPENSSL_SHA {
            return Err(fail("custody OpenSSL source mismatch"));
        }
        let result: serde_json::Value = parse(&read(
            &members.join("extra-openssl-build-result.json"),
            checksums::MAX_MANIFEST_BYTES,
        )?)?;
        if result["native_exit_code"].as_i64() != Some(0) {
            return Err(fail("custody OpenSSL result mismatch"));
        }
    }
    Ok(())
}
fn listing(archive: &Path, cwd: &Path) -> Result<BTreeMap<String, u64>> {
    let bytes = tool(
        Path::new("tar"),
        vec![
            "--list".into(),
            "--verbose".into(),
            "--numeric-owner".into(),
            "--full-time".into(),
            "--quoting-style=literal".into(),
            "--ignore-zeros".into(),
            "--file".into(),
            archive.as_os_str().into(),
        ],
        cwd,
        128 * 1024,
    )?;
    let text = std::str::from_utf8(&bytes).map_err(|_| fail("custody archive listing rejected"))?;
    let mut entries = BTreeMap::new();
    let mut total = 0;
    for line in text.lines() {
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        if fields.len() != 6
            || !fields[0].starts_with('-')
            || fields[0].len() != 10
            || !checksums::safe_name(fields[5])
        {
            return Err(fail("custody archive nonregular or unsafe entry"));
        }
        let size: u64 = fields[2]
            .parse()
            .map_err(|_| fail("custody archive size rejected"))?;
        if size > checksums::MAX_FILE_BYTES
            || entries.insert(fields[5].into(), size).is_some()
            || entries.len() > checksums::MAX_FILES + 1
        {
            return Err(fail("custody archive count/size/duplicate rejected"));
        }
        total += size;
        if total > checksums::MAX_TOTAL_BYTES + checksums::MAX_MANIFEST_BYTES {
            return Err(fail("custody archive total rejected"));
        }
    }
    Ok(entries)
}
fn member(archive: &Path, name: &str, size: u64, cwd: &Path) -> Result<Vec<u8>> {
    let bytes = tool(
        Path::new("tar"),
        vec![
            "--extract".into(),
            "--to-stdout".into(),
            "--file".into(),
            archive.as_os_str().into(),
            "--".into(),
            name.into(),
        ],
        cwd,
        size as usize + 1,
    )?;
    if bytes.len() as u64 != size {
        return Err(fail("custody archive member size mismatch"));
    }
    Ok(bytes)
}
fn verify(options: &Options, transport: &Path, identity: &Path) -> Result<()> {
    let parent = options
        .output
        .parent()
        .ok_or_else(|| fail("custody output parent required"))?;
    posix::directory(parent)?;
    outside(&parent.canonicalize()?)?;
    let receipt_bytes = read(&transport.join(RECEIPT), checksums::MAX_MANIFEST_BYTES)?;
    let receipt = receipt_parse(&parse(&receipt_bytes)?)?;
    if receipt.schema != 1
        || receipt.binding != options.binding
        || receipt.recipient_sha256 != RECIPIENT_SHA
        || receipt.ciphertext != CIPHER
    {
        return Err(fail("custody receipt binding rejected"));
    }
    let ciphertext = read(&transport.join(CIPHER), CIPHER_LIMIT)?;
    if receipt.size != ciphertext.len() as u64 || receipt.sha256 != tooling::sha(&ciphertext) {
        return Err(fail("custody ciphertext mismatch"));
    }
    let scratch = scratch()?;
    let cipher = scratch.path().join(CIPHER);
    write(&cipher, &ciphertext)?;
    let archive = scratch.path().join("evidence.tar");
    write(&archive, &[])?;
    tool(
        &options.age,
        vec![
            "--decrypt".into(),
            "--identity".into(),
            identity.as_os_str().into(),
            "--output".into(),
            archive.as_os_str().into(),
            cipher.into_os_string(),
        ],
        scratch.path(),
        4096,
    )?;
    read(&archive, ARCHIVE_LIMIT)?;
    let listed = listing(&archive, scratch.path())?;
    let size = *listed
        .get(INVENTORY)
        .filter(|s| **s <= checksums::MAX_MANIFEST_BYTES)
        .ok_or_else(|| fail("custody inventory unavailable"))?;
    let raw = member(&archive, INVENTORY, size, scratch.path())?;
    if tooling::sha(&raw) != receipt.inventory_sha256 {
        return Err(fail("custody inventory digest mismatch"));
    }
    let inventory = inventory_parse(&parse(&raw)?)?;
    validate_inventory(&inventory)?;
    if inventory.binding != options.binding
        || inventory.original_exit != receipt.original_exit
        || inventory.status != receipt.status
        || inventory.complete != receipt.complete
        || listed.len() != inventory.files.len() + 1
    {
        return Err(fail("custody inventory/receipt mismatch"));
    }
    let stage = stage(&options.output)?;
    let private = stage.path().join("private");
    let extras = stage.path().join("extras");
    let members = scratch.path().join("members");
    for dir in [&private, &extras, &members] {
        fs::DirBuilder::new().mode(0o700).create(dir)?;
    }
    for e in &inventory.files {
        if listed.get(&e.member) != Some(&e.size) {
            return Err(fail("custody archive/inventory mismatch"));
        }
        let bytes = member(&archive, &e.member, e.size, scratch.path())?;
        if tooling::sha(&bytes) != e.sha256 {
            return Err(fail("custody evidence digest mismatch"));
        }
        write(&members.join(&e.member), &bytes)?;
        write(
            &if e.origin == "private" {
                private.join(&e.name)
            } else {
                extras.join(&e.name)
            },
            &bytes,
        )?;
    }
    verify_extras(&inventory, &members)?;
    if inventory.original_exit == 0 {
        checksums::run("verify", &private)?;
    }
    write(&stage.path().join(INVENTORY), &raw)?;
    let result = serde_json::json!({"schema":1,"binding":binding_value(&options.binding),"recipient_sha256":RECIPIENT_SHA,"ciphertext_sha256":receipt.sha256,"inventory_sha256":receipt.inventory_sha256,"original_exit":inventory.original_exit,"status":inventory.status,"complete":inventory.complete,"local_decryption_and_inventory_verified":true,"private_manifest_verified":inventory.original_exit == 0,"notice":"Local readback only; not builder authentication, durable custody or release approval"});
    write(
        &stage.path().join("local-readback-receipt.json"),
        &tooling::json_bytes(&result),
    )?;
    publish(stage.path(), &options.output)
}
pub fn run(options: &Options) -> Result<()> {
    let result = (|| {
        options.binding.validate()?;
        recipient(&options.recipient)?;
        match &options.operation {
            Operation::Capture {
                private,
                extras,
                exit,
            } => capture(options, private, extras, *exit),
            Operation::Verify {
                transport,
                identity,
            } => verify(options, transport, identity),
        }
    })();
    result.map_err(|_| {
        fail("custody NOT SAVED or readback rejected; private diagnostics suppressed")
    })?;
    println!("custody operation PASS; transport/readback is not release approval");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_listing_rejects_duplicate_unsafe_links_and_oversize_members() {
        use std::os::unix::fs::symlink;
        let scratch = scratch().unwrap();
        assert_eq!(
            fs::metadata(scratch.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
        let t = tempfile::tempdir().unwrap();
        let stage = stage(&t.path().join("output")).unwrap();
        assert_eq!(
            fs::metadata(stage.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
        fs::write(t.path().join("good"), b"fixture").unwrap();
        let archive = t.path().join("evidence.tar");
        let make = |members: &[&str]| {
            let mut args = vec!["-cf".into(), archive.as_os_str().into(), "--".into()];
            args.extend(members.iter().map(OsString::from));
            tool(Path::new("tar"), args, t.path(), 4096).unwrap();
        };
        make(&["good"]);
        assert!(listing(&archive, t.path()).is_ok());
        make(&["good", "good"]);
        assert!(listing(&archive, t.path()).is_err());
        symlink("good", t.path().join("link")).unwrap();
        make(&["link"]);
        assert!(listing(&archive, t.path()).is_err());
        fs::create_dir(t.path().join("directory")).unwrap();
        make(&["directory"]);
        assert!(listing(&archive, t.path()).is_err());
        fs::write(t.path().join("unsafe name"), b"x").unwrap();
        make(&["unsafe name"]);
        assert!(listing(&archive, t.path()).is_err());
        fs::File::create(t.path().join("large"))
            .unwrap()
            .set_len(checksums::MAX_FILE_BYTES + 1)
            .unwrap();
        make(&["large"]);
        assert!(listing(&archive, t.path()).is_err());
    }
    #[test]
    fn inventory_rejects_duplicate_json_and_unsafe_or_mismatched_sets() {
        assert!(parse(br#"{"a":1,"a":2}"#).is_err());
        assert!(parse(br#"{"a":{"b":1,"b":2}}"#).is_err());
        let v = json!({"schema":1,"binding":{"source":"a".repeat(40),"workflow_sha":"a".repeat(40),"workflow":workflow(),"git_ref":git_ref(),"run":"1","attempt":"1"},"recipient_sha256":RECIPIENT_SHA,"original_exit":17,"status":"FAILED_INCOMPLETE","complete":false,"private_manifest_sha256":null,"missing_extras":EXTRAS,"files":[{"member":"private-fixture","origin":"private","name":"fixture","size":1,"sha256":"a".repeat(64)}]});
        let mut i = inventory_parse(&v).unwrap();
        validate_inventory(&i).unwrap();
        i.files[0].member = "../escape".into();
        assert!(validate_inventory(&i).is_err());
        i = inventory_parse(&v).unwrap();
        i.files[0].size = checksums::MAX_FILE_BYTES + 1;
        assert!(validate_inventory(&i).is_err());
        i = inventory_parse(&v).unwrap();
        i.missing_extras.clear();
        assert!(validate_inventory(&i).is_err());
    }
}
