# YT-247 — Dynamic SDK Resolution Implementation

Date: 2026-11-10 (Asia/Taipei). **Phase B implemented; independent Reviewer PASS.**
The Owner accepted the [Contract](YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md) and
[ADR 0010](adr/0010-dynamic-sdk-version-resolution.md) after the
[Phase A delivery](YT-247-PHASE-A-HANDOFF.md), then authorized implementation,
independent review and Strict/Builtin guarded Git delivery. This record describes
checkout source, not a new published CLI. YT-248 is not started.

## Delivered behavior and file boundaries

| Surface | Implementation |
|---|---|
| `tools/yetlet-cli/src/sdk_version.rs` | Canonical exact SemVer, typed request/result, sparse-record validation, pure compatibility filtering and stable-first selection. Only reviewed alpha.6/alpha.7 combinations are allowed by the shipped policy. |
| `src/sdk_registry.rs` | Injectable registry transport, fixed crates.io sparse HTTPS paths, complete-set atomic cache, metadata digests/timestamps, conditional revalidation, bounded reads/deadline and no silent stale fallback. Offline never calls the transport. |
| `src/creation.rs`, `lib.rs`, `main.rs` | Parse exact/offline choices, validate protected destination and installed tools, resolve once, pass the result into scaffold, generate/validate root lock in private staging, publish once and report source/freshness. |
| `src/scaffold.rs`, `browser.rs` | Thread selected version into exact registry pins for native and equal-version browser pairs. Template source and SDK source remain separate. Generated instructions retain the new app lock. |
| `src/platform/fs/`, `src/platform/process/` | Extend existing owned-staging and child-process adapters for bounded Cargo lock output, cancellation, cleanup and no-replace publication. Preserve the nested Style lock and all emitted template inputs. |
| `resources/sdk-compatibility.json` | `sdk-compat-v1`: native/browser alpha.6 and alpha.7; native-style/browser-style alpha.7; MSRV 1.99; required features and equal-version browser pair. Empty stable set and explicit reviewed-prerelease fallback. |
| `resources/sdk-identity.json`, `scripts/sync_resources.py`, resource tests | Record immutable snapshot SDK/source and resource/policy digests. Future explicit sync uses committed SDK inputs. Existing snapshot bytes are not regenerated. |
| `scripts/release_docs.py`, `release_registry.py`, `sync_workspace_versions.py` | Distinguish CLI build SDK and app policy; verify snapshot closure/digests and archive identity instead of requiring a later checkout's SDK to equal the old snapshot. Legacy metadata remains supported. |
| `scripts/release-publish.py` | Keep historical CLI consumers on their immutable bindings. A future nonlegacy CLI consumer passes an explicit supported app version and verifies its separate app receipt/lock; no CLI0.3 binding is added here. |
| CLI README and adjacent architecture/contract/index/backlog documents | Describe source-only resolver behavior and accepted version authorities while retaining the published release and Reader Edition identities. |

The CLI package stays **0.2.0**, build SDK stays **exact alpha.6**, and embedded
snapshot stays **alpha.6** at SDK source
`c7e63724b2fc303abcf85ff503cc8f230794cb41`. Default app resolution selects
**alpha.7** when available and compatible. Its source comes from Cargo's registry
package, never the embedded source map. Ordinary native/browser can deliberately
select alpha.6. The Style templates require alpha.7. New dependencies are pinned
Rust `semver` and `ureq` with rustls; existing `tempfile` becomes a runtime
dependency for atomic cache replacement. No Python dependency is introduced.

The [Contract's diagrams and online/offline table](YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md#minimal-module-boundary)
remain normative. The concrete module flow is:

```mermaid
flowchart LR
    N["new parse + protected/tool preflight"] --> C["creation orchestration"]
    C --> R["sdk_registry: refresh or offline complete cache"]
    R --> P["sdk_version: pure policy + ordering"]
    P --> V["ResolvedSdk + provenance"]
    V --> S["native/browser/Style exact manifests"]
    S --> L["owned staging: Cargo lock + validation + receipt"]
    L --> F["create-only publication"]
```

Online refreshes every required package and fails on network failure. Offline
(`--offline` or `CARGO_NET_OFFLINE=true`) requires both CLI metadata and Cargo
dependency caches; the two caches serve different purposes. Metadata results show
source and age. Creation preserves Cargo's registry checksums and writes
`.yetlet/sdk-resolution.json` with SDK/policy/metadata identity and app-lock digest.
Later build/run/watch remain frozen and resolver-free. Exact SDK identity is
stable; different creation dates can still produce different transitive locks.
Retaining the actual app Cargo.lock is required for the latter identity.

## Verification

Evidence root: `artifacts/verification/yt247-resolver-20261110/` (ignored).
Rebuildable Cargo output: `.cache/yetlet/builds/yt247-resolver-20261110/`.
Both use the existing `scripts/dev_artifacts.py` layout. Original failures,
remote prerequisite failures and prior qualification evidence remain retained.
Commands below use `CARGO_TARGET_DIR` set to this build directory.

| Check | Result and scope |
|---|---|
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --locked --offline` | PASS; 182 tests (120 library and 62 integration), 8 ignored, no failures. Resolver fixture/cache/clock/transport cases, real Cargo local vendor-source locks, exact manifests/receipt, repeated creation, failed lock cleanup, post-creation frozen-build preservation and owned combined-output cancellation are exercised. Windows-only tests run zero cases on Linux. |
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --locked live_registry_native_browser_and_default_locks -- --ignored --nocapture` | PASS, separate opt-in crates.io lane: native/browser explicit alpha.7 and default selection with real Cargo locks and index checksum/source verification. |
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --locked registry_template_compile_compatibility -- --ignored --nocapture` | PASS; all six policy combinations compile from real registry packages: native alpha.6/alpha.7, browser WASM alpha.6/alpha.7, native-style alpha.7 and browser-style WASM alpha.7. Rust 1.99, `cargo check --locked`. |
| `cargo clippy --manifest-path tools/yetlet-cli/Cargo.toml --all-targets --locked --offline` | PASS with warnings; existing unused watch helpers and a nonblocking test suggestion are not treated as platform/runtime proof. |
| `PYTHONPATH=scripts:tools python3 -m unittest test_release_docs test_sync_workspace_versions test_release_publication` | PASS, 70 tests; build/policy separation, snapshot tamper/closure, historical publication bindings and separate dynamic app receipt/lock. |
| `python3 scripts/release_docs.py check` | PASS; published identities and guide edition remain unchanged. |
| `cargo xtask check docs`, Cargo formatting, tracked/untracked whitespace and local document links | PASS; results retained with the review/delivery receipts. |

Default tests do not depend on the current crates.io state. Opt-in registry lanes
are intentionally separate and may fail if publication, yank or availability
changes. No native server, browser runtime, Node/Vite build or CSP journey is
claimed by these compile tests. YT-248 retains that separate installed-consumer
acceptance and the original independent Style plan PASS.

Initial test failures were fixture/environment failures, not silently discarded:
a temporary test parent needed the existing protected-directory mode; two old
source-export fixtures incorrectly paired the alpha.6 snapshot with the alpha.7
checkout; the binary preflight assertion assumed safe installed tool permissions.
Focused corrections use the existing alpha.6 fixture and assert the doctor result.
The subsequent complete CLI suite passes. Product preflight was not weakened.

## Platform and environment limits

The local installed Rust/Cargo directory chain is rejected by the existing doctor
policy because of group-writable tool directories. The binary therefore refuses
new before metadata/cache/target work with infrastructure exit 125. Resolver and
creation tests inject the existing preflight seam; real Cargo lock/compile lanes
exercise Cargo directly. This is not an end-to-end successful binary-new claim
on this host. No tool installation or Runner permission was changed.

Two existing Windows environments were observed through WebCodex without editing
their checkouts. One lacks `link.exe`; the other has MSVC but lacks the required
native OpenSSL installation. Both stopped at prerequisites, with original logs
transferred and no active jobs at closeout. The changed Windows staging/process
code has **not** completed compile or runtime verification. The first independent
review found incompatible staging handle sharing and a missing READ_CONTROL
right during lock adoption. The correction releases creation handles while a
normal directory pin protects Cargo preparation, then reacquires and verifies
original identities before cleanup/publication. Lock adoption requests the
security rights and records ownership before checking ACLs. A focused test in
the existing Windows staging test module covers input rereads, successful lock
adoption, repeated-destination refusal and failed-lock cleanup; it is not claimed
as executed on Linux. The Linux library suite passes after the correction.
Corrective review confirmed both filesystem fixes and found the immediate
Windows process caller still refused `cargo generate-lockfile`. The cwd-pinning
allowlist now admits that literal Cargo action while retaining all executable
and directory checks; an assertion in the existing Windows process tests checks
admission and refusal of unrelated commands without spawning Cargo.
Existing historical
Windows evidence does not qualify this new code. Windows Style revalidation
remains paused/unverified as in the accepted plan.

## Independent review and Git boundary

Initial independent read-only review failed on those two Windows correctness
defects; the result is retained as `reviewer-result.json`. The first corrective
review found the process admission defect, retained as
`reviewer-correction-result.json`. Final corrective review by
`/root/yt247_resolver_review_20261110` is **PASS**, with zero issues and required
followups. All 53 reviewed file hashes matched before these factual result/status
annotations. Native trace matches role `reviewer`, configured model
`gpt-6-astra` and effective effort `high`; the shared resolver observation is
enforced without degradation. `reviewer-final-result.json` and
`reviewer-final-trace.json` retain the result and bounded telemetry:

```json
{"overall_status":"pass","issues":[],"required_followups":[],"optional_notes":[]}
```

After PASS, delivery uses explicit Strict/Builtin staged and
outgoing-range guards, commit, PR, exact-head CI, squash merge and master CI.
Commit/PR/CI identities are external receipts because this precommit record
cannot contain its own future hash. No release workflow is dispatched.

Historical CLI0.1.0/0.1.1 -> SDK alpha.5 and CLI0.2.0 -> alpha.6 bindings are
unchanged. No accepted Release Record, SDK alpha.7 tag/archive or public guide
edition is rewritten. The public Reader Edition stays alpha.6 / CLI0.2.0.

## Next handoff and stop

After independent PASS and Git delivery, stop. YT-248 is the next backlog item,
and requires a separate Owner instruction. It retains the reviewed Style plan,
selects CLI0.3.0 and its alpha.7 build/snapshot inputs separately, and qualifies
installed exact/default consumers, native HTTP/CSS and browser Vite/WASM/runtime/
CSP with owned cleanup. This implementation does not select a release candidate,
publish a CLI or enter that qualification. Resolve the documented Windows native
toolchain prerequisites in an authorized environment before claiming Windows
support for the changed creation path.
