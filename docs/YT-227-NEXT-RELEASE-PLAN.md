# Next SDK and CLI release plan — YT-227

**Subsequent completion (2026-11-02):**
[SDKalpha.6](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01)
and [CLI0.2.0](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02)
are published with verified registry consumers. The original planning-time
manifests, proposed decisions and Pending gates below remain historical; they
do not describe current publication status. The live guide edition stays pinned
and registry OIDC remains deferred.

**Authorized continuation (2026-11-01):** the owner selects preparation through
the final publication gate. [YT-228](ALPHA6-RELEASE-PREPARATION.md) records the
selected SDKalpha.6-first/CLI0.2.0-later sequence and the retained token route.
Planning-time Pending decisions below preserve their original review checkpoint;
the continuation does not authorize publication.

## Scope and status

Documentation-only planning, 2026-11-01. Inventory the unreleased SDK/CLI changes,
propose version and delivery order, identify affected qualification, and evaluate
crates.io Trusted Publishing. Independent review PASS, zero issues/followups. Completing this item
accepts a plan, not a new candidate, version, authentication policy or release.

No manifest, dependency, workflow, Environment or live Pages change is made.
No Commit/Push, tag, hosted execution, credential exchange, signing or publication
is authorized by this document. Existing release and failure records remain intact.

## Published baseline and proposed scope

Planning anchor: merged master
`09233b118848d24d7ce6c2a626150edb5beb83a4`. Its tree matches the current local
closeout commit `ace96c1bf817441c7c36c92006a84b0b13b7eac6`; neither is selected
as a new release candidate.

| Cohort | Actual published baseline | Proposed next delivery; not selected |
|---|---|---|
| SDK: `yetlet`, then `yetlet-browser` | `0.9.0-alpha.5`, tag `v0.9.0-alpha.5`, C `0a9fa8a594e3dd2ce753422dd848df10cd8c98df` | `0.9.0-alpha.6`, tag `v0.9.0-alpha.6`; schema2/`consolidated`, exactly two archives |
| Independent `yetlet-cli` | `0.1.1`, tag `yetlet-cli-v0.1.1`, C `ae5dd4c55e1124f56fcc3b07a9fc9e5bf083b25f` | `0.2.0`, tag `yetlet-cli-v0.2.0`; schema3/`cli`, exactly one archive |

The published [SDK record](ALPHA5-HOSTED-RECORD.md) and
[CLI installed-consumer record](YT-216-CLI-011-PATCH-RELEASE.md#completed-publication-and-installed-consumer--2026-10-31)
remain authoritative for those bytes. Read-only crates.io observations confirm
all three names exist and list `bohewu` as owner. This does not establish Secret
validity, token scope or a configured Trusted Publisher.

### Changes already implemented but not distributed in these baselines

| Surface | Included source changes | Compatibility and limits |
|---|---|---|
| SDK DOM | [YT-220](YT-220-HTML-DOM-IMPLEMENTATION.md): 89 ordinary elements, scoped attributes, canonical structure and native controls | Bounded inventory; not all HTML, foreign namespaces or executable content |
| SDK URL/image options | [YT-221](YT-221-DOM-URL-IMAGE-CONTRACT.md): `DomOptions`, `NavigationPolicy`, `ImagePolicy`, additive validation/mount entry points | Same-origin navigation and disabled images remain defaults. Per-mount opt-in HTTPS origins do not authorize arbitrary resources. Native validation is provisional; browser mount enforces origin and frozen-destination checks |
| SDK SSR | Initial `<pre>` LF/CR/CRLF preservation, included in YT-220 | Serialized leading newline bytes intentionally change to preserve parsed text; escaping and security checks remain |
| SDK documentation | [YT-217](YT-217-DOCS-PREVIEW.md) API doc comments and examples | No new Session behavior or mandatory feature |
| CLI | [YT-223](YT-223-STATIC-HTML-IMPLEMENTATION.md): `yetlet build --static`, finite native export protocol, root/project paths, create-only output and private receipts; explicit optional Wasm islands | Published CLI0.1.1 lacks this command. Do not promise `yetlet new --static` or a general Markdown-site generator. Normal commands remain compatible |

CLI0.2.0 is proposed for the additive static-build feature. The two-package SDK
boundary and optional server/browser capabilities remain unchanged. No third-party
upgrade is proposed. Current manifests still say SDKalpha.5/CLI0.1.1; those numbers
do not mean the newer source has already been published.

## Preparation gates discovered in current source

### G1: keep the published guide usable during development version changes

`site/pages.toml` selects SDKalpha.5/CLI0.1.1. The public-mode check in
`tools/build_docs.py::page_label` requires that edition to equal workspace and
CLI package versions. `examples/docs-site/prepare.py` also admits only that pair;
the guide consumer explicitly depends on the registry SDKalpha.5. `docs.yml`
builds in public mode on relevant changes, including root `Cargo.toml`.
Consequently, a direct version bump breaks the public build. Turning deployment
off does not disable this check.

Before a version bump, make the smallest reviewed metadata change that admits
the already published guide edition independently of development workspace
versions. Keep explicit approved edition/registry-consumer correspondence, clean
source and immutable source links. Reject unreviewed or unpublished candidate
editions; do not automatically turn them into public documentation or silently
switch a failing public build to preview. Focused checks must cover preserved
published edition with newer workspace versions and rejected dirty/unadmitted
inputs. Existing content/browser proof remains reusable when its inputs match.

Promoting the guide to SDKalpha.6/CLI0.2.0 is a later, explicit consumer update
after both versions are actually published. The live guide and its deployment
policy remain unchanged in this planning item.

### G2: bind each CLI version to its actual published SDK

The CLI manifest pins registry SDK `=0.9.0-alpha.5`. Its package-owned
`resources/workspace.toml`, `framework.json` and `framework.lock` supply generator
versions and embedded source. `scripts/release_record.py::CLI_SDK_VERSION` is
also fixed to alpha.5; `release_registry.py` and `release-publish.py` use it to
check archives, generated consumers and receipts.

Recommend SDK first, then CLI. After SDKalpha.6 is public, coherently update the
new CLI's registry pin, bundled qualified source/lock, generated dependencies and
release checks. A minimal trusted version-specific binding may retain
CLI0.1.0/0.1.1 → SDKalpha.5 and admit CLI0.2.0 → SDKalpha.6. Review the implementation
before adopting it. Keep schema3 closed; do not accept an arbitrary request-supplied
SDK version or reinterpret historical CLI records by replacing one global constant.
Check both the preserved old binding and the proposed new binding.

Do not bulk-replace independent registry consumers with an unavailable version.
During SDK preparation, update affected workspace/path dependency edges and
lockfiles without requiring the not-yet-published SDK for a CLI build.

### G3: registry OIDC is an optional policy change

Current publisher P deliberately has no OIDC under
[ADR0009](adr/0009-release-build-and-signer-identities.md#single-publication-policy-amendment--2026-10-30).
The final `release-publish` job uses the Environment's `CRATES_IO_TOKEN`; it runs
GitHub publication/readback and registry upload in one Python invocation.
Trusted Publishing therefore needs a reviewed privilege and phase change, not
just a replacement environment variable. The existing token route remains the
current contract unless a separate decision adopts the option below.

## Optional crates.io Trusted Publishing

The [current official crates.io documentation source](https://github.com/rust-lang/crates.io/blob/9839ef7bf9b776dcd322a9765ab3d2809faf45ea/svelte/src/routes/docs/trusted-publishing/%2Bpage.svelte)
requires an existing crate and owner-configured publisher, matching repository,
workflow filename and optional Environment. It supplies a 30-minute token and
requires `id-token: write`. All three names now meet the name-existence prerequisite.
Proposed registration for each is owner/repository `bohewu/yetlet`, workflow
`release-publish.yml`, Environment `release-publish`; this has not been configured
or tested here.

The [official auth action](https://github.com/rust-lang/crates-io-auth-action)
outputs a token and revokes it in its post step. The inspected action has no TTL
input. Its inspected main SHA is research evidence, not a selected production
pin; select a reviewed released full SHA during any implementation. The
[official Rust update](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/)
documents optional Trusted-Publishing-only enforcement and blocked
`pull_request_target`/`workflow_run` triggers. Current explicit dispatches do not
use those triggers. Registry authentication is separate from Sigstore signing.

Recommended migration, if selected:

1. Amend ADR0009 and the publication contract explicitly. Keep the final owner
   gate, exact master/source CI checks, no age identity, and isolated independent
   verifier. [Job permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions)
   are job-scoped: all code in the authorized job needs
   review; a late action alone does not make that permission step-scoped.
2. Establish a late authentication boundary after successful GitHub publication
   and readback, immediately before registry promotion. Preserve the same frozen
   archives, order and partial-publication stop behavior. Do not spend the short
   credential lifetime on earlier GitHub work.
3. Mask the ephemeral token, keep it out of artifacts and generated consumers,
   and revoke on completion. No automatic token fallback, renewal or upload retry.
4. Obtain authority for per-crate registration and a bounded auth-only exchange/
   revocation rehearsal in the protected publication context. That rehearsal
   needs its own one-time owner gate; it is not an upload test. Dry-run does not
   prove registry authentication. Never probe an existing-version PUT or create a
   throwaway crate as a credential check.
5. Keep the current Secret until a separately authorized cutover. Enabling
   Trusted-Publishing-only enforcement disables traditional-token publishing;
   Secret removal and rollback choices must be explicit.

Recommend evaluating this bounded migration before the next publication, while
allowing the existing token route if the owner defers it. Research alone neither
blocks a token-based release nor grants new publisher permissions.

## Execution order after separate authorization

1. Confirm the proposed versions/order and choose whether to adopt OIDC. Resolve
   G1 before changing versions; resolve G3 only if that auth option is selected.
2. Prepare SDKalpha.6 and affected locks/packages, reconcile the evidence below,
   independently review and deliver via authorized Git/CI. Only then select
   immutable C_sdk. Future tag/build/custody/prepare/sign/result/publication each
   need their applicable authority and fresh version-specific bindings.
3. Publish the two verified SDK archives in dependency order. After registry
   readback, resolve G2, prepare the independent CLI0.2.0, verify its actual binary
   and generated registry consumer, deliver and select C_cli.
4. Execute the independently bound CLI release; do not republish SDK. Each closed
   cohort retains its own final owner publication approval. This is two releases,
   not an invented three-package cohort or a single approval for arbitrary bytes.
5. Separately qualify and promote the new public guide edition after both
   publications. Do not treat a development version bump as that promotion.

C_sdk/C_cli and their distinct signer S, independent verifier V and publisher P
are all unselected. Preserve existing tags, failed attempts, signatures and
checksums; changed bytes require new records, hashes and signatures. Derive the
SDK's six public files and CLI's five through existing closed-cohort helpers.

## Reuse and missing verification

| Area | Reuse at its recorded scope | New proof needed for the later delivery |
|---|---|---|
| SDK behavior | YT-217/220/221 doc/native/Wasm/Chrome/Firefox proof when actual product inputs correspond | Final normally verified archives, exact base checksum staged for Browser, affected native/Wasm external consumers exercising the new options and unchanged defaults; final MSVC archive compilation where changed inputs require it |
| CLI behavior | YT-223 export, refusal/cancellation/cleanup, GNU/MSVC and browser results for matching inputs | Version/resource/binding consistency, normal final archive and installed real binary exact version/help, no-clone generation, selected root/project static export and optional island; affected MSVC final archive/build |
| Public guide | YT-225/226 accepted content, build artifact, HTTPS and reader journeys | G1 metadata checks; later edition/registry-consumer promotion separately qualified |
| Release helpers | Existing closed schemas, build/sign/verify/publication isolation and retained successful receipts | G2 focused old/new binding checks; if OIDC is adopted, G3 focused phase/auth tests and separately authorized real rehearsal |
| Inventory/security | Existing dependency and feature-isolation evidence with matching input closure | Inventory, checksums and security-coverage reconciliation for new archive/lock graphs; only uncovered changed edges require checks |

Old candidate PASS never becomes new candidate certification by changing a label.
Do not replay the full Auth/Valkey/Azure/GPU/benchmark matrix for metadata changes.
No dependency upgrades, audit ignores or unrelated feature changes belong to this
plan. Retain failures and stop on failed gates; do not automatically retry.

## Decisions and acceptance

| Decision | Recommendation | State |
|---|---|---|
| Versions and order | SDKalpha.6 first, independent CLI0.2.0 second | Pending owner selection |
| Guide lifecycle | Preserve published edition through development; promote after both releases | Proposed prerequisite; no implementation |
| CLI/SDK binding | Trusted per-CLI-version binding and coherent bundled resources | Proposed prerequisite; no implementation |
| Registry authentication | Reviewed OIDC migration/rehearsal, or explicitly retain current token route | Pending owner selection |
| Candidates and execution | Select only after reviewed preparation/qualification | Unselected / not authorized here |

Planning evidence is retained outside the repository under run
`yt227-next-release-plan-20261101`: Git/tag deltas, public registry/owner snapshots
and pinned official-source research. No credential was inspected. This round's
validation is limited to current documentation links/structure/backlog/whitespace
and independent review; prior product/hosted evidence is not rerun.

Initial documentation check: `cargo xtask check docs` PASS (75 required documents,
227 backlog items; links, Markdown structure, whitespace and dependency DAG).
`git diff --check` and explicit six-file whitespace check PASS. Independent
Reviewer PASS0: all31 bound source hashes, six frozen documents and Git diff/status
match. Registered reviewer/model and effective high effort are verified.
YT-227 is Done at planning scope only. Subsequent edits record acceptance/status;
the proposed decisions, product inputs and execution limits remain unchanged.
