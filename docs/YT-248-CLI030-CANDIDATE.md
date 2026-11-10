# YT-248 — CLI 0.3.0 candidate preparation

**Checkpoint: local candidate qualification and independent implementation
review PASS (2026-11-10), zero issues/followups. Authorized Git/CI delivery follows
this checkpoint with receipts retained alongside the review. Stop
before formal release execution or publication; Reader Edition stays fixed.**

## Authority and scope

The Owner authorized this slice under the already independently reviewed
[CLI 0.3.0 plan](CLI-030-RELEASE-PREPARATION.md), after accepting
[YT-247](YT-247-SDK-VERSION-RESOLUTION-IMPLEMENTATION.md) and
[ADR 0010](adr/0010-dynamic-sdk-version-resolution.md). Preserve the original
Style plan review and YT-247 evidence. This checkpoint prepares and verifies the
candidate locally, then delivers the reviewed changes through normal PR CI and
master CI. It does not select an immutable release C/tag, dispatch protected
release/sign/V/publish workflows, upload a crate or change the guide edition.
YT-248 remains In progress until the later release work has accepted receipts.

Base: `5dc8a9a990ac083e07081ea3be008748239c8a58` (master after PR #70).
Local evidence: `artifacts/verification/yt248-cli030-candidate-20261110/`.
Logs, original failures, archive, executable, source/resource hashes, metadata,
locks and runtime/Chrome receipts are retained independently of Rust build output.

## Selected inputs and preservation

| Authority | Candidate value and responsibility |
|---|---|
| CLI package | `yetlet-cli 0.3.0`, one schema3/cli package; later tag would be `yetlet-cli-v0.3.0` |
| CLI build SDK | Exact registry dependency `yetlet = "=0.9.0-alpha.7"`; only CLI and SDK package identities/checksum changed in its existing lock |
| Embedded source export | Canonical sync from committed SDK source at base `5dc8a9a`; snapshot alpha.7, 81 SDK source files, root workspace/lock and byte-equal browser startup assets |
| Template/app selection | Existing `sdk-compat-v1`: native/browser alpha.6 or alpha.7; Style templates alpha.7; explicit/default/offline choices keep their own exact pins, lock and receipt |
| Historical release bindings | CLI0.1.0/0.1.1 → alpha.5; CLI0.2.0 → alpha.6; closed new binding CLI0.3.0 → alpha.7; unknown bindings still rejected |
| Accepted publications | SDK alpha.7 and CLI0.2.0 unchanged; no tag/archive rebuild, upload or publication-authority update |
| Reader Edition | SDK alpha.6 / CLI0.2.0, unchanged `site/pages.toml`, first-app text and docs-site manifest/lock |

The SDK source files/manifests/assets match the immutable alpha.7 tag. The
embedded root lock comes from the later accepted base, including the already
merged SQLite remediation; it is not a claim that the published SDK archive or
its original lock was rewritten. `sdk-identity.json` records the actual snapshot
source and closure digests. No SDK snapshot is copied into generated registry
apps. Source export retains its existing qualified scope.

The version synchronization tool previously derived current published CLI/guide
text from the new CLI build SDK. Advancing that dependency exposed a real
Reader-pin drift. It now obtains the published CLI build SDK from the immutable
historical binding and the first-app pin from `site/pages.toml`. Focused tests
cover the alpha.7-build/alpha.6-reader split and idempotence.

## Candidate package and installed consumers

Normal `cargo package --locked --allow-dirty` verifies the local candidate;
`--no-verify` is not used. The dirty-tree flag describes this precommit local
checkpoint, not an immutable formal release source. The verified archive is
installed from its extracted package with normal `cargo install --path ...
--locked --offline`; the executable is then run outside the checkout. Package
manifest, included files, archive and executable SHA-256 values are retained in
`candidate-metadata.json`, `candidate-package-files.json` and
`candidate-identity.json`. Corrected archive SHA-256:
`0cc27fb24294f4d90e472b540e8dc6399334e6b5d09b6c5e9e1cad1af2157fea`.

This is **candidate-archive CLI installation with registry-only SDK consumers**.
CLI0.3.0 has not been uploaded, so this is not a claim of a fresh crates.io
CLI0.3.0 installation. The existing publication consumer now invokes the same
installed-consumer qualification after its real anonymous exact registry CLI
installation; that future path records `cli_install_source = crates.io` and its
own installed-package lock separately from app locks. Legacy consumers retain
their fixed generated-pin checks.

The isolated Cargo home starts without resolver metadata. Only a small explicit
environment allowlist crosses into the consumer; no publish token, SDK path,
patch, replacement or source override is forwarded. Registry metadata/checksums,
resolution receipts and both app/Style locks are saved. The qualification covers:

- Real binary version/help/new-help, default native plus each of the four template
  selectors, fixed explicit alpha.7 and a separate default checked against
  captured refreshed metadata and the shipped compatibility policy.
- Exact registry manifest pins, paired browser SDK versions, lock checksum/receipt
  agreement, offline cache miss without a project, prepared browser Style offline
  hit, and repeated creation refusal without lock changes.
- Dependency preparation through explicit `cargo fetch --locked` and browser
  `npm ci --ignore-scripts`; CLI creation/build does not install tools or packages.
  Both dependency-free Style generator closures and separate locks are retained.
- Actual installed-CLI build/run of native Style: no-JS loopback SSR, expected
  escaped content and class, linked fetchable generated CSS, MIME/CSP and hashes.
- Actual installed-CLI browser Style Vite/WASM build and static preview: real
  index, stylesheet and WASM fetches, Chrome WASM readiness, computed flex/12px/
  bold Style output after mount and a keyed update, no inline styles, no browser
  console/CSP/network errors. Preview changes only the app-owned Vite port to an
  ephemeral loopback port, retaining/restoring the original configuration.
- Bounded readiness/commands and finally cleanup of owned server/Chrome trees;
  actual termination exits, closed ports and removed Chrome profiles are recorded.

Original browser qualification exposed a real defect: the keyed row updater
replaced its complete class attribute, removing `app-toolbar` even during initial
mount. The smallest Style scaffold correction preserves that class in both
initial View and updater output. Ordinary browser templates keep their existing
class behavior. A focused regression and the corrected archive-installed Chrome
run pass. Original failed browser receipts and the initial archive remain under
`initial-*`; no failed record was overwritten. The first new test used swapped
Style fixture options; its original failure and the corrected suite are retained.

## Validation and limits

| Check | Result |
|---|---|
| Rust 1.99 CLI tests, locked/offline | 183 PASS / 8 ignored after correction; initial 182 PASS / 8 ignored before the new regression |
| All release Python tests | 149 PASS / 1 skipped (150 total); synthetic network failure/wait cases never contact a registry |
| Version sync tests | 12 PASS; combined release/docs/sync focused suite also passes |
| Normal package verification and archive installation | PASS; complete Style/resource inclusion and exact alpha.7 registry build dependency |
| Installed candidate consumers | Six creations, exact/default/offline, locks/repeat, native and browser runtime/Chrome PASS |
| Source docs/version/resource gates | PASS; docs/DAG, sync check, release-doc check, formatting and workflow lint recorded in evidence |
| Windows | New CLI0.3.0 Style/browser runtime remains paused/unverified; YT-247 Windows resolver proof is historical, separately scoped |

Local tools are existing Rust/Cargo1.99, wasm-bindgen0.2.129, Node24 and Chrome.
The existing group-writable Rust installation is copied byte-for-byte into a new
private task-owned tree for the CLI's protected tool probes; no global/Runner
permissions or checks change. The future schema3/CLI0.3.0 publication job prepares
its wasm32/wasm-bindgen prerequisites and checks Node/npm/Chrome before any
publication mutation. Normal fixture tests remain network-independent. Hosted
release execution and a real registry CLI installation are still outstanding.

The eight ordinary-suite ignores comprise one deadline child helper (launched by
its parent test), two opt-in live registry/compile tests, Cloud and Config
cached-app tests, two explicit Vite/Cargo integration tests and one separately
provisioned standalone Tailwind test. The candidate consumer evidence above is
separate from those opt-in tests; Cloud/Config/standalone Tailwind were not rerun
at this checkpoint.

The first independent review found one P1 in the new consumer: Chrome launched
without the sanitized environment, so a future publication invocation could
forward its ambient registry/GitHub tokens. The narrow correction threads the
consumer environment and PATH into Chrome, with a sentinel-environment launch
test and an actual bounded Chrome preview delta. The initial review JSON is
preserved; corrective independent review PASS has zero issues/followups. Native
trace verifies reviewer / gpt-6-astra / xhigh; shared resolver observation is
enforced, with the selector value matching the parent effort.

Independent Reviewer JSON, trace/effort evidence, Strict/Builtin guards and Git/CI
receipts are retained in this run's evidence directory. Only the requested local
candidate/Git-delivery checkpoint can close here; no publication success is
inferred from a local test or a passing PR.

## Next handoff and cleanup

After independent review and successful PR exact-head/master CI, stop for the
Owner's next release instruction. The next unblockable YT-248 work is selecting
an immutable clean release source, normal hosted one-package build/evidence
readback, schema3 signing, independent V and the approved five-file public
allowlist/registry plan. Actual publication stays behind the existing Owner gate;
do not supply a token or retry a failed upload automatically. After accepted
publication receipts, update current-publication documentation independently of
the still-pinned Reader Edition.

The Owner authorizes deleting this round's rebuildable Rust cache and temporary
consumer/tool copies after qualification. Preserve logs, original failures,
archives, accepted executable, manifests, metadata, locks, review and Git/CI
receipts. Cleanup inventory and actual deletion results belong in `cleanup.json`;
shared Cargo caches, historical evidence, old mixed targets and the GPUI worktree
remain outside this deletion scope.
