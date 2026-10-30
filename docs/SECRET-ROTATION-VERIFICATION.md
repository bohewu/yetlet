# SecretProvider rotation, Windows and Azure verification

Date: 2026-10-18 (Asia/Taipei). Task: separately authorized YT-090 follow-up.
Status: local, native Windows and actual Azure checks passed;
[independent review](SECRET-ROTATION-REVIEW.md) PASS, zero issues or required
followups. Subsequent local commit is owner-authorized; no push occurred.

The owner expanded the [initial read contract](SECRET-PROVIDERS.md) to try
rotation and Windows/cloud verification. The implementation remains an opt-in,
unpublished native crate. The previous read-only review is historical evidence;
this follow-up receives a separate independent review.

## Explicit local adoption

`SecretProvider::get` remains read-only. `SettingsSecrets::replace(values)`
accepts the application's newly selected `BTreeMap<String, SecretString>` and
replaces the complete snapshot through the same shared handle. The application
first updates its secret source, calls its existing Loader again and selects
the fields to provide. There is no second parser, filesystem writer, random
credential generator or implicit action during lookup.

The provider holds a native `RwLock` only for cloning or replacing the map,
without I/O or an await while locked. A poisoned lock returns fixed
`SecretError::Unavailable`. Each lookup observes a complete snapshot; separate
lookups are not an atomic multi-secret transaction. The caller supplies all
names to retain because replacement replaces the whole map.

The same `Arc<dyn SecretProvider>` injected into an existing service observes
new values on later lookups. Previously returned protected strings and clients
that copied credentials retain their old values. External credential systems,
client reconnection and coordinated adoption remain application/operator work.
This is explicit local adoption, separate from automatically changing a database
password, scheduling rotation or restarting consumers.

## Azure experiment boundary

The [read-only Azure CLI example](../crates/yetlet/examples/secrets/azure_cli.rs)
implements the same trait with a current-version or pinned-version lookup.
It uses a separately installed/authenticated official Azure CLI through literal
argv, captures JSON without trimming secret contents and discards backend error
details. Recognized missing/access-denied responses map to the existing
contract; unknown failures, malformed output and a 30-second timeout become
Unavailable. Process drop kills the owned CLI child.

The example is an explicit experiment, with three parser/error tests that make
no cloud call. The general library still depends only on secrecy/zeroize;
Tokio process/time and serde_json are example/test dependencies. The two stdin
checkpoints permit an operator to create v2 and disable v1 between actual reads.
The example performs no cloud writes and prints only fixed status markers.

The live plan uses one new Standard vault and resource group in the previously
approved subscription/East Asia scope, with synthetic values only. A private
temporary file supplies each value to `az keyvault secret set --file`; no value
appears in argv or retained command output. It checks fresh latest-version reads,
a retained old value, a missing name and denied reads of a disabled old version.
Only get/set/list/delete secret permissions are assigned inside the disposable
vault; no external credential system or global role assignment is changed.
The dedicated resource group is deleted in a finally block and absence is
verified. Soft-deleted retention is recorded; no purge is performed.

Official operation semantics:

- [Azure CLI secret commands](https://learn.microsoft.com/en-us/cli/azure/keyvault/secret?view=azure-cli-latest).
- [Key Vault secret rotation and consumer updates](https://learn.microsoft.com/en-us/azure/key-vault/secrets/tutorial-rotation).

## Local results

All commands exited 0:

```text
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --doc
cargo clippy --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path crates/yetlet-secrets/Cargo.toml --check
cargo build --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --example azure_cli
```

The final combined suite passed 13 tests: one lock-poison unit test, four
provider tests, five State/Loader tests and three Azure parser tests. One
rustdoc test passed. Local rotation writes v2 to an owned synthetic Development
secret file, reloads through the same Loader and replaces the same provider
used by an existing service. A previously returned v1 value remains unchanged.
The suite also covers replacement/removal of names, safe errors, Send behavior,
empty/missing distinctions, SSR/CSP, current profile rules and fixed Debug output.
Temporary fixture directories were removed; no listener/browser was started.

Logs, native receipts, cloud operations and preservation evidence are retained
under `artifacts/secret-rotation-20261018/`. The Windows package contains 208
selected current source files; its inventory is `source-hashes.json`. Packaging
is source transfer for direct native tests, separate from a release artifact.

## Scope limits

Production/cloud SDK
support, automatic scheduled rotation and browser secret handling are outside
this bounded experiment. YT-094 remains a separate next core discussion.

## Native Windows result

The [native Windows record](SECRET-PROVIDER-WINDOWS-VERIFICATION.md) contains
the exact commands, tool identities, failures, results and cleanup references.

WebCodex ran the real MSVC Cargo 1.99 toolchain in the isolated protected
`windows-secret-rotation-20261018` directory on project
`agent:ncut-bohe-244143825ae3488e:localtasks`, session
`wc_sess_v49VHB4rF9p5ckzy`. Job `wc_job_dg3H7A3qqxAhZevz` completed with exit 0.
All-target tests passed the same 13 cases, including actual Loader file
rotation and same-handle service injection; one rustdoc test also passed.
Strict Clippy and formatting exited 0. The configured State example exited 0
and reported `Example request status: 200 OK`.

The first offline attempt stopped before compilation because the dedicated
cache lacked `signal-hook-registry`, newly required by the Azure example's
Tokio process feature. A task-authorized `cargo fetch --locked --target
x86_64-pc-windows-msvc` filled the locked dependency cache; the final commands
remained `--offline --locked`. The original exit 101 and the fetch receipt are
preserved; no product failure or source/lock modification was demonstrated.

All 208 transferred build-input files matched before and after testing. The
prior 196/197-file task sources and Cargo config/credentials remained unchanged.
No task process or temporary fixture remained. No listener/browser was started,
no cloud command ran on Windows and no global tool, trust or ACL was changed.
Primary commands and raw results are in `windows-native-logs/`, including
`native-results.json`, `all-targets.stdout.log`, `doc.stdout.log` and `cleanup.json`.

The crate README received only a documentation link/status clarification after
the package was tested. The other 207 packaged files, including all runtime
sources, tests, manifests and locks, still match the current checkout. Both
README hashes are retained in `post-verification-documentation.json`; a
documentation-only update does not claim a new native runtime test.

The Windows artifact export gateway could not read back its export URI. Public
UTF-8 log contents were instead transferred in bounded text segments and checked
against native hashes. This transport issue did not rerun or alter the native
test. Native Windows execution and the Linux-hosted Azure experiment are
separate observations, not an Azure-on-Windows authentication qualification.
The WebCodex Work Result summary retains a mixed/current-failed aggregate from
the initial cache-miss observation, while the final native Job is completed/
passed. This historical aggregate is not relabeled or repaired; the exact final
command exits, test counts and independent review establish the scoped result.

## Actual Azure result and cleanup

The real Rust example completed all three markers and exited 0. It read the
initial latest and pinned version through `Arc<dyn SecretProvider>`, observed
the newly created v2 through the same latest handle, rejected a pinned disabled
v1 with AccessDenied and still read v2. The retained owned v1 string remained
unchanged. A missing remote name returned None. The live experiment never
printed a secret value or backend diagnostic.

All 14 recorded lifecycle Azure CLI commands exited 0, in addition to the
example's six read operations. Version metadata confirmed v1 disabled and v2
enabled. The dedicated resource group no longer exists and the active vault
list is empty. The soft-deleted vault remains scheduled for purge on
2026-10-25; no purge was performed. The default subscription was unchanged.
The private temporary secret files were removed and the owned Rust process
exited. Exact resource/version IDs, safe command metadata, observations and
cleanup are in `azure-plan.json`, `azure-result.json` and `azure-live.log` under
the evidence directory.

The resource was one Standard vault in East Asia, dedicated to this experiment.
The owner had requested cloud verification and previously authorized this
same-session independent test-vault/create/delete scope. No existing vault,
application credential target, global role or subscription selection changed.

## Minimal local contract follow-up

Date: 2026-10-18 (Asia/Taipei). The owner subsequently selected documentation
and necessary tests for the minimal local contract, explicitly without a source
revision. This is a separate follow-up to the historical Windows/Azure checks
above; it does not claim another Windows or cloud execution.

The [guide](SECRET-PROVIDERS.md), crate README and `replace` rustdoc now state
that publication does not prove source freshness. All reload writers must use
one application-owned entry that acquires its gate before loading, then validates,
selects the complete map and replaces it. Load/validation errors skip replacement;
the healthy snapshot remains usable. Equal values may be replaced, restart loads
current inputs, and no source revision, snapshot counter or persistent floor is
added. External rollback, cross-process coordination, multi-file transactions,
client reconfiguration and external credential acceptance remain separate.

The State example's test-private synchronous helper demonstrates that entry.
The executable's once-at-startup behavior, settings schema, read trait, provider
implementation, dependencies and manifests/locks are unchanged. Two focused
tests close the previously uncovered application-pattern behavior:

- Competing reloads: A reads old source while holding the gate; B observes
  contention. After an external file update, A publishes first, then B observes
  A's publication and freshly loads/publishes the new source. Explicit channels
  control order, with bounded five-second receive timeouts and no sleeps.
- Failed reload: malformed TOML and the test application's empty-required-token
  validation each retain the same service's old snapshot. A subsequent valid
  reload succeeds; an already returned protected value remains unchanged.

Fresh local commands all exited 0:

```text
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --doc
cargo clippy --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path crates/yetlet-secrets/Cargo.toml --check
```

All-target tests passed 15 cases: one lock-poison unit test, four provider tests,
three Azure parser tests and seven State/Loader tests. One rustdoc test passed.
The documentation gate and independent review are recorded in the
[review record](SECRET-ROTATION-REVIEW.md#minimal-local-contract-follow-up).

The guide also records supported local inputs and the recommended separation of
business application reads/signing from operator or dedicated automation
creation/rotation. Azure source versions, generating a credential, publishing
it and updating its target are distinct. Signing and TLS certificate renewal
retain their existing contracts. No Azure write, live cloud test, new CLI command,
scheduler, registry, listener or browser was introduced by this follow-up.

Changed files for this follow-up are `crates/yetlet-secrets/examples/state.rs`,
`crates/yetlet-secrets/src/lib.rs` (rustdoc only), the crate README,
`docs/SECRET-PROVIDERS.md`, `docs/API-CONTRACTS.md`, this verification record and
the review record. Existing unrelated work and previous validation records are
preserved. No staging, commit or push is authorized by this follow-up. YT-094
remains the next separate core discussion.
