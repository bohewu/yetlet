# YT-089d — Independently Accepted Configuration-only Kubernetes Generator

**Date:** 2026-10-16. **Current state: YT-089d Done at bounded Linux/amd64 Kubernetes config-generator scope.** Linux implementation, native Windows GNU/MSVC output parity, and isolated Multipass K3s Pod/Service/HTTP/cleanup passed. Distinct read-only Reviewer `wc_job_1SJSI37ZToF_od0U` accepted the implementation and native receipts (zero issues/followups). Original coding Job itself did not deploy a Pod; parent-operated K3s smoke remains separate. YT-089e final integration and M15 closeout remain open; YT-020 Blocked.

## Authority and preservation

Work stays in the existing `$HOME/repos/yetlet`, branch `master`, HEAD
`d24a05033c57e5b98dd57bf83189bc1297f6f28c`, with accepted 089a/b/c uncommitted.
The canonical Generalist contract, REASONING_POLICY, MATERIALITY_GATE,
AGENTS.md, ADR 0006, [M15 plan](YT-089-CLOUD-NATIVE-GENERATORS-PLAN.md) and
[089a](YT-089A-VERIFICATION.md)/[089b](YT-089B-VERIFICATION.md)/[089c](YT-089C-VERIFICATION.md)
verification were read. No native managed dispatch, role identity, verified
model/effort or independent review is claimed.

The accepted Reviewer PASS results `wc_job_0dvTDNAV22Vbpc5j` (089a),
`wc_job_IC92mEXoQMsG4nTg` (089b) and `wc_job_OCVGSYl5uAGET8w3` (089c) remain
unchanged at their recorded bounds. All earlier failures and raw evidence remain.
Before editing, `target/m15/yt089d/` captured a 692-file source/artifact SHA-256
baseline, 213 prior evidence-file hashes and original HEAD/status/index receipts.
No worktree, stash, reset, clean, stage, commit, push, release, host modification,
Docker/K3s/Azure/registry action or external environment mutation is authorized.

## Delivered contract

```text
yetlet generate k8s --manifest-path APP/Cargo.toml --output NEW_DIRECTORY --image EXPLICIT_IMAGE [--replicas COUNT]
```

The manifest, output and image are mandatory unique literal options. Replicas
are unsigned ASCII decimal 1-100, default 1; leading decimal zeros are accepted
without changing the numeric result. Unknown, duplicate, misplaced, missing or
blank arguments and invalid image/count syntax produce usage exit 2. Input,
profile, name and output-safety failures produce exit 1. Paths may contain
spaces, Unicode and literal shell punctuation within the existing OS contracts.

Only `deployment.yaml`, `service.yaml` and static `README.md` are published.
Both YAML documents use JSON syntax (a YAML 1.2 subset) through the existing
serde_json serializer: explicit typed strings, booleans and integers, sorted
deterministic object keys, UTF-8/LF/final newline, with no new dependency.
The Kubernetes module and generator are private. The public CLI enum/command
execution boundary is the only new caller surface; no public Kubernetes module,
SDK, reusable generator function or framework API was added.

`image::validate` is reused unchanged, including its exact registry/repository,
non-latest explicit tag or lowercase SHA-256 digest contract and injection
refusals. Image bytes are preserved as a serialized scalar, never put into a
shell command. Package names have an identity mapping: 1-63 lowercase ASCII
letters/digits/hyphens, letter prefix, letter/digit suffix. Underscores,
truncation and translation are rejected to avoid generated-name collisions.
Resource conflicts across separately selected apps/namespaces remain operator
responsibility.

`docker::generate_checked` is reused without changing its inspection or safety
contract: standalone generated native-ssr-v1 manifest, exact accepted cloud
source, explicit lock, supported framework closure and full snapshot reinspection
before publication. `compose::validate_lock` only becomes `pub(crate)` and is
reused unchanged, refusing stale/unsupported fixed-profile lock graphs; it is
not a Cargo solver or an image attestation. The existing operator image build
must still compile with its accepted locked build. No source files are exported.

Linux publication uses the existing pinned directory-FD/RENAME_NOREPLACE writer;
Windows retains protected local fixed NTFS, ACL/reparse-safe pinned handles,
owned staging and no-replace publication. The existing Windows deployment-parent
ACL gate now also recognizes the two fixed Kubernetes filenames. No replacement,
arbitrary nested output paths or blind recursive deletion is introduced.

## Manifest semantics and limits

The `apps/v1` Deployment and `v1` ClusterIP Service share the unchanged app name
and `app.kubernetes.io/name` label. Deployment selector, Pod labels and Service
selector match. Replicas are explicit, default 1, with no HA claim. Scheduling
selects Linux/amd64 nodes. One container named `app` uses the exact image with
`IfNotPresent`; Kubernetes may pull a missing image after an operator deploys.
The CLI never builds, pulls, publishes, imports or invokes tools.

Container port 8080 is named `http`, with `YETLET_HOST=0.0.0.0` and string
`YETLET_PORT=8080`. Service TCP port 80 targets `http`. Startup and liveness use
`/health/live`; readiness uses `/health/ready`. Startup period/failure threshold
are 2 seconds/30; liveness 10 seconds/3; readiness 5 seconds/3, all with 1-second
timeouts. They reflect local process state only, without DB/Redis health claims
or exec helpers. Termination grace is 30 seconds, an operator example budget,
not an application-enforced drain deadline or synchronous routing guarantee.

Example CPU request/limit are 100m/500m and memory 64Mi/128Mi, without capacity
claims. Pod security sets runAsNonRoot, numeric UID/GID 10001, RuntimeDefault
seccomp and automountServiceAccountToken=false. Container security sets
readOnlyRootFilesystem=true, allowPrivilegeEscalation=false and drops ALL
capabilities. The accepted Docker executable is readable/executable by this UID
and the cloud app needs no writable filesystem/user entry. Actual image/runtime
compatibility is a pending parent gate, especially for other operator images.
No namespaces, secrets, value-bearing ConfigMaps, Ingress/TLS assumptions, RBAC,
HPA, PVC, volumes, privileged mode, host ports or credentials are emitted.

The generated README describes the Linux/amd64 OCI image built separately with
the three-context Docker BuildKit contract, explicit node import or operator
registry publication, exact image availability, namespace/resource collision
ownership and explicit context/namespace client/server dry-run and apply
examples. These are optional operator commands; even client dry run can fetch
schema/discovery data. They are never executed by CLI generation or this job.
The README also documents ports, signals, resource/probe limits and network/TLS
ownership. No namespace is embedded in the manifests.

## Local evidence and remaining gates

Rust/Cargo 1.99.0 local results, exact command arrays, native exits, logs, source
identities and preservation receipts are retained under `target/m15/yt089d/`.
All of these commands returned native exit **0** on the first validation attempt:

```sh
cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --all -- --check
cargo check --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets
cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets
cargo clippy --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets -- -D warnings
cargo --offline --locked xtask check docs
cargo --offline --locked xtask check samples
git diff --check
```

The CLI suite passed **50 unit tests and 7 integration tests**. The pre-existing
explicitly opt-in cloud native-runtime case stayed ignored; Windows-specific
binaries execute zero Linux tests, not Windows PASS. Docs reported 75 documents
and 89 backlog items; samples reported 12. `linux-cli-results.json` and
`repository-checks.json` bind native exits to their raw logs. Final documentation
and new-file whitespace checks are recorded separately. For an untracked file,
`git diff --no-index --check -- /dev/null FILE` exit 1 with empty diagnostics is
the expected content difference, not a whitespace failure; tracked `git diff
--check` alone does not inspect such files.

Focused tests cover strict flags/images/replicas/names, structural YAML semantics,
labels/selectors, ports, probes, numeric/string types, Linux scheduling, resource
and security settings, deterministic image/count combinations, prohibited
sensitive fields, synthetic secret/path absence, literal Unicode paths, stale or
missing lock, unsupported profile, edited/extra source, snapshot substitutions,
late destination races, symlinks, no-overwrite and staging cleanup. New native
Windows tests cover protected parent ACLs, junctions, no-replace races and
rollback; their execution remains pending. Existing 089a/b/c assertions are
unchanged; only the Compose unit fixture import moved to a shared test module.
Structural manifest tests are not external Kubernetes schema validation or a
working-Pod claim.

The executable integration prepares a real cloud app and explicitly generates
its Cargo.lock offline before generation. A separate retained real-source run
created `target/m15/yt089d/k8sproof`, explicitly generated its lock offline, and
generated `target/m15/yt089d/real-source k8s/` with an unusable tools PATH/Cargo
home, invalid Docker endpoint and nonexistent kubeconfig. All three commands
returned 0. Manifest/lock/main hashes stayed unchanged and no tool state was
created; see `real-source-generation.json` and `real-source-preservation.json`.
Its default-one-replica output SHA-256 values are:

| File | SHA-256 |
|---|---|
| deployment.yaml | `c19eb9d95494186f5611bd782986e103e78ede6df48d021dccb15db174836c67` |
| service.yaml | `870ea7c53768c6de9891f12e8798c37aba2553fa922b4505f9654461399fbc5c` |
| README.md | `972a0ec3c324ecfcbf48b2d7e8760f39ae5a98c15a32bd671491f1f2d7c2607f` |

The accepted 089c source archive SHA-256
`969881f6c265de8d392fe3c9c6916b57d673f553e93dc4902e965b570297ccd4`
matches all 45 baseline CLI source files. `accepted-cli-incremental.patch`
records only this task's changes to those accepted files, separately from the
full dirty diff against HEAD. `cli-source-sha256.json` binds the current CLI
inputs; `preservation.json` checks all baseline source/artifact and prior evidence
hashes, unchanged HEAD/index and the bounded changed/new path set.

Counters: **0/3 product repairs**, no harness repair, **1/2 operational retries**.
The sole operational correction was a read-only ADR lookup using an outdated
filename (ENOENT); bounded file discovery found and read the canonical ADR 0006.
No validation command failed or required retry. No server, browser, watcher,
container or cluster was started. Foreground validation commands completed,
test-owned temporary fixtures cleaned up, and retained app/output/log fixtures
are evidence, not background resources. `cleanup.json` records the final bounded
process/staging observation. Compile/unit evidence here is separate from browser,
CSP runtime, external schema and cluster validation; no such fresh result is
claimed for 089d.

## Incremental changed paths

- `tools/yetlet-cli/src/k8s.rs`, `src/k8s/tests.rs`, `tests/k8s.rs` and
  `templates/k8s/README.md`: private renderer, direct product tests and static
  operator instructions.
- `tools/yetlet-cli/src/lib.rs`, `src/main.rs`: strict CLI parsing/help and command
  execution; `src/compose.rs`: only internal lock-validator visibility.
- `tools/yetlet-cli/src/compose/tests.rs`: shared fixture import only;
  `src/platform/fs/safe_dir.rs`, `src/platform/fs/windows_tests.rs`: added safety
  cases; `src/platform/fs/windows.rs`: Kubernetes filenames join the existing
  protected deployment-parent gate.
- `tools/yetlet-cli/README.md`, this record, `docs/BACKLOG.md`, `docs/ROADMAP.md`
  and `docs/YT-089-CLOUD-NATIVE-GENERATORS-PLAN.md`: candidate documentation and
  status/index only. No backlog item is marked Done.

The cloud/default templates, scaffold implementation, Docker inspection/rendering,
image validator, framework source, Cargo manifests/locks and all earlier
verification/evidence files remain byte-identical to the baseline.

Kubernetes reference points for the selected fields are the official
[probe documentation](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
and [security context documentation](https://kubernetes.io/docs/tasks/configure-pod-container/security-context/).
The tested cluster version and effective image/runtime behavior still need
parent evidence; documentation lookup is not an execution result.

The next unblockable action is parent read-only source/receipt inspection,
then native GNU/MSVC checks and exact three-file parity, external schema checks,
separately authorized isolated Linux K3s namespace runtime/cleanup, and a distinct
read-only Reviewer. Windows runners need deterministic generator parity only,
not Windows Kubernetes. YT-089e final integration remains separate.

## Parent native parity, schema and isolated K3s acceptance candidate (2026-10-16)

After the original 089d source implementation job, the parent ran native Windows GNU/MSVC verification on the **same archived CLI source** (source ZIP SHA-256 `e5643da23aa53b29fa9ce956596373986aa846976cbb7f5cf2f3fe497e6249d0`). Each real Windows Runner recorded native Rust 1.99 fmt, check, tests and strict Clippy exit 0, including **51 unit tests** and successful deterministic Kubernetes output on a real generated cloud project. Windows generated `deployment.yaml`, `service.yaml` and `README.md` were byte-identical to the canonical Ubuntu Linux generator output. In both Windows environments, an initial inherited-ACL deployment parent was correctly refused; tests then used an owner-protected permitted parent instead of weakening the product ACL gate. Exact native receipts and their SHA-256 files are retained at `target/m15/yt089d/windows/{gnu,msvc}/` and `windows/summary.json`.

The parent independently performed a bounded Kubernetes manifest schema/behavior smoke on the existing **Ubuntu Multipass three-node K3s v1.36.4+k3s1** lab, using the unique namespace `yt089d-smoke-20261016`. It did not use Azure, install a controller or modify OpenBao or other application namespaces. The operator-built Linux/amd64 app image was explicitly imported into the three K3s nodes. Actual client-side **and server-side** Kubernetes dry-run succeeded, followed by applying the generated standard Deployment + ClusterIP Service in only that namespace. A Pod reached **1/1 Ready** on `k3s-lab-02`; Service ClusterIP HTTP returned **200**, application liveness and readiness each returned **200**, and actual SSR output passed CSP and escaped-HTML checks. The test then scaled the Deployment to zero, verified the Pod terminated, deleted the exact test namespace, removed only the test image from the K3s nodes and original Docker host, and cleaned temporary node transfer files. A later read-only `kubectl get namespace yt089d-smoke-20261016` returned **NotFound**. No other namespace was touched. This is an actual native K3s runtime result, not a substituted dry-run or claim of generic production qualification.

Retained source-bound parent evidence: `target/m15/yt089d/k3s-smoke/result.json`, `schema-dryrun.log`, `k3s-apply.log`, `k3s-rollout.log`, `k3s-pods.log`, `http-headers.txt`, `http-body.txt`, `liveness-headers.txt`, `readiness-headers.txt`, `pod-termination.log`, `cleanup.log`, and `node-image-import.log`, with original SHA-256 identities recorded inside the result JSON. The result states `namespace_created_and_deleted=true`, `scale_zero_and_pod_deleted=true`, `test_image_imported_then_removed_from_3_nodes=true`, `other_namespace_touched=false`, `azure_created=false` and `result=PASS subject to independent Reviewer gate`.

**Independent final Reviewer YT-089d PASS:** Read-only Reviewer `wc_job_1SJSI37ZToF_od0U` returned `overall_status=pass`, `issues=[]`, `required_followups=[]` after verifying the current 49-file CLI hash archive, 50 Linux unit + 7 integration tests, GNU/MSVC native results (51 unit tests each and three identical generated files), source-level K8s safety/probe/security assumptions, and all 26 Windows plus 14 K3s receipt hashes. The reviewer made no modifications and did not run tests, build images or touch the cluster. **YT-089d is Done at bounded config-only Linux/amd64 Kubernetes scope**; YT-089e full M15 integration remains open. The user selected **Linux OCI images and Linux Kubernetes as deployment targets**; native Windows Runners verify the Rust generator/output parity only, not Windows Kubernetes. No Git commit/push, image publication or persistent K8s namespace is authorized by these tests.
