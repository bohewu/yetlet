# YT-209 — Versioned release automation

Status: Done at implementation, repaired integration CI and Environment setup
scope. Positive release-result hosted qualification remains Pending. Automatic
post-merge master CI PASS; earlier failures are retained below.


## Scope and authority

The owner authorized a versioned, data-only release record, a separately protected
independent signing-result verifier, and exact Release-ID-based public attachment
operations. This item implements those seams locally. It does not authorize a
hosted run, Environment/Secret changes, signing, a tag, publication, registry
upload, Commit or Push. The accepted alpha release and all failed attempts remain
immutable evidence; no old qualification is replayed. See [the release SOP](RELEASING.md)
and [the signer contract](YT-096-SIGNER-CONTRACT.md).

## Versioned input contract

One strict JSON schema (`schema: 1`) carries `version`, full lowercase product
`source` C and the closed `sixteen` cohort. The version is canonical SemVer
(without build metadata); tag is derived as `v` plus version, never supplied
independently. Unknown/duplicate fields, invalid types, oversized input, unknown
cohorts, malformed identities and executable/URL/path fields are rejected. Trusted
repository/owner IDs, workflow paths, tool/root/recipient pins and API endpoints
remain in reviewed code.

A build snapshot needs only that core. A custody-readback snapshot additionally
has `build`: exact run, attempt, artifact ID and ZIP SHA-256. Before prepare the
record is completed with `custody_verifier` (run, attempt, Environment ID),
`signer_commit` S, `provenance` (raw SHA-256 and byte size) and `release_id`.
All IDs are positive decimal strings; hashes are lowercase SHA-256. S must differ
from C. Completed records must include the earlier build snapshot unchanged.
Keep each snapshot and its canonical JSON digest in custody. Never edit an
accepted record; corrections create a separately reviewed snapshot and retain the
old one. Later run IDs and receipts are external to the frozen record/manifest,
so neither C nor a payload needs to contain its own hash.

Build/readback/sign requests take the record as data. No default alpha fallback
is allowed in command execution. The selected source's package recipe/version
must match the record and the exact sixteen archive names; version changes still
require the normal reviewed Cargo/lockfile migration. Event/workflow/checkout,
existing immutable tag, protected master ancestry, clean source and exact original
run/artifact checks remain. Prepare/sign still run at separately approved S on
protected master with the existing Environment gate; live master must equal S.
The signer receipt binds the complete record digest and existing frozen payload
SHA values. Signing uses that same record, not a fresh build or mutable tag lookup.

## Independently protected signing-result verification

A new manual workflow runs at a separately reviewed verifier commit V on protected
master. V can follow S after signing; it does not change C, S, the signature or
accepted payloads. A closed request pins V, the complete release record, original
prepare run/attempt/artifact/digest/receipt and actual successful signing
run/attempt/artifact/digest/execution-receipt digest. No retries or success claim
from the signer job substitute for this check.

Preflight has read-only Actions/contents permissions and no key. It checks exact
source/workflow identity, protected master = V, successful original prepare/sign
runs at S and their exact artifact metadata. The protected job uses a dedicated
`release-result-verify` Environment: required reviewer bohewu, master only,
admin bypass disabled, explicit owner approval. Environment installation and
`AGE_IDENTITY` provisioning are future separately authorized operations.
Neither job has OIDC write, content write or a signing action.

Authenticate the existing pinned verification tools before key access. Decrypt
only in runner-private storage outside the checkout; verify the frozen receipt,
complete manifest and every entry, both exact payload hashes, all sixteen package
subjects, both actual Sigstore bundles and their original S/run/attempt certificate
claims. The first bundle must contain the exact raw statement, not a reconstructed
one. Reject partial/failure artifacts, altered receipts and a private bundle hash
mismatch. Delete plaintext/key files on success and failure; cleanup failure is
failure. Export only a reduced verification receipt, including record/input/result
digests and verifier source/run; no manifest body, private bundle, raw logs or key.
Final owner acceptance and durable custody remain separate steps.

## Public attachment and publication contract

A local explicit helper consumes the same completed record plus a closed public
plan: exact Release ID, release name, notes SHA, twenty asset names/sizes/SHA-256,
independent-verification receipt SHA and reduced owner-acceptance summary. Paths
and credentials are local invocation parameters, never record-provided programs.
The list is sixteen versioned `.crate` files, original `PROVENANCE-UNSIGNED.json`,
public `RELEASE-CHECKSUMS.json`, `provenance.sigstore.json`, and
`SIGNING-APPROVAL.json`. Private material is never selected by a directory glob.
The complete private signed manifest is not the public package-set checksum.

Separate `stage`, `readback` and `publish` commands require explicit invocation;
publication additionally requires a literal confirmation and the accepted
independent result/owner summary. Never create/move tags or discover a Draft by
published-tag lookup. Before any write, resolve the existing tag to C, fetch the
exact Release ID and verify tag/C/name/body digest/pre-release identity. Preserve
all bindings explicitly on publication (`draft=false`, `prerelease=true`,
`make_latest=false`). This slice supports Experimental pre-releases only.

Stage refuses duplicates, unexpected existing assets and any clobber. It validates
all local public bytes before its first upload, uploads missing assets once, and
checks actual downloaded bytes, names, sizes and IDs. Failure stops immediately;
partial state remains for inspection, not automatic retry. Readback verifies the
exact list and each actual byte digest. After publication, anonymous metadata and
actual bundle download must agree; a post-write observation failure stays failure
and never causes automatic re-publication. Generated source ZIP/tar downloads are
not signed subjects. Receipts are exclusive local files; no secret URL/token or
private evidence is emitted.

## Bounded verification and completion

New local synthetic tests cover changed version/identity/schema boundaries,
record-to-receipt binding, independent verification dispatch/claims and failure
cleanup, public allowlist/Release identity/partial-write/readback behavior. Use
existing test seams with fake API/crypto commands; no formal key, signing, network,
full package rebuild or old accepted qualification is exercised. Changed workflow
structure and affected Rust tag derivation receive focused checks.

Independent Reviewer PASS is required before marking Done. Done means reviewed
implementation and focused local evidence, not hosted qualification or permission
to operate it. The next operational step is a separately approved source/record,
Environment setup and bounded hosted rehearsal; do not retrofit that result to
the already published alpha.

## Integration reference

This is maintainer tooling for Python 3.11+ on POSIX and Ubuntu hosted runners;
it adds no application dependency. `scripts/release_record.py` owns schema 1.
The build and original-custody workflows accept a string input named `record`.
The signer and independent-result workflows accept `record` plus their separate
closed `request`. Inputs are JSON data, never shell fragments. Old tagged workflow
sources retain their historical input interfaces; new workflow inputs do not
retroactively change their accepted runs.

| Snapshot or request | Exact fields / binding |
|---|---|
| Core record | `schema`, `version`, `source`, `cohort`; derive tag from version |
| `build` | `run`, `attempt`, `artifact`, `sha256` (original Actions ZIP digest) |
| `custody_verifier` | `run`, `attempt`, `environment_id`; actual approved original readback |
| `provenance` | `sha256`, integer `size`; exact original public statement bytes |
| Complete record | Core plus `build`, `custody_verifier`, `signer_commit`, `provenance`, `release_id` |
| Signer request | Existing closed prepare/sign fields; S must equal record S, complete record digest enters frozen receipt |
| Result request | `verifier_commit`, `signing_request` (original exact sign request), `sign_run`, `sign_attempt`, `sign_artifact`, `sign_digest`, `execution_sha256` |
| Public plan | `schema`, `record_sha256`, `release_id`, `name`, `notes_sha256`, `assets`, `verification_sha256`, `approval_sha256` |
| Each public asset | `name`, integer `size`, `sha256`; all twenty closed names required |

`sign_digest` and the original sign request's `digest` are Actions ZIP hashes;
`execution_sha256` pins raw `SIGNING-EXECUTION.json`; `receipt_sha256` in the sign
request pins raw `SIGNING-INPUTS.json`. `record_sha256`/`plan_sha256` hash canonical
JSON (sorted keys, ASCII escapes, compact separators, trailing newline). Other
file hashes always cover the original bytes. Do not interchange these digests.
A complete record is frozen before prepare and remains byte-equivalent under
canonicalization through sign, independent verification and public operations.

Local data validation needs no GitHub credential:

```bash
(
  release_record_file=/secure/release/release-record.json
  export RELEASE_RECORD="$(cat "$release_record_file")"
  python3 scripts/release_record.py check complete
)
```

Build/readback source guards additionally compare the record with all sixteen
Cargo versions. Rust preparation derives tag/version from its compiled source's
root workspace manifest, so the recipe no longer needs an alpha version constant.
Prepare/sign retain original trust pins; key/tool rotations remain separate work.
The result workflow emits only `RESULT-VERIFICATION.json`, after deleting its
plaintext directory. Retain that receipt, frozen inputs, ciphertext and signing
artifacts durably outside the repo; artifact expiry still applies.

For public preparation, place only the exact public files in an explicitly
selected directory and review their plan. `SIGNING-APPROVAL.json` is a reduced
owner decision with exact fields: `schema: 1`, `record_sha256`, `source`,
`signer_commit`, `verification_sha256`, `manifest_sha256`,
`provenance_bundle_sha256`, `accepted: true`,
`scope: "Experimental source packages"`, `publication: "GitHub pre-release"`,
`registry_upload: false`. It is separate from immutable signing execution receipts.
Keep the full independent receipt private; its digest is sufficient in this
public acceptance file. Never use the public helper to upload the private receipt.

After separate authority for the selected operation, invoke `scripts/release-public.py`
with `stage`, `readback` or `publish` and explicit `--plan`, `--public-dir`,
`--notes`, `--verification` and `--output` paths. `RELEASE_RECORD` supplies the
reviewed complete record; `GH_TOKEN` supplies the owner's narrowly scoped GitHub
credential. Output is an exclusive file outside the repo. `publish` additionally
requires `--confirm publish-exact-approved-plan`; this literal is an assertion,
not a replacement for the owner's actual authorization.

The Draft must already exist under separate authority, with exact tag/C/name/body
and pre-release state. Remove temporary encrypted handoff assets under their
existing cleanup authority before public staging; unexpected assets are rejected,
never automatically deleted. Stage uploads missing intended files once and never
clobbers an existing file. Its repeated invocation is a new explicit operator
action after inspecting any retained partial state, not an automatic retry.
Readback/publication inspect actual owner/anonymous downloaded bytes and ensure
the release is not Latest. Unsupported stable/registry publication is rejected
by scope; no automatic promoter or scheduled signing job is added.

## Local evidence and remaining qualification

Evidence is retained at `target/yt209-release-automation-20261029/` (local, ignored).
Contract independent Reviewer PASS0 precedes implementation. New checks use
synthetic versions and non-default run attempts, fake API replies and fake crypto
commands; no formal identity, crypto issuance, network or hosted job is exercised.
Initial test-fixture pin/mapping failures and their focused corrections remain
visible. Existing browser, package, crypto, MSVC and hosted proofs are reused at
their original scope, not relabeled as YT-209 hosted qualification.

Final implementation Reviewer `/root/yt096_d6_confirmation_review_20261029`
returned PASS0 and confirmed all seventeen frozen source hashes; native role/model/
effort verified as reviewer / gpt-6-astra / actual max. Actual `release-result-verify` Environment installation, Secret
provisioning, a new source/tag/record, hosted qualification, signing, public writes
and durable/off-host custody remain separately authorized future operations.


Accepted focused evidence: **30 distinct checks** (23 new Python checks, five
changed signer checks, two changed Rust preparation checks). The workflow check
also parses all changed Bash blocks and Python heredocs. `cargo xtask check docs`
passes the existing fixed75-document/209-item matrix; all changed Markdown files
additionally pass local link/structure/whitespace inspection. `git diff --check`,
Python compilation and `cargo fmt --manifest-path xtask/Cargo.toml --check` PASS.
The first synthetic age-pin fixture mismatch and the affected signer fixture's
mapping mistake were each corrected once, rerunning only their failed check;
both original failures remain in local evidence. No original qualification was
replayed. Completion-only documentation metadata follows the frozen review.

| Changed paths | Purpose |
|---|---|
| `.github/workflows/release.yml`, `evidence-verify.yml`, `release-sign.yml`, `release-result-verify.yml` | Versioned inputs and protected independent verification |
| `scripts/release_record.py`, `release-evidence-preflight.py`, `release-sign.py`, `release-result-verify.py`, `release-public.py` | Closed record, retained identities, result verification and exact-ID public operations |
| `scripts/test_release_automation.py`, `test_release_sign.py` | New/affected local behavior evidence |
| `xtask/src/release/preparation.rs`, `private.rs` | Version derived from selected compiled manifest |
| `docs/YT-209-RELEASE-AUTOMATION.md`, `RELEASING.md`, `BACKLOG.md`, `ROADMAP.md` | Contract, operator interface, evidence and bounded completion |

The seventeen implementation paths were subsequently committed as
`c70a7431226c307a097c218050348477de85671d` after strict/builtin/native-hook
PASS; no Push. [The hosted/Environment plan](YT-209-HOSTED-VERIFICATION-PLAN.md)
records current read-only settings, old-receipt compatibility limits and the
proposed minimum authority packets. It grants no operational authority. YT-209
remains Done at the accepted local scope; hosted qualification is Pending.


## Subsequent integration CI — 2026-10-29

Owner-authorized tooling delivery created [PR20](https://github.com/bohewu/yetlet/pull/20)
at `c0fae63871ee9910eb9182007615950e96f627d7`, preserving the original branch.
The original staged/range/native strict/builtin checks passed. CI
[40871149693/1](https://github.com/bohewu/yetlet/actions/runs/40871149693) passed
MSRV but failed Stable's Rust-tool test step before the later audit/browser steps.
No Merge, Environment/Secret installation or release workflow followed that failure.

The concrete failure is `ModuleNotFoundError: release_record` when the existing
Rust custody test loads the preflight script through `importlib` from repo root.
Initial YT-209 checks covered the new helper/direct invocation but omitted this
existing caller. The custody metadata fixture and the separate preparation
workflow test also retained obsolete fixed-alpha input/identity assumptions.
Those are changed consumers, not unrelated qualification to replay.

The minimum repair changes three source/test files: resolve the trusted sibling
module directory in `release-evidence-preflight.py`, migrate the custody fixture
through actual schema-1 inputs using a synthetic alpha.2 tag, and update the
preparation/custody workflow assertions to the reviewed record/source guard.
The identity, protected reviewer/no-bypass, exact tag/ZIP and secret-order checks
remain; no legacy fallback, ignore, skipped assertion or weaker workflow is added.
The caller inventory covers direct workflow execution, signer module loading and
the Rust `importlib` caller. Existing new-record behavior evidence remains scoped
and retained.

Local evidence: the two affected Rust cases PASS individually, xtask all-target
Clippy with `--offline --locked -- -D warnings` PASS, plus changed-file format,
Python syntax and whitespace PASS. The initial local edit script aborted before
writing the Rust fixture; its subsequent targeted run therefore still failed on
missing `tag`. That editing failure and local test failure are retained with the
first remote failure, then corrected at the existing fixture seam. No CI retry
or new remote source follows automatically. Independent repair Reviewer `/root/yt096_d6_confirmation_review_20261029`
PASS0; all six reviewed hashes match and native reviewer/gpt-6-astra/actual max
identity is verified. Only completion metadata changes after this review.

At that failure, original PASS results remained historical evidence, not
permission to merge the failed head. YT-209 was reopened for integration repair.
New repair delivery/CI and continuation to conditional Merge/Environment setup
required a separate owner decision under the no-automatic-retry instruction; the
subsequent authorized recovery is recorded below.


## Owner-authorized recovery delivery and setup — 2026-10-29

The owner explicitly approved the six-file repair, one new-source required CI
run and continuation to the previously approved Merge/Environment batch. Repair
commit `cdde3c4e785cd0822b73d0535a1501305dde3489` passed staged/range/native
strict/builtin checks and was pushed once to PR20. Both required jobs in
[40873628263/1](https://github.com/bohewu/yetlet/actions/runs/40873628263) PASS:
MSRV and Stable native/samples/Chrome, including the previously unstarted audit
and browser phases. First failed run `40871149693/1` and local failures remain.

PR20 was squash merged as `7de77c84e0e56e683cea4567048397caa0614590`.
Protected master and Git tree `95c65e754a51fadce648645791f41620b56f8bca` match
the qualified repair source; candidate/merged-range strict/builtin checks PASS.
Original branch/history and the local integration branch remain preserved. This
is tooling delivery, not selection of a new product C, signer S or version tag.

Environment `release-result-verify` ID `25684187945` was created with owner
reviewer bohewu/35250002, `prevent_self_review=false`, admin bypass disabled,
and exactly one custom branch policy for master. Configuration passed the same
reviewed Environment predicate before Secret installation. The existing custody
identity was supplied only through stdin to GitHub's Environment Secret installer;
no plaintext copy/log, key rotation or local evidence decryption was performed.
`AGE_IDENTITY` name readback and final configuration PASS. The original identity
file and the other two Environments/policies/Secret metadata remain unchanged.

Installation does not qualify decryption: only a separately approved compatible
future protected run can establish that. No release workflow, new tag, formal
signing, registry upload or public release mutation occurred. The accepted alpha
D6 remains Complete. Automatic post-merge baseline CI
[40875245510/1](https://github.com/bohewu/yetlet/actions/runs/40875245510) PASS,
both required jobs;
it is a normal master push trigger, not a release dispatch or manual CI retry.

Local operational evidence is retained in `target/yt209-delivery-20261029/`:
`repair-ci-success.json`, `merge-result.json`, `environment-setup-result.json`,
`result-environment-readback.json`, Guard outputs and original failure logs.
These closeout documents remain uncommitted unless separately authorized.
