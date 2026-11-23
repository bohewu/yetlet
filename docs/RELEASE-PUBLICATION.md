# Single-gate release automation and registry promotion

Current distribution checkpoint, 2026-11-12:
**Published SDK:** `0.9.0-alpha.8` (`yetlet` / `yetlet-browser`).
**Published CLI:** `0.4.1` (`yetlet-cli`).
The [SDK publication record](ALPHA8-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-12)
and [CLI publication record](CLI-041-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-17)
retain separate GitHub/crates.io archives, actual signatures and consumers. The
CLI resolves the latest available non-yanked SDK per project by default and writes exact pins/locks;
its build SDK/snapshot remain separate fixed release inputs. The public Reader
Edition follows the verified published pair selected in `site/pages.toml`; its
reviewed update and Pages readback are required for each publication closeout. Earlier release records retain their
original bindings and failures. [RELEASING.md](RELEASING.md) is the general SOP;
The original YT-227 proposal considered crates.io OIDC Trusted Publishing,
but it has **not** been adopted or activated. Existing registry-token
ownership, protected Environments and ADR 0009 remain unchanged; a future
OIDC policy would need new approval and actual credential/issuer proof.
The original version plan is recoverable at fixed local Git `ad564ad1ef07e5fca063ae88587c0f21e6edb5b1`.
## Independent CLI cohort — YT-215

[CLI first-release preparation](COMPATIBILITY.md#historical-first-cli-releases-and-consumer-failures) adds closed schema3/cli:
only yetlet-cli, version from its own manifest, tag yetlet-cli-vVERSION, one source
archive and five public files. Its dependency on the published SDK is not another
release subject or upload. The existing source-bound builder, signer identity,
separate verifier/custody and final owner gate apply. The final consumer installs
the executable with its packaged lock. Existing SDK schemas/outputs remain
unchanged. [CLI0.1.0 is published](COMPATIBILITY.md#historical-first-cli-releases-and-consumer-failures),
with successful installation but failed exact version-output validation.
[CLI0.1.1's completed publication](COMPATIBILITY.md#historical-first-cli-releases-and-consumer-failures)
repairs that bounded interface: install, exact version, help and scaffold pass.
It does not replace0.1.0 or re-publish SDK; every future CLI cohort retains its
own final owner gate.

## Historical cohort planning boundary — alpha.5

The [two-package plan](ALPHA5-RELEASE-PLAN.md) uses schema2/`consolidated`, exactly
`yetlet` followed by `yetlet-browser`, unreleased `0.9.0-alpha.5`. Existing
helpers derive the subject/asset sets from that validated cohort. Alpha.4's
schema1/sixteen execution and the original one-plus-fifteen continuation below
remain historical; the continuation is paused and invalid for schema2.
There is no new C/tag, hosted execution, signature or publication approval here.

## Owner-selected policy

The owner requested that the assistant handle preparation, signing and
verification, with human intervention only for final publication. The final
request covers **GitHub Experimental pre-release, followed by crates.io for the
same frozen archives in the selected closed cohort** (sixteen for schema1;
two for schema2). Git delivery still uses explicit authority and
strict/builtin Guard. This local work does not execute any hosted stage, access
a registry token, change an Environment or publish anything.

The assistant orchestrates the existing explicit dispatches; push/PR events do
not sign or publish. Keep exact C/S/V/P identities, reviewed scope, normal CI,
bounded attempts, private evidence retention and independent review. The
workflow's committed `RELEASE_AUTOMATION=single-publication-v1` selects this
policy; request JSON cannot choose a reviewer bypass. Old tagged workflows and
legacy requests retain their original interface and approval rules.

Signing still creates public certificate/identity and transparency-log metadata
**before** publication. The standing signing authority accepts only the already
reviewed [D1/D2 disclosure fields](RELEASE-SUPPLY-CHAIN.md#adopted-release-trust-and-disclosure-policy--yt-096); new fields,
identities or custody policies need another decision. No private evidence is
promoted by this change, and an unsuccessful publication cannot retract a log.

## Separation of privileges

| Stage | Environment / authority | Human gate under the new policy |
|---|---|---|
| Unsigned build at C | Existing `release.yml`, no OIDC/signature or publishing token | None |
| Original readback | `release-evidence-verify`, existing age identity, selected tag only, read-only | None |
| Frozen preparation at S | `release-sign`, existing age identity, master only, no OIDC | None |
| Sign exact frozen payloads at S | `release-sign`, master only; identity removed before OIDC; no registry token | None |
| Independent result verification at V | `release-result-verify`, existing age identity, master only; no OIDC or publishing writes | None |
| Publication preflight at P | No Environment Secret, read-only CI/artifact/public-tag checks; no Draft read | None |
| GitHub then crates.io at P | `release-publish`, master only; GitHub contents write and `CRATES_IO_TOKEN`, no OIDC or age identity | **Owner `bohewu` approves this exact run** |

S differs from C; V and P may equal S or be later reviewed commits. Live protected
master must equal the executing S/V/P. The helpers require successful exact-source
master CI, including both actual MSRV and Stable/Chrome jobs, before automatic
signer/result/publisher execution. Independent verification remains a separate
job without signing authority; removing repeated owner clicks does not merge it
into the signer. Every protected Environment retains disabled admin bypass and
its exact ref policy. See the [ADR amendment](adr/0009-release-build-and-signer-identities.md#single-publication-policy-amendment--2026-10-30).

## One-time rollout, after local acceptance

These changes are **not installed** by editing this document:

1. Obtain Git delivery authority, deliver the reviewed code via PR/green CI and
   Merge. Select the next version/candidate R/C separately; the old alpha.3 tag
   does not contain this source-cycle correction or automation implementation.
2. Retain existing `AGE_IDENTITY` secrets in the three worker Environments.
   Remove their required-reviewer rules only for this reviewed policy rollout.
   Keep `release-evidence-verify` restricted to the exact selected new-version
   tag; keep `release-sign` and `release-result-verify` restricted to master.
   Keep custom ref policies and `can_admins_bypass=false`. Runtime preflight
   rejects the old reviewer configuration in automation mode instead of hanging
   behind an unexpected gate. Do not copy an age identity to publication.
3. Create `release-publish`: sole required reviewer User `bohewu` ID35250002,
   `prevent_self_review=false`, admin bypass disabled, exact master branch only.
   Store a publish-authorized, **90-day** crates.io token as its Environment
   Secret `CRATES_IO_TOKEN`. Restrict scope to the selected cohort's names,
   including first-name creation authority where required. No repository-wide
   publish token. The workflow exposes it as `CARGO_REGISTRY_TOKEN` only to the
   final publishing step. Expiry needs renewal before the next release; never
   print it, pass it as an argument or include it in an artifact.
4. Confirm crates.io owner/email/name permissions before a live request. The
   owner's existing local `cargo login` is not proof of token scopes, identity or
   CI installation. No credential file is imported automatically. Later
   Trusted Publishing is a separate decision after first-name ownership exists.

Configuration readback and a successful new hosted execution remain necessary
to qualify this path. Local mock tests do not establish them. No live token is
needed for local protocol tests.

## Each version

1. Select version/scope and complete only affected qualification; reconcile
   existing evidence. Complete the [release documentation gate](RELEASING.md#release-documentation-gate)
   and semantic reader-document review before freezing C or packaging. Package
   READMEs describe the release without premature upload claims or stale pending
   text; actual status is reconciled from receipts after publication.
   Merge the reviewed source and confirm master CI. Select
   immutable C, then use strict/builtin Guard and the existing authorized tag
   procedure. Never move a tag or reuse an accepted version for changed bytes.
2. Dispatch one bounded unsigned build and original readback using the existing
   versioned release record. The recipe has one vendor step and one normal
   verified package step per selected library, in all-kind dependency order
   (two package steps for schema2, sixteen for historical schema1).
   Each predecessor is staged with its **actual `.crate` digest**; no bootstrap
   archive, `--no-verify` or null dependency checksum. Original nine-package
   historical recipes remain unchanged.
3. Preserve evidence outside transient storage. Select S, dispatch prepare,
   freeze both raw payload hashes and retain the actual frozen receipt. Generate
   the existing age-encrypted short-lived handoff only immediately before the
   relevant job. Remove owned temporary handoff assets after verified retention;
   the final Draft must contain no unapproved extra file.
4. Dispatch sign with `d6="reviewed-automation"`, the exact frozen hashes and
   static automation mode. This is standing policy authority, not a claim that
   the owner manually approved that run. Dispatch independent result readback at
   V. Keep all original failures; no automatic write/job retries.
5. The independent verifier exports `verified-publication-RUN-ATTEMPT` only after
   verifying both bundles, the exact selected subjects and the full private
   manifest, and cleaning private work. It contains five allowlisted public
   files for schema2 (nineteen for schema1), plus `RESULT-VERIFICATION.json`.
   Private manifest/bundle/evidence remain excluded.
6. Freeze the publication request and read-only registry plan. The assistant
   obtains full P/V/run/attempt/artifact IDs and digests from actual results, the
   raw verification receipt SHA, notes/name/notes SHA and registry-plan SHA.
   `PUBLICATION_REQUEST` is the closed schema in
   [release-publish.py](../scripts/release-publish.py); use the same complete
   `RELEASE_RECORD` accepted by the verifier. Derive the plan from those exact
   archives with `release_registry.prepare(record, directory, subjects)` and
   `release_record.digest(plan)`. Do not use historical alpha.3 archives: their
   Config/Secrets development cycle correctly fails first-publication planning.
7. Dispatch [release-publish.yml](../.github/workflows/release-publish.yml) at P.
   Its no-secret preflight re-derives both plans and the joined approval from the
   original successful verifier artifact, checks CI and the existing immutable
   tag, and presents the exact request/plan hashes in the job
   summary. **Only now does the owner approve `release-publish`.**
8. The approved job repeats source/CI/Environment/run/approval and archive checks,
   then checks Draft metadata and all existing asset bytes before any mutation.
   Draft API reads require the approved job's contents-write token; the read-only
   preflight deliberately does not access those endpoints.
   It stages only the closed public set (six files for schema2, twenty for
   schema1) using `gh api`, publishes the
   existing Experimental pre-release without Latest, and completes authenticated
   and anonymous byte readback. Only then does it PUT the unchanged archive bytes
   to crates.io in dependency order. No `cargo package` regeneration is allowed.
   Each upload needs exact index checksum and downloaded-byte readback before a
   dependent upload. A fresh registry-only consumer pins both schema2 versions
   (all sixteen for historical schema1),
   uses no token/path patch/source replacement and must resolve/check successfully.
   Only that complete result can close YT-020; reflect actual results in the
   release record and retain receipts outside temporary CI artifacts.

## Failure and artifact boundaries

GitHub and crates.io publication is not atomic. If registry upload or the final
consumer fails, the public GitHub Release and any accepted packages remain.
Retain `PUBLICATION-RESULT.json`, `GITHUB-PUBLICATION.json`,
`REGISTRY-RESULT.json`, consumer logs/lock and the frozen plans as applicable;
report incomplete delivery. An uncertain PUT is recorded before sending and is
never retried automatically. Investigate existing remote versions/bytes before
requesting an explicitly authorized continuation. Readiness GET polling is
bounded and cannot repeat a write. Never delete/re-upload, clobber or yank as
automatic recovery.

The combined schema-2 publication plan/acceptance summary binds both registry
plan and request digests. The legacy local GitHub-only CLI accepts schema1 only;
it cannot claim that a combined registry request was fulfilled. Public attachments
remain the exact selected `.crate` files, original unsigned provenance, public checksum,
public Sigstore bundle and reduced acceptance summary. Publication receipts and
the complete private evidence are not added through a directory glob.

## Exact public plan and upload boundary

Local `stage`, `readback` and `publish` are separate explicit operations by exact
Release ID. Before writes, verify immutable tag→C, Release ID/tag/C/name/body
digest and prerelease identity; preserve `draft=false`, `prerelease=true` and
`make_latest=false` on publication. `--confirm publish-exact-approved-plan` is an
operator assertion, not owner authorization. Validate all local bytes first;
reject duplicates, unexpected existing assets and clobber. Upload each missing
asset once, verify actual downloaded names/IDs/sizes/hashes, and stop on failure
with partial state retained. Anonymous metadata and actual bytes must agree after
publication; failed observation never authorizes republishing.

The public set derives from the cohort: archives, original unsigned provenance,
public checksum, public DSSE bundle and reduced signing approval (twenty files
for historical sixteen, six for the two SDKs, five for CLI). The complete private
manifest is not the public checksum manifest. Full verification receipts/private
bundles/raw findings are never public attachments. Reduced approval binds
`record_sha256`, C, S, verification/manifest/public-bundle digests and the accepted
scope/destination; the combined policy additionally binds its registry plan and
request. Retain exact plan/schema distinctions; generated GitHub source ZIP/tar
downloads are not signed package subjects.

The 2026-10-30 alpha.3 Python upload failed with zero assets and an unknown
original HTTP status. Successful later `gh api` transport did not diagnose that
old failure. The canonical transport uses authenticated `gh api`, fixed upload
origin, explicit binary Content-Type/Content-Length and byte-exact stdin;
metadata reads retain bounded HTTP handling without credential forwarding to
public asset hosts. Timeout, unavailable CLI, command/HTTP failure and malformed
response produce bounded phase/class/status receipts, never raw stderr/bodies,
tokens or signed URLs. Preserve the separate alpha.4 length failure/correction in
its [accepted record](ALPHA4-HOSTED-RECORD.md#owner-approved-publication-failure-and-local-repair).
There is no silent fallback, automatic retry, tag change or implicit registry grant.

## Registry-only continuation contract — 2026-10-30

The owner authorizes a local parser correction and preparation of one explicit
continuation after publication41028646006/1. GitHub is already public with the
original twenty assets; `yetlet-actions0.9.0-alpha.4` is published under bohewu
and its API/index/download match the frozen archive. The old incomplete receipt
is retained. The other fifteen packages and registry consumer were not attempted.
This local slice does not authorize Git delivery or another network write.

Cargo's [publish response contract](https://doc.rust-lang.org/cargo/reference/registry-web-api.html#publish)
returns optional warnings, without requiring `krate.name`. Accept a successful
JSON object with no errors and optional valid warning arrays; retain only warning
counts. Identity/version/content acceptance still requires index/download
readback. Never log raw provider warning/error strings or treat a parse failure
after PUT as proof that the write did not happen.
The [upstream API type](https://github.com/rust-lang/crates.io/blob/daa3c146d99e5db0a2fb74062ac480f3f03797a0/crates/crates_io_api_types/src/lib.rs#L1133)
renames Rust's `krate` member to JSON `crate`; that extra field is accepted but
is not an identity substitute for readback.

Keep the normal closed publication request unchanged. An optional, closed
`continuation` object selects only this known one-plus-fifteen case:

| Field | Binding |
|---|---|
| `run`, `attempt` | Original failed publication run/attempt |
| `artifact`, `sha256` | Original `publication-result-RUN-ATTEMPT` ZIP and digest |
| `existing` | Exactly `["yetlet-actions"]` |
| `remaining` | Exactly the other fifteen names, in the unchanged registry-plan order |

Validate the original completed failure, source/workflow/repository, owner
approval and original five-file result artifact. Reconstruct its original
publication request using its actual P; its digest, original public plan,
registry plan and GitHub-success/registry-failure receipts must agree. Preserve
the original public acceptance summary rather than generating a replacement.
Current published Release/tag/name/notes and all twenty asset IDs/sizes/digests
must still match that accepted plan and original readback receipt.

The new reviewed P receives its own request and owner approval. A separate
`registry-continuation` job uses the existing `release-publish` Environment,
GitHub contents/actions **read**, registry token only after the owner gate, and
no OIDC/age identity. It never calls GitHub stage/publish/update operations.
The read-only preflight and protected phase reconcile the existing package's
index/download/not-yanked state and owner, and require all fifteen selected
versions absent before the first write. Recheck each selected version before
its single PUT; a newly observed existing version stops instead of being
silently skipped or resent. There is no generalized or automatic resume.

Use a fresh continuation receipt. Record the first package as verified existing,
and only the fifteen newly attempted writes as uploads. Keep the old failure
immutable. After all fifteen readbacks succeed, run the existing fresh,
credential-free registry consumer for all sixteen exact versions. Any partial
or uncertain failure remains incomplete without retry, deletion, yank, version
bump, re-signing or GitHub asset mutation. C/tag/S/V, original package bytes and
signatures remain fixed. Hosted continuation remains separately authorized;
YT-020 stays Blocked until complete registry and consumer evidence exists.

First-name capacity is not established by the read-only checks. The pinned
[upstream rate limiter](https://github.com/rust-lang/crates.io/blob/daa3c146d99e5db0a2fb74062ac480f3f03797a0/src/rate_limiter.rs#L22)
defaults to a five-name burst and one new-crate token per ten minutes, with
deployment configuration and per-user overrides. These are source defaults,
not observed production/account limits. Before a live fifteen-new-name batch,
confirm sufficient account capacity or separately agree a different bounded
batch/budget. The owner selects a conservative fixed cadence: wait660seconds
before every selected package's unique PUT, including the first. Fifteen waits
total165minutes; the registry-only protected job is bounded to210minutes,
including setup, readbacks and consumer checks. The normal publication job
remains60minutes. Request JSON cannot shorten or disable this policy.

After each wait, recheck protected master=P, CI, the actual owner approval and
the unchanged public Release/assets, then require the target version absent
immediately before its PUT. Keep master at P during the batch; source or state
drift stops it. This cadence is conservative against the documented defaults,
not a guarantee of an unknown production/account allowance. No quota is probed
by uploading, and no override is requested. Any HTTP429 or other failure stops
with the original partial receipts, without retry or automatic batch splitting.

## CLI registry-only continuation after a GitHub readback failure

The original [CLI 0.4.2 publication run](https://github.com/bohewu/yetlet/actions/runs/45086142841)
made the GitHub pre-release public and then stopped on an HTTP 403 read
in the GitHub publish phase. The receipt records that no registry upload
was attempted. Later anonymous downloads matched the original five-asset
plan. Preserve the original failed run and public Release.

This schema-3 continuation is limited to the recorded run
`45086142841`, attempt 1, artifact `14779392510`, and its verified ZIP
SHA-256. The closed `continuation` object carries those identities with
`existing=[]` and `remaining=["yetlet-cli"]`. The workflow reconstructs the
original request and matches both frozen plans. Its read-only preflight
checks the approved failure receipt, the public release metadata with the
read-only GitHub token and asset bytes through the canonical anonymous
downloads. It also confirms absence of the exact version in the registry. A new
protected `registry-continuation` job requires the owner's approval, repeats
the checks, then sends the original archive once to crates.io. It uses
read-only GitHub permissions and writes a separate continuation receipt.
The fresh crates.io-installed consumer runs after package readback. Any
attempted registry write falls outside this continuation contract.

The first [continuation run](https://github.com/bohewu/yetlet/actions/runs/45089671256)
also stopped on HTTP 403 while checking the public release. Its receipt
records no GitHub or registry write attempt. The continuation now uses
authenticated release and asset metadata and validates each fixed
`github.com` public download URL before fetching bytes without a credential.
The original plan's release identity, asset sizes and SHA-256 checks still
apply in both phases.

## Local verification and remaining limits

Focused policy/protocol/order/receipt tests and the revised Cargo preparation
recipe are recorded under `target/yt020-release-implementation-20261030/`.
Twenty-seven distinct affected Python tests and three Rust tests PASS; the latter
include the real Cargo sixteen-package synthetic fixture and its deliberate
verification refusal. Clippy, docs/links/Markdown/DAG and whitespace checks PASS.
A direct call of the existing packaging function on an isolated current-source
snapshot normally verifies all sixteen actual packages; twenty-six internal
registry checksums in their archive lockfiles match the staged original bytes.
All sixteen exact-byte protocol plans parse those archives. This uses an
explicitly synthetic transport record, not a selected new release candidate.
Seven moved example/manifest/fixture inputs exactly match the earlier accepted
prototype, so its metadata and example compile evidence is reused.

The initial local compile/receipt failures and initial independent FAIL are
retained. Two review blockers (duplicate workflow `env`, read-only preflight
access to Draft metadata) received the smallest correction and focused checks.
Independent Reviewer `/root/yt020_single_publication_review_20261030` followup
**PASS**, zero issues/required followups; all thirty-nine reviewed path hashes
match before/after review. Only this completion record follows acceptance.
Owned recipe scratch and the temporary invocation binary are removed; sixteen
local archives, logs and cleanup receipt remain retained outside Git tracking.

No registry network write, real token use, Environment change, new candidate,
hosted job, signature or publication occurs in this slice. Historical evidence
is reused at its original scope. New source/package qualification, rollout and
actual publication/readback remain separate work; YT-020 remains **Blocked**.

## Alpha.4 timing and bounded CI maintenance — 2026-10-30

The actual interval from candidate master CI creation at06:18:42Z to failed
publication completion at07:51:09Z is92m27s. This measures an incomplete delivery.
The retained original run/job timestamps separate the elapsed categories:

| Category | Approximate elapsed minutes |
|---|---:|
| Candidate master CI and signer-document PR/master CI | 25.87 |
| Build, original readback, prepare, sign and independent verification | 9.25 |
| Publication run, including owner waiting and failed execution | 3.35 |
| Gaps between the recorded workflows | 53.98 |

Gaps include evidence selection/retention, a75MB local prepare-artifact download,
document/review/Git delivery, Draft corrections and assistant orchestration.
They are not all runner compute or owner waiting. The signer master Stable job
spends146s on RustSec installation/audit and136s on browser tooling/build;
Chrome E2E itself takes23s. These observations identify work, not a promised
future duration.

The minimal current workflow change caches only the pinned `cargo-audit0.22.2`
and `wasm-bindgen-cli0.2.129` installation root. Its exact key includes OS,
architecture, complete `rustc -vV` digest and both tool versions, with no broad
restore prefix. A miss retains the normal locked installs. Restored tools must
report the expected versions before use. The SHA-pinned
[official cache action](https://github.com/actions/cache/tree/caa296126883cff596d87d8935842f9db880ef25)
uses Node24; the observed hosted runner2.338.0 exceeds its2.327.1 minimum.
Only unprivileged baseline CI uses this cache. No Cargo home/credential, private
evidence, product target directory or release bootstrap/signing tool is cached.
MSRV, format, audit, native/WASM, Chrome and all required statuses remain.
Local syntax/review does not prove a hosted cache hit or performance gain;
observe cold/warm behavior in subsequent necessary CI, without an extra replay
of product qualification solely to obtain timing.

Further optimization is a proposal, not an implemented trust-policy change:

1. Connect existing fixed-source worker stages and process safe receipts without
   serial chat/operator handoffs. Preserve separate no-OIDC verification and
   the sole owner publication gate. Do not merge worker permissions.
2. Evaluate parallel encrypted long-term retention against the existing custody
   acceptance boundary. Do not discard retention or claim that expiring Actions
   artifacts are durable storage.
3. Evaluate repeat full CI for status-only document delivery. Any reuse must bind
   the exact reviewed tree, workflows, toolchain and locks and amend the existing
   exact-source green-master contract through review. No required status or
   immutable source check is skipped by this slice.

Fixing the publisher needs a reviewed P and refrozen publication request/plan,
not another product build, prepare, signature or independent verifier when
their frozen inputs remain unchanged. Retain the failed attempt and inspect
remote state before requesting a bounded continuation. No Git delivery,
Environment change, hosted retry or publication is performed by local maintenance.

## First registry publication and immutable artifact contract

The 2026-10-30 YT-020 contract studied sixteen historical alpha.3 libraries at C
`557ab76e03b619e959eae7529dbe03317fe1f952`, S/V `30baf25`. Current accepted
distribution is [SDK alpha.8 / CLI0.4.1](RELEASE-READINESS.md).
Select current cohort/version/source independently. Old nine/sixteen-package
orders,404 name observations, owner-reported cargo login and private dry-runs
provide no present name reservation, account/email/scope or upload authority.
Package disclosure and repository visibility require their respective approval.
Keep reachable private reporting/applicable security controls and concrete
release/source/security acceptance bound to the selected distribution.

### Historical first-name dependency correction

The old normal DAG put Secrets before Config because Config optionally depends
on Secrets. Secrets' development dependency on Config nevertheless created an
initial-name cycle. The observed crates.io source derived normal/build/target/dev
declarations from archive Cargo.toml and rejected unknown names even for dev
dependencies. Altering upload JSON, native batch packaging or installing a token
could not solve that condition. The dated upstream source observation did not
attest the deployed service revision.

The selected bounded correction moved listener-free `secrets_state` and its four
synthetic settings files from Secrets to a distinct Config example directory,
changed only the fixture path, registered Config's target behind `secret-provider`
and removed Secrets' reverse Config dev edge. Core/Server dev edges remained.
Preserve APIs/features/workspaces, update live commands/CI consumers when applying
the correction, reconcile affected locks without unrelated upgrades and compile
the relocated target before selecting a new source. Do not leave an example
importing an omitted dependency or publish placeholder names. The isolated
prototype proved16nodes/17all-kind declarations, one removed edge/no added edge
and no cycle. Original signed alpha.3 had18declarations and remained blocked.

The corrected historical layers were Actions/Core/Reactive/Signing/wgpu,
then DOM/DOM-keyed/HTML, then Server Axum, then Request-ID/Secrets/Session/
Session-Redis, then Auth/Config, then Rate Limit. Current batches derive order
from every normal/build/dev/optional/target declaration and require intended
prerequisite version/checksum indexed and downloaded before dependent upload.
The older normal-only order cannot substitute for this all-kind gate.

### Freeze, request and readback

Build with normal Cargo extracted-package verification, staging each predecessor's
final archive and actual checksum before packaging dependents. Keep dev entries
in published manifests and actual predecessor checksums in packaged locks. Private
prospective directory sources are a disclosed packaging model. They cannot claim
live registry presence or form a live source-replacement publish configuration.
Never bypass verification, remove checksums or transfer old signatures to changed
archives. New source/version/archive bytes require new approved subjects.

Freeze the full selected set and an immutable publication plan before writes.
Reject missing/Pending plan fields. Retain subsequent receipts separately by plan
digest and initialize outcomes as `not_attempted`. Future outcomes are completion
gates rather than impossible prerequisites for the first operation.

| Frozen binding | Required data |
|---|---|
| Source/version | Full commit/tree, selected version, source archive digest, recipe/toolchain/input-lock identities |
| Cohort/artifact | Exact allowed names, version, filename, compressed size/SHA-256, normalized manifest/packaged-lock identity and provenance subject |
| Metadata | Archive-derived JSON/README, aliases, normal/build/dev kinds, target, optional/default features and feature map, serialized metadata SHA-256 |
| Order | All-kind acyclic prerequisites with exact intended version/index checksum |
| Request | Framed body digest, metadata/archive lengths and fixed official crates.io endpoint, without arbitrary destination or redirects |
| Authority | Approved source/cohort/batch, intended publisher, allowed attempts and failure limits |

Transmit four-byte little-endian JSON length, JSON bytes, four-byte little-endian
archive length and original compressed `.crate` bytes. Metadata/request hashes
supplement archive hashes. Index/download SHA must equal the archive SHA.
Reject name/version mismatch, unpacking/recompression, lock edits or repackaging
after freeze. The exact-byte transport promotes verified archives through Cargo's
official registry protocol. It does not run cargo publish to regenerate them.

Use the selected supported credential arrangement explicitly, including a
configured provider. Reuse owner-installed login where selected. Verify effective
account, verified email and `publish-new` rights at the authorized stage without
logging tokens or putting them in argv. Initial restricted credentials must have
the selected names' required rights without implied yank/owner-management rights.
The later deployed Environment token and90-day policy remain in the
[rollout contract](#one-time-rollout-after-local-acceptance). Trusted Publishing
setup/only-mode requires separately verified first-name ownership and actual
repo/workflow/ref/trigger constraints. Registry authentication does not supply
Sigstore trust or release approval.

Upload sequentially once per allowed package. Wait for matching index and actual
downloaded bytes before dependents. Record native attempt/response/readback as
it occurs. Timeout,403,429, digest mismatch and uncertain writes stop with partial
receipts. Inspect actual registry state before separately authorized continuation.
Never automatically retry, overwrite versions, move tags, yank or split a batch.
Complete every selected download and a fresh credential-free registry-resolved
consumer without path/patch/source replacement before declaring completion.

### Dated local evidence and remaining provenance

On2026-10-30, normal Secrets and Config/secret-provider publish dry-runs passed
under disclosed accepted-archive directory sources. The baseline offline command
failed101 at required HTTP before sibling diagnosis. First Config failed101 at
rustup concurrent stable updates, then passed with existing Rust1.99 compiler/
rustdoc/toolchain pinned. Secrets' compiler proxy was not equivalently pinned.
Source manifests/locks remained unchanged. VCS metadata in old vendor input was
corrected once before Cargo by extracting accepted archives. Accepted versus
dry-run archive hashes differed only through four added Yetlet checksums in each
packaged lock. Their new archives did not form an approved closed registry set.

| Package | Signed accepted SHA-256 | Dry-run SHA-256 |
|---|---|---|
| Secrets | `c4a3f8162f9451049b6528be15a6272640264d068b15ecb2de8764b976953a2b` | `a9e07c75bd1bc77336b9a6597f747cd0f5bb89f85a6a2c0f82c5c89388f38da3` |
| Config | `0ad1d3fe2aee9fc02b7286ed923c7d5655c4973504f22767865ce3a9bb090d69` | `85dcaaaae592c91c25362b139c64022856d55ad4bf05dd1d7042e34fd9728044` |

The isolated correction's offline/locked metadata and relocated example
`cargo test --example secrets_state --features secret-provider --no-run` passed
with pinned Rust/Cargo1.99. Sixteen offline envelopes decoded to original signed
archives and manifest identities with zero HTTP/credential reads. They remained
ineligible for upload. A graph wrapper first failed on string-versus-table
dependencies. Independent review first failed because pre-upload requirements
included post-upload results. Both original failures and smallest corrections
remain retained. Re-review PASS0 accepted only contract/prototype scope.
No original product suite was replayed. Raw stores are
`target/yt020-first-publish-20261030/` and
`target/yt020-registry-binding-20261030/`. The historical contract selected no
new R/version/tag and left YT-020 Blocked. Later actual publications retain
their own separate accepted records and cannot backdate those observations.
