# CI and Security Review — Post-M9 Prototype

**Date:** 2026-10-10
**Scope:** CI reliability, dependency security, repository controls and next qualification work after the M9 evidence freeze.
**Decision:** The current source remains an experimental private prototype. The changes in this review improve CI and close one concrete RustSec advisory; they do not authorize public package publication or production deployment.

**Later private preparation update (2026-10-11):** PR #4 delivered bounded YT-065/066 qualification; required PR CI and post-merge `a61f409` CI passed. The maintainer then selected [experimental 0.1.0 Private preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) for the original nine library candidates. This update reconciles remaining activation work below without relabeling earlier tests/scans or authorizing publication, settings changes or production work.

## Public source preview and private reporting

The Private/Pending statements in this post-M9 review describe **2026-10-10**.
The **2026-10-27 YT-206** owner-approved disclosure/activation subsequently made
the repository Public as an Experimental source preview. PR12 squash
`d7065bbfb5b733c418f65ef55324f4d0ce6ac4d6`, tree
`cca0153c3e56bc5e13e651a4a4bb1469dc042cfe`, matched reviewed3d63f0b;
CI40497321153 PASS preserved required checks/admin enforcement/no-force and
collaborator-only PR policy. The bounded inventory covered1,191 tracked files,
114 reachable commits/2,177 unique blobs,15 PNGs,38 Actions runs (32 success/six
failure),16 artifacts and12 PR bodies. It found historical workspace disclosures
at six representative document paths and operational/reviewer identifiers;
bounded zero credential-marker matches were not comprehensive secret scanning
or credential validation. The owner accepted normal source/history/PR/image and
Actions disclosure, including failures; hiding links or deleting current reports
does not privatize Git history. No history/failure deletion or formal signing
disclosure approval followed from source-preview acceptance.

GitHub Private Vulnerability Reporting was enabled and the public entry verified.
Settings-only subscription/enablement originally left the signed-in form and
actual notification delivery **Not verified**. A separately authorized controlled
**non-vulnerability** test then completed on2026-10-27: a non-admin reporter's
authenticated form/submission created one private Triage record at20:25:29
Asia/Taipei; maintainer/collaborator messages reached the configured owner Gmail
at20:25:50/51 (21/22 seconds), with matching Delivered-To and GitHub DKIM/SPF.
The owner closed it at20:29:04, API Closed, without publication/CVE/private fork.
Independent PASS accepted this bounded form/delivery/closeout scope, not an SLA.
Private advisory IDs, mailbox addresses, tokens and mail bodies remain private.

Earlier owner/admin draft-form API submission **403**, zero reports/notifications,
signed-out/robots/browser observations, optional On GitHub Watching readbacks that
did not persist, and the To-header parser failure remain failures/limits. The
corrected parser reused the same messages and checked Delivered-To; no second
report was sent. Only Email delivery was proved, not Web inbox delivery. The
owner selected GitHub reporting as sufficient; a separate email/domain is
optional. Reverify form/notification routing when advertising a changed route
under [SECURITY.md](../SECURITY.md#reporting-a-vulnerability).

Current distribution is separately the [published SDKalpha.8/CLI0.4.0](RELEASE-READINESS.md),
not the old preview About text saying packages unpublished. Source visibility
did not publish a Reader site, registry packages or resolve D1–D6 at that checkpoint.
The original [disclosure/reporting record](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-206-PUBLIC-PREVIEW-DISCLOSURE.md#reporting-completion)
and inventories are recoverable in local Git; remote SHA reachability is
unverified. Raw proof stays in ignored
`target/yetlet-public-preview-inventory-20261027-yot7fza7/evidence/activation/reporting-completion/`,
not Git-guaranteed private evidence storage.

## Versioned release automation integration history

YT-209's local30 distinct focused checks and implementation PASS at
`c70a7431226c307a097c218050348477de85671d` did not qualify hosted execution.
On **2026-10-29**, PR20 CI40871149693/1 passed MSRV but failed Stable at the
existing Rust importlib caller (`ModuleNotFoundError: release_record`) before
audit/browser. Old custody/preparation fixtures also used obsolete fixed-alpha
inputs. Local edit/fixture failures are retained; focused sibling-import and
actual schema-1 fixture corrections preserved identity, artifact and secret-order
checks. YT-209 reopened for integration repair; old local PASS never authorized
merging that failed head.

Separately authorized repaircdde3c4 was delivered once, CI40873628263/1 PASS;
PR20 squash `7de77c84e0e56e683cea4567048397caa0614590`, tree
`95c65e754a51fadce648645791f41620b56f8bca`, matched qualified repair and
master CI40875245510/1 PASS. `release-result-verify` Environment25684187945
was installed with owner reviewer/master-only/no-admin-bypass and existing
AGE_IDENTITY via protected stdin. Configuration/name readback was not decryption
qualification; later alpha.3 supplied actual positive protected verification.
No old candidate/signature or failure result was replaced. This setup history
predates the separately adopted [single publication policy](RELEASE-PUBLICATION.md).
Original [integration and recovery](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-209-RELEASE-AUTOMATION.md#subsequent-integration-ci--2026-10-29)
Markdown is locally recoverable (remote SHA unverified); logs/settings receipts
remain in `target/yt209-delivery-20261029/`, not guaranteed by Git.

## 1. Remote CI failure diagnosis

The later [separate-signer failure chronology](#separate-signer-integration-failures-and-corrections--2026-10-28-and-29)
retains each original failure independently of successful final output acceptance.

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
| Browser/WASM | Eight wasm32 packages; Todo/Search, keyed lifecycle and multi-island Chrome release suites; hosted Firefox/form-keyboard qualification in run `38043565238` | Real IME, spoken assistive-technology output, WebKit/mobile and zoom remain Not verified optional coverage; no general support/conformance claim |
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
- YT-065 hosted run `38043565238` passed Ubuntu, Windows MSVC, macOS ARM64 and Linux Firefox/form-keyboard qualification; local Windows GNU and Docker also passed. The bounded automated scope is complete, while real screen-reader/IME/WebKit/mobile/zoom results remain explicitly Not verified optional coverage;
- no fuzzing/property program covers cookie/form/path/HTML/SSE parsers;
- no external penetration test, formal security audit or incident-response policy has been performed;
- repository/crate licensing is selected as `MIT OR Apache-2.0`, but public SemVer activation, scanner availability and release ownership remain unresolved.

## 6. Remaining activation and separately selected application work

1. **Public-transition activation.** Re-enable/verify GitHub Private Vulnerability Reporting, secret scanning/push protection and CodeQL immediately after changing repository visibility; retain the private channel in `SECURITY.md` until then.
2. **Repository governance review.** Keep the two required checks and collaborator-only PR policy; revisit approval count/bypass policy only when another maintainer is added and define signed release-tag policy in YT-064.
3. **Prepared compatibility policy.** YT-062 completed compile fixtures and additive document metadata. YT-020 preparation selects experimental `0.1.x`, patch API/MSRV preservation, latest-patch best-effort maintenance and source-aware qualification. Actual public activation is pending; no stable API, SLA or broader platform claim is introduced.
4. **Application production edge (not a framework publication prerequisite).** Apply the reviewed [Threat Model](THREAT-MODEL.md) to an application's selected proxy/IdP/Session store/limiter/secrets topology and qualify its required abuse cases under separately authorized scope.
5. **Release activation.** YT-064 completed design/preflight. Current private preparation adds versioned package dependencies, nine locally verified source archives, selected dependency inventories and unsigned checksum binding. Clean immutable source/tag, real registry ownership/resolution, independent rebuild, trusted provenance/signatures and approved immutable promotion remain unperformed gates; do not publish directly from a mutable build job.
6. **Evidence retention and optional compatibility extension.** Hosted platform/Firefox execution is complete in run `38043565238`; retain future scheduled evidence. Perform real-device, screen-reader, IME or zoom qualification only when a selected application/support scope requires it, without redoing successful automated lanes or treating those optional cells as a framework-release threshold.
7. **Optional broader provider/application operations.** YT-066 is Done at single-host Docker multi-instance and selected experimental single-primary/no-persistence Redis-provider scope, with actual qualification and independent reviews. Broader durability/restore, HA/cross-host/load/SLO and production operations remain Not verified and application-selected; they are not framework publication prerequisites.
8. **Internal security assessment.** The maintainer selected Codex Security Standard rather than paid external engagement. The later current-source scan `fc4d4af8-5864-4ace-b098-23ebfe249b31` completed (227/227 regular tracked files; one Low finding); a local endpoint-validation fix passes regressions and independent fix reviewer `wc_job_sTA8TdP3N3Veu9uC` PASS. The first older-source scan remains incomplete. Neither CI nor a document-review PASS substitutes for source-scan evidence or release approval.

Licensing/private governance and YT-064 release design/preflight are independently reviewed. YT-065 is Done at its automated desktop/native/browser scope after hosted run `38043565238`; manual real-device/assistive-technology/IME/WebKit/mobile/zoom cells remain Not verified optional coverage. YT-066 is Done at its recorded single-host Docker/selected Redis-provider scope; broader deployment qualification is optional and unverified. Revised YT-067 retains the completed pre-patch source-scan report and independently reviewed Low finding correction, with no external assessment or post-patch full rescan claim; the earlier initial scan has no report. The former external handoff is an optional future template. Private experimental `0.1.0` preparation does not establish trusted release evidence or public activation. YT-020 remains Blocked for its concrete release and public-transition gates.

## 7. Delivery evidence boundary

The `run-ci` artifacts under `.pipeline-output/20261010-yetlet-ci-audit/` are ignored documentation plans. They do not generate or authorize workflow/deployment configuration. The tracked harness, workflow, dependency and documentation changes are separately authorized by the user and require:

1. complete non-browser local checks and focused harness evidence;
2. independent read-only reviewer PASS;
3. commit guard, milestone commit and push;
4. a new GitHub Actions run observed to terminal status, with every required job/step reported.

GitHub Actions run `38024582602` is the terminal hosted evidence for commit `86c30a42114ca74f4197f0389707861308a844a7`; both required jobs completed successfully. Future commits require their own terminal CI result. A green CI run does not close public-release task `YT-020`, which remains Blocked.

## Separate signer integration failures and corrections — 2026-10-28 and 29

The independent-S design preserves C `bf3363801132f4481263ceb411d9bba3349df86d`,
both original tags and exact build/verifier bytes. Local integration PASS did
not qualify hosted execution. Every replacement S required separately approved
delivery, exact reviewed tree, actual PR/master CI and source selection.
Original failures remain ineligible even where later payload hashes match.

| Checkpoint | Actual failure, correction and preserved limit |
|---|---|
| Local signer implementation2026-10-28 | Closed prepare/sign, isolated custody, exact S/OIDC and partial-output sealing were reviewed PASS0. Initial TUF API/test-block errors remained. Reviewer runtime trace was unavailable, so configured model/effort stayed unverified. No hosted or formal-assurance proof followed |
| PR15 S `d2562e14af75d08c177c01e976cbd8f13124b8ff`, tree `74dd75a7f132f9080885190e1cecdc99245d57af` | Before dispatch, real original layout exposed duplicate OpenSSL log/result names in private and extras. Synthetic fixture had missed extras. No hosted/key attempt occurred. Unique BUILD-EXTRA labels/closure mapping and real-layout regression preserved both copies. Followup PASS0 retained unavailable-trace limitation |
| Qualification selection |53original records, reviewed proposal SHA `91dc6c988e2d3452a1614b157ad1e71bfef1694cd65f6ccfe55cbe464683e999`, then external review receipt yielded54records plus closure/manifest,56flat files/18,267,605bytes. Manifest `20757af3039310399145a93acf9bab06439d88235a55aa7e776aa90280bece3c`, encrypted ZIP `c90d98d20d579ed76c9ba8e705a6b204e1e43186eb3e02ec0121e91d15596168`. Neither was the final signing manifest |
| PR16 delivery | PR CI40710158856/1 failed Chrome debugger startup before assertions. Owner-authorized failed-Stable-only attempt2 passed, original MSRV timestamps retained. Selected S `87d24e9e63b4c9486d63a4b0141c20368ceaf540`, tree `977b82db0323e57f8692b8c22172f609a7c8c008`, master40716085116/1 PASS |
| Prepare40717870186/1 | Generic read-only preflight rejection. Prepare/sign skipped, no key/OIDC/approval/output. Exact cause remained unknown. Owner-context cached metadata did not prove hosted job-token access. Draft visibility stayed a hypothesis. Failure custody23files/200,037bytes, index `351cf237ce2010d36ca262f370da50d11ab8d1684f5288b7563aef8f6082d160` |
| Safe diagnostics | Only fixed stage, http/network/metadata/internal class and bounded integer HTTP status. Closed error responses without reading private bodies. Missing-entry and HTTP-finalizer warning failures were retained. Focused correction/PASS0 changed no gate/permissions. This review had matching native reviewer/model/high evidence |
| PR17 S `0e8fb600af7aaebd4641a78c477035afe637c67e`, tree `a4abae728a957991611c2f362ba5b005522b1abd` | PR40788520807/1/master40789389079/1 PASS. Prepare40789535074/1 failed `gate=qualification-release class=http status=403` after earlier gates passed. Protected work skipped. This observed status did not diagnose the earlier generic failure. Provider body/permission headers were deliberately unretained. Failure custody41files/224,168bytes, index `517c584090f91c6318275263dbb51f0b770a9755d229c29b35e0fb7ea258824d` |
| Encrypted capability correction | Reused existing Draft asset685300857 through owner-resolved age-encrypted read capability. Actual local transfer with test age identity and nine focused cases passed. No owner key/qualification plaintext read. No owner credential forwarded. Independent PASS0 had matched native reviewer/max evidence. Hosted source qualification remained separate |
| PR18 S `e2d4a9f64925f2a20c35c7e7aeae6e352e4938ef`, tree `07404415d79443c83109a1205b0ccb6a25584cd7` | PR40794539309/1/master40795652719/1 PASS. One owner-approved prepare40795794183/1 passed preflight123191847380, all tools/data/full-manifest/freeze/upload, then cleanup failed200 permission-denied entries inside task-private Go module cache. Job123191897780/overall run stayed FAIL, sign123191900050 skipped. Logs showed no errors elsewhere, without independent filesystem-cleanup certification |
| Failed frozen candidate readback | Artifact12863583130,74,815,048bytes, ZIP `561058586354c54c935af000f7b5fac3d1084d93ae12f606dd257262cfc49b4b`, safe receipt `b6b79e3b98a21c1caa226ea27a004fab367ac7d2eb951e6785340f06025e19a5`, ciphertext `e47f1d0544acc240f31310eec7f1c79d169187312f5e1a4bec716086b83a4f27`. Final-manifest hash matched later success, but this run remained ineligible. No local decrypt or automatic retry |
| Scoped cleanup repair | Task-private bootstrap GOFLAGS=-modcacherw made its new Go cache removable. One focused native offline case reproduced old removal failure and proved corrected removal. Global settings, tool pins, sumdb, keys and ordinary strict cleanup gate stayed intact. Original failure/local review PASS0 did not claim repaired hosted success |
| PR19 S `d65e3180ad3076659f03c069fc71cd0e7e3f0518`, tree `5812dcfbc9e50a76c4d230bd5d32a336568db12a` | PR40805140129/1/master40806189605/1 PASS. New separately approved prepare40806988578/1 and sign40811275645/1 passed, followed by independent cryptographic verification and exact-output owner acceptance. The [accepted identities](RELEASE-READINESS.md#alpha1-frozen-inputs-signatures-and-acceptance--2026-10-29) retain the spent one-use local verification exception and wrapper failures |

The supplemental selection preserved actual C export, locks/recipes, accepted
qualification/support/scans and the original370-member security ZIP. Denied
Windows ZIP transfer remained denied, represented only by admitted remote text/
index/reference scope. The Tokio audit ZIP remained a referenced reconciliation,
without a claim that its raw bytes were in the local selection. Neither source
export nor review created complete compiled/native/system inventory or a new
all-source scan. Final source-specific acceptance closed D6/YT-096 only at its
documented signing/integrity scope. The earlier ce56e08 FAIL/release-ready NO,
13historical Guard findings and all custody/rotation/reviewer/wrapper failures
retain their original results. No repeated test, key operation or cache change
is performed by this consolidation.

Raw records remain at `target/yt096-alpha-signer-preparation-20261028/implementation/`,
`target/yt096-signer-prepare-batch-20261028/` and
`target/yt096-owned-cache-cleanup-delivery-20261029/`, plus indexed owner custody.
The [original dated signer narrative](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-SIGNER-CONTRACT.md#hosted-preflight-failure-and-safe-local-diagnostics--2026-10-29)
is locally Git-recoverable with unverified remote reachability. This preserves
provenance of claims and raw-store locations, without asserting current raw-byte
availability or granting operations against those stores.

## M10 policy, governance and scan evidence

These are dated **2026-10-10** observations, not a readback of current GitHub
settings. Current reporting and distribution follow the public-source section
above and [Release Readiness](RELEASE-READINESS.md).

| Historical slice | Accepted evidence and original limits |
|---|---|
| YT-060 licensing/policy | `MIT OR Apache-2.0` lets the consumer choose either license; Apache supplies its patent grant. All 28 original manifests carried license/repository metadata, 19 applications were nonpublishable, nine library package lists included both texts through 18 exact aliases. Reviewer `wc_job_MPF4l2t8JaJXBguv` PASS. Licensing grants fork/use rights, not upstream PR acceptance, publication or legal assurance. Security reporting remains private, best effort and without response SLA under SECURITY.md |
| YT-061 Private governance | Collaborator-only upstream PRs, sole owner, squash-only/no auto-merge/no DCO, automatic deletion of merged branches; protected master required PRs with zero human approvals, conversation resolution, linear history and administrator enforcement, no force/delete. Required checks were `Rust 1.88 MSRV` and `Stable native, samples, and Chrome`. Actions were read-only, unable to approve PRs, GitHub-owned/full-SHA, with checkout credentials disabled; Dependabot alerts/security fixes enabled. Private reporting returned404, secret scanning/push protection422, CodeQL default setup403: none was silently enabled or treated as PASS |
| Governance review correction | First reviewer `wc_job_ISJjq_EUBG8tmQfQ` found missing check-run proof and stale monitoring claims. Retained `target/m10ab-evidence/check-runs.json` proves both contexts succeeded at `86c30a42114ca74f4197f0389707861308a844a7`, Actions app15368. Corrected text distinguishes alerts/security fixes/per-CI audit from absent scheduled updates and scanning. Repeat reviewer `wc_job_lu2WynqvKYemLlmd` PASS; this closed Private-state governance only |
| Original YT-067 Standard scan | Codex Security0.1.32 scan `580f6c9a-ded9-4f69-a3c9-16bbcc29ab97`, source `3128d23591feee15ce976d7b83a56dcd587048d7`, coordinator `wc_job_oCTu5__mlpWYcsLr`: process completed but semantic result **incomplete**, ten reviewed files, no final report/counts. A group-writable Codex-home ancestor blocked checkpointing; owner changed0775→0755. Same-scan continuation was refused before a Job ID. No bypass, handwritten replacement report or repository-clean claim followed |
| Later distinct source scan | Official Standard scan `fc4d4af8-5864-4ace-b098-23ebfe249b31`, Job `wc_job_qawaocVOmnaMdOj2`, clean source `25bff77ccc5a3641b30be27c4ca5d862c06f535d`: semantic completion/exit0, 227/227 regular files plus18 license symlinks, complete coverage, no deferred questions; **one Low/medium-confidence finding**, zero Critical/High/Medium. Canonical manifest/findings/coverage/threatmodel/report remain private in scanner-owned state; no attack traffic, external audit or complete dependency audit |
| Bounded finding correction | Public-library `HttpExecutor::new` could disagree with browser URL normalization on ASCII controls. The local patch rejects every ASCII control byte before browser Request parsing. The shipped Todo used a fixed endpoint and restrictive CSP; no working shipped exploit was demonstrated. Three focused tests on stable/MSRV, strict Clippy, stable/MSRV Wasm, 196 workspace tests and five doctests per toolchain, eight Wasm packages and Chrome release E2E passed. Todo Wasm became294,576 bytes (+21), digest27597bad9c53b93e. Reviewer `wc_job_sTA8TdP3N3Veu9uC` PASS closes scan/fix scope; **no post-patch whole-repository rescan** |

The maintainer replaced the paid external-assessor prerequisite with the internal
scan. The [external template](EXTERNAL-SECURITY-REVIEW.md) remains optional, with
no commissioned assessor or external attestation. Earlier M10 release blockers
describe their checkpoint; they do not undo the accepted SDKalpha.8/CLI0.4.0
receipts. No scan here certifies current production security.

Full original policy, governance, delivery and scan records are recoverable from
local Git `40b66f1ad9bd0eb5666a9e1c8f113310993a3544` as
`docs/M10A-VERIFICATION.md`, `M10B-VERIFICATION.md`,
`M10-DELIVERY-VERIFICATION.md` and `M10H-VERIFICATION.md`. Remote SHA accessibility
is unverified. Raw `target/yt067-evidence/` logs and private scan reports stay at
their recorded locations; Git recovery restores Markdown, not those raw files.
