# YT-093 — Signing and local key CLI verification

Date: 2026-10-18 (Asia/Taipei). Status: PASS at the bounded signing/key-administration scope.
Base checkout HEAD: `c726af4a4f2f3cf3cf6225de0c793541b564249d` plus the preserved
dirty working tree. No commit or push is included.

## Authorized scope

The owner selected the [replaceable signing contract](YT-093-SIGNING-CONTRACT.md),
established cryptographic libraries, first-version CLI management, automatic
settings updates after rotation approval, and multiple keys/certificates. The
owner separately authorized a disposable Standard/software EC P-256 Azure lab
in the default Visual Studio Professional subscription/East Asia, with deletion
after testing. No cloud adapter, SDK, production credential or release signing
is included.

Implementation paths are the new nonpublishable nested-workspace
`crates/yetlet-signing`, standalone `examples/signing`, and the existing
`tools/yetlet-cli` key command/native filesystem seams. Core/browser/default
framework packages are unchanged. Existing YT-092 preview/data and unrelated
YT-103 work are preserved.

## Native cryptographic library

The local host provides a runtime OpenSSL binary but no discoverable development
headers. Qualification therefore uses an isolated static OpenSSL 3.5.9 LTS build,
without changing global libraries or package installations:

```sh
OPENSSL_DIR=/tmp/yt093-native-tools/openssl-install
OPENSSL_STATIC=1
```

The official source archive SHA-256 is
`603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a`.
The maintained Rust binding is `openssl 0.10.81` / `openssl-sys 0.9.117`.
No library source patch or custom cryptographic/ASN.1 algorithm was introduced.
An official distro make package was extracted only into task-local temporary
storage for this native build. Initial inspection against the installed runtime
and an unqualified vendored dependency was not relabeled as final evidence.

PFX requires a present password-verified MAC through OpenSSL; this does not assert
every imported archive bag was encrypted. The reference's generated private PEM
is encrypted, private files/directories are protected, and no secret is written
to public TOML or argv.

## Linux verification

With the native library environment above:

| Command / observation | Result |
|---|---|
| `cargo test --locked --manifest-path crates/yetlet-signing/Cargo.toml` | PASS, 11 tests |
| Core `cargo fmt -- --check` and `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| CLI full `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | PASS, 80 library tests, 7 initial key integration tests and existing default integration suites; existing opt-in ignores retained |
| Focused CLI key integration after public-parent and comment-preservation repairs | PASS, 9 tests |
| Focused key units, CLI fmt and strict all-target Clippy | PASS |
| Example strict all-target Clippy | PASS |
| Actual `yetlet build` / `yetlet run` in `examples/signing` | PASS, native exit 0 each |

The console application demonstrated encrypted PEM signing, PFX signing,
tamper rejection, documents v1-to-v2 rotation, independent invoices signing,
public-only verification and revoked-v1 rejection. It generates disposable
in-memory identities; persisted CLI management is a separate journey.

The actual persistent CLI journey also passed: terminal `yes` confirmation
initialized documents v1, explicit invoices initialization left documents intact,
and documents rotation selected v2 while invoices stayed at v1. Status succeeded
without a password. Base settings remained byte-identical, the old version was
retired, and generated directories/files had 0700/0600 permissions. The owned
fixture and encrypted private material were removed afterward. See
`manual-cli-results.json`, `manual-interactive-rotation.log` and public before/after
profile snapshots under the evidence directory.

Root command logs are retained under `artifacts/yt093-signing-20261018`.
The initial sample-bootstrap failure with an ordinary 0755 named-key parent is
retained as `sample-bootstrap-initial.log`. The smallest repair permits an owned,
non-group/world-writable public parent, while generated immutable versions remain
0700 and private files 0600. A 0775 parent remains rejected. A separate initial
sample build failure caused by the task-created directory's group-write mode was
resolved by preparing the new sample with the CLI's existing ownership policy.
Independent review then found loss of comments attached to the selected active
version and historical array entries. The repair retains original TOML values
and decorations while changing state/appending the new record. One parameterized
regression covers inline arrays and arrays of tables in both the selected profile
and inherited base, including strict-loader reload. The reviewer closed this
source finding after checking the exact repaired file hashes.

## Native Windows

Source-matched execution through WebCodex used an isolated source bundle that
contains 196 files and has SHA-256
`643e0a90c82355d49874c0320a601d344b47507916f7279d282d32043fe5833a`;
`windows-source-manifest.json` records every file digest. This is independent of
the old Windows checkout's HEAD.

The later two-file comment-preservation delta has SHA-256
`6f236a283c71c493b5afff9ccbc5388e3371dda8736caa83647890f963d5098c`.
`windows-final-source-manifest.json` binds all 196 files after that explicit
overlay. The native worker verified the original bundle, the two permitted delta
paths and the final complete inventory before testing the CLI.

Existing Rust 1.99 MSVC/Visual C++/Windows SDK are reused. Native portable Perl
and official OpenSSL source are prepared only inside a new task-local directory.
Compilation or prerequisite detection alone is not native lifecycle evidence.

Observed native results:

| Execution | Result |
|---|---|
| Core `--test signing` | PASS, 11/11 |
| CLI `--lib` | PASS, 72/72 |
| CLI `--test key` | PASS, 7/7 |
| Native Cargo build/run of the console sample | PASS, six demonstrated outcomes |
| Actual key CLI with noninteractive `--yes` | PASS, documents v1, invoices v1, documents v2 and both status commands; five unselected files preserved |

The [native report](../artifacts/yt093-signing-20261018/windows-yt093-native-report.md),
structured result, cleanup observation and fourteen exported logs are retained
under the common evidence directory. All 196 source hashes still matched after
testing. Cleanup exited 0, removed the complete private fixture and temporary
tool/cache/build directories, and found zero matching task processes. Only public
source/baseline snapshots, source archives, logs and reports remain remotely.

Platform-specific test counts differ from Linux; no filter was used to hide an
executed failure. An initial core `--lib` command selected no tests and is retained
as compilation-only evidence; the actual integration command then executed all
11 signing tests. An extra wrapper assertion incorrectly expected the persistent
lock file to disappear. Its failure is retained; a focused exclusive-open check
confirmed the lock handle was released without repeating successful rotations.

The existing `yetlet build` Cargo-supervision path exited 125 (`CLI_SPAWN`,
`Other`) on this Runner. The exact pre-change CLI from HEAD
`c726af4a4f2f3cf3cf6225de0c793541b564249d`, built in the same environment,
produced the same failure against the same sample. Baseline bundle SHA-256:
`d073c5bc39f9eb77d049fdcf66be6a58caab61442081e55fc652131dc1e8a493`.
The underlying cause was unproven at this checkpoint; nested Job incompatibility
was not established.
Actual Windows key administration and native crypto/sample execution passed,
while this Runner's pre-existing CLI build/run supervision remains unqualified.
Linux's actual Yetlet CLI sample build/run evidence is separate. This task does
not repair or relabel the older supervision qualification.

A separately authorized [Windows CLI follow-up](WINDOWS-CLI-125-VERIFICATION.md)
subsequently identified untrusted write access on the selected Cargo executable
as the refusal before child creation. It adds safe executable-stage diagnostics
and records native build/run/cancellation with a protected task-local Cargo path.
That follow-up has its own source-bound evidence and review; the original failed
logs above remain unchanged.

## Azure experiment

Azure CLI 2.91.0 read-only account/vault inventory calls passed. Neither visible
subscription had an accessible existing vault. The owner then explicitly approved
an isolated lab, bounded synthetic signing/verification/rotation, and deletion.
The [retained experiment report](../artifacts/yt093-signing-20261018/azure-report.md)
and `azure-result.json` record a bounded PASS:

- Independent documents/invoices software EC P-256 keys signed synthetic data.
- Documents rotation produced a distinct version and public key; invoices was
  unchanged. Old signatures verified with v1 and failed with v2; new v2 signatures
  verified. Expected invalidity was `isValid: false` with native exit 0.
- The existing Rust public-only verifier independently verified three Azure
  signatures and rejected tampering, wrong-version routing and local revocation.
- The successful retry recorded 29 Azure commands with native exit 0 each.

The first CLI response-projection failure and cleanup remain under
`azure-attempt1`; one canonical scratch correction/retry used a different unique
name because the original vault was soft-deleted. A separate initial scratch
Rust compilation error and its smallest correction are also retained. Neither
changed product source or was relabeled as successful evidence.

For both attempts, synthetic keys and the own-vault access policy were removed,
vault and dedicated resource group were deleted, and active key/vault/group
absence was verified. The original subscription default is unchanged. Two
recoverable soft-deleted vault records remain until their scheduled retention
expiry on 2026-10-25; no purge was performed. Their exact IDs/dates are in the
structured result and report. No active HSM, VM, rotation schedule or cloud SDK
was introduced. The experiment did not measure billing/credit consumption.

Azure CLI verification may run locally through its SDK; it does not prove every
verify call used remote REST verification. Remote private-key signing plus the
independent Rust public-key interoperability are the demonstrated boundaries.
This experiment does not implement an Azure provider adapter.

## Review and boundaries

Final independent implementation review: **PASS**, zero issues and required
followups. The reviewer checked the comment-preservation repair, all 196 final
Windows source hashes, the fourteen exported log hashes, native executions,
Azure interoperability and both platforms' cleanup evidence. The Windows
Cargo-supervision limitation above remains explicitly unqualified by this PASS.
The source review's comment-preservation finding is closed. The earlier contract review found
and closed the duplicate-public-key/per-version revocation substitution problem;
that design result is not implementation acceptance.

Settings updates affect the next settings load; an application must be restarted
to adopt them. File watching, distributed policy rollout, cloud adapters, trusted
timestamps/historical certificate validation, chain building, production custody,
browser/HTTP signing UI and release activation remain outside this qualification.

The authorized commit preparation has a separately recorded
[formatting and Guard boundary](WINDOWS-CLI-125-VERIFICATION.md#commit-preparation-boundary):
eight sample files lose trailing blank lines and the unchanged public PEM label
uses compile-time concatenation. Eleven local signing tests passed again; the
retained native logs keep their original source identities.
