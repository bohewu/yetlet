# Single-gate release automation and registry promotion

YT-020 / YT-209 followup, 2026-10-30. **Worker rollout verified by alpha.4;
first combined registry publication remains incomplete.** See the
[actual hosted outcome and local repair](ALPHA4-HOSTED-RECORD.md#owner-approved-publication-failure-and-local-repair).
[RELEASING.md](RELEASING.md) remains the general SOP; dated alpha.3 results and
immutable tags/archives are preserved.

## Owner-selected policy

The owner requested that the assistant handle preparation, signing and
verification, with human intervention only for final publication. The final
request covers **GitHub Experimental pre-release, followed by crates.io for the
same sixteen frozen archives**. Git delivery still uses explicit authority and
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
reviewed [D1/D2 disclosure fields](YT-096-D1-D5-RELEASE-TRUST-PLAN.md); new fields,
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
   Secret `CRATES_IO_TOKEN`. Restrict scope to the sixteen selected names,
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
   existing evidence. Merge the reviewed source and confirm master CI. Select
   immutable C, then use strict/builtin Guard and the existing authorized tag
   procedure. Never move a tag or reuse an accepted version for changed bytes.
2. Dispatch one bounded unsigned build and original readback using the existing
   versioned release record. The sixteen-package recipe now has one vendor step
   and sixteen normal verified package steps, in all-kind dependency order.
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
   verifying both bundles, sixteen subjects and the full private manifest, and
   cleaning private work. It contains nineteen allowlisted public files plus
   `RESULT-VERIFICATION.json`. Private manifest/bundle/evidence remain excluded.
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
   It stages only the closed twenty-file public set using `gh api`, publishes the
   existing Experimental pre-release without Latest, and completes authenticated
   and anonymous byte readback. Only then does it PUT the unchanged archive bytes
   to crates.io in dependency order. No `cargo package` regeneration is allowed.
   Each upload needs exact index checksum and downloaded-byte readback before a
   dependent upload. A fresh registry-only consumer pins all sixteen versions,
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
remain sixteen `.crate` files, original unsigned provenance, public checksum,
public Sigstore bundle and reduced acceptance summary. Publication receipts and
the complete private evidence are not added through a directory glob.

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
batch/budget. The current sixty-minute job does not establish that all fifteen
can complete under those defaults. No quota was probed by an upload, no account
override was requested, and no sleep/retry/batch-splitting behavior was added;
HTTP429 stops with the original partial receipts.

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
