# YT-096 — Alpha candidate and minimum hosted build/readback plan

Date: 2026-10-28 (Asia/Taipei). Status: Plan independently reviewed PASS, zero issues/required followups.
D6 Pending; YT-096 In progress; YT-020 Blocked. This packet authorizes no execution.

## Current operator checkpoint — used binding retired

On 2026-10-28, the owner separately authorizes the exact reviewed local
`v0.1.0` used-binding retirement and preparation of the next alpha steps.
The earlier plan/review statements below retain their original dates, scope
and acceptance results; their then-pending rollout gates are historical.
This checkpoint records execution of the approved metadata transition, not
a new independent review or authority for any later delivery/release action.

The published Guard release is now `0.45.0`, source
`140548a26ce88a85b79aef4c5262c33a3cea3dca`. Verified installation through
WebCodex is complete on the local Linux runner and both Windows runners.
The three installations report healthy global support; all twenty registered
workspace configurations remain unchanged, with fifteen healthy/current/eligible
profiles and five global-inheritance selections. No project hook/profile rewrite
was needed. Release and installation receipts remain in the owning Guard
repository's ignored `guard-v045-release-20261028` and
`guard-v045-runner-rollout-20261028` evidence packets. The installation packet
preserves original preflight/readback failures and advisory session flags;
only the actual corrected installation/readback checks are claimed PASS.

Before retirement, fresh local checks match the three installed Guard JavaScript
entry/lifecycle/hook files to that release commit. `doctor` reports owned hooks
and automatic strict/builtin policy PASS; the Yetlet workspace is healthy,
current and eligible. The active record still matches the reviewed SHA-256
`43494ed5f4c1c06c33567769e8ad49c210e019303c824a299485162061b7955e`,
`used` state and historical `v0.1.0` source. No fingerprint substitution occurs.

The installed `retire-remote-tag --binding-sha256 HASH --ack-retire-used
--format json` runs exactly once, without policy overrides, from this checkout.
It exits zero and reports `binding_state=retired`, `archived_state=used`,
strict/builtin and `coverage=metadata-only-no-push-approval`. The complete
327-byte record remains byte-identical in the private digest archive under
`.git/commit-guard-remote-tag-retired/HASH/binding/acceptance.json`; its state
remains `used`, and the active selection is absent. No raw endpoint or private
record payload is copied into documentation. All local refs, HEAD, hooks,
workspace profile and policy hashes match the pre-transition snapshot. The
worktree was clean at the transition. Existing unsigned `v0.1.0` remains at
`3eacfe31f5e3281411d3bc4653a6baccab668b30`; historical hosted/custody evidence
is untouched. No new acceptance is armed and no Push/native-hook execution is
claimed. Retirement makes no remote calls and grants no Push authority.

### Alpha source and next delivery order

| Binding | Current observation or next gate |
|---|---|
| Owner-selected version candidate C | `5d7b91842f8188d45e5af22367245aebec7ca7d1`; sixteen Experimental `0.9.0-alpha.1` libraries |
| Reviewed fixed-binding implementation | Local `9ee7f98767851c9a3d7357f5a68446d539cd3df9`; existing affected-test and independent review results reused |
| Local checkout before these notes | `e8be2319585b48bf01f476a0f93892bc9ff73cb2`, branch `codex/yt096-hosted-custody` |
| Live `origin` master observation | `3eacfe31f5e3281411d3bc4653a6baccab668b30` |
| Live old source branch observation | `9b7638c3d5b1557a225dc0887c30f0ff10b68241` |
| History relationship | Live master is not an ancestor of the local checkout; its prior integration was squashed. Nine local commits are not reachable from live master; tree comparison has 78 changed paths before these notes |
| Alpha execution C | Pending explicit selection after source integration and required CI; neither the selected version C nor local HEAD is silently substituted |
| Alpha tag | Absent in the bounded read-only `origin` observation; no local/remote tag created, no acceptance prepared |

The remote query is a dated read-only fetch-endpoint observation, not proof of
the effective Push endpoint, an accepted history baseline or future tag absence.
Fresh exact destination/ref checks and strict/builtin scanning remain necessary
before any separately authorized delivery.

1. **Deliver reviewed source first.** After separate Commit/Push/PR authority,
   retain the existing local branch and use the freshly accepted remote master
   as the base for a normal integration branch/PR. Reconcile the reviewed tree
   explicitly; do not force-push, assume fast-forward ancestry or rewrite the
   historical branch. The new integration branch's historical base must be
   explicitly accepted under Guard's existing rules. Scan its exact outgoing
   range with strict/builtin and retain the normal hooks and required CI. Merge
   requires separate authority; compare the actual merged SHA/tree with the
   reviewed content and explicitly select `C_release_alpha` afterward.
2. **Prepare the readback policy.** Separately authorize the exact replacement
   of the existing single tag policy with `v0.9.0-alpha.1` in
   `release-evidence-verify`. Keep the reviewer, bypass policy, existing
   `AGE_IDENTITY` Secret and pinned public recipient. Live settings/Secret
   availability are not rechecked or changed by this checkpoint; never add a
   second tag policy, read the key or rotate it implicitly.
3. **Authorize the alpha tag and one build.** Once execution C, ancestry and
   policy are concrete, obtain separate authority for the immutable unsigned
   `v0.9.0-alpha.1`, fresh exact new-tag history acceptance and normal guarded
   Push. Then authorize one manual `release.yml` build with the full selected
   `expected_commit`, `cohort=sixteen` and 45-minute job cap. No tag or hosted
   operation occurs now; failures must remain and no automatic retry follows.
4. **Read back and retain new evidence.** After a successful complete capture,
   separately authorize the same-source verifier using actual run/attempt,
   artifact ID and ZIP digest. The owner approves the real Environment gate.
   Verify the sixteen alpha subjects, public18 inventory, ciphertext and private
   manifest, then retain the distinct alpha packet outside the repository before
   transport expiry. Reuse the accepted unchanged product/platform/security
   evidence with its original scope; do not relabel the old `0.1.0` payloads.

The separate signer source S, both frozen alpha payload hashes, D6 signing
approval and publication remain later gates. No Commit/Push, Environment/Secret
change, new tag, hosted execution, signing or publication accompanies this
checkpoint. D6 Pending; YT-096 In progress; YT-020 Blocked.

Local receipts are in ignored `target/yt096-used-binding-retirement-20261028/`:
preflight/tool/profile/doctor checks, before-ref snapshot, exact command output,
archive-byte/ref/hook preservation proof and bounded remote/source observations.
No old functional, native-hook fixture, packaging, platform, browser, security,
hosted or custody validation is replayed.

## Guard version, release, installation and actual retirement plan

This is planning only. The owner authorizes the two reviewed implementation
commits below, not their Push, a version edit, a release, installation, hook
changes, actual binding retirement or Yetlet hosted execution. D6 remains Pending.

| Completed local commit | Binding and Guard result |
|---|---|
| Yetlet eleven paths | `9ee7f98767851c9a3d7357f5a68446d539cd3df9`, tree `6009f8cfc983d706a7ba00e22cafb3e7a91ed9a0`; staged strict/builtin and native hook PASS, zero findings |
| agents_pipeline five paths | `f1decf7c966bddb4ecf1c587dddb6d9cb6e2ac0f`, tree `00d41063fffdfaa4f3dfd8f45d10aeea4fe3e3c3`; explicit staged strict/builtin exit0, zero blocking findings and four existing generic Windows-drive warnings |

Both worktrees were clean after their commits; neither commit is Pushed. The
Guard checkout has no local pre-commit hook, so no native-hook PASS is claimed
there and no hook was added. Its first commit attempt stopped for missing Git
identity; the successful retry uses the same configured owner identity as the
Yetlet commit through per-command options only. No Git configuration is changed;
the failure, corrected invocation and fresh Guard receipt are retained.

### 1. Prepare the next Guard version

Recommend **agents_pipeline `0.45.0` / `v0.45.0`** for the additive public
`retire-remote-tag` command. This is a proposed version, not an edited file or tag.
Follow that repository's `CONTRIBUTING.md` version contract: only `VERSION`, a
new `CHANGELOG.md` entry and the managed current-release block in `README.md`
need version preparation. Preserve the original `0.44.0` release and history.
Use its existing `python3 scripts/sync-readme-version.py` and `--check`, plus
whitespace/version metadata checks and independent review. Reuse the four
already accepted retirement tests; no extra local functional suite or harness.

A separately authorized version commit needs a fresh strict/builtin staged
Guard check, matching `tree_oid` and exact three-file contents. The resulting
full immutable Guard release SHA, called R here, is Pending and differs from
feature commit `f1decf7`; do not invent it or select an unreviewed current tip.

### 2. Publish through the existing Guard release workflow

After explicit agents_pipeline publication authority, freshly inspect the live
remote, allowed branch/protection path, exact outgoing base and proposed tag
absence. Scan the exact outgoing range with strict/builtin before the permitted
branch/tag Push. No force, moved tag, protection change or historical-scan bypass.
Release R must contain the reviewed feature and version metadata. The actual
release tag must resolve to R and remain immutable throughout publication.

Use the existing tag-triggered `release.yml` once. It runs the required reusable
cross-platform/minimum-runtime CI, creates tar/zip bundles and checksum text,
issues GitHub OIDC build-provenance attestations, rechecks the tag, verifies
checksums/attestations and creates the GitHub Release/assets. These are public
**agents_pipeline release effects**, not Yetlet signing/D6 authority. No extra
manual dispatch is needed after the normal tag trigger. Preserve failed runs;
no automatic rerun or asset replacement, since the workflow's existing rerun
path can overwrite release assets. A failure needs a separately reviewed next step.

The future assets are `agents-pipeline-bundle-v0.45.0.tar.gz`, the matching `.zip`
and `.SHA256SUMS.txt`. Require actual successful jobs bound to R and that tag;
old `0.44.0` CI/results cannot substitute. Existing required release CI is a
new-source release gate, not authorization for extra repeated local tests.

### 3. Verify the release and install only the current local Codex runtime

Recommend reusing the accepted `0.44.0` verification model: the release job
cryptographically verifies the exact archive attestations before publication;
local readback matches downloaded digests, verified subjects, exact source R,
signer workflow, immutable tag, GitHub-hosted runner identity and actual release
run to that successful verification record. Also compare selected extracted
source/tool files with R before executing the installer. Record each binding.
Published checksum text alone is insufficient. If the verified hosted subjects
and source/run correspondence cannot be established, stop before installation.
This model is hosted cryptographic verification plus local correspondence,
**not independent local cryptographic verification**.

The previous `0.44.0` packet explicitly records that distinction. Current default
`gh` 2.46.0 still has no `attestation` command; the bootstrap script can silently
skip its provenance step in that situation. Do not use that checksum-only
bootstrap path as verification. Use explicit verified download/extraction and
the extracted release installer after the correspondence checks above. Optional
independent local verification would require a separately approved, pinned,
attestation-capable task-local CLI; no system upgrade is required by this plan.

After separately authorized download/installation, preview the same verified
bundle with `scripts/install-codex.sh --target <current Codex home> --dry-run`,
then perform that exact installation with default backups. This updates global
support/tools, generated roles/configuration/guidance and managed user skills;
it does not install Codex. Preserve the prior verified `0.44.0` bundle as rollback
input. Default backups do not retain the entire old support tree after success;
do not claim they provide that retained snapshot. No install is performed here.

The Guard feature does not change `hooks.js`, which is byte-identical to the
installed template. Existing Yetlet hooks use the stable installed Guard path.
All 56 premium/openai role-generation inputs match the installation after the
canonical support-reference rewrite; version-only profile differences are
explicitly accommodated by the existing profile manager. Therefore the minimum
rollout requires **no hook reinstall and no workspace profile set/clear**.

After installation, require matching installed release/tool hashes, readable
`retire-remote-tag` help, healthy global support and healthy/current/eligible
Yetlet workspace status with unchanged configuration identity. Run read-only
Guard `doctor`; compare actual owned hook hashes and strict/builtin automatic
policy with the preinstall snapshot. Doctor is structural evidence, not engine
execution or native Push proof. Unexpected differences stop for review; do not
refresh profiles or rewrite hooks automatically. Other runners are outside this
minimum local rollout and need separate scope.

### 4. Retire only the exact existing Yetlet used binding

The read-only retirement target observed for this plan is:

| Assertion | Required value |
|---|---|
| State / policy | `used`, strict/builtin |
| Historical accepted tag/source | `refs/tags/v0.1.0` / `3eacfe31f5e3281411d3bc4653a6baccab668b30` |
| Exact record-byte SHA-256 | `43494ed5f4c1c06c33567769e8ad49c210e019303c824a299485162061b7955e` |

The digest identifies the complete private record bytes, including its newline;
it is neither a source commit nor new Push authority. Raw record/endpoint values
are not exported. Actual retirement requires separate owner authorization tied
to this digest, installed reviewed code and a fresh unchanged-state check. A
changed digest/state stops; do not silently adopt a new fingerprint.

Using `GUARD_TOOL` for the absolute verified installed entry and
`EXPECTED_BINDING_SHA256` for the approved digest, the only proposed mutation is:

```sh
node "$GUARD_TOOL" retire-remote-tag \
  --binding-sha256 "$EXPECTED_BINDING_SHA256" --ack-retire-used --format json
```

Run from the Yetlet checkout only after approval. Do not add profile/engine
flags: this command rejects overrides and requires the actual automatic
strict/builtin policy and successful builtin initialization. Require a PASS
receipt with `binding_state=retired`, `archived_state=used`, matching digest and
`coverage=metadata-only-no-push-approval`. Verify exact original bytes remain
in the private digest archive and active selection is absent. Preserve the
archive; do not unlink, rewrite, rearm or replace it. No tag/ref/hook/policy or
remote operation is part of retirement. No new crash-durability claim follows.

For later source-delivery planning, inspect fresh remote boundaries and use an
exact reviewed outgoing range or ref-input for a read-only strict/builtin check.
That is scanner evidence only, not a real native-hook Push or permission to Push.
No real Push is needed to retest retirement: the four accepted disposable native
fixture tests already prove ordinary scanning resumes and new findings block.
Any future new-tag history acceptance requires its own fresh plan, remote proof
and explicit acceptance; no consumed authority is renewed here.

### Required owner authorizations

| Gate | Exact additional scope to approve |
|---|---|
| A — Version preparation | Adopt Guard `0.45.0`, edit the three version documents, run affected metadata checks and independent review; version commit requires explicit authority |
| B — Guard publication | Strict/builtin version commit and exact permitted agents_pipeline source/tag Push, required hosted CI, public release assets and OIDC provenance; stop on failure without auto-retry |
| C — Local verified installation | Release download/readback, current machine's global Codex installation and backup writes using the verified bundle; no hooks/profile changes or all-runner rollout |
| D — Exact actual retirement | The one digest-bound Yetlet local metadata transition above and archive verification; no Push/tag/hosted authority follows |

Recommend authorizing A first, keeping B-D as separate concrete gates. This
packet grants none of them. Optional new verifier acquisition is separate from
C if chosen. Yetlet source Push/Merge, execution-C selection, alpha tag,
Environment policy, build/readback, signing payloads, D6 and publication remain
outside this Guard rollout plan.

Independent Reviewer `/root/guard_retirement_rollout_plan_independent_review_20261028`
returns PASS, zero issues/required followups. It verifies all three document and
seven evidence hashes before/after, actual commit/tree/11-and-5-path receipts,
five unchanged protected hashes and all 56 canonical profile inputs. It inspects
the actual version/release/installer/profile/retirement contracts and retained
original `0.44.0` verification packet. No tests, builds, scans, network operations,
private-state mutations or gate execution occur during review. Native trace
verifies registered reviewer/model and actual high effort; the resolver's
observed decision is enforced without conflict or degradation.

Documentation checks PASS: 75 required documents, 207 backlog items; internal
links, Markdown structure, whitespace and dependency DAG, plus direct changed
links/fences/EOF and `git diff --check`. Only permitted three-document review/status
metadata changes after acceptance. Gates A-D remain unapproved and D6 Pending;
this plan PASS is not installation, retirement or release authorization.

## Local fixed-binding and Guard lifecycle implementation

The owner separately authorizes the minimum local implementation after the
nine-document plan commit `cd2502fb26577cf970ccc3c22f10c754218c44d6`.
That commit passed explicit staged strict/builtin Guard and the normal native
pre-commit hook, zero findings; its nine paths and tree match the guarded index.
No Push occurred. This checkpoint is implementation only, not hosted execution.
The selected alpha candidate remains `5d7b918`; no new execution C is selected.

The working-tree changes set the five fixed build/capture/readback seams to the
exact `v0.9.0-alpha.1` identity and update only affected fixtures. Source/event/
workflow/checkout/expected/tag equality, master ancestry, clean source, manual
entry points, read-only permissions, protected readback, timeouts, encryption
and bounded public/private allowlists remain required. Old `v0.1.0` evidence
retains its immutable source/helper identity; the new alpha helper must reject
old-tag bindings rather than reinterpret them.

The companion `agents_pipeline` source adds an explicit consumed-binding
retirement command. It requires the exact binding SHA-256 and explicit
`--ack-retire-used`, validates effective strict/builtin policy and the private
repository-bound record, and permits only `used` state. It archives the intact
private directory under a collision-resistant digest name using the existing
consume lock and an exclusive archive reservation, preserving the original
record bytes. Only then can ordinary native pre-push scanning resume. This
operation grants no remote-history acceptance, Push/retry authority or new tag.
Pending, malformed, mismatched, unsafe, locked or colliding state must fail
without retiring the active binding. Ordinary strict scanning still blocks new
findings after retirement; fresh tag acceptance remains a separate operation.

The reviewed source is now committed locally in the owning Guard repository at
`f1decf7`; this is not an installed-tool or hook update. The installed 0.44.0 tool, real managed hooks and real consumed
binding remain unchanged. No retirement command is executed against either real
repository. Later reviewed tool delivery/installation and explicit actual
retirement authority remain necessary before future Push. No Environment/Secret
or recipient change is made; the exact alpha Environment policy is still Pending.

Alpha affected checks PASS: nine focused Rust tests across seven offline/locked
Cargo commands, plus formatting, Python syntax and whitespace checks. They cover
alpha source/workflow/tag equality, explicit old-version and wrong-SHA refusal,
exact metadata/policy and public18/ciphertext workflow boundaries. The rotation
tool changes only its two synthetic binding strings to match the current helper;
its old age/key drill is not rerun. No package/platform/browser/security/custody
integration or hosted check is replayed. Native trace verifies the registered
implementation role/model and actual effort; the shared resolver observation is
enforced. Independent alpha and Guard implementation reviews both PASS with zero
issues/required followups. Four new Guard retirement tests PASS, including actual
native Git hook invocations against disposable local repositories: clean outgoing
Push succeeds after explicit retirement; a new outgoing credential is rejected
and the disposable remote stays at the prior clean commit. Refusal cases preserve
active records; archived bytes remain identical and fresh acceptance stays
separately explicit. Only these new Guard tests run; all disposable fixtures are
cleaned. No actual repository metadata, hook or installed-tool operation occurs.
Earlier plan/package and historical proof keep their original source/scope labels.

The independent alpha review verifies eleven source and five evidence bindings,
141 protected files and five real hook/binding/installed-tool hashes, before/after;
its task is `/root/yt096_alpha_fixed_bindings_independent_review_20261028`.
The separate Guard review verifies five source and two evidence bindings and its
focused log; task `/root/guard_remote_tag_retirement_independent_review_20261028`.
Both reviews are read-only and replay no tests/builds/scans or network operations.
Native traces verify both registered reviewers, models and actual high effort;
shared resolver observations are enforced, without conflict or degradation.
Only permitted three-document status/review metadata changes after acceptance;
all eight Yetlet implementation files and all five Guard files remain byte-identical.

Evidence labels: `target/yt096-alpha-bindings-20261028/` contains the alpha command
logs, preservation checks, immutable review snapshots, trace/resolver observations
and acceptance receipts. Guard source/test hashes and native fixture proof are in
its ignored `.pipeline-output/guard-retirement-20261028/` evidence directory.
Documentation checks PASS (75 required documents, 207 backlog items), including
links/Markdown/whitespace/DAG; no earlier functional qualification is repeated.
The real installed tool still has no retirement command; actual deployment and
used-binding retirement remain Pending, alongside the alpha Environment policy,
execution-source selection, hosted run/readback, exact payloads and D6.

## Selected candidate and retained evidence

The owner explicitly authorizes the strict/builtin commit of the 59 reviewed
version-migration paths, then selects that resulting commit as the new candidate C.

| Binding | Selected or proposed value |
|---|---|
| Selected immutable alpha C | `5d7b91842f8188d45e5af22367245aebec7ca7d1` |
| C tree | `6adff26fc4fec65e9aa8f40d7172ff266b7e500d` |
| Parent | `edfb0d68b947b5c571aef94d0af6ac7b942683fb` |
| Local branch | `codex/yt096-hosted-custody` |
| Product scope | Sixteen Experimental `0.9.0-alpha.1` libraries; API/features/MSRV unchanged |
| Commit checks | Explicit staged strict/builtin and normal native pre-commit hook PASS, 59 paths, zero findings; committed tree equals Guard `tree_oid` |
| Proposed future source tag | Immutable unsigned lightweight `v0.9.0-alpha.1`; not created or authorized |
| Future executable release source | Pending after the required identity-binding patch and separate source selection |
| Historical source/tag | `3eacfe31f5e3281411d3bc4653a6baccab668b30` / existing immutable unsigned `v0.1.0`; unchanged |

C is selected now, not silently substituted with a future patch or squash commit.
The [local alpha preflight](YT-096-ALPHA-VERSION-PREFLIGHT.md) and its independent
PASS remain the accepted version/package evidence. Selecting C does not convert
those dirty-tree preflight archives into clean immutable-C archives or qualify
new hosted execution. Exact new output bytes, provenance and custody must bind
the actual future build source/run. No new product test, platform run or scan is
performed by this plan.

## Two blockers before executable hosted preparation

### Fixed version identities require a source change

C deliberately preserves the old hosted identity gates. The new archive names
are alpha, but the build, capture and readback still require the old tag/ref.
Running the alpha tag at C would fail closed. The smallest later implementation
changes only the fixed version-specific identities and their focused fixtures:

| Existing seam | Required later delta |
|---|---|
| `.github/workflows/release.yml` | Exact event/ref/workflow/tag checks for `v0.9.0-alpha.1`; retain manual-only dispatch, sixteen cohort, source/workflow equality, read-only permissions and 45-minute job cap |
| `xtask/src/release/preparation.rs` | Matching fixed tag/workflow/source checks and provenance ref; retain exact sixteen archive/public allowlist and source ancestry/cleanliness |
| `xtask/src/release/custody.rs` | Matching fixed alpha capture/verify binding; keep encryption, bounded inventory and original-exit handling |
| `.github/workflows/evidence-verify.yml` | Matching alpha verifier/ref/tag checks; keep preflight before secret job, same-source workflow equality, protected isolated readback and cleanup |
| `scripts/release-evidence-preflight.py` | Exact alpha build branch/path and Environment tag-policy checks; keep original run/attempt/artifact/digest and repository checks |

Do not generalize these gates to arbitrary tags, accept an old workflow identity
for new code, move `v0.1.0` or weaken a failed check. Only tests affected by these
fixed bindings need new execution. Existing packaging, runtime, native/browser,
security and custody behavior proofs remain source-bound without replay.

Any such patch changes C's tree and commit. Under the existing co-located build
contract, checkout HEAD = event SHA = workflow SHA = expected commit = peeled tag
target, with clean source and ancestry on `origin/master`. After separately
approved patch/review/delivery and required CI, the owner must explicitly select
the actual reviewed execution commit as the next C / `C_release_alpha`. A squash
merge's SHA must be recorded and compared with the reviewed tree; do not keep
claiming that `5d7b918` contains code it does not contain. This is a future
selection gate, not a selection performed here. The separate signer design does
not authorize splitting the build's source/workflow equality.

### Consumed tag Guard selection needs a supported disposition

Read-only local observation finds the prior one-use remote-tag acceptance in
`used` state. Installed Commit Guard 0.44.0's selected path fails closed for used
state, including later ordinary Push. Its documented CLI advertises no supported
retire/clear transition. This is a separate delivery prerequisite, not permission
to discard the record, rearm it, reinstall/disable hooks or force a Push.

Before future source delivery, obtain a reviewed supported operator/tool-lifecycle
disposition that preserves consumed-state evidence and strict/builtin native-hook
enforcement. If unavailable, stop before Push and address that narrow tool gap
under separate authority. This plan performs no metadata or hook mutation.

## Minimum proposed sequence and authority

1. **Local fixed-binding patch:** separately authorize the five seams above and
   affected focused fixtures. Independent Reviewer PASS before acceptance. No
   old successful functional/platform/security/custody suite is replayed.
2. **Source delivery and execution-source selection:** separately authorize
   Commit/Push/PR and Merge as needed; preserve existing branches/history and
   required checks. Scan the exact outgoing range and run the native strict/builtin
   hook after the consumed-selection prerequisite is resolved. Required new-source
   PR/master CI is distinct from replaying historical qualification. Record the
   actual source/tree and obtain explicit next-C selection after any squash.
3. **Protected readback policy:** freshly inspect the existing
   `release-evidence-verify` Environment. Recommend reusing its current matching
   `AGE_IDENTITY` Secret and pinned public recipient, with no rotation, key read,
   Secret duplication or new signing Environment. The current API gate requires
   exactly one tag policy; adding a second alpha policy would fail it. Separately
   authorize replacing that exact policy with `v0.9.0-alpha.1`, retaining reviewer
   `bohewu`, the single-owner self-review choice and administrator bypass disabled.
   Recheck actual settings before dispatch and again before key access. Historical
   old-tag executions retain their records; re-enabling an old-tag rerun would
   require separate settings authority. This plan does not inspect live settings
   or claim that a future Secret is available.
4. **Unsigned tag and one build:** after source/settings are concrete, separately
   authorize strict/builtin preparation and native-hook Push of only the immutable
   unsigned `v0.9.0-alpha.1` at the approved execution C. Freshly verify exact remote
   tag absence and accepted branch/source; no remote write or absence claim now.
   Then authorize one `release.yml` manual attempt with `expected_commit` equal to
   that full C, `cohort=sixteen`, 45-minute build-job cap. Preserve all failures;
   no automatic retry, tag move, certificate, signature, GitHub Release or publish.
5. **One readback:** after successful complete capture and encrypted upload,
   record actual build run/attempt, encrypted artifact ID and GitHub ZIP digest.
   Separately authorize one manual `evidence-verify.yml` at the same approved
   tag/source with exactly those inputs. Keep existing 5-minute preflight and
   15-minute protected-job caps. The owner approves the real Environment job;
   dispatch authority and a chat reply do not bypass that gate. Retain original
   failures, including any incomplete build/readback disposition; no auto-retry.
6. **Correspondence and retention:** before the seven-day transport expiry,
   preferably within 24 hours, verify the new public18 allowlist, sixteen actual
   alpha provenance subjects, source/workflow/run/lock/tool bindings, artifact
   ZIP digests and ciphertext/safe receipt. Readback must authenticate/decrypt and
   verify the private inventory/full manifest. Retain encrypted evidence, safe
   receipts, source/recipe inputs and a verified index/receipt outside the repo,
   under a distinct alpha source/run directory. Never overwrite the old custody
   directory. No local private-key/plaintext copy is needed. Persistent owner-host
   custody is not offsite backup, replication or indefinite availability proof.

Only a complete new-source build, complete encrypted custody and approved actual
readback/correspondence satisfy the alpha preparation checkpoint. Public upload
alone or a safe receipt alone cannot substitute for complete private custody.
Capture failure means NOT SAVED; runner loss has no guaranteed capture. Use the
existing ciphertext-only failure transport and original exit status, with no raw
logs, environment dumps or private plaintext fallback. The legacy private-rebuild
workflow's plaintext upload is not a substitute for this D2 boundary.

## Reuse and next signing boundary

Reuse unchanged library/API/feature, third-party-lock, platform/browser/security
and historical custody evidence with their original source/scope labels. New
package/provenance identities must be measured at the new actual build. No new
Windows, browser, GPU, benchmark, full security scan or unrelated dependency
upgrade follows from this version-only preparation plan.

Keep the [independent fixed-SHA signer contract](YT-096-SIGNER-CONTRACT.md) /
[ADR0009](adr/0009-release-build-and-signer-identities.md): later signer S is
separate from the actual build C. The alpha public provenance and complete private
evidence manifest both need their own frozen SHA values and binding receipts.
Old `0.1.0` payload hashes cannot be relabeled as alpha hashes. Exact S/tool/roots,
final evidence closure, Environment/custody authority, D6 signing approval and
actual publication remain separate Pending gates. No signature is issued here.

## Plan evidence and acceptance

Local commit/source and read-only binding observations are under ignored
`target/yt096-alpha-version-20261028/commit-20261028/` and
`target/yt096-alpha-hosted-plan-20261028/`. This is a proposed sequence, not a live
GitHub settings/CI attestation. No network environment, GitHub setting, Secret,
Git ref or Guard metadata changes accompany this documentation.

Documentation checks PASS: `cargo xtask check docs` reports 75 required documents
and 207 backlog items, with internal links, Markdown structure, whitespace and
DAG PASS. Direct nine-document links/fences/EOF and tracked/new-file whitespace
checks PASS. The new file's no-index exit 1 denotes its expected content difference
against /dev/null only with empty whitespace diagnostics.

Independent Reviewer `/root/yt096_alpha_hosted_plan_independent_review_20261028`
returns PASS, zero issues/required followups. It verifies all nine source and four
evidence hashes before/after review; HEAD/tree/status and staged-zero remain
unchanged. The actual commit has the 59 authorized paths, matching strict/builtin
staged and native-hook PASS0 receipts. Native trace verifies registered reviewer,
model and effective high effort; the shared resolver observation is enforced.
Review is read-only and replays no check/build/test/scan, network operation,
private-binding read or mutation. No persistent resource was created.

Only this permitted review/check closeout changes after PASS; the other eight
reviewed documents remain byte-identical. Consumed Guard state remains a local
observation; live GitHub settings, future execution source/output/custody and D6
are still unverified or Pending. Plan acceptance authorizes no implementation,
delivery, tag, hosted execution, key access, signing or publication.
