# CI and Security Review — Post-M9 Prototype

**Date:** 2026-10-10
**Scope:** CI reliability, dependency security, repository controls and next qualification work after the M9 evidence freeze.
**Decision:** The current source remains an experimental private prototype. The changes in this review improve CI and close one concrete RustSec advisory; they do not authorize public package publication or production deployment.

## 1. Remote CI failure diagnosis

GitHub Actions run `38012171376` executed commit `292b1524ff2ed0e223aeb2575e5b744cda14dfc5`. Documentation, format, strict Clippy, native tests, doctests, wasm32 compilation, browser asset generation and the native server build passed. The only failed step was the Chrome Browser E2E smoke.

The failure had two harness-level causes:

1. immediately after `Page.navigate`, the readiness predicate dereferenced `document.documentElement` before the new document had created it, producing a transient JavaScript `TypeError` instead of returning false and continuing to poll;
2. the failure path terminated the Chrome parent but did not own and reap the complete helper-process group before `TemporaryDirectory` removed the profile, producing a secondary `Directory not empty` cleanup error that obscured the first failure.

The bounded repair is restricted to the test harness and CI configuration:

- navigation readiness checks use null-safe document access;
- Chrome runs in an owned POSIX process group and bounded cleanup terminates, kills if necessary and reaps that group;
- profile removal retries transient late writes;
- failure logs are copied to `target/ci-browser-logs/` for artifact upload;
- the smoke accepts an explicit debug/release profile, and CI now builds and tests matching release artifacts;
- focused standard-library tests cover profile selection, process-group cleanup and profile-tree retry behavior.

No browser runtime, renderer, server endpoint or application behavior changed for this repair. Two local Chrome attempts after a host reboot reached a distinct CDP `Page.navigate` response timeout. Server and Chrome startup were healthy, cleanup left no owned process, and the same operational signature repeated twice, so the local browser lane was stopped rather than repeatedly changing product code. The pushed GitHub Actions run is the required external validation of the original hosted-run failure.

## 2. CI coverage after the repair

The checked-in workflow uses read-only repository permissions, cancels superseded branch runs and pins external actions to full commit SHAs. It separates a Rust 1.88 MSRV job from the current-stable native/browser job.

| Boundary | Per-push/PR evidence | Remaining qualification gap |
|---|---|---|
| Documentation/catalog | Documentation DAG, sample catalog and focused Python harness tests | None for repository consistency |
| Native Rust | Format, strict Clippy, workspace tests and doctests | Other OS/architecture combinations |
| MSRV | Rust 1.88 native tests/doctests and eight wasm32 package checks | Upgrade policy and future dependency drift |
| Dependencies | `cargo-audit 0.22.2` against `Cargo.lock` | License/source policy, vendoring and continuous advisory services |
| Native samples | All bounded native application smokes | Load, soak and production topology tests |
| Browser/WASM | Eight wasm32 packages; Todo/Search, keyed lifecycle and multi-island Chrome release suites | Firefox is retained locally but not yet hosted/scheduled; no WebKit/mobile/IME matrix |
| Artifacts | Release browser build, digest checks in repository tests and retained baseline artifact | No release SBOM, provenance, signing or public promotion pipeline |

This workflow is intentionally a CI baseline, not a release pipeline. Renderer benchmarks remain informational and are not pass/fail performance thresholds.

## 3. Dependency and supply-chain finding

A new RustSec scan found `RUSTSEC-2026-0009` in transitive `time 0.3.45`, reached through `cookie 0.18.2` and the opt-in Session crate. The advisory covers denial of service by stack exhaustion in RFC 2822 parsing. The patched line begins at `time 0.3.47`.

`time 0.3.47` requires Rust 1.88, while the workspace previously declared an unverified Rust 1.85 baseline. The security repair therefore:

- updates the lockfile to `time 0.3.47`;
- raises the declared workspace MSRV to Rust 1.88;
- validates the complete native workspace, 190 tests and five doctests with Rust 1.88;
- validates the patched lockfile with `cargo-audit`, returning zero vulnerabilities and zero warnings;
- adds Rust 1.88 and RustSec checks to hosted CI.

This is an intentional MSRV change for a concrete security patch, not an accidental dependency drift. Future MSRV increases require the same explicit explanation and CI evidence.

Other supply-chain controls in this round:

- GitHub Actions are pinned to full commit SHAs;
- workflow permissions are `contents: read`, and checkout credentials are not persisted;
- tool installs use explicit versions and Cargo locked resolution;
- tracked-source secret-marker inspection found no obvious token/private-key pattern.

Open supply-chain gaps remain: no `cargo-deny` license/source allowlist, no SBOM or build provenance, no artifact signing, no secret-scanning/code-scanning service, and no dependency-update automation.

## 4. Demonstrated security controls

The repository has meaningful prototype security evidence:

- strict CSP defaults and context-aware HTML escaping;
- no inline event-handler or `eval` authoring surface;
- typed request extraction, explicit body/media limits and fail-closed validation;
- bounded opaque Sessions with secure-cookie defaults, HttpOnly, SameSite, expiry/capacity, CSRF tokens, login/logout rotation and stale-request protections;
- synthetic Auth Notes tests for fixation, old-cookie/old-CSRF rejection, IDOR and per-user authorization;
- parameterized SQLite, explicit transactions, rollback, optimistic versions and fail-closed schema checks;
- file path/symlink defenses, private temporary storage, upload limits, cancellation cleanup and protected downloads;
- bounded SSE replay/subscriber queues and disconnect cleanup;
- browser/server and optional-provider dependency isolation;
- reviewer-discovered defects repaired before each M9 stage closed.

These controls apply to the bounded samples and their stated topology. They are not a production security certification.

## 5. Current security and repository gaps

Observed GitHub repository settings on 2026-10-10:

- repository visibility is Private;
- Actions are enabled and default workflow token permission is read-only;
- all third-party Actions are allowed and repository-wide SHA pinning is not enforced by settings;
- `master` has no branch protection;
- Dependabot alerts are disabled;
- secret scanning is disabled;
- code scanning is not enabled;
- web commit sign-off is not required.

Product/operations gaps:

- Auth Notes uses terminal-only synthetic identities, not a production IdP, passkey, password, MFA or recovery flow;
- no brute-force/rate-limit policy, trusted TLS reverse-proxy/header contract or deployment threat model exists;
- Session state is in-memory and single-process; no distributed persistence, revocation or multi-node semantics are qualified;
- SQLite and temporary-file samples do not define backup, encryption-at-rest, retention, restore or multi-process operations;
- no production secrets/key-management integration exists;
- no complete screen-reader, IME, WebKit, mobile or multi-OS test matrix exists;
- no fuzzing/property program covers cookie/form/path/HTML/SSE parsers;
- no external penetration test, formal security audit or incident-response policy has been performed;
- repository/crate licensing, public SemVer support and release ownership remain undecided.

## 6. Prioritized next qualification work

1. **Maintainer decision — licensing and vulnerability policy.** Select repository/crate licenses and a supported vulnerability-reporting channel before any distribution.
2. **Repository governance.** Require the CI checks on protected `master`; enable dependency, secret and code scanning; restrict or require SHA-pinned Actions; decide signed-commit/tag policy.
3. **Compatibility contract.** Add compile-level API compatibility fixtures, define SemVer scope across crates and replace or explicitly freeze the document language/head metadata contract.
4. **Production edge threat model.** Specify TLS termination and trusted headers, real authentication, rate limiting, CSRF/CORS policy, secret/key rotation and authorization ownership.
5. **Release supply chain.** Add license/source policy, SBOM, checksums, provenance/attestation, signing and immutable artifact promotion; do not publish directly from a mutable build job.
6. **Scheduled qualification matrix.** Add hosted Firefox and additional OS/browser/accessibility/IME coverage without making flaky browser timing a product threshold.
7. **Provider operations.** Qualify distributed Session/storage behavior, backup/restore, concurrency/load, cancellation, observability and incident recovery.
8. **External security review.** Perform an independent review only after the production topology and release inputs are defined.

The first technically unblockable work after a green hosted CI run is the API/document-metadata compatibility contract. Repository-governance settings can proceed in parallel when the maintainer authorizes those external changes. Licensing remains the first public-release blocker and requires a maintainer decision.

## 7. Delivery evidence boundary

The `run-ci` artifacts under `.pipeline-output/20261010-yetlet-ci-audit/` are ignored documentation plans. They do not generate or authorize workflow/deployment configuration. The tracked harness, workflow, dependency and documentation changes are separately authorized by the user and require:

1. complete non-browser local checks and focused harness evidence;
2. independent read-only reviewer PASS;
3. commit guard, milestone commit and push;
4. a new GitHub Actions run observed to terminal status, with every required job/step reported.

Until that remote run succeeds, the hosted CI repair is not complete. Regardless of CI outcome, public-release task `YT-020` remains Blocked.
