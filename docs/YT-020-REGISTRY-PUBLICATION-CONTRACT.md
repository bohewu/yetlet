# YT-020 — First registry publication and sixteen-artifact binding

Date: 2026-10-30 (Asia/Taipei). Contract and isolated verification accepted after
independent re-review PASS, zero issues/required followups. The initial P2 FAIL
remains retained. **No registry upload is authorized.**

## Authority and identities

The owner authorized strict/builtin commit of the five preceding dependency-study
documents, then this contract, minimal new verification and independent review.
That commit is `15abc44c2add0bf1aaff37d1921599c89a5a164e`, tree
`fc82cffde41f82ceb77b2113fb44fbbb34516d4b`; direct Guard/native hook PASS,
no Push. The initial Guard refusal of two slash-separated noun sequences is
retained; only prose separators changed before the successful check. New work
remains uncommitted and may not publish, sign, create tags or run hosted jobs.

The owner reports having completed `cargo login`. Treat credential installation
as owner-reported complete, not a request to install another token. This work
does not read credential values, invoke a credential provider, validate token
scope/email/account rights, or use an authenticated registry endpoint.

The historical product baseline remains C
`557ab76e03b619e959eae7529dbe03317fe1f952`, alpha.3, with S/V `30baf25` and its
accepted sixteen subjects. Existing tags, artifacts, provenance, signatures and
failed observations remain immutable. A future changed registry source **R** and
coordinated version **V** must be selected after source preparation/review and an
authorized commit. Recommend the next prerelease train, rather than silently
assigning different source contents to the existing alpha.3 tag. No R, V or new
tag is selected here; prototype alpha.3 inputs are not a new release candidate.

## What the registry actually requires

The public [crates.io upload implementation](https://github.com/rust-lang/crates.io/blob/main/src/controllers/krate/publish.rs)
converts normal, build, target and development declarations from the archive's
manifest. Its `add_dependencies` lookup rejects a dependency whose crate name is
unknown; it does not exclude development kinds. Changing only the upload JSON
cannot hide a declaration: the server derives dependencies from Cargo.toml.
This is an observation of upstream source, not proof of the exact deployed
revision or permission to test a production upload.

Therefore the unchanged alpha.3 Config/Secrets name cycle has no approved first
upload order while both names are absent. A token, a native multi-package Cargo
command or exact-byte API transport does not solve that server-side condition.
The earlier [prospective-source dry-runs](YT-020-FIRST-PUBLISH-CONTRACT.md) remain
valid local compilation observations; they did not prove initial name acceptance.

## Selected source correction and order

Keep Rust library APIs, runtime behavior, Cargo feature boundaries and separate
workspaces. Make the following source correction in a later preparation slice:

1. Move the listener-free Settings-to-State example from Secrets to Config,
   which owns the Settings integration. Move its four synthetic settings files
   into a distinct Config example directory; change only that fixture path.
2. Register the relocated `secrets_state` example with Config's existing
   `secret-provider` required feature. It must not enter default builds.
3. Remove Secrets' reverse Config development dependency. Do not leave a
   published example that imports an omitted development dependency, introduce
   a runtime cycle, or create placeholder registry versions to reserve names.
   Existing Core/Server development declarations may remain for this bounded
   change; no general dependency cleanup is needed.
4. Update live example commands, current documentation and affected CI consumers
   when implementing the move. Preserve dated evidence and original commands
   in historical records. Reconcile affected lockfiles without third-party
   upgrades and compile the relocated target before accepting new source R.

This turn applies the move **only to an isolated prototype**, not repository
manifests, examples or locks. It validates the selected design without silently
changing the published source baseline.

Publication order includes **all** internal dependency kinds and optional/target
declarations. Every referenced internal name must exist before its dependent
upload; require the intended version/checksum to be indexed as an additional
batch control. The corrected graph must contain sixteen nodes and seventeen
internal declarations with no cycle. An explicit total order is recorded by the
prototype result; unrelated packages at the same dependency layer may vary.
No repeat of old runtime, browser, platform, security or load tests is required
merely to prove this graph.

## Selected artifact and upload method

Use Cargo for normal package build verification, then promote the **frozen
`.crate` file**, rather than invoking `cargo publish` to regenerate it after
artifact acceptance. The official [Registry Web API publish protocol](https://doc.rust-lang.org/cargo/reference/registry-web-api.html#publish)
accepts metadata and the original archive bytes. A future small maintainer
transport may implement that protocol after separate implementation review;
this contract does not add a network-capable uploader.

For R, build the sixteen final artifacts in the corrected dependency order.
Reuse the established private package-preparation machinery, with normal
extracted-package verification enabled. Stage a prerequisite's **final archive**
and checksum before packaging a dependent. Keep development declarations in the
published manifests and use actual predecessor digests in generated package
locks. The resulting acyclic graph avoids trying to solve mutually referential
archive hashes. Prospective directory inputs are a disclosed private build
arrangement, not a claim that the versions already exist in crates.io. They are
not passed to a live Cargo publish operation. No verification bypass or false
registry-presence receipt is permitted.

Freeze the full set before any real upload. The immutable publication plan must
bind the following fields, and reject missing/Pending **plan fields** before
network mutation:

| Binding | Required contents |
|---|---|
| Source and version | Full immutable R commit/tree, selected V, source archive digest, exact preparation recipe/toolchain and input-lock identities |
| Exact cohort | The sixteen library names in the existing release inventory; no CLI, sample, benchmark, vendor or loader package |
| Each artifact | Name, V, exact filename, byte size, SHA-256 of the compressed `.crate`, normalized manifest/packaged-lock identity and corresponding provenance subject |
| Upload metadata | JSON derived from that archive's manifest/README, including aliases, normal/build/dev kind, target, optional/default features and feature map; serialized metadata SHA-256 |
| Order | Corrected all-kind dependency order; prerequisite name/version/index checksum constraints |
| Request | Exact framed-body digest plus its metadata and archive lengths; destination fixed to crates.io's official publish endpoint, no redirects or arbitrary destination |
| Authority | Approved batch/source/cohort, intended publisher, permitted attempts and failure limits |

Keep subsequent receipts separate from that frozen plan, linked by its digest.
Each package initially has status `not_attempted`; future outcomes and readback
proof are not prerequisites for starting the batch. Append each actual attempt,
upload response, index checksum and downloaded-archive result as it occurs. A
dependent upload requires its predecessors' matching index/download receipts.
The final registry consumer result is required before declaring batch completion,
not before the first upload. Never fill an unexecuted receipt with a synthetic
PASS or mutate frozen identities/expected digests to fit an observed result.

Keep new registry subjects/provenance separate from the old alpha.3 record.
Reusing unchanged Rust verification does not make new metadata/archive bytes
covered by an old signature. Any signed registry distribution requires its own
accepted artifact subjects; no signing occurs in this phase.

The request is four-byte little-endian JSON length, JSON bytes, four-byte
little-endian archive length and **unaltered compressed archive bytes**. Metadata
and request hashes supplement the artifact hash; the registry index/download
checksum must equal the artifact hash, not the request-body hash. Do not unpack,
recompress, patch Cargo.lock or remove dependencies while transmitting a frozen
artifact. Reject any manifest/metadata name or version mismatch.

After separate batch authorization, upload sequentially using the selected
credential mechanism, without token arguments, token logging or automatic
retry. The existing Cargo login is the starting credential arrangement; verify
its effective account, verified email and `publish-new` rights at the proper
authorized stage. A future transport must support that selected credential
arrangement explicitly; it must not silently bypass a configured provider.
Successful PUT alone is insufficient: wait for the exact index entry and compare
downloaded archive bytes before proceeding to dependents. On timeout, 403, rate
limit, byte mismatch or an uncertain write, retain the result and stop. Inspect
actual registry state before any separately authorized retry. At completion,
all sixteen downloads and a clean registry-resolved consumer must pass; local
directory-source proof does not substitute for that final gate.

## Minimum new verification fixed before execution

- Reuse accepted alpha.3 source/archive identities and prior qualification.
  Inspect its all-kind graph as the negative control: it must remain blocked by
  the original Config/Secrets cycle; never label it upload-ready.
- In one temporary source copy, perform only the proposed example relocation,
  fixture-path adjustment, target registration and reverse-edge removal. Inspect
  Cargo metadata, prove sixteen-node all-kind order/closure and compile the
  relocated example's test target without executing old tests. Pin existing
  Rust/Cargo 1.99 directly; copy no credentials and make no registry write.
- For each of the sixteen **historical** accepted archives, construct an offline
  publish-envelope preview under the documented API format. Decode the framed
  bytes and compare its archive slice to the accepted artifact/provenance digest,
  and metadata identity to the archive's manifest. This is new transport/identity
  evidence, not another package build or server-acceptance test. Every preview
  remains explicitly ineligible for upload because the original graph is blocked.
- Preserve commands, failures and results, remove temporary source, build and Cargo home
  directories, check the new English documents and obtain independent Reviewer
  acceptance. No new general validator, mock registry, uploader, installed tool,
  actual name claim or authenticated endpoint is needed for this verification.

This selects the formal method and binding rules, not the future R artifacts or
upload authority. Subsequent source preparation and exact-byte transport
implementation are distinct from this contract/prototype acceptance. YT-020
remains Blocked until their evidence and actual registry activation gates close.

## Results and review

| New observation | Actual result and limit |
|---|---|
| Original signed cohort graph | Eighteen internal declarations; Config/Secrets cycle remains and Rate Limit waits on Config. Not an approved initial-name upload set. |
| Isolated relocation prototype | Sixteen nodes, seventeen internal declarations, one removed development edge, no added dependency edge and no cycle. Runtime libraries unchanged. |
| Cargo metadata for Config and Secrets | Both offline/locked observations exit 0; Secrets no longer has the Config dependency/old state target, and Config's relocated example requires `secret-provider`. |
| Relocated example test-target compile | Pinned Rust/Cargo 1.99, offline/locked `cargo test --example secrets_state --features secret-provider --no-run` exits 0. No old test was executed and no new runtime-behavior pass is claimed. |
| Prototype preservation | Example Rust bytes differ only in the settings fixture path; all four fixture files and three relevant source locks remain byte-identical to C. |
| Exact-byte protocol previews | Sixteen offline request envelopes decode to the original archive bytes and accepted provenance digests. Manifest name/version and metadata identity match. Zero HTTP requests or credential reads. All previews explicitly ineligible for upload. |

The new all-kind layers are:

| Layer | Packages |
|---|---|
| 1 | `yetlet-actions`, `yetlet-core`, `yetlet-reactive`, `yetlet-signing`, `yetlet-wgpu` |
| 2 | `yetlet-dom`, `yetlet-dom-keyed`, `yetlet-html` |
| 3 | `yetlet-server-axum` |
| 4 | `yetlet-request-id`, `yetlet-secrets`, `yetlet-session`, `yetlet-session-redis` |
| 5 | `yetlet-auth`, `yetlet-config` |
| 6 | `yetlet-rate-limit` |

The first independent review found one P2: the original pre-upload table also
required completed post-upload results. This would make the first operation
impossible. The correction separates immutable plan requirements, initially
unattempted receipts, dependency readback gates and final completion evidence.
The initial FAIL remains retained; no compilation, package or upload observation
is replayed for this document-only correction. Independent re-review PASS,
zero issues and required followups, accepts the correction.

The first graph-collection wrapper assumed an inherited workspace dependency
was always a table. It failed before Cargo started when it encountered a version
string. A single normalization correction reused the same temporary source;
the first failure remains. No product change, fresh verification system or
replay of accepted compilation was used to address it.

Local observations, commands, prototype inputs, upstream source snapshot/digest,
metadata, graph and sixteen envelopes remain in
`target/yt020-registry-binding-20261030/`. The upstream source is a dated main-branch
observation, not a deployed-service attestation. Owned temporary source, Cargo
home and build directories are removed; prototype inputs/logs and protocol
previews remain retained. Independent Reviewer
`/root/yt020_registry_publication_review_20261030` re-review PASS0; all five source
hashes match before/after review, with native reviewer/model/effort verified.
Only completion metadata follows acceptance. Docs/links/Markdown/DAG and focused
whitespace checks PASS; accepted old tests were not rerun. Repository library inputs and
existing release evidence are unchanged. No token/scopes were read, actual
registry write, new release candidate/version, signing or publication occurred.

**Next source slice:** implement the reviewed example/metadata correction and
topological final-artifact preparation, reconcile affected locks/consumers and
prepare the exact-byte transport. Apply only necessary affected verification;
then select/freeze R/V and all sixteen actual registry subjects before obtaining
upload authority. The current protocol previews are historical-artifact test
inputs, not future registry artifacts. YT-020 remains **Blocked**.
