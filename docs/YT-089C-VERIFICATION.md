# YT-089c — Accepted Configuration-only Compose Generator

**Date:** 2026-10-16. **Final state:** Linux/amd64 configuration-only Compose generator independently accepted by Reviewer `wc_job_OCVGSYl5uAGET8w3` PASS, zero issues and followups, after native GNU/MSVC source/output parity and disposable real Compose runtime verification. The initial implementation-only status and failed receipts below remain historical. YT-089 remains In progress;
YT-020 remains Blocked.

## Authority, checkout and preservation

Work stayed in the original dirty `$HOME/repos/yetlet` checkout, branch
`master`, HEAD `d24a05033c57e5b98dd57bf83189bc1297f6f28c`. The canonical Generalist
contract, REASONING_POLICY, MATERIALITY_GATE, AGENTS.md, ADR 0006, [M15 design](YT-089-CLOUD-NATIVE-GENERATORS-PLAN.md),
[accepted YT-089a](YT-089A-VERIFICATION.md) and [accepted YT-089b](YT-089B-VERIFICATION.md)
were read before implementation. This is role-directed WebCodex work, with no
native managed-agent identity/model/effort assertion or child dispatch.

The original YT-089a Reviewer `wc_job_0dvTDNAV22Vbpc5j` PASS and YT-089b Reviewer
`wc_job_IC92mEXoQMsG4nTg` PASS (zero issues) remain accepted at their recorded
bounds. Their historical failures, actual Linux image/HTTP/CSP/SIGTERM and native
GNU/MSVC eight-file parity evidence were preserved; they are not new Compose
runtime or Windows results.

A 678-file SHA-256 baseline and original status/index listing were captured under
`target/m15/yt089c/` before editing. Existing cloud/default templates, framework
source, root/xtask and CLI Cargo manifests/locks, prior verification documents
and artifacts remain unchanged. The Docker inspection/publication boundary was
shared without weakening its checks; original Docker test assertions remain
unchanged, with only their framework-fixture import moved to one shared module.
No reset, stash, clean, clone, worktree, staging, commit or push occurred.

## Delivered command and image contract

```text
yetlet generate compose --manifest-path APP/Cargo.toml --output NEW_DIRECTORY --image localhost:5000/team/app:v1
```

All three options are explicit, unique and literal. Missing, unknown, duplicate,
misplaced or blank arguments and invalid images produce usage exit **2**.
Generation/source/output preflight failures use exit **1**. The existing
`generate docker` and `new --cloud` contracts remain intact. No new dependency,
SDK, package or Cargo graph change was needed.

The internal `image::validate` helper is available for the later YT-089d slice.
The initial contract requires a lowercase ASCII **registry/repository** and
either an explicit non-`latest` tag or `@sha256:` plus exactly 64 lowercase hex
digits. Tags allow ASCII letters/digits/underscore/dot/hyphen, starting with a
letter/digit/underscore, up to 128 bytes. Repository components use the bounded
OCI lowercase component grammar. Registry hosts must be dotted DNS/IPv4 or
`localhost`, optionally with decimal port 1-65535 without leading zeros. The
repository reference is at most 255 bytes, the whole image at most 512.
Unqualified names, implicit/default/latest tags, IPv6, tag-plus-digest, Unicode
lookalikes, controls/newlines, whitespace, quotes, dollar signs, backticks,
backslashes and escapes are rejected, without echoing the rejected value.

Every YAML scalar is explicitly single quoted; the image grammar excludes the
characters that would need escaping or trigger Compose interpolation. The image
is substituted only into the YAML scalar, never a shell command or script.
The generated README is static and contains no selected image or personal paths.

## Source, lock and output boundaries

`docker::generate_checked` is a thin shared entry to the accepted `inspect`
implementation: pinned native app handles, exact standalone native-ssr-v1
manifest, explicit existing lock, exact parsed/text cloud source and supported
core/HTML/server-axum framework source/dependency closure. Metadata-only, edited
source, extra targets, build scripts, unknown dependencies/features/layouts and
unsafe paths remain refused. A second inspection compares the full snapshot
before the native output writer. No app/source file is edited or copied.

Compose adds static lock consistency checks for this fixed profile: Cargo.lock
v4; unique app/core/HTML/server root packages at 0.1.0 without registry sources;
their exact direct dependency lists; stable axum 0.8, Tokio 1, serde 1 and Tower
0.5 registry identities; registry checksum syntax; unique referenced lock nodes;
and Tokio's signal dependency. Missing/stale app names, versions, roots or
dependency nodes are refused. This is **not a Cargo solver**, registry content
verification, image provenance check or promise that an arbitrary lock builds.
The separate operator image build must still use the accepted locked build.
The unit fixture is deliberately a structural lock, not a runnable Cargo fixture;
the separate real-source smoke uses the existing actual Cargo-generated lock.

Exactly **two files** are generated: `compose.yaml` and `README.md`. The service
is `app`, with the literal selected image, `platform: linux/amd64`,
`pull_policy: never`, container `YETLET_HOST=0.0.0.0`, `YETLET_PORT=8080`, and host
mapping **127.0.0.1:8080:8080**. The existing YT-089b image is Linux OCI/amd64
GNU/glibc only. Operators build/audit it separately; generation does not build,
pull, start or publish it. A digest must already resolve locally by that exact
identity; a loaded tag alone may not do so.

SIGTERM and `stop_grace_period: 30s` are explicit. The app has no automatic drain
deadline: 30 seconds is a documented example operator budget, not a guaranteed
drain bound. Long requests/streams require measurement and adjustment. No
HEALTHCHECK assumes curl/wget/a helper exists, and no runtime health claim is
made. There is no implicit build, host networking, privileged mode, source or
secret volume, credential, environment-file interpolation or shell script.
The CLI does not read `.env` or environment secrets. The README explains that
separately invoked Compose can itself read a project `.env` and that the emitted
YAML has no substitution or `env_file`.

Publication reuses Linux directory-FD/RENAME_NOREPLACE and Windows protected local
NTFS/ACL/reparse/no-replace adapters, with their owned staging, rollback and
revalidation rules. Windows explicitly applies the deployment parent ACL gate
to Compose as well as Docker; ordinary `new` retains its historical policy.
No arbitrary nested output paths were added: only the existing fixed
`src/main.rs` and `cargo-overlay/Cargo.toml` child paths are permitted by the
writer. Root-only Compose output creates no child directory.

## Actual Linux verification and retained failures

Rust/Cargo **1.99.0** were used with cached dependencies. Exact argument arrays,
native exit codes and full logs are in `target/m15/yt089c/`:

```sh
cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --all -- --check
cargo check --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets
cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets
cargo clippy --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets -- -D warnings
cargo xtask check docs
cargo xtask check samples
git diff --check
```

Final CLI fmt/check/test/strict Clippy each returned **0**
(`linux-cli-final-results.json`). Tests: **44 unit passed**, plus **4 integration
passed** (cloud/default scaffold, existing frozen build/dev, Docker, Compose);
the existing opt-in generated-cloud native runtime case stayed **ignored**.
Windows-specific integration binaries run zero Linux tests, not Windows PASS.
Repository docs/sample and tracked/new-file whitespace receipts are recorded
separately in `repository-checks.json` and `new-file-checks.json`.
All eight new-file `git diff --no-index --check` probes returned **1** with empty
diagnostics: the expected new-file content difference, with no whitespace error.
This is separate from the tracked diff check and does not require staging.

Focused cases cover parser exit codes, ASCII/tag/digest validation and injection,
two deterministic outputs, no paths/secrets, source preservation, missing/stale
locks, edited source and framework closure, existing/traversing/missing output,
source/lock/framework races, symlinks and unsafe parent ownership, pinned app
rename, late destination, partial staging cleanup and rejected arbitrary tree
paths. Executable generation succeeds with a nonexistent tools PATH/Cargo home
and an invalid Docker endpoint. No tool cache, staging residue or extra file is
created. The existing build/dev regression itself explicitly invokes Cargo as
its established test workload; generation never does.

`real-source-generation.json` records successful Compose generation against the
existing YT-089b `cloudproof` app, real framework source and actual lock, with
tools unavailable and no Cargo home. Manifest/lock/main checksums are unchanged.
Output is retained at `target/m15/yt089c/real-source compose/`. This is local
configuration evidence only.

Retained development failures and bounded corrections:

- Initial strict Clippy returned **101**, `clippy::duplicate_mod`, because the
  new test fixture loaded the existing framework fixture a second time. One
  shared test module fixed this harness seam; strict Clippy and focused Compose
  tests then passed. Existing Docker assertions were not weakened.
- The first real-source smoke returned **1** because the evidence parent created
  under umask 0002 was group-writable. The adapter correctly refused it. Only the
  task-owned evidence directory was changed to mode 0700; the single rerun passed.
  The initial JSON/log remain as `real-source-generation-initial.*`.
- Source inspection found a static lock-check gap for declared serde/Tower
  version ranges. One bounded product correction added those stable-version
  checks and stale-version negatives; final verification passed.
- One patch was rejected before applying edits because its context predated
  rustfmt. The exact lines were inspected and the patch corrected once.

Counters: **1/3 product repair**, **1/2 operational retry**, two distinct harness
seams each corrected once; no repeated unresolved failure. Full accounting is in
`repair-accounting.json`. No server, browser, container, cluster or background
service was started by this job. Foreground validation processes completed;
test-owned temporary resources use the existing bounded test cleanup.

## Changed paths and remaining gates

Incremental YT-089c changes, separate from all earlier dirty M15 work:

- `tools/yetlet-cli/src/lib.rs`, `src/main.rs`: parser, help, dispatch and shared
  test-fixture import.
- `tools/yetlet-cli/src/compose.rs`, `src/compose/tests.rs`, `src/image.rs`:
  deterministic generator, static lock consistency, shared image validation.
- `tools/yetlet-cli/src/docker.rs`, `src/docker/tests.rs`: thin existing
  inspection/publication entry and one shared fixture import; Docker rendering
  and test assertions unchanged.
- `tools/yetlet-cli/src/platform/fs/safe_dir.rs`, `src/platform/fs/windows.rs`,
  `src/platform/fs/windows_tests.rs`: Compose native safety coverage and Windows
  deployment-parent ACL gate.
- `tools/yetlet-cli/templates/compose/compose.yaml`, `templates/compose/README.md`,
  `tests/compose.rs`, `tests/fixtures/compose_app.rs`: two templates and focused
  cross-platform fixtures/integration tests.
- `tools/yetlet-cli/README.md`, `docs/README.md`, `docs/ROADMAP.md`,
  `docs/BACKLOG.md`, `docs/YT-089C-VERIFICATION.md`: candidate documentation and
  explicit pending acceptance. No task was marked Done.

Parent must perform native GNU/MSVC source-byte and two-file output comparison,
actual `docker compose config`, separately authorized bounded disposable up/down
HTTP/CSP/SIGTERM and cleanup, and a separate independent Reviewer. None was run
or claimed PASS here. Added Windows ACL/junction/race tests remain unexecuted
on Windows in this job. No Docker, Compose, K8s/K3s namespace/Pod/cluster action,
registry access, image build/publication, browser or external environment change
occurred.

The next unblockable step is parent-owned YT-089c native parity/config/runtime
verification and independent review of this exact candidate. YT-089d Kubernetes
implementation remains a later separately bounded task; no K3s action follows
from this result. Final source/preservation/index receipts accompany this handoff
under `target/m15/yt089c/`.

## Parent-run native Windows and real Compose runtime supplement (2026-10-16)

These are **additional independent validation runs**, not results from the earlier Generalist Job. A separate read-only YT-089c Reviewer is still required.

**Actual Linux Compose schema and runtime:** The generated real-source compose.yaml passed Docker Compose config --format json against an intentionally invalid DOCKER_HOST. It declares only one Linux/amd64 app service, a literal operator-selected image, localhost-only 127.0.0.1:8080 host publication, YETLET_HOST=0.0.0.0 and YETLET_PORT=8080, with no privileged, secret-volume or implicit image build settings. The normalized model is retained at target/m15/yt089c/compose-config-real.json.

The operator separately rebuilt a disposable local image, tagged localhost:5000/yetlet/cloudproof:yt089c, using the previously independently accepted **three physically sanitized BuildKit contexts**. The generated Compose file was used with up -d --no-build --pull never. Real HTTP GET / returned **200**, strict CSP and escaped HTML without scripts; health/live and health/ready returned **200**. Docker Compose stop --timeout 30 sent real container SIGTERM and the listener stopped accepting connections. Compose down --remove-orphans returned **0**, the temporary image deletion returned **0**, and post-run Docker inspection confirmed **zero task-owned containers and no retained task image**. Native build, HTML, headers, stop and cleanup evidence is retained under target/m15/yt089c/compose-* and the source inventories under target/m15/yt089b/sanitized-validation/. The CLI generator itself made no Docker/registry/network calls.

**Native GNU/MSVC negative fixture history:** The first Windows GNU source candidate ran fmt/check/strict Clippy but failed one of 45 unit tests. The test hook attempted to write to the already-pinned native Cargo.lock and received Windows ERROR_SHARING_VIOLATION (32), then erroneously unwrapped the failure. Initial investigation separately addressed the late-destination test's CreateDirectory assumption; the numbered source line showed the remaining root cause was the **correct read-only HANDLE protection of Cargo.lock**. The bounded final test-only correction recognizes only the exact native sharing refusal, asserts the lockfile stayed unchanged, mutates an already inspected application source to preserve the generator's fail-closed output contract, and uses an injected foreign file to test no-replace publication. Dedicated Windows filesystem tests still cover protected ACLs, junctions and late destination races. **No production source inspection, lock policy or filesystem protection was relaxed.** Historical failed runs remain retained.

**Source-bound latest GNU/MSVC PASS:** The final corrected 45-file CLI archive has SHA-256 969881f6c265de8d392fe3c9c6916b57d673f553e93dc4902e965b570297ccd4. GNU and MSVC independently verified all 45 file digests and each ran Rust 1.99 native fmt, Cargo check --offline --locked --all-targets, Cargo test --offline --locked --all-targets and strict Clippy, all **exit 0**. Each native suite passed **45 unit tests**, the cloud/default, Docker and Compose integrations and the existing 5 Windows frozen build plus 3 Windows new tests; the explicitly opt-in generated-app runtime case remains ignored by default. Linux latest source passed 44 unit plus 4 integration, fmt/check/strict Clippy, docs (75 documents, 89 backlog items), samples (12) and git diff --check.

**Exact deterministic output parity:** Each Windows native CLI generated Compose against a real explicit-lock cloud application with local framework dependencies, a native ACL-protected output parent and an intentionally unusable external tools PATH/Docker host/kubeconfig. Both exited 0 and preserved application manifest, lock and main source hashes. The generated files' SHA-256 matched the real Ubuntu output byte-for-byte: compose.yaml = bbe12ff1bdc7eee6eb5d044a18c9ddfd95affcb22d8c739d4188997f4d1dacfe, README.md = 3797bb5ed6ea05b76764db79bd9d3076b94aa3e358f4f759d1dd6e3e6fec548e. The native raw receipts and cross-transfer checksums are retained under target/m15/yt089c/windows/{gnu,msvc}/, with summary.json mapping the exact archive/source hashes.

**Acceptance boundary:** This verifies configuration-only Linux/amd64 Compose generation and one disposable operator-initiated real container lifecycle; it does not establish Windows container images, image publication, external ingress, production readiness or Kubernetes support. The 30-second Compose stop grace is an example, not an application-enforced drain deadline. No K3s/Azure resource was created for this slice. **YT-089c remains a reviewed-candidate only until its own independent Reviewer PASS.** YT-089d/e and release YT-020 remain pending/Blocked, and no Git staging, commit or push was authorized.

## Final independent YT-089c implementation acceptance

Distinct read-only Reviewer Job `wc_job_OCVGSYl5uAGET8w3` returned **PASS**, `issues=[]`, `required_followups=[]`. It examined the corrected 45-file source ZIP and native GNU/MSVC hashes, inspected the source-only Win32 HANDLE sharing test correction, validated literal-image/YAML/Compose parser and safe no-replace output semantics, and reconciled the Linux real Compose BuildKit/HTTP/CSP/SIGTERM/cleanup evidence and two-file native byte parity. The original first Windows failures and one blocked MSVC admission remain historical, not PASS. The Reviewer performed no edits, tests, Docker actions or deployment. **YT-089c is Done only at bounded configuration-only Linux/amd64 Compose scope with native Windows generator parity**. YT-089d Kubernetes and YT-089e final integration remain open; YT-020 remains Blocked. No source commit, push, image publication or K3s resource was authorized by this review.
