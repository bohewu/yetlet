# YT-206 — Public source-preview disclosure preparation

Current status: 2026-10-27 (Asia/Taipei). **The repository is Public as an
Experimental source preview. Owner disclosure approval and the authorized
Merge/visibility/reporting setup are complete. The signed-in reporting form and
actual notification-delivery checks PASS. Independent completion Reviewer PASS,
zero issues and required followups, marks YT-206 Done at this selected scope;
no check is waived or moved to another item.**

## Reporting completion

The owner requested completion of both remaining checks, signed in to GitHub,
explicitly authorized one private non-vulnerability test and supplied an existing
non-admin reporter account. The test is clearly labelled as a reporting/delivery
check, with no affected package/version, real security impact, severity or CVE
assertion. No private advisory ID, mailbox address, reply token or mail body is
published here; exact bindings remain in the local evidence below.

| Check | Executed result | Scope |
|---|---|---|
| Authenticated private-reporting form | PASS: the non-admin reporter sees Report a vulnerability, the security-policy link, Title/Summary/Details/Proof of concept/Impact fields, private-maintainer visibility notice and Submit report; the browser submission creates one Triage record | Actual external-reporter flow, rather than the owner's draft-advisory interface; AI assistance disclosed |
| Account-level notification route | PASS: the owner's configured default email matches the connected Gmail mailbox; Watching selects Email and the repository subscription is subscribed/not ignored | Existing personal mailbox, no new domain or dedicated address; no Web inbox delivery claim |
| Actual maintainer notification delivery | PASS: GitHub's private-report email arrives21seconds after submission; a related collaborator notice arrives22seconds after submission | Matching advisory reference and approved test marker, Gmail delivery timestamps, exact Delivered-To recipient match and passing GitHub DKIM/SPF; one bounded sample, not a delivery SLA |
| Controlled-test closeout | PASS: the owner closes the single private test after receipt; authenticated API readback reports Closed with no publication, CVE or private fork | Closed record and evidence retained; it is not a real vulnerability or published advisory |

The report was created on2026-10-27 at20:25:29 Asia/Taipei; the two emails arrived
at20:25:50 and20:25:51, and the record was closed at20:29:04. The API/source
receipts retain the corresponding UTC timestamps. Only this newly authorized
reporting/delivery check executed; accepted package/browser/provider/load
qualification was not replayed. Independent completion Reviewer PASS, zero issues
and required followups, accepts the actual form, delivery and private-test closeout
evidence. Only this completion metadata follows review acceptance.

The following earlier outcomes remain recorded as failures or limits:

- The owner initially saw the admin draft-advisory form. A private-report API
  submission under that identity returned403; readback confirmed0reports and
  0notifications. The owner then supplied the non-admin reporter login. No
  duplicate report was created and no permissions were weakened.
- Attempts to add the optional On GitHub Watching channel did not persist in
  their readbacks. The existing Email route was preserved and is the route
  actually verified above; Web inbox delivery is not claimed.
- The first mail-binding assertion expected the user in the To header, but
  GitHub addressed a repository mailing list. One parser correction reused the
  same two messages and checked Gmail's Delivered-To envelope instead. It did
  not send another report or repeat the delivery test.

This completion supplies the previously missing evidence. It does not reinterpret
the earlier Not verified results as passes, waive notification delivery, create a
future backlog substitute or approve formal D1–D6 decisions.

## Post-activation status

This section records activation before the reporting execution above. Its
settings-only verification limits are retained as the earlier observation.

The owner explicitly accepted the inventoried source/history/PR/image and Actions
disclosure, preserving failed records, then authorized PR12 Merge, Public
visibility and GitHub Private Vulnerability Reporting. This subsequent authority
supersedes the pending disclosure/operation choices in the retained snapshot below.

- [PR12](https://github.com/bohewu/yetlet/pull/12) was squash-merged as
  `d7065bbfb5b733c418f65ef55324f4d0ce6ac4d6`. Its complete tree,
  `cca0153c3e56bc5e13e651a4a4bb1469dc042cfe`, exactly matches reviewed head
  `3d63f0bcaa12d23ab075bfd63e784c8d50f39cc4`. Both required jobs passed in
  [CI40497321153](https://github.com/bohewu/yetlet/actions/runs/40497321153), including
  RustSec and Chrome E2E. Accepted product qualification was not replayed.
- The nine-commit merge range passed strict/builtin screening with exit0,
  462file operations,0blocking findings and9non-blocking path warnings. Those
  warnings remain recorded; no policy downgrade or suppression was introduced.
- The [repository](https://github.com/bohewu/yetlet) is Public. The delivery branch
  is retained and automatic merged-branch deletion is disabled. Both required
  checks, administrator enforcement, force-push prohibition and collaborator-only
  PR creation were preserved. No history or failed-run deletion was performed.
- GitHub reports private vulnerability reporting `enabled=true`. The public
  [Advisories page](https://github.com/bohewu/yetlet/security/advisories) displays
  **Report a vulnerability**. This verifies enablement and the public entry,
  without claiming that the signed-in form was rendered or submitted.
- The maintainer's repository subscription reads `subscribed=true` and
  `ignored=false`. This is notification-setting evidence, not proof of GitHub
  inbox/email routing or delivery. A separate mailbox/domain remains optional.
- Immediately before activation, metadata listed39runs and16artifacts. Only
  new CI40497321153 logs and artifact12731428615 were read at the existing bounded
  marker scope;28log members and1artifact member had0personal-path/private-key
  marker matches. Earlier artifact12262540945 was no longer listed; the retained
  download and absence observation were preserved, without a deletion operation.
  The original38-run inventory below remains unchanged as historical evidence.
- The subsequently authorized GitHub About description now states:
  "Experimental HTML-first Rust UI framework with native SSR, optional browser
  WASM, and Axum/Tower integration. Source preview; packages unpublished."
  Repository topics are `rust`, `ui-framework`, `web-framework`, `html-first`,
  `ssr`, `webassembly`, `axum`, `tower` and `experimental`; these are discovery
  metadata, not Git version tags or release approval.

Technical C remains `118f3d0f496b487ef20fe243f4e73179f9eadc43`; no successor
candidate, release-artifact signing, version tag, package publication or reader
site export was selected or performed. YT-096 remains In progress, YT-020 Blocked,
and all formal D1–D6 decisions remain Pending.

## Remaining reporting verification

**Historical pending table:** These were the open checks at the3ecf6ff document
checkpoint. Both are now exercised in [Reporting completion](#reporting-completion);
the original Not verified wording and authorization boundary remain below.

| Check | Accepted evidence | Still Not verified |
|---|---|---|
| Signed-in private reporting form | API enablement and the public Report a vulnerability entry | The authenticated reporter can open the form and see its private-reporting fields/notice. The available browser session was signed out; page-control observations failed and crawler access to the form was refused by robots policy. No alternate form fetch or submission was used |
| Actual maintainer notification delivery | Repository watch is subscribed and not ignored | Delivery of a genuine private report to the intended GitHub inbox/email route, and any account-level email-routing preferences. No report was submitted, so no delivery or response-time claim is made |

The remaining form check can be a read-only observation in a signed-in GitHub
session; it does not require submitting a vulnerability. Delivery evidence can
come from a genuine report or a separately authorized controlled notification
check. This document does not authorize a fabricated vulnerability, outbound
message or test submission. Preserve any later failed result and distinguish
settings checks from actual delivery. YT-206 must not be marked Done from this
documentation reconciliation or the earlier inventory Reviewer PASS alone.

## Retained pre-activation snapshot

All remaining sections retain the original pre-approval inventory, recommendation
and settings observations. Their Private/Pending/no-external-mutation statements
describe that earlier phase, not the current repository state. Original counts,
failed checks, evidence identities and qualification limits are preserved.

Date: 2026-10-27 (Asia/Taipei). **Inventory and recommendation prepared;
owner disclosure/activation decisions remain Pending. GitHub private reporting
is selected; YT-206 is In progress.**

## Authority and recommendation

The owner accepts the proposed bounded public-content inventory. Prepare a
reviewable disclosure choice and necessary document corrections; do not infer
Commit, Push, Merge, visibility change, evidence deletion, history rewriting,
signing, tags or package publication authority.

The clean source is `778423e6c682e2a030f8946ddf11c9875bea5652`, tree
`06446ec680ee1e9558f8820f5a4eed50ad95d0eb`: the seven-document closeout commit
is local, not Pushed. Technical C remains immutable
`118f3d0f496b487ef20fe243f4e73179f9eadc43`. PR12 is open at C; default master
remains `71b5f084fdeedd3feade202740077fc199f950ae`.
[YT-205](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md) remains Done; no accepted
product/package/browser/provider/load qualification is replayed.

**Recommend the existing repository as an Experimental source preview, retaining
normal history and truthful evidence, if the owner accepts O1–O3 below.**
The observed metadata does not justify automatic history rewriting, removal of
failed runs or a replacement root/repository. A source preview does not promise
stable APIs, production support, trusted-release provenance or registry access.
Keep visibility Private until the disclosure and activation choices are settled;
verify the selected GitHub reporting form/notifications during authorized public
activation before announcing the preview. If historical/maintainer/Actions records must stay private, retain this
repository Private while separately choosing a publication route. Moving tracked
files to an internal directory or hiding links does not privatize their history.

## Concrete inventory

| Surface | Observed result and exposure |
|---|---|
| Current tracked tree |1,191files;366 under docs, including236 YT task records. Publishing the repository exposes more than the46 selected reader manuscripts |
| Bounded current text/filename rules |0 personal/workspace absolute-path matches,0 private-key markers and0 credential-like filenames; no skipped over-2MiB blobs |
| Maintainer identifiers |129files/453matching lines contain runner/reviewer identifiers; ARCHITECTURE and SAMPLES retain historical annotations among selected reader sources |
| Private-address matches |3files/12lines; inspected matches are synthetic address-admission/upstream parsing tests, not live deployment inventory |
| Remote reachability |20advertised refs,17unique tips,114reachable commits,2,177unique blobs/38,476,193bytes; includes5branches and PR refs |
| Historical workspace paths |8unique blob versions at6representative document paths retain owner workspace information; current cleanup does not remove historical disclosure |
| Git contact metadata |All114author addresses use GitHub noreply; committers use102noreply and12GitHub service addresses, with no additional personal-email finding |
| Existing binaries |All15current/historical binary blobs are PNGs, visually inspected: synthetic Lab data/identities, some loopback browser origin and IME/desktop chrome |
| Actions runs/logs |38runs:32success/6failure. All retained log sets read at bounded marker scope:36downloaded/2reused from accepted YT-205 records,11,742,364 unfolded bytes including provider duplicates |
| Actions artifacts |16unexpired artifacts:15archives downloaded/read, current C rebuild reused.49members inspected in the15 archives, including9historical source packages |
| PRs/comments |12PR bodies inspected,2open/10closed;0issue comments/0inline review comments. No private-path/private-key-marker match in those bodies |
| Tags/releases/forks |0tags,0GitHub releases,0listed forks; no release activation implied |

The six historical documents are M10G-VERIFICATION.md,
M9-PLANNING-VERIFICATION.md, YT-066-PERSISTENCE-RECOVERY-PLAN.md,
YT-066B1-VERIFICATION.md, YT-066B2-VERIFICATION.md and
YT-066B3-VERIFICATION.md under docs. Private evidence gives blob IDs/lines without
repeating raw workspace values. History counts use unique blobs and one
representative path per object, not every historical occurrence. Unadvertised or
dangling provider objects and unrelated account/project contents are not certified.

No nonstandard user-directory/private-key-marker matches occur in the inspected
Actions log text;75members contain ordinary runner/build identity. Three apparent
artifact user-path matches are standard hosted macOS runner directories. No
private-key marker appears in the inspected artifact text. These limited patterns
are not complete credential scanning, credential validation or independent
publication certification. Operational identifiers are not credentials merely
because they exist. Existing synthetic screenshots are not new browser/IME proof.

The nine nested historical source archives keep their original identity and are
not requalified here. Current C's artifact/sixteen archives use accepted YT-205
input/review bindings. Raw downloaded evidence stays ignored/private locally.
The Windows ZIP denied in YT-205 is not reopened, moved, reconstructed or retried.

## Recorded pre-activation settings and reporting requirement

- Visibility is Private; Pages/Discussions are disabled. Wiki/Projects feature
  flags are enabled; no successful Wiki content-ref read is observed. Feature
  flags do not prove content exists or certify an empty Wiki/project inventory.
- Pull requests are enabled with actual `pull_request_creation_policy` set to
  `collaborators_only`, agreeing with [CONTRIBUTING](../CONTRIBUTING.md) and
  GitHub's [collaborator-only setting](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/disabling-pull-requests).
- Master requires `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`, with
  administrator enforcement enabled and force pushes disabled;0repository
  rulesets listed. No setting/protection is changed by this inventory.
- The owner selects GitHub Private Vulnerability Reporting as sufficient for
  this source preview. [SECURITY.md](../SECURITY.md#reporting-a-vulnerability)
  now makes a separate mailbox/domain optional. The earlier inventory's mandatory
  fallback/contact request is superseded by that choice; no email or domain
  purchase is needed to complete the selected reporting setup. The mentioned
  tunnel domain is not inspected or reconfigured. No test report/outbound message
  is sent.
- The Private Vulnerability Reporting read returns404 while Private; do not
  interpret that as permanent disablement. GitHub documents
  [enablement for public repositories](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository).
  Actual enablement/form/notification verification belongs to authorized activation;
  do not require the public-only feature to be enabled before the visibility change.

## Owner choices and minimal delivery before approval

| Choice | Recommendation | State |
|---|---|---|
| O1 — Source/history/PR/image disclosure | Retain normal history and accept the identified historical workspace paths, operational identifiers and synthetic screenshots as public source-preview context | Pending |
| O2 — Existing Actions disclosure | Retain38runs, including6failures, and16available artifacts with true source/experimental limits; do not delete evidence just to hide failed results | Pending |
| O3 — Reporting and launch | Use GitHub Private Vulnerability Reporting; explicitly approve the final source/ref and delivery/Merge/visibility/reporting operations, then verify the form and maintainer notifications during activation before announcing the preview | GitHub channel selected; operations/verification Pending |

These preview choices do not resolve formal [D1–D6](YT-096-TRUST-DISCLOSURE-DECISIONS.md#pending-owner-decisions)
or consent to Sigstore/public signing payloads; YT-096 stays In progress and
YT-020 Blocked. Visibility approval must acknowledge that
[Actions history/logs become public](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility)
and signed-in repository readers can
[download retained artifacts](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts).
A private workflow/artifact name does not impose separate access control.

After O1–O3: obtain explicit strict/builtin Commit/Push authority for the reviewed
documents, observe existing required CI, obtain PR12 Merge authority, then perform
separately authorized visibility/reporting activation. Reobserve refs, run/artifact
IDs and settings immediately before activation and inspect only new/changed
inputs. Preserve both required checks and collaborator-only PR creation. Do not
automatically select a successor C, tag/sign/publish or change formal release policy.

## Evidence and document verification

Private records are under
`target/yetlet-public-preview-inventory-20261027-yot7fza7/`: exact source/remote
inventories, contextual disclosure locations, screenshot observations, all-run
log/artifact inventories and activation-setting observations. Downloads remain
ignored; original outcomes and qualification limits are preserved.

Initial gh2.46 parsing rejects unsupported --slurp before HTTP dispatch. Preserve
those failures; one canonical correction uses supported --paginate plus paged
JSON decoding. Wiki/PVR unavailable observations remain explicit, with no admin
recovery loop. No test/build/workflow rerun, external mutation or fresh runtime
qualification occurs. Run only changed-document links/structure/whitespace and
Backlog DAG checks. Inventory completion does not activate publication.

The later `evidence/activation/` records add owner authorization, actual merged
source/tree, pre/post settings, bounded new-CI disclosure observations, About
metadata, merge-guard acceptance and activation closeout. The initial overstrict
wrapper assertion on non-blocking Guard warnings is retained; it was corrected by
reading the same exit0 result, without rerunning the Guard or changing policy.
Browser/form failures and the unverified notification-delivery boundary remain
explicit. No old failure or qualification result is replaced. This documentation
reconciliation does not repeat product tests/builds/scans, submit a report, make
further GitHub changes or authorize Commit/Push. The separately authorized About
metadata update is recorded above.

The later `evidence/activation/reporting-completion/` records preserve the explicit
test authorization, owner403 refusal, authenticated reporter form/submission,
created and closed private-record API responses, redacted account-routing
observations, actual Gmail receipt bindings, mail-parser correction and browser
screenshots. Mailbox addresses/reply tokens stay private. The newly executed
checks do not authorize Commit/Push, artifact signing, package publication or a
change to D1–D6.
