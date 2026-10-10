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

GitHub Actions run `38024582602` subsequently completed successfully for commit `86c30a42114ca74f4197f0389707861308a844a7`. The `Rust 1.88 MSRV` job passed native tests/doctests and the eight wasm32 packages. The `Stable native, samples, and Chrome` job passed documentation/catalog checks, format, strict Clippy, native tests/doctests, RustSec, all native smokes, eight wasm32 packages, release asset builds, all three Chrome E2E suites and the informational baseline artifact. This hosted result closes the prior Browser E2E repair gate for that commit.

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

YT-064 now supplies a passing cargo-deny license/source policy, an actual native CycloneDX inventory and bounded unsigned checksum preflight. Open release gaps remain: trusted build provenance, signing, independent reproducibility and immutable release inputs; secret/code scanning and scheduled version-update configuration also remain unavailable/unconfigured beyond enabled Dependabot vulnerability alerts/security fixes and per-CI cargo-audit. Local inventory and hashes do not authenticate a release.

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

Observed GitHub repository settings after the authorized YT-061 update on 2026-10-10:

- repository visibility remains Private;
- pull-request creation is `collaborators_only`; the sole direct collaborator is the repository owner;
- `master` is protected for administrators, requires a PR, zero approvals in the sole-maintainer phase, both existing CI checks, conversation resolution and linear history;
- force pushes and branch deletion are disabled;
- Actions use a read-only default token, are restricted to GitHub-owned actions and require full commit-SHA pinning;
- Dependabot vulnerability alerts and automated security fixes are enabled;
- private vulnerability reporting is unavailable on the current private repository (HTTP 404);
- secret scanning/push protection is unavailable on the current private repository/account plan (HTTP 422);
- CodeQL default setup is unavailable because code scanning is not enabled/available (HTTP 403);
- web commit sign-off remains disabled because the project does not accept external code contributions or require DCO sign-off.
Product/operations gaps:

- Auth Notes uses terminal-only synthetic identities, not a production IdP, passkey, password, MFA or recovery flow;
- [M10d Threat Model](THREAT-MODEL.md) now defines the required TLS edge, trusted forwarding-header, OIDC, rate-limit, CSRF/CORS, secrets and authorization contracts, but no production deployment/provider has implemented or qualified them;
- Session state is in-memory and single-process; no distributed persistence, revocation or multi-node semantics are qualified;
- YT-066 adds actual local SQLite two-process concurrency and online backup/isolated restore evidence; encryption/retention, off-host durability, production recovery budgets and multi-node operations remain unqualified;
- no production secrets/key-management integration exists;
- YT-065 hosted run `38043565238` passed Ubuntu, Windows MSVC, macOS ARM64 and Linux Firefox/form-keyboard qualification; local Windows GNU and Docker also passed. Real screen-reader/IME/WebKit/mobile/zoom results remain unverified;
- no fuzzing/property program covers cookie/form/path/HTML/SSE parsers;
- no external penetration test, formal security audit or incident-response policy has been performed;
- repository/crate licensing is selected as `MIT OR Apache-2.0`, but public SemVer activation, scanner availability and release ownership remain unresolved.

## 6. Prioritized next qualification work

1. **Public-transition activation.** Re-enable/verify GitHub Private Vulnerability Reporting, secret scanning/push protection and CodeQL immediately after changing repository visibility; retain the private channel in `SECURITY.md` until then.
2. **Repository governance review.** Keep the two required checks and collaborator-only PR policy; revisit approval count/bypass policy only when another maintainer is added and define signed release-tag policy in YT-064.
3. **Compatibility activation.** YT-062 already completed the bounded compile-level compatibility fixtures, provisional coordinated SemVer policy and additive title/language document contract. Public SemVer/support activation and broader compatibility qualification remain future release decisions; do not reimplement the completed slice.
4. **Production edge implementation.** Apply the reviewed [Threat Model](THREAT-MODEL.md) to a selected proxy/IdP/Session store/limiter/secrets topology and prove every required abuse case.
5. **Release activation.** YT-064 already completed release design/preflight, the passing license/source policy, native SBOM experiment and unsigned checksum tooling. Clean release candidates, independent reproducibility, trusted provenance/signing and approved immutable promotion remain unperformed activation gates; do not publish directly from a mutable build job.
6. **Manual compatibility qualification.** Hosted platform/Firefox execution is complete in run `38043565238`; retain future scheduled evidence and complete the explicitly unverified real-device, screen-reader, IME and zoom coverage without redoing successful automated lanes or making timing a product threshold.
7. **Provider operations.** Qualify distributed Session/storage behavior, backup/restore, concurrency/load, cancellation, observability and incident recovery.
8. **Internal security assessment.** The maintainer selected Codex Security Standard rather than paid external engagement. The later current-source scan `fc4d4af8-5864-4ace-b098-23ebfe249b31` completed (227/227 regular tracked files; one Low finding); a local endpoint-validation fix passes regressions and independent fix reviewer `wc_job_sTA8TdP3N3Veu9uC` PASS. The first older-source scan remains incomplete. Neither CI nor a document-review PASS substitutes for source-scan evidence or release approval.

Licensing/private governance and YT-064 release design/preflight are independently reviewed. YT-065 automated hosted qualification passed run `38043565238`; its manual gates remain open. YT-066 source/local slices passed review, but actual distributed qualification remains open. Revised YT-067 uses an internal Codex-assisted scan; no external assessor is required. The current-source scan generated its canonical report, and its single Low finding's bounded local patch passed independent reviewer (no post-patch full rescan); the earlier initial scan has no report. The former external handoff is an optional future template. Public-transition scanner/reporting activation and YT-020 remain Blocked.

## 7. Delivery evidence boundary

The `run-ci` artifacts under `.pipeline-output/20261010-yetlet-ci-audit/` are ignored documentation plans. They do not generate or authorize workflow/deployment configuration. The tracked harness, workflow, dependency and documentation changes are separately authorized by the user and require:

1. complete non-browser local checks and focused harness evidence;
2. independent read-only reviewer PASS;
3. commit guard, milestone commit and push;
4. a new GitHub Actions run observed to terminal status, with every required job/step reported.

GitHub Actions run `38024582602` is the terminal hosted evidence for commit `86c30a42114ca74f4197f0389707861308a844a7`; both required jobs completed successfully. Future commits require their own terminal CI result. A green CI run does not close public-release task `YT-020`, which remains Blocked.
