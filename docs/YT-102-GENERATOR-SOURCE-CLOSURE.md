# YT-102 follow-up — Config/Secrets generator source closure

Date: 2026-10-22. Status: Done at the bounded source/configuration scope after
focused verification and independent Reviewer PASS, zero issues/followups. This is a separate repair after the
owner-authorized 65-path naming commit `8c8fba3ae2e84c7493d56b74d0ccc8d8f26920ed`.
The repair remains uncommitted; no push or deployment is selected.

## Problem and selected contract

The [naming audit](NAMING-AUDIT-20261022.md) retained an actual
`generator_selected` exit 101: plain service generation succeeded, but a service
using runtime settings failed with `unsupported config crate layout`. At that
checkpoint, `yetlet-config/src/secrets.rs` already existed while the export
guard required `src` to contain only `lib.rs`. Legacy runtime-settings layouts
used the same guard. This was an existing source-closure incompatibility, not
a naming regression. The original failed output and historical generator PASS
remain source-bound evidence.

The owner selected an independent, bounded repair. Preserve the existing
exact-source allowlists and extend the selected configuration closure to these
six files only:

```text
crates/yetlet-config/Cargo.toml
crates/yetlet-config/src/lib.rs
crates/yetlet-config/src/secrets.rs
crates/yetlet-config/examples/secret_provider.rs
crates/yetlet-secrets/Cargo.toml
crates/yetlet-secrets/src/lib.rs
```

Cargo must resolve the optional `yetlet-secrets` path dependency's manifest even
when the `secret-provider` feature is disabled. Including its reviewed source
does not enable that feature, instantiate a provider or copy application secret
values. The `secrets.rs`/`yetlet-secrets` names here identify public implementation
source, not credentials, local secret files or provider configuration.

The first five-file implementation passed metadata resolution and 16 Docker unit
tests, but an actual sanitized-export `cargo check` exited 101: Cargo required the
explicitly declared `secret_provider` example file even while that feature was
disabled. The smallest product correction includes that one exact approved
example, without exporting other examples or editing the manifest. The original
failure remains in `implementation-selected-tests.*`. Metadata success alone
is not treated as compilation proof.

Keep byte/normalized-source comparisons, directory restrictions, protected path
and symlink checks, consistent explicit locks and inventory hash/size declarations.
Do not accept arbitrary extra Rust modules, modified dependencies, build scripts,
private application inputs or unrestricted directory export. No optional feature,
dependency version, CLI syntax, settings schema, Core API or default runtime graph
change is selected. Operators still explicitly prepare sanitized contexts from
the declared inventory; generators do not execute Cargo, npm, Docker or kubectl.

## Verification boundary

Use existing focused product checks for the four current service profiles and
Docker/Compose/Kubernetes outputs, plus exact closure inventory, tamper/layout/link
refusal and representative offline Cargo resolution/compilation from exported
sources. Keep fake structural Vite assets distinct from actual frontend builds.
Retain original command output, exit codes and any failed attempt in the
repo-external `yetlet-generator-source-closure-20261022-w1c3hrbw` evidence directory.

This repair does not rerun or transfer historical browser/CSP, native Windows,
OCI image, Kubernetes/Valkey/IdP, Auth or release qualification. Registry-only
delivery, arbitrary app layouts and publication remain outside the source
selection. The [current source matrix](../tools/yetlet-cli/README.md#current-generator-source-matrix)
must reflect the actual focused result before acceptance. The
[2026-10-18 verification](YT-102-GENERATOR-COMPATIBILITY.md) keeps its original
source identity, PASS, failures and operational limits.

## Results and independent acceptance

The initial five-file build failure above consumed one in-place product repair.
Two earlier temporary probe setup misses (an incomplete baseline, then a temporary
manifest parsing typo) and an edit-script assertion correction are retained in
the evidence directory. They are not passing product evidence; no manifest,
dependency or feature was changed to bypass the source-closure defect.

| Focused command/check | Observed result |
|---|---|
| Initial CLI Docker unit tests | 16 passed for the initial five-file attempt; the later exported build still failed as recorded above |
| Corrected `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test generator_selected -- --nocapture` | PASS: four current profiles and all 12 Docker/Compose/Kubernetes generations; 14 framework-context entries for plain service, 20 for runtime settings |
| Sanitized Vite-source export inside that integration | Exact inventory sizes/SHA-256 and physical allowlists checked; only the overlay's fixed container mount paths relocated to temporary native paths. `cargo metadata --offline --locked --format-version 1` and `cargo check --offline --locked --bin selected-gen-2` both exited 0; lock unchanged, `secret-provider` feature and normal `yetlet-secrets` dependency inactive |
| Corrected CLI unit filter `delivery_config_secrets_closure` | 2 passed: 12 missing/tampered-file cases, six extra-module/build-script/config cases and six Unix symlink cases rejected |
| Corrected unit `delivery_profile_matrix_exact_inventory_env_and_no_secrets_or_developer_paths` | 1 passed: retained legacy delivery fixture, corrected inventory/bytes/hashes and private-input exclusion |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | PASS for the corrected six-file implementation |
| CLI `cargo fmt -- --check` and scoped `git diff --check` | PASS |

Exact commands, stdout/stderr and exits are retained as `implementation-*` and
`legacy-positive-final.*`. The integration uses finite structural Vite assets;
the successful Rust compilation is real, but no actual npm/OCI/browser/deployment
build is claimed. All command sessions completed and owned temporary export
trees were removed. No server, watcher or external service was started.

Actual changed files:

| Area | Files |
|---|---|
| Source closure | [delivery.rs](../tools/yetlet-cli/src/docker/delivery.rs) |
| Focused product tests | [Docker tests](../tools/yetlet-cli/src/docker/tests.rs), [selected generator integration](../tools/yetlet-cli/tests/generator_selected.rs) |
| Current guidance | [root README](../README.md), [CLI guide](../tools/yetlet-cli/README.md), [documentation index](README.md) |
| Evidence/history | This record and the dated note in the [naming audit](NAMING-AUDIT-20261022.md); original generator and naming results retained |

Independent Reviewer `/root/generator_source_closure_review_20261022` PASSed
with zero issues and required followups. All eight reviewed path hashes and
HEAD/status remained unchanged during review. After PASS, only this record
is updated with the acceptance result. `cargo xtask check docs` PASSed
(75 required documents, 126 backlog items); direct changed-file links, English,
fences, EOF/whitespace and source continuity checks PASS. Final evidence is
retained as `closure-checks.json`, with the original review in
`independent-review.json`. No implementation check is rerun for this bookkeeping.

YT-020 remains Blocked and YT-096 remains In progress; no release candidate or
other backlog status is expanded by this repair. The next independent M21 study
remains YT-118. All eight repair paths remain uncommitted with staged files zero;
this repair adds no commit or push.
