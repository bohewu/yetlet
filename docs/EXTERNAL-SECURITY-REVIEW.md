# External Security Review Handoff

**Scope:** Optional future external engagement template; former YT-067 / M10h preparation.
**Status:** Superseded as a current milestone requirement by the maintainer on 2026-10-10. YT-067 now uses an internal Codex-assisted scan; no external assessment has been purchased or performed. This template does not create a paid-assessor prerequisite.
**Repository:** `bohewu/yetlet`, Private, maintainer-controlled contributions.

## 1. Mandate and independence

Commission an external production-security assessment only after the maintainer approves the precise source, deployable artifacts, topology, auditor and rules of engagement. A local Codex role-directed reviewer can inspect this handoff and source changes, but cannot issue an independent external audit result. That distinction remains true even though current YT-067 no longer requires an external engagement. Passing CI, checksums, a vulnerability scanner or a documentation review is not a substitute.

The current work authorizes no engagement, purchase, disclosure of the private repository, remote penetration testing, credentials, production access, release or deployment. No auditor identity, date, sign-off or assessment outcome has been invented. Do not send this repository or evidence to an outside service without separate approval.

## 2. Entry gates and named decisions

| Required entry | Current state | Evidence/owner required before assessment |
|---|---|---|
| Maintainer authorization | Not granted for external engagement | Named approver, approved auditor, scope and dates |
| Immutable source and release candidate | Not established | Reviewed clean commit/tag, complete archive and digest; current uncommitted work is not baseline HEAD |
| Release build identity | Design/preflight only | Toolchain/target/features, artifact/SBOM/checksum/provenance binding and independent rebuild/signature verification |
| Production topology | Not selected/deployed | Edge, private network, IdP, Session store, database/storage and secrets/telemetry products, versions and owners |
| Distributed-provider qualification | YT-066 remains In progress | Cross-node update/rotate/revoke, TTL, outage/failover and restore/load evidence |
| Hosted compatibility | YT-065 automated scope complete | Recorded hosted workflow/source identities are available; any manual browser/OS/assistive-technology scope is selected explicitly for the engagement and is not implied complete |
| Legal/security policy | YT-060 local policy complete | Current license/security policy plus agreed confidentiality, reporting and access handling |
| Findings lifecycle | Not assigned | Maintainer/security owner, severity decisions, fix and independent retest process |

At this template's original preparation, source HEAD `7c5d1267fd2ea3490738da105b476e271278ae7a` identified only that historical committed baseline and did not include the then-uncommitted M10 source. Later merges and qualification runs are recorded elsewhere, but none is automatically an auditable release candidate. Before any engagement, select a current approved immutable source/archive and artifact digest; do not use a mutable branch name or any historical baseline as if it were the engagement target.

## 3. Evidence index and limitations

| Artifact | Useful input | It does not prove |
|---|---|---|
| [Threat model](THREAT-MODEL.md) | Twenty threats, fifteen abuse scenarios, control ownership, private-cache/CSRF boundaries | Implemented production controls |
| [Release design](RELEASE-SUPPLY-CHAIN.md), [M10e](M10E-VERIFICATION.md) | License/source policy, native SBOM experiment, checksum tests, promotion/rollback design | Signed release, independent reproducibility or trusted build origin |
| [Qualification matrix](QUALIFICATION-MATRIX.md), [M10f](M10F-VERIFICATION.md) | Actual local and hosted native/Firefox/keyboard evidence with recorded source identities | Real IME, spoken screen-reader output, WebKit/mobile/zoom behavior, accessibility conformance or a general support range |
| [Provider operations](PROVIDER-OPERATIONS.md), [M10g](M10G-VERIFICATION.md) | Real process-local Session limits, two-process local SQLite concurrency and isolated backup/restore | Distributed Session, HA, failover, production RPO/RTO or load/SLO |
| [API contracts](API-CONTRACTS.md), [compatibility](COMPATIBILITY.md) | Experimental/Stable Candidate boundaries and source compatibility fixtures | Stable framework or production support contract |
| [Security policy](../SECURITY.md), [governance](M10B-VERIFICATION.md) | Reporting/contribution controls and private-state repository settings | Complete security-service coverage or incident response execution |
| [M9 freeze](M9J-VERIFICATION.md) | Historical sample, browser, lifecycle and dependency evidence | Current deployment acceptance or externally signed conclusions |

Ignored `target/` outputs are local evidence, not durable committed audit records. Before sharing, regenerate or export only approved sanitized evidence, record its source and SHA-256, and verify completeness and confidentiality. Never export Cargo caches, browser profiles, raw cookie/token/CSRF values, environment dumps or unrelated files.

## 4. Proposed technical scope

Review the selected application/deployment, not an imaginary framework feature set. Keep ordinary Axum/Tower and explicit third-party provider boundaries visible; do not require Yetlet to grow an IdP, ORM, DI container, distributed store, proxy, vault or policy engine merely to be audited.

The auditor should trace:

- TLS edge, private-hop identity, direct-access prevention, Host/forwarded metadata, allowed public origins and redirects;
- real OIDC issuer/key/claims/state/nonce/PKCE, callback replay/mix-up and account/privilege revocation;
- Session ID/CSRF rotation, dependent updates, stale responses, expiration, cross-node consistency and fail-closed outage;
- resource/tenant authorization, IDOR, mass assignment, privileged/operator actions and security audit semantics;
- cache isolation for personalized/authentication/Set-Cookie responses, same-origin/CORS policy and synchronizer-token delivery;
- HTML/JSON/form escaping, body/media/schema limits, file traversal/symlink/upload cleanup, SQL parameterization/transactions and SSE replay/subscriber bounds;
- cancellation, admission/rate/concurrency limits, queue/pool exhaustion, graceful drain, storage outage and backup/restore;
- server-secret isolation from WASM, logs and metrics, secret/key rotation, dependency/build trust and immutable artifact promotion.

Separate upstream dependencies' documented guarantees from actual integration behavior. A provider name, a third-party audit or an SBOM does not prove the application's authentication/authorization or concurrency contract.

## 5. Rules of engagement to approve

Default external testing authorization is **none**. Before testing, record exact approved source archives, target URLs/IPs/ports/accounts, time window, allowed methods and rate/concurrency/volume limits. Prefer a disposable nonproduction environment with synthetic tenants, synthetic secrets, snapshots and working recovery procedures. Production targets, real user data, neighboring services and underlying cloud accounts are excluded unless individually authorized.

Define a private findings channel, access expiration, least-privilege accounts, evidence encryption/retention/deletion, emergency contacts and stop procedure. No persistence, exfiltration, social engineering, destructive action, exploit chaining into another tenant/service or denial-of-service is authorized by this handoff. Destructive/availability experiments need separate written limits and recovery ownership. Stop on unexpected production routing, real sensitive data, uncontrolled resource growth, scope ambiguity or loss of monitoring; preserve minimal redacted evidence and notify the approved contact.

## 6. Required auditor deliverables

The actual auditor must provide a dated report identifying its organization/person, methodology, exact immutable source/build/deployment inputs, authorized scope and testing limitations. Include reproducible redacted findings with affected component/route, violated requirement, prerequisites, impact, severity rationale and a minimal remediation. Explicitly identify untested areas and distinguish exploitation evidence, static observations and assumptions.

For each material finding, maintain a record of owner, accepted fix, commit/build identity, regression test and the external auditor's retest on the exact revised candidate. Independent local code review supports remediation but does not replace that retest. A risk acceptance requires the maintainer/security owner, rationale, compensating controls, expiration and residual risk; it is not an automatic PASS. Do not average away a critical/high issue using an overall score.

## 7. Acceptance and release separation

An optional future external engagement may be accepted only when its actual approved external report is received, its identity/scope/input digests verified, required fixes independently retested, remaining risks explicitly accepted by the authorized owner and evidence stored through the approved private channel. A blank template or local role review is insufficient.

Even a completed optional external engagement does not automatically publish Yetlet or open the repository. Public transition, package publication, release signing and deployment remain separate actions requiring explicit maintainer authorization and all relevant YT-020 gates.

## 8. Unfilled acceptance record

```text
External assessor: NOT APPOINTED
Engagement authorization: NOT GRANTED
Target source commit/artifact digests: NOT FROZEN
Target deployment/tenant scope: NOT SELECTED
Rules of engagement and dates: NOT APPROVED
Report identifier and date: NOT ISSUED
Findings and severity inventory: NOT ASSESSED
Independent remediation retest: NOT PERFORMED
Authorized residual-risk acceptance: NOT ISSUED
Optional external assessment: NOT COMMISSIONED; NOT A CURRENT YT-067 REQUIREMENT
```

[M10h verification](M10H-VERIFICATION.md) records the maintainer's scope revision, historical handoff review and actual incomplete internal scan; it is not an external report.
