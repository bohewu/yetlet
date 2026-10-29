# YT-209 — Minimum hosted verification and Environment plan

Status: proposed operational plan; no new operational authority granted.

## Current disposition — 2026-10-29

Tool delivery and the protected `release-result-verify` Environment/Secret
installation are [complete](YT-209-RELEASE-AUTOMATION.md#owner-authorized-recovery-delivery-and-setup--2026-10-29).
PR21 closeout merged as `c2d4bd085682cf86748bdd3ff18d6ca9f26534fc`;
its required CI and automatic master CI `40884588178/1` PASS. The owner then
approved the [alpha.2 version/package preflight](ALPHA2-VERSION-PREFLIGHT.md),
without selecting C or authorizing a tag, hosted execution, signing or publication.
Positive schema-1 hosted result verification is still Pending. Alpha.1 D6 and
publication remain Complete. The inventory/proposals below preserve their dated
pre-installation observation; do not recreate the Environment or reinstall its
Secret merely because this historical plan proposed doing so.

## Recorded baseline — 2026-10-29

The seventeen reviewed YT-209 paths were committed as
`c70a7431226c307a097c218050348477de85671d`, tree
`55a1cd70d7a9cd89f8ad683e57564a105e7aa824`, after strict/builtin Guard and the
native hook passed. Commit content/tree match the approved set; no Push occurred.
[Local implementation acceptance](YT-209-RELEASE-AUTOMATION.md) remains Done.
This plan does not claim hosted qualification.

Read-only GitHub API inventory found:

| Surface | Observed state |
|---|---|
| Protected remote `master` | `d65e3180ad3076659f03c069fc71cd0e7e3f0518`; new YT-209 workflow is not delivered there |
| `release-sign` | Environment ID `25580949441`; owner reviewer bohewu, admin bypass disabled, exactly the `master` branch policy; `AGE_IDENTITY` name present |
| `release-evidence-verify` | Environment ID `25525950662`; same reviewer/bypass controls, exactly the `v0.9.0-alpha.1` tag policy; `AGE_IDENTITY` name present |
| `release-result-verify` | Absent from the complete two-Environment list; direct GET also returned404 |

Only Secret names/settings were read, never Secret values or the local formal
identity. Inventory is a dated observation, not a future preflight or continuing
authority. Refresh changed identities/policies before an authorized operation.
Local receipts are under `target/yt209-hosted-plan-20261029/` (ignored).

## Recommended minimum

First deliver the reviewed tooling and configure its new verification Environment.
Run the positive hosted sequence alongside the next actual release candidate;
do not invent another product release or re-sign the accepted alpha just to
exercise this tooling. A possible next version is `0.9.0-alpha.2`, **not selected
or authorized by this plan**. Its package/version changes require their own
review and affected checks before C is selected.

Two constraints prevent a misleading shortcut:

- Existing `v0.9.0-alpha.1` is immutable at product C
  `bf3363801132f4481263ceb411d9bba3349df86d`. Current tooling commit/version is
  not permission to move that tag or build another source under its name.
- The old signer S and actual prepare/execution receipts predate `record_sha256`.
  New `signing_input` and `execution` require that complete-record binding.
  Old receipts must not be edited or supplemented to make the new verifier pass.
  They remain valid historical evidence under their original implementation.

Consequently build/readback/prepare can be rehearsed without signing, but a
successful end-to-end **new signing-result verifier** needs actual new-format
signing results. Its positive verification stays Pending until a separately
approved real signing produces them. Do not dispatch a predictably incompatible
legacy input just to obtain a hosted failure; no legacy adapter is proposed here.

## Environment settings to approve

Create only `release-result-verify` after delivery of the reviewed workflow:

| Setting | Minimum selected proposal |
|---|---|
| Required reviewer | User bohewu, ID `35250002`; single-owner approval remains explicit |
| Prevent self-review | `false`, matching existing single-owner policy; the initiating owner can approve |
| Admin bypass | Disabled |
| Deployment branch policy | Custom policies, exactly one entry: branch `master`; no wildcard/tag allowance |
| Secret | Environment-scoped `AGE_IDENTITY`; reuse the existing recipient-compatible identity, no rotation |
| Workflow permissions | `contents: read`, `actions: read`; no OIDC/content write, signing or registry action |
| Secret exposure | Only the protected verification step, after authenticated tools and input preflight; runner-private files deleted on success/failure |

GitHub Secret storage is write-only: listing an existing Secret does not recover
its value for another Environment. Provision from approved existing owner custody
under explicit installation authority, without printing it or putting it in a
repo/PR/Actions input. This is Secret installation, not permission for local
private-evidence decryption or renewal of the spent alpha key-use exception.
Verify setting/name readback; only a later protected job can establish that the
installed key decrypts the selected evidence. An empty or wrong Secret is not
qualified merely because its name exists.

Keep `release-sign` unchanged. Keep the existing evidence-verifier tag policy
unchanged now. Once a new exact C/version/tag is approved, separately authorize
replacing its single tag policy with that exact new tag; do not append a second
policy or use `v*`, because the current metadata gate requires exactly one.
Capture before/after settings and historical policy/approval evidence. Policy
rollover must wait until all executions using the old tag are closed.

## Later hosted sequence and limits

No C, S, V, new tag, Draft or run is selected by this table. Each input packet
must first contain actual fixed identities/hashes. The schema-1 data record is
only the closed implemented subset; platform scope, tool pins and owner decisions
remain in the surrounding release dossier/trusted code, not invented JSON keys.

| Order | Operation and limit, once separately authorized | Required output / stop boundary |
|---|---|---|
| 1 | Normal tooling PR delivery and required CI; Merge only under its explicit authority | Actual merged tooling SHA/tree and CI; this delivery commit is not automatically new C or S |
| 2 | Create/configure new result Environment and provision its Secret; no dispatch | Exact settings and Secret-name readback; key use remains unqualified |
| 3 | Next real candidate: reviewed version/package migration, select C and unsigned immutable version tag; one `release.yml` invocation,45 minutes | Tag/C/workflow equality, exact sixteen source packages, public provenance/checksum and encrypted build evidence |
| 4 | Roll evidence Environment to the exact new tag; one `evidence-verify.yml` invocation, preflight5/protected15 minutes | Original build run/attempt/ZIP/artifact binding, actual owner gate, verified complete custody and durable receipt |
| 5 | Select separately reviewed S distinct from C, using a real reviewed follow-up commit; complete/freeze record and original evidence; one `release-sign.yml` prepare, preflight5/protected30 minutes | Actual frozen receipt/artifact, canonical record digest, closure and both raw payload hashes; no OIDC/signature |
| 6 | Present exact new payload hashes/disclosure for owner signing approval; one sign invocation, preflight5/protected30 minutes | Actual public/private bundles, original S/run/certificate claims and immutable execution receipt; no publication |
| 7 | One `release-result-verify.yml` invocation, preflight5/protected30 minutes | Both original bundles, all sixteen subjects/all manifest entries, record/receipt/certificate identities; cleanup then safe `RESULT-VERIFICATION.json` |
| 8 | Later public attachment staging/readback; publication remains a distinct owner decision | Exact Release ID, approved twenty-file plan/notes/owner acceptance, actual downloaded hashes; no private attachments |

The smallest V choice is **V = S**, if that reviewed commit already contains the
accepted independent workflow and remains protected-master HEAD after signing.
Independence comes from the separate no-OIDC job, protected Environment and
actual-output checks; it does not require an otherwise unnecessary code commit.
If master advances, explicitly review/select V rather than silently substituting
it. Keep master at S from its selection through prepare/sign. No empty source
commit, forced ref or history rewrite is needed to satisfy C/S separation.

The original-evidence/prepare/keyless phases remain distinct. Confirm an approved
Draft and encrypted qualification handoff are ready before prepare; its exact
Release ID/ciphertext, capability expiry and source must pass existing gates.
A stale handoff stops, never triggers automatic renewal or run retry. Protected
approval waiting is separate from job execution timeouts; the owner must approve
before relevant capability/retention deadlines. Each table invocation is at most
once, with no job/phase retry or artifact resurrection implied by this plan.
The execution ceilings total at most170 minutes across stages3–7; no simultaneous
whole-sequence dispatch is proposed, and CI/approval waiting is separate.

Retain original successful/failed runs, receipts, ciphertext and outside-repo
custody indexes before seven-day artifact expiry. Do not replay prior package,
browser, MSVC or cryptographic qualification for unchanged scope. New-source
workflow preparation and required CI provide their own exact-source evidence;
accepted old PASS labels do not transfer to new bytes. Cleanup failure is failure.

## Owner authorization packets

These are proposed future packets, not questions or granted authority:

1. **Tool delivery:** accepted live remote-history baseline, strict/builtin Commit
   where required, branch/Push/PR plus required CI; separately state whether
   successful CI also authorizes Merge. Preserve the original branch/history,
   no force Push or automatic master replacement. Current instruction grants no
   Push or new-plan Commit.
2. **Environment installation:** create/configure `release-result-verify` exactly
   as above; install the existing key into its `AGE_IDENTITY`, without rotation
   or hosted execution. This does not change the other two Environments.
3. **Candidate/build/readback/prepare:** actual version/cohort, full C/tag, evidence
   policy rollover, exact original run/artifact/hash packets, full S/record/Draft
   bindings, one-invocation limits and custody destination. A later bounded packet
   can combine ready stages; it cannot pre-approve unknown hashes or signing.
4. **Signing and independent verification:** after prepare, actual two payload
   hashes, complete-record/receipt digest, public metadata disclosure, exact S/V,
   actual result inputs and attempt limits. Owner personally approves each
   protected Environment gate; never automate those clicks. Signing needs its
   own concrete approval; proposed verifier authority alone cannot issue a signature.
5. **Public writes:** exact Draft/Release ID, public plan, notes and output
   acceptance, staging/cleanup operations; publication and Latest/pre-release
   choice explicitly stated. No crates.io authority follows.

Recommended next approval is tooling delivery, followed by the new Environment
installation. Full hosted execution waits for the next actual candidate and its
concrete packet. The accepted alpha D6 remains Complete and its public release
unchanged; it is not reopened or treated as approval of a future version.
