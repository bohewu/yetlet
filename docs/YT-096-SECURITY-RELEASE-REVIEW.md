# YT-096 — final candidate security and release review

Date: 2026-10-20. **Review complete; release gate FAIL; release-ready NO.**
The independent reviewer accepts the bounded candidate technical/security
evidence with its recorded limits and identifies no new material defect in the
inspected source delta and adjacent boundaries. At that review checkpoint,
five existing release-evidence and approval gates were unmet. The subsequent
[owner disposition](#subsequent-owner-support-and-inventory-disposition)
resolves only support/inventory acceptance; four release conditions remain
open. The original verdict is unchanged. YT-096 stays In progress;
YT-020 stays Blocked.

## Authority and immutable review target

The owner requested committing the reviewed reconciliation, then conducting
this final review using existing evidence and preserving known limitations.
Reruns, signing and publication were excluded. The eight reconciliation
documents were committed as
`c0726d22d55aa3a23e1a1d37efd7dd0147fbada8`; strict/builtin Commit Guard passed
with the exact scanned tree and two nonblocking generic Windows-path warnings.
That local commit was not pushed.

The product candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`: the original nine experimental,
unpublished 0.1.0 source packages, Rust MSRV 1.99 and toolchain pin 1.99.0.
Documentation commits do not change those package bytes or API classifications.
CLI/config/secrets/signing/Redis/Valkey/GPU, application HA/durability/IdP and
certificate custody remain outside this distributed package scope.

Reviewer `/root/yt096_candidate_security_release_review_oct20` performed the
independent source/evidence assessment. Native trace verification matched the
registered reviewer role, gpt-6-astra model and effective max effort. The shared
resolver's final-gate selection was strict and enforced. These execution
identities do not establish external certification or approve release activation.
The assessment used existing records and source inspection; no test, build,
checksum command, scanner, browser or hosted workflow was rerun.

The exact terminal result is retained in
[REVIEW-RESULT.json](../artifacts/yt096-security-release-review-20261020/REVIEW-RESULT.json).
[Verified receipt](../artifacts/yt096-security-release-review-20261020/REVIEW-ACCEPTANCE.json)
accepts the result's identity while explicitly retaining its failed gate
outcome. This report relays that verdict after the review; the reviewer did not
review this subsequently authored prose.

## Accepted technical and security evidence

- **Historical security assessment and remediation:** the reviewer inspected
  the five canonical private scan files for completed Standard scan
  `fc4d4af8-5864-4ace-b098-23ebfe249b31`, source
  `25bff77ccc5a3641b30be27c4ca5d862c06f535d`. Coverage was 227/227 regular files
  and 18 license aliases, with one Low finding. The actual HttpExecutor
  ASCII-control guard/tests and retained focused/full regression evidence
  support its bounded remediation. The original scan remains pre-patch;
  historical findings are not reduced to zero and no current full rescan is
  claimed. [M10h](M10H-VERIFICATION.md) retains that boundary; private exploit
  details are not copied into repository documentation.
- **Current source/API delta:** comparison with the scanned source and the
  historical signed source confirms the selected changes. Since signed source
  `8e557e760407a3d4b3f5a8b8f833748946be1387`, all nine manifests and eight
  libraries' Rust sources are unchanged. The Rust addition is the reviewed
  weak-handle snapshot/shared helper. Implementation/tests preserve weak
  ownership, published-state timing, Disposed-before-Faulted and borrow release
  before user Clone. Read capability, retained clones, reentrancy, shared
  interior mutability and freshness limits remain explicit. No new material
  defect was found within this inspection. Browser state remains untrusted and
  must not carry server-only secrets or authorization capabilities.
- **Package/dependency preparation:** normal verification of nine archives and
  four packaged-input consumer tests, 23 CycloneDX inventories, 23 license/source
  checks and nine package-lock RustSec outputs are accepted at their recorded
  scopes. The 29 license warnings are unused allowances. RustSec's zero
  reported vulnerabilities/warnings apply to database
  `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`, completed 2026-10-20 07:53:16 UTC;
  this review performs no advisory refresh.
- **Execution and rebuilding:** existing raw GNU/MSVC evidence supports
  83 native tests/four doctests, and five wasm library builds. Nine complete
  archive matches across Linux/Windows/Azure/GitHub remain accepted. Hosted
  run 39418279958 attempt 1/job 118612562396 succeeded at the exact candidate
  using the inspected pinned, read-only, fresh-input workflow. Separately
  operated execution and unsigned provenance do not authenticate a builder or
  create an independent human approval.
- **Cleanup:** retained receipts support browser/server teardown, container
  removal, terminal native/hosted execution and deletion of Azure resources,
  including failed SSH attempts. The reviewer identified no cleanup blocker.

The [final reconciliation](YT-096-FINAL-RECONCILIATION.md) retains every
inventory/browser qualification limit. The two native server-axum Serde derive
extras and two test/dev-only packages remain recorded; the 140 actual Cargo
events have no unexplained package/version difference. Exact per-library
compiled features/output hashes, sysroot/native/system closure and new Windows
compiler-event coverage remain unverified. Firefox is source-reconciled on
Ubuntu 26.04.1; historical Chrome has no original candidate-wide manifest.
Windows OIDC and repeated IME/assistive-technology waivers remain intact.

## Five unmet release gates at the review checkpoint

| Gate | Reviewer disposition and requirement | Next authority/evidence |
| --- | --- | --- |
| Formal integrity and trust | P1 evidence gap: unsigned 36-entry source/SBOM bundle and separate hosted provenance do not authenticate a combined release statement. Approved source/tag, builder/signing identity, verifier/roots, certificate/chain and transparency/inclusion evidence are absent | Separately approve and authorize the release trust/identity/source/tag policy and independently verified bound evidence set under [supply-chain policy](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals) |
| Support and inventory acceptance | P2 evidence gap at the review checkpoint: historical Chrome's missing original candidate source binding and conservative/dev-only/compiled-system inventory omissions had no final maintainer disposition | Subsequently resolved as a maintainer disposition only by the [owner's bounded acceptance](YT-096-FINAL-RECONCILIATION.md#owner-support-and-inventory-acceptance-2026-10-20); missing evidence stays Not verified |
| Actual registry qualification | P2 evidence gap: local directory-source consumer simulation does not establish account/name rights, uploaded/downloaded bytes or real registry consumption | Keep blocked until separately authorized registry activation supplies those proofs under the existing release policy |
| Public activation controls | P2 evidence gap: retained records do not establish the required reachable private reporting fallback or applicable public reporting/scanning controls | Before separately authorized public activation, verify the route/fallback and applicable controls; retain Private state until then |
| Concrete maintainer release approval | P2 evidence gap: candidate selection and reviews do not approve a concrete tag, evidence digests, residual risks or rollback target | After applicable gates pass, record the maintainer's exact source/tag/artifact/evidence/support/risk/rollback decision |

At that checkpoint these were mandatory deferred release conditions, not new product
vulnerabilities or authorization to execute activation work. The review request
is complete with this rejection of release readiness. No additional repair,
rerun, policy weakening or signing/publication was admitted in response to the
findings. [Followup disposition](../artifacts/yt096-security-release-review-20261020/FOLLOWUP-DISPOSITION.json)
retains those authority boundaries. Public-control observations remain bound to
their historical records; no current entitlement or setting is invented.

## Preservation and next decision

All 743 prior evidence files, 63 selected source inputs and five canonical
private scan files were unchanged at review closeout. That checkpoint used
`feat/yt090-configuration`, HEAD `c0726d2`, with the new report/status updates
uncommitted; they were subsequently committed as `2118495` under separate
owner authorization. No runtime resource or new external operation was started. Signing,
key access, tags, publication, visibility changes and deployment were not
performed. The final documentation and preservation checks are recorded in
[FINAL-STATE.json](../artifacts/yt096-security-release-review-20261020/FINAL-STATE.json).

At the review checkpoint, the next decision needing no signing or publication
was the maintainer's explicit disposition of the support/inventory limits in
[the reconciliation](YT-096-FINAL-RECONCILIATION.md#residual-decisions-and-release-boundary).
Acceptance cannot create missing source evidence or erase Not verified claims.
Formal integrity/registry/public-activation execution remains separately
authorized work; final release approval follows the applicable completed gates.

## Subsequent owner support and inventory disposition

On 2026-10-20, after commit `2118495`, the owner explicitly accepted the
[recommended scope and known inventory differences](YT-096-FINAL-RECONCILIATION.md#owner-support-and-inventory-acceptance-2026-10-20)
and required missing evidence to remain Not verified. This resolves the
support/inventory finding's missing maintainer disposition only. Historical
Chrome exact-candidate binding and complete compiled/binary/system inventory
are still Not verified; native/wasm/Firefox evidence keeps its original scope.
Neither raw reviewer result nor historical evidence is rewritten.

The original final review remains FAIL / release-ready NO. Four conditions
still need their own authority and evidence: formal integrity/trust, actual
registry qualification, public activation/reporting controls and concrete
final maintainer release approval. This documentation acceptance does not
authorize signing, publication, tags, visibility changes or deployment, does
not weaken existing supply-chain policy, and does not mark YT-096 Done or
unblock YT-020. Independent review of this update is a bounded documentation
review, not a replacement final release verdict.

## Subsequent formal trust proposal

After commit `0bd7596`, the owner requested
[minimum trust/provenance research](YT-096-FORMAL-TRUST-PROPOSAL.md) and
independent documentation review, excluding signing/publication. The proposal
identifies conditional public Sigstore/GitHub OIDC identities, exact evidence
bindings and future operations. It does not select a service policy, disclose
signing metadata, authenticate a past builder or replace the original failed
release verdict. Formal trust and the other three remaining conditions are
still open; no new candidate, workflow or activation is approved here.
