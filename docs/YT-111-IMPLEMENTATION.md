# YT-111 — Project-aware CLI inspection

Date: 2026-10-22. Implementation base:
`7b2b361f6f9fac6a28a27a602405312daff526a9`, current checkout
`feat/yt111-112-dx-roadmap`. Status: **Done** at the selected read-only inspection scope.
This bounded owner-selected task implements YT-111 only. YT-112 remains Todo.

## Selected contract and implementation

The [registration](YT-111-112-DX-SCOPE.md) required a design comparison before
implementation. The separate
[`inspect` contract](CLI-CONTRACT.md#read-only-project-inspection--yt-111) was
documented before source edits. Extending environment-only doctor would couple
project loading/errors to tool probes and change its schema scope. Instead,
`yetlet inspect [--manifest-path PATH] [--json]` has its own explicitly scoped
project-execution-plan schema 1; doctor source and schema 1 remain unchanged.

The implementation reuses protected standalone Cargo.toml/Cargo.lock selection
and the existing public Loader. A small internal return-value extension exposes
the already validated finite profile without a second selector. Existing
settings callers discard that metadata and retain their validation ordering.
The new command explains build/run/watch intentions, native/browser target,
configured Rust watch, Vite enabled/relative root, auto_build/auto_start and
fixed selection/skipping reasons. Human relative roots use character escapes;
JSON uses ASCII/UTF-16 escapes, preserving the decoded relative string.

Only allowlisted settings are rendered. Inspection never invokes process code,
reads frontend directories/assets or TLS key files, enables Development secrets,
calls SecretProvider, writes output/settings or uses the network. Exit 0 means
an explanation was produced, including unsupported/skipped stages. It is not
tool availability, asset validation, runtime startup or readiness evidence.
Existing manifest and settings failures remain exit 1/stderr with empty stdout;
source/profile/values grouping and ordering are retained. Usage stays exit 2.

## Changed files

- `tools/yetlet-cli/src/inspect.rs`: narrow plan construction and safe rendering.
- `tools/yetlet-cli/src/lib.rs`, `src/main.rs`: grammar, help and dispatch.
- `tools/yetlet-cli/src/settings.rs`: return the existing profile for inspection.
- `tools/yetlet-cli/tests/commands.rs`: real CLI cases and doctor shape assertions.
- `docs/CLI-CONTRACT.md`, `tools/yetlet-cli/README.md`: selected public contract.
- `docs/BACKLOG.md`, `docs/ROADMAP.md`, `docs/YT-111-112-DX-SCOPE.md`, this record:
  truthful current state, retaining planning provenance and accepted evidence.

No dependencies, locks, generators, native execution/lifecycle adapters, browser
assets or release sources changed.

## Local verification

Raw logs and progress notes are retained in
`/tmp/yetlet-yt111112-20261022-xd7g59vt`. Commands select the already prepared
Rust/Cargo and Node 24.21.0 installations through process-local PATH entries.
No shell startup/global configuration was changed. The existing OpenSSL prefix
under the signing crate's ignored build cache was reused with `OPENSSL_DIR`
and `OPENSSL_STATIC=1` for CLI compilation/tests/Clippy. Exact host-local paths
remain in the raw logs rather than tracked documentation; no installation or
upgrade was performed.

All required checks passed with exit 0:

| Command | Result | Raw log |
|---|---|---|
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test commands inspect -- --nocapture` | 3 passed | `yt111-inspect-tests.log` |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test commands` | 7 passed, including real native Cargo/cleanup, discovery and doctor | `yt111-commands-tests.log` |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib settings::tests` | 8 passed | `yt111-settings-tests.log` |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib doctor::tests` | 1 passed | `yt111-doctor-tests.log` |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib --bin yetlet --test commands -- -D warnings` | Passed | `yt111-clippy.log` |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --check` | Passed | `yt111-fmt.log` |
| `cargo xtask check docs` | Passed: 75 documents, 112 backlog items, links/structure/whitespace/DAG | `yt111-docs.log` |
| `git diff --check` | Passed | `yt111-diff-check.log` |

Before independent review, no product/harness check failed and no repair cycle
was used. One initial documentation patch had a context mismatch; the corrected tool invocation
succeeded before source edits (one operational retry). No capability recovery
or validation infrastructure was added.

The inspection tests use existing command-fixture patterns and protected private
temporary roots. Cases cover native defaults, browser static build/preview,
unsupported browser watch, all three profiles, explicit public root, ENV
precedence, watch override/build skipping, invalid sources/profile/values,
failure ordering, token/parser/path redaction, malicious terminal characters
and Unicode round-trip. Tool tripwires and unchanged file/entry assertions
verify no child/output-tree effects. No frontend/source tree is required for
inspection. Invalid secret-file content and backend token/password sentinels
are ignored. Existing command regressions exercise real native frozen Cargo,
lock/cache failures and owned-descendant cleanup separately from inspection.

## Review correction

The first independent review (`wc_job_G7XgP6BIxBO-8qL8`) found one P2 defect:
JSON serialization passed ASCII DEL through the printable-ASCII branch. The
smallest correction emits it as `\\u007f`; the existing real CLI escaping test
now includes DEL and verifies no raw control byte plus unchanged decoded value.
The focused test, targeted strict Clippy and formatting pass. This is one
material product repair, not a new validation framework. The original failed
review and correction receipts are retained with the task evidence.

## Independent closeout

Reviewer follow-up role-directed Codex Job `wc_job_42fZ6Oced30fFBOS` completed
exit 0 with PASS, zero issues and zero required followups. Runtime metadata
matches the saved reviewer model and selected effort; reviewed source hashes
were unchanged during the read-only review. This is independent review, not
self-review or native managed-role identity. The initial failed review is
retained, not overwritten. Final status metadata is recorded after acceptance;
no commit or push is included.

## Limits, cleanup and next work

No browser or CSP runtime verification was needed or run: this task changes
read-only CLI reporting only. Linux native evidence does not claim new Windows
runtime qualification; no platform adapter changed. Public release and
production qualification remain open, and old accepted evidence keeps its
recorded scope. There are no remaining contract decisions for this slice;
independent Reviewer follow-up PASS closes the selected YT-111 scope.

All verification uses bounded commands. No server, browser, watcher or background
service was started. The existing native command test verifies owned-descendant
cleanup; temporary fixture directories are owned by TempDir and cleaned on test
completion. Build caches and raw evidence are retained. No stage/commit/push,
branch switch, global installation, external write or release action occurred.

The next independently unblockable backlog item is YT-112, whose browser
watch/lifecycle design gate remains separate; browser watch is still unsupported.
