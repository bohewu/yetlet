# M10 Remaining Qualification Plan

**Date:** 2026-10-10
**Tasks:** YT-064–YT-067
**Status:** Independent planning reviewer `wc_job_n6i2DoBNKaltAWs2` PASS (terminal exit 0, requested/observed `gpt-6-astra`, high, read-only). No release, deployment, external audit engagement, commit or push is authorized by this plan.

## Baseline and constraints

The sole development checkout was on branch `master`, baseline HEAD `7c5d1267fd2ea3490738da105b476e271278ae7a`. Existing uncommitted YT-060–YT-063 work must be preserved. Dual licensing, maintainer-only PR creation and protected master are established. Only a maintainer branch/PR may eventually merge new work; no protection bypass is permitted.

Keep Yetlet tiny: these tasks add development tooling, tests and documents, not mandatory runtime dependencies, new framework subsystems, ORM/DI, an in-house distributed store or an authentication platform. Use existing external tools for license analysis and SBOM rather than implementing SPDX parsing or a custom supply-chain format. Public release YT-020 remains Blocked.

## Sequential slices and reviewer gates

### YT-064 / M10e — Release supply-chain design and executable preflight

Deliver a concise `docs/RELEASE-SUPPLY-CHAIN.md`, a default-deny dependency license/source policy for an established checker, and bounded repeatable tooling/tests for release-evidence checksums. Use a pinned mature CycloneDX generator for a local SBOM experiment; retain its input identity, dependency/target coverage and limitations. Inventory all nine library candidates and 19 nonpublishable samples using Cargo metadata. No package publication, tag, key generation, signing-service call, registry upload or GitHub Environment mutation.

Define: clean reviewed source/tag/lock/toolchain/target identity; license/source acceptance; package ordering and path dependency version blockers; SBOM per selected artifact; SHA-256 verification with bounded path handling; independent rebuild versus same-source equality; provenance/signature identity and issuer checks; approvals, immutable digest promotion, revocation and rollback. A dirty local evidence bundle must be labeled development-only and must not be mistaken for a signed or reproducible release.

Validation: license/source checker on the actual workspace, an actual generated dependency SBOM, manifest success/tamper/missing-file/unsafe-path tests, documented candidate inventory and no new runtime dependencies. Independent reviewer PASS closes **the design/preflight task only**, not release qualification or signing.

### YT-065 / M10f — Scheduled compatibility harness

Add one opt-in scheduled/manual qualification workflow using only approved full-SHA GitHub-owned actions, read-only permissions, bounded concurrency and timeouts. Preserve the existing two required checks. Explicitly install/observe toolchain and Firefox/geckodriver prerequisites; do not assume a local installed binary exists on hosted runners. Include native portability lanes where feasible and a Linux Firefox lane for the three existing release suites. Keep architecture/browser/version coverage explicit; no unsupported browser or screen-reader claims.

Add a checked-in matrix/runbook covering Linux/Windows/macOS, Chrome/Firefox/WebKit/mobile, keyboard/labels/focus and real IME/screen-reader/manual testing. Unexecuted cells remain Not verified, not PASS. Reuse existing harnesses, add narrowly scoped accessibility/keyboard checks where a current suite lacks them, and preserve explicit ownership/cleanup. Do not run repeated visible probes; execute one bounded local headless sequence and classify failures before any repair.

Validation: workflow policy/commands and target prerequisites; local available Firefox suites; deterministic markup/keyboard checks and report parsing. An uncommitted workflow cannot supply hosted evidence: YT-065 stays In progress until the authorized maintainer PR is merged and the actual new workflow reaches terminal success. No new cron automation outside the repository is created.

### YT-066 / M10g — Operational evidence, not a new provider framework

Document the real current topology: `yetlet-session` is process-local with no pluggable distributed-store contract. Do not silently replace it with a generic provider or claim Redis/multi-node semantics from a shared in-process map. Qualify the deployed capability that actually exists.

Add bounded local operation experiments reusing existing sample binaries: process-local Session isolation/restart limits, SQLite backup/restore integrity with committed state and failed/corrupt restore rejection, deterministic concurrent-write/cancellation regression where existing tests apply, and owned-process/temp-directory cleanup. Use synthetic users/data and redact cookies, codes and secrets from reports. Capture concrete outcomes and durations as local observations, not throughput/SLO claims.

Write the future application-selected distributed-provider acceptance contract: atomic read/update/rotate/revoke, stale cookie response protection, TTL/revocation consistency, fail-closed outage, bounded retries/backpressure and drain; storage RPO/RTO/backup/restore and secret/redaction/telemetry ownership. Evaluate mature external integration rather than hand-writing a store. The tower-sessions documentation explicitly warns that concurrent dependent session mutations can lose updates; selecting a persistent store alone does not prove Yetlet's stronger transaction semantics.

Local operations tests can PASS while **distributed-provider qualification remains blocked**. YT-066 must remain In progress until a concrete external topology/store is selected and its real cross-node/outage/rotation tests pass. Preserve this unmet gate; no substitute Python model counts as provider validation.

### YT-067 / M10h — External security review handoff

Prepare `docs/EXTERNAL-SECURITY-REVIEW.md` with scope, threat-model links, exclusions, release/build identity, evidence index, known gaps, rules of engagement, required auditor deliverables, severity/fix/retest/acceptance and redaction requirements. Include an unambiguous gate checklist and blank attestation fields rather than invented results.

No external auditor, production deployment, release candidate or paid engagement is currently designated. A local role-directed reviewer can review this handoff but **cannot satisfy the external production-security audit**. Do not transmit the private repository to a third party, purchase services, run attacks on remote systems or mark YT-067 Done. It remains Blocked pending original prerequisites and an explicitly authorized real external assessment.

## Evidence and scope accounting

Each slice has its own verification document and independent read-only reviewer, with retained exact Job/model/effort/terminal result. First review the plan, then each source freeze. Material fixes invalidate only affected validation and require repeat review. Prior clean evidence is not re-run solely to produce redundant proof.

Track current source separately from baseline HEAD: all pre-existing Rust/Cargo/sample files remain unchanged by the remaining tasks, unless a specific operational defect in an existing harness is demonstrated and reviewed. Workflows and development scripts are not runtime dependencies. Neither local tests nor written requirements prove hosted, distributed, reproducible, signed or externally audited behavior.

Required closeout: docs/catalog/whitespace checks, focused tool/harness tests, applicable Rust/MSRV baseline, unchanged runtime/dependency/bundle evidence, task-specific findings and status, no active uncertain Jobs, no staged files, no commits or pushes, no visibility/branch-policy changes. Report unmet gates plainly rather than weakening their acceptance criteria.

## Primary references

- Cargo-deny license and source policy: https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html and https://embarkstudios.github.io/cargo-deny/checks/sources/cfg.html
- CycloneDX Cargo generator: https://github.com/CycloneDX/cyclonedx-rust-cargo
- GitHub workflow triggers and default-branch constraints: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows
- Tower Sessions concurrent mutation caveat: https://docs.rs/tower-sessions/latest/tower_sessions/#data-races-under-concurrent-conditions
