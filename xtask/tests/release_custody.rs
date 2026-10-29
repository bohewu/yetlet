#![cfg(target_os = "linux")]
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use yetlet_xtask::{
    release::{
        checksums,
        custody::{self, Binding, Operation, Options},
    },
    repository_root, tooling,
};

fn options(temp: &Path, exit: i32) -> Options {
    Options {
        operation: Operation::Capture {
            private: temp.join("private"),
            extras: temp.join("extras"),
            exit,
        },
        binding: Binding {
            source: "a".repeat(40),
            workflow_sha: "a".repeat(40),
            workflow: "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.9.0-alpha.1".into(),
            git_ref: "refs/tags/v0.9.0-alpha.1".into(),
            run: "1".into(),
            attempt: "1".into(),
        },
        age: PathBuf::from("/unavailable-age"),
        recipient: repository_root().join(".github/release-evidence-recipient.txt"),
        output: temp.join("transport"),
    }
}
fn fixture(temp: &Path) {
    fs::create_dir(temp.join("private")).unwrap();
    fs::create_dir(temp.join("extras")).unwrap();
    fs::write(
        temp.join("private/local-fixture.txt"),
        b"LOCAL synthetic evidence; no product qualification",
    )
    .unwrap();
}
fn configured() -> Option<(PathBuf, PathBuf)> {
    match (
        std::env::var_os("YT096_CUSTODY_AGE"),
        std::env::var_os("YT096_CUSTODY_IDENTITY"),
    ) {
        (Some(age), Some(identity)) => Some((age.into(), identity.into())),
        _ => None,
    }
}
#[test]
fn closed_cli_rejects_unknown_duplicate_missing_and_invalid_bindings() {
    let t = tempfile::tempdir().unwrap();
    let mut args: Vec<OsString> = [
        "capture",
        "--age",
        "/age",
        "--recipient",
        "/recipient",
        "--output",
        "/output",
        "--source",
        &"a".repeat(40),
        "--run",
        "1",
        "--attempt",
        "1",
        "--private",
        "/private",
        "--extras",
        "/extras",
        "--exit",
        "17",
    ]
    .map(Into::into)
    .into();
    assert_eq!(
        Options::parse(&args).unwrap().binding,
        options(t.path(), 17).binding
    );
    let verify: Vec<OsString> = [
        "verify",
        "--age",
        "/age",
        "--recipient",
        "/recipient",
        "--output",
        "/output",
        "--source",
        &"a".repeat(40),
        "--run",
        "1",
        "--attempt",
        "1",
        "--transport",
        "/transport",
        "--identity",
        "/identity",
    ]
    .map(Into::into)
    .into();
    assert_eq!(
        Options::parse(&verify).unwrap().binding,
        options(t.path(), 17).binding
    );
    args.extend(["--exit".into(), "0".into()]);
    assert!(Options::parse(&args).is_err());
    args.truncate(args.len() - 2);
    args[1] = "--unknown".into();
    assert!(Options::parse(&args).is_err());
    args[1] = "--age".into();
    args[8] = "main".into();
    assert!(Options::parse(&args).is_err());
    let mut o = options(t.path(), 17);
    fixture(t.path());
    o.binding.workflow_sha = "b".repeat(40);
    assert!(custody::run(&o).is_err());
    assert!(!o.output.exists());
}
#[test]
fn alpha_capture_and_verify_reject_legacy_identity_and_sha_mismatch_before_tools() {
    let t = tempfile::tempdir().unwrap();
    for verify in [false, true] {
        for (workflow, git_ref, workflow_sha) in [
            (
                "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.1.0",
                "refs/tags/v0.9.0-alpha.1",
                "a".repeat(40),
            ),
            (
                "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.9.0-alpha.1",
                "refs/tags/v0.1.0",
                "a".repeat(40),
            ),
            (
                "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.1.0",
                "refs/tags/v0.1.0",
                "a".repeat(40),
            ),
            (
                "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.9.0-alpha.1",
                "refs/tags/v0.9.0-alpha.1",
                "b".repeat(40),
            ),
        ] {
            let mut o = options(t.path(), 17);
            o.binding.workflow = workflow.into();
            o.binding.git_ref = git_ref.into();
            o.binding.workflow_sha = workflow_sha;
            if verify {
                o.operation = Operation::Verify {
                    transport: t.path().join("transport"),
                    identity: t.path().join("unavailable-identity"),
                };
            }
            assert!(custody::run(&o).is_err());
            assert!(!o.output.exists());
        }
    }
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
}
#[test]
fn unsafe_nonregular_and_changed_recipient_inputs_fail_without_publication() {
    use std::os::unix::fs::symlink;
    let t = tempfile::tempdir().unwrap();
    fixture(t.path());
    let mut o = options(t.path(), 17);
    fs::write(t.path().join("recipient"), b"age1other\n").unwrap();
    o.recipient = t.path().join("recipient");
    assert!(custody::run(&o).is_err());
    o.recipient = repository_root().join(".github/release-evidence-recipient.txt");
    symlink("local-fixture.txt", t.path().join("private/symlink")).unwrap();
    assert!(custody::run(&o).is_err());
    fs::remove_file(t.path().join("private/symlink")).unwrap();
    fs::create_dir(t.path().join("private/directory")).unwrap();
    assert!(custody::run(&o).is_err());
    fs::remove_dir(t.path().join("private/directory")).unwrap();
    fs::write(t.path().join("private/unsafe name"), b"bad").unwrap();
    assert!(custody::run(&o).is_err());
    assert!(!o.output.exists());
    assert!(!fs::read_dir(t.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".custody-")
    }));
}
#[test]
fn empty_and_oversized_failed_capture_are_not_saved() {
    let t = tempfile::tempdir().unwrap();
    fixture(t.path());
    fs::remove_file(t.path().join("private/local-fixture.txt")).unwrap();
    let o = options(t.path(), 17);
    assert!(custody::run(&o).is_err());
    let file = fs::File::create(t.path().join("private/oversized")).unwrap();
    file.set_len(checksums::MAX_FILE_BYTES + 1).unwrap();
    assert!(custody::run(&o).is_err());
    assert!(!o.output.exists());
}
#[test]
fn actual_age_local_roundtrip_manifest_partial_tamper_binding_and_noclobber() {
    let Some((age, identity)) = configured() else {
        eprintln!(
            "authenticated local age/identity not configured; direct crypto case not executed"
        );
        return;
    };
    let t = tempfile::tempdir().unwrap();
    fixture(t.path());
    checksums::run("create", &t.path().join("private")).unwrap();
    let original_manifest = fs::read(t.path().join("private/RELEASE-CHECKSUMS.json")).unwrap();
    let mut o = options(t.path(), 0);
    o.age = age;
    custody::run(&o).unwrap();
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        fs::metadata(&o.output).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for entry in fs::read_dir(&o.output).unwrap() {
        assert_eq!(
            entry.unwrap().metadata().unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let receipt: Value =
        serde_json::from_slice(&fs::read(o.output.join(custody::RECEIPT)).unwrap()).unwrap();
    assert_eq!(receipt["complete"], false); // Missing OpenSSL inputs never qualify a complete preparation.
    assert_eq!(receipt["status"], "FAILED_INCOMPLETE");
    assert!(custody::run(&o).is_err()); // Atomic publication never replaces a prior transport.
    let transport = o.output.clone();
    o.operation = Operation::Verify {
        transport: transport.clone(),
        identity,
    };
    o.output = t.path().join("readback");
    custody::run(&o).unwrap();
    for directory in [
        &o.output,
        &o.output.join("private"),
        &o.output.join("extras"),
    ] {
        assert_eq!(
            fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        for entry in fs::read_dir(directory).unwrap() {
            let metadata = entry.unwrap().metadata().unwrap();
            assert_eq!(
                metadata.permissions().mode() & 0o777,
                if metadata.is_dir() { 0o700 } else { 0o600 }
            );
        }
    }
    assert_eq!(
        fs::read(o.output.join("private/RELEASE-CHECKSUMS.json")).unwrap(),
        original_manifest
    );
    checksums::run("verify", &o.output.join("private")).unwrap();
    let readback: Value =
        serde_json::from_slice(&fs::read(o.output.join("local-readback-receipt.json")).unwrap())
            .unwrap();
    assert_eq!(readback["private_manifest_verified"], true);
    assert_eq!(readback["complete"], false);
    o.output = t.path().join("rejected");
    for key in ["source", "run", "attempt", "recipient_sha256"] {
        let mut bad = receipt.clone();
        if key == "recipient_sha256" {
            bad[key] = json!("b".repeat(64));
        } else {
            bad["binding"][key] = json!("2");
        }
        fs::write(
            transport.join(custody::RECEIPT),
            serde_json::to_vec(&bad).unwrap(),
        )
        .unwrap();
        assert!(custody::run(&o).is_err());
        assert!(!o.output.exists());
    }
    fs::write(
        transport.join(custody::RECEIPT),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    let ciphertext = fs::read(transport.join(custody::CIPHER)).unwrap();
    let mut tampered = ciphertext.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    fs::write(transport.join(custody::CIPHER), &tampered).unwrap();
    assert!(custody::run(&o).is_err());
    let mut bad = receipt.clone();
    bad["sha256"] = json!(tooling::sha(&tampered));
    fs::write(
        transport.join(custody::RECEIPT),
        serde_json::to_vec(&bad).unwrap(),
    )
    .unwrap();
    assert!(custody::run(&o).is_err()); // age authentication also rejects it.
    fs::write(transport.join(custody::CIPHER), &ciphertext).unwrap();
    fs::write(
        transport.join(custody::RECEIPT),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    // A non-identity file is rejected by age, with its diagnostics suppressed.
    fs::write(t.path().join("wrong-identity"), b"not an identity").unwrap();
    let identity = match &o.operation {
        Operation::Verify { identity, .. } => identity.clone(),
        _ => unreachable!(),
    };
    o.operation = Operation::Verify {
        transport: transport.clone(),
        identity: t.path().join("wrong-identity"),
    };
    assert!(custody::run(&o).is_err());
    assert!(!o.output.exists());
    // A native failure with available evidence roundtrips while remaining incomplete.
    let partial = t.path().join("partial");
    fs::create_dir(&partial).unwrap();
    fixture(&partial);
    let mut p = options(&partial, 17);
    p.age = o.age.clone();
    custody::run(&p).unwrap();
    let transport = p.output.clone();
    p.operation = Operation::Verify {
        transport,
        identity,
    };
    p.output = partial.join("readback");
    custody::run(&p).unwrap();
    let v: Value =
        serde_json::from_slice(&fs::read(p.output.join("local-readback-receipt.json")).unwrap())
            .unwrap();
    assert_eq!(v["original_exit"], 17);
    assert_eq!(v["complete"], false);
    assert_eq!(v["private_manifest_verified"], false);
    assert!(!p.output.join("private/RELEASE-CHECKSUMS.json").exists());
    assert!(!fs::read_dir(t.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".custody-")
    }));
}
#[test]
fn success_capture_rejects_tampered_original_manifest_before_encryption() {
    let t = tempfile::tempdir().unwrap();
    fixture(t.path());
    checksums::run("create", &t.path().join("private")).unwrap();
    fs::write(t.path().join("private/local-fixture.txt"), b"changed").unwrap();
    let o = options(t.path(), 0);
    assert!(custody::run(&o).is_err());
    assert!(!o.output.exists());
}
#[test]
fn workflow_keeps_build_key_free_and_explicit_ciphertext_allowlist() {
    let s = fs::read_to_string(repository_root().join(".github/workflows/release.yml")).unwrap();
    assert!(!s.contains("secrets."));
    assert!(!s.contains("environment:"));
    assert!(!s.contains("always()"));
    assert!(s.contains("steps.custody_capture.outcome == 'success'"));
    assert!(s.contains("steps.custody_transport.outcome == 'success'"));
    assert!(s.contains("target/release-custody/private-evidence.tar.age\n            target/release-custody/transport-receipt.json"));
    assert!(s.contains("persist-credentials: false"));
    let bootstrap =
        fs::read_to_string(repository_root().join("scripts/bootstrap-release-age.sh")).unwrap();
    assert!(!bootstrap.contains("@latest"));
    assert!(!bootstrap.contains("GOSUMDB=off"));
    assert!(
        bootstrap.find("sigsum-verify\" -k").unwrap()
            < bootstrap.find("\"$BOOTSTRAP/age/age\" --version").unwrap()
    );
}
#[test]
fn gnu_tar_is_available_for_the_closed_archive_boundary() {
    let result = Command::new("tar").arg("--version").output().unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("GNU tar"));
}

#[test]
fn isolated_manual_verifier_checks_documented_metadata_and_zip_allowlist() {
    let python = r#"
import copy, hashlib, importlib.util, io, json, os, stat, tempfile, zipfile
spec=importlib.util.spec_from_file_location('custody', 'scripts/release-evidence-preflight.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
os.environ['RELEASE_RECORD']=json.dumps({'schema':1,'version':'0.9.0-alpha.2','source':'a'*40,'cohort':'sixteen','build':{'run':'1','attempt':'2','artifact':'3','sha256':'b'*64}})
e=m.inputs();assert e=={'source':'a'*40,'tag':'v0.9.0-alpha.2','run':'1','attempt':'2','artifact':'3','digest':'b'*64}
r={'id':1,'run_attempt':2,'head_sha':'a'*40,'head_branch':e['tag'],'event':'workflow_dispatch','path':'.github/workflows/release.yml','repository':{'id':4,'full_name':'bohewu/yetlet'},'head_repository':{'id':4,'full_name':'bohewu/yetlet'},'status':'completed','conclusion':'failure'}
a={'id':3,'name':'encrypted-private-'+'a'*40+'-1-2','expired':False,'size_in_bytes':100,'digest':'sha256:'+'b'*64,'workflow_run':{'id':1,'head_sha':'a'*40,'repository_id':4,'head_repository_id':4}}
v={'name':'release-evidence-verify','id':5,'can_admins_bypass':False,'protection_rules':[{'type':'required_reviewers','reviewers':[{'type':'User','reviewer':{'login':'bohewu','id':35250002}}]}],'deployment_branch_policy':{'protected_branches':False,'custom_branch_policies':True}}
p={'total_count':1,'branch_policies':[{'name':e['tag'],'type':'tag'}]}
m.check_metadata(r,a,v,p,e)
for path in ['.github/workflows/release.yml@refs/tags/'+e['tag'],'.github/workflows/release.yml@'+e['tag']]:
 good=copy.deepcopy(r);good['path']=path;m.check_metadata(good,a,v,p,e)
def rejects(run=r,artifact=a,environment=v,policies=p):
 try:m.check_metadata(run,artifact,environment,policies,e)
 except (ValueError,KeyError,StopIteration):return
 raise AssertionError('untrusted metadata accepted')
for key,value in [('run_attempt',1),('head_sha','c'*40),('event','pull_request'),('path','.github/workflows/other.yml'),('path','.github/workflows/release.yml@refs/tags/v0.1.0'),('path','.github/workflows/release.yml@v0.1.0'),('head_branch','master'),('head_branch','v0.1.0'),('head_branch','v0.9.0-alpha.1'),('conclusion','cancelled')]:
 bad=copy.deepcopy(r);bad[key]=value;rejects(run=bad)
bad=copy.deepcopy(r);bad['head_branch']='v0.1.0';bad['path']='.github/workflows/release.yml@refs/tags/v0.1.0';rejects(run=bad)
for key,value in [('name','other'),('expired',True),('digest','sha256:'+'c'*64),('id',9)]:
 bad=copy.deepcopy(a);bad[key]=value;rejects(artifact=bad)
bad=copy.deepcopy(a);bad['workflow_run']['head_sha']='c'*40;rejects(artifact=bad)
bad=copy.deepcopy(v);bad['protection_rules']=[];rejects(environment=bad)
bad=copy.deepcopy(v);bad['can_admins_bypass']=True;rejects(environment=bad)
bad=copy.deepcopy(v);bad['protection_rules'][0]['reviewers'][0]['reviewer']['id']=7;rejects(environment=bad)
bad=copy.deepcopy(v);bad['protection_rules'][0]['reviewers'][0]['reviewer']['login']='other';rejects(environment=bad)
bad=copy.deepcopy(v);bad['deployment_branch_policy']=None;rejects(environment=bad)
bad=copy.deepcopy(p);bad['branch_policies'][0]['type']='branch';rejects(policies=bad)
bad=copy.deepcopy(p);bad['branch_policies'][0]['name']='*';rejects(policies=bad)
bad=copy.deepcopy(p);bad['branch_policies'][0]['name']='v0.1.0';rejects(policies=bad)
bad=copy.deepcopy(p);bad['total_count']=2;bad['branch_policies'].append({'name':'v0.1.0','type':'tag'});rejects(policies=bad)
def zipped(names,link=False):
 b=io.BytesIO()
 with zipfile.ZipFile(b,'w') as z:
  for name in names:
   n=zipfile.ZipInfo(name)
   n.external_attr=((stat.S_IFLNK if link else stat.S_IFREG)|0o600)<<16
   z.writestr(n,b'fixture')
 return b.getvalue()
with tempfile.TemporaryDirectory() as d:
 body=zipped([m.CIPHER,m.RECEIPT]);expected=dict(e,digest=hashlib.sha256(body).hexdigest())
 m.unpack(body,expected,os.path.join(d,'transport'))
 assert set(os.listdir(os.path.join(d,'transport')))=={m.CIPHER,m.RECEIPT}
 for names,link in [([m.CIPHER,'../'+m.RECEIPT],False),([m.CIPHER,m.CIPHER],False),([m.CIPHER,m.RECEIPT],True),([m.CIPHER,m.RECEIPT,'extra'],False)]:
  body=zipped(names,link);expected=dict(e,digest=hashlib.sha256(body).hexdigest())
  try:m.unpack(body,expected,os.path.join(d,'bad'))
  except ValueError:pass
  else:raise AssertionError('unsafe archive accepted')
  assert not os.path.exists(os.path.join(d,'bad'))
 try:m.unpack(body,e,os.path.join(d,'bad'))
 except ValueError:pass
 else:raise AssertionError('wrong digest accepted')
print('documented metadata and two-file ZIP boundary: PASS')
"#;
    let result = Command::new("python3")
        .args(["-c", python])
        .current_dir(repository_root())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let source =
        fs::read_to_string(repository_root().join(".github/workflows/evidence-verify.yml"))
            .unwrap();
    let (preflight, verify) = source.split_once("  verify:\n").unwrap();
    for required in [
        "on:\n  workflow_dispatch:",
        "contents: read",
        "actions: read",
        "timeout-minutes: 5",
        "timeout-minutes: 15",
        "ref: ${{ github.sha }}",
        "record:",
        "RELEASE_RECORD: ${{ inputs.record }}",
        "persist-credentials: false",
    ] {
        assert!(source.contains(required), "missing {required}");
    }
    for required in [
        "[[ \"$GITHUB_EVENT_NAME\" == workflow_dispatch && \"$GITHUB_REPOSITORY\" == bohewu/yetlet ]]",
        "[[ \"$GITHUB_SHA\" == \"$RECORD_WORKFLOW_SHA\" ]]",
        "assert r['source'] == os.environ['GITHUB_SHA']",
        "assert os.environ['RECORD_WORKFLOW_REF'] == 'bohewu/yetlet/.github/workflows/evidence-verify.yml@' + os.environ['GITHUB_REF']",
        "python3 scripts/release_record.py source evidence-verify.yml",
    ] {
        assert_eq!(
            source.matches(required).count(),
            2,
            "missing repeated {required}"
        );
    }
    assert!(!source.contains("v0.1.0"));
    assert!(!source.contains("v0.9.0-alpha.1"));
    assert!(!preflight.contains("secrets."));
    assert!(!preflight.contains("environment:"));
    assert!(verify.contains("environment: release-evidence-verify"));
    assert_eq!(source.matches("secrets.AGE_IDENTITY").count(), 1);
    assert!(!source.contains("id-token:"));
    assert!(!source.contains("always()"));
    assert!(!source.contains("workflow_run:"));
    assert!(!source.contains("pull_request:"));
    assert!(
        verify
            .find("python3 scripts/release-evidence-preflight.py download")
            .unwrap()
            < verify.find("secrets.AGE_IDENTITY").unwrap()
    );
    assert!(verify.contains("path: ${{ runner.temp }}/yt096-safe-readback-receipt.json"));
    assert!(verify.contains("trap 'rm -rf \"$custody_work\"' EXIT HUP INT TERM"));
}
