# YT-211 — Two implementation packages and consumer migration

Date: 2026-10-30. Status: Done at the authorized migration/local verification scope after independent Reviewer PASS0. The owner's continuation after
[YT-210](YT-210-PACKAGING-STUDY.md) authorizes the production source migration,
consumer/tool updates and affected local proof. It does not authorize Git
delivery, tag/hosted execution, signing, registry uploads or GitHub publication.
The owner subsequently explicitly selects two packages, `yetlet` and
`yetlet-browser`, before any production source migration. This replaces the
initial one-package contract; the initial P2 omission of rate limiting and its
failed review remain retained. The complete map below includes that capability.

## Selected package and API shape

Use two implementation packages at `crates/yetlet` and `crates/yetlet-browser`, with
unreleased development version `0.9.0-alpha.5`. Base default features are `["html"]`;
`default-features = false` on `yetlet` exposes the view model without external normal
dependencies. Preserve the root workspace's resolver3 and Rust1.99 baseline.
This development version is not a selected release C, tag or signed subject.

`yetlet` owns `view` (root `View`/`Element`/`Attribute` re-exports), `html`,
portable `actions` contracts/local executor/envelopes, `reactive`, `server_axum`,
`request_id`, `session`, `auth`, `session_redis`, `config`, `secrets`, `signing`
and `rate_limit`. Its matching optional features are `html`, `actions`,
`reactive`, `server-axum`, `request-id`, `session`, `auth`, `auth-session`,
`session-redis`, `session-redis-tls`, `config`, `secrets`, `secret-provider`,
`signing` and `rate-limit`. Rate limiting retains its native Axum/Tower/governor
boundary, accounting/Policy/Budget semantics, settings consumer and tests.

`yetlet-browser` depends on `yetlet` with defaults disabled, never conversely
(including development dependencies). It owns `dom`, `dom_keyed`, browser
`actions::HttpExecutor` and `wgpu`. Its features are `dom` (default), `dom-keyed`,
`actions`, `wgpu` and `wgpu-browser`. Browser `actions` enables portable
`yetlet/actions`; its HTTP implementation keeps its existing same-origin and
wire/error rules. The locally owned HttpExecutor implements the shared
ActionExecutor trait. Browser root View/Element/Attribute and its actions module
may re-export shared definitions for convenient imports; never duplicate them.
Portable reactive logic remains in `yetlet::reactive`, not a browser-only API.

`auth-session` selects the old Auth/session
bridge; `session-redis-tls` selects the old provider/tls feature; `wgpu-browser`
selects the old GPU/browser feature. No `full` preset or old-crate facade fleet.

Keep qualified capability names, existing type/method names and typed options.
Memory Session stays synchronous; the Redis/Valkey protocol provider stays
asynchronous. Settings loading and startup enablement remain separate choices.
No new runtime service, protocol, retry, admission policy, authorization rule,
cache, tracer or DI/AOP subsystem is introduced.

Move actual implementation/resources into modules in the two archives,
including the existing Lua and WGSL files. Old library Cargo manifests/source
are retired from the working tree; Git history preserves them. Historical guide
entry paths may contain short documentation redirects, not compiling shims.
Applications/examples/CLI remain separate unpublished packages, depending only
on these selected libraries for framework functionality. Native-only apps need
only `yetlet`; browser consumers select `yetlet-browser` and portable features
they use. The two-package order is `yetlet` then `yetlet-browser`, with no
reverse normal/build/development dependency and no registry bootstrap cycle.

## Feature and target boundaries

Default HTML/Core must not compile the HTTP, provider, cryptographic, browser
or GPU stacks. Native capability modules require both the feature and native
target; DOM/WebGPU bindings and browser HTTP execution require wasm32. The
base package has no browser SDK dependency. A feature union cannot expose native
Auth/Session/HTTP APIs or native networking/crypto/storage dependencies on
wasm32. Keep explicit `cfg` and optional target dependencies, not mutually
exclusive server/browser presets.

Serde/JSON are shared by browser Actions and native APIs. A union selecting
those features may compile shared serialization dependencies even when the
native module itself is target-gated. Do not claim zero extra shared-dependency
compilation for an arbitrary feature union. The YT-210 four-module prototype's
stronger no-serde result remains valid for its original inputs only.
The attempted dual-target alias probe is retained: Cargo rejects the same serde
version under two different names. No alias-based isolation promise is made.

Keep currently used third-party versions, not an ecosystem upgrade. Consolidate
compatible lock identities from the existing workspace/isolated locks. Native
Redis requires existing serde_json1.0.151; use that existing version for the
unified package rather than the standalone Secrets/Signing dev-only1.0.154 pin.
The separate CLI also aligns its former exact serde_json1.0.149 pin to this
already-selected1.0.151 identity; no new third-party version or checksum is introduced.
Record these compatible pin alignments explicitly. Preserve native TLS,
provider secrecy and browser serialization exclusions. Optional features do not
promise an empty resolver/lockfile or unchanged generated asset bytes.

## Consumer, guide and tool migration

Replace each active consumer's former library dependencies with `yetlet` and,
where needed, `yetlet-browser`, explicit additive features and defaults disabled where
appropriate. Preserve existing application feature names/runtime keys and
default-off settings. An optional application Auth/Valkey dependency becomes
an application feature forwarding to `yetlet/auth` or `yetlet/session-redis`,
not an always-enabled package feature. Retain one View type and migrate imports
coherently; no old/new crate-type interoperability is promised.

Migrate library integration tests/examples with feature requirements so base
Cargo commands do not force all providers. Reuse their existing test bodies
where the moved crate/import context changes; do not replay unchanged deployment,
cloud, capacity, hardware GPU or benchmark qualification. Existing ignored
external-service tests remain ignored unless a newly demonstrated affected path
requires a separately bounded local check.

Update current README, guides/rustdoc, CLI scaffolds, framework source closure,
delivery generator, sample/build/smoke source paths and active verification
commands. Keep build-time source selection closed and generated projects
independent of unshipped local crates. Retain historical validation narratives,
results, tags and versioned fixtures with their original cohort; do not rewrite
historical PASS to claim the new layout was tested.

## Release record and artifact boundary

Preserve schema1/`sixteen` records and their validation for old artifacts. Add
only the closed schema2/`consolidated` cohort with exactly
`{"yetlet", "yetlet-browser"}` as the new
library subject set; do not accept arbitrary package arrays. Package/source,
provenance, manifest, signature/result and public attachment checks derive the
expected count and filenames from the validated cohort. Old fixed payload bytes,
checksums, signature identities and acceptance records remain immutable.

New build/rebuild preparation normally verifies both final source packages in
dependency order, using the final base archive/checksum for the browser package;
native/browser consumers select the documented features. Preserve exact source/
version/tool/lock/archive binding, private custody and signer/verifier/publisher
separation. An old-source rebuild remains an operation at its old checkout;
the consolidated checkout must not pretend old manifests still exist.

Keep the final owner publication Environment gate and strict/builtin Git policy.
The old one-plus-fifteen continuation remains a legacy-only path and is not
valid for a schema2 record. No new concrete release record, candidate C/S/V/P,
tag, run, payload SHA or release approval is selected by this implementation.
Existing GitHub alpha.4, `yetlet-actions0.9.0-alpha.4`, all signatures and failures
stay untouched. PR29 remains open/unmerged and publication remains paused.

## Bounded verification and completion

Use the existing local Cargo/Python/xtask tests. Verify changed module imports,
feature boundaries, native API/security tests, doctest/current guide paths,
default/core-only and selected native/provider combinations, wasm32 with browser
features and a broad mixed feature union. A compile-refusal check confirms that
native modules stay unavailable on wasm32. Audit the real normal dependency
graphs; do not infer isolation from `cfg` spelling alone.

Verify normal source packaging and external consumers of the extracted archive,
then required CLI scaffold/delivery source changes and affected release-record/
subject checks. Compile changed example/workload sources; retain old runtime
measurements without claiming fresh qualification. No full feature power-set
matrix, new validator framework, benchmark campaign, Azure resources, live
Windows/browser qualification or hosted execution is part of this slice.

After local checks, obtain independent read-only final review. Only a PASS with
no material required followups can mark YT-211 Done. Report affected scope and
unexecuted qualification separately. Keep original failures and all existing
YT-210 work; leave new changes uncommitted and unstaged.

## Implementation and affected evidence — 2026-10-30

Both implementation archives, all 34 example manifests and the unpublished CLI
now use the selected module paths and feature boundaries. Old library sources,
Cargo manifests and independent locks are removed; short guide redirects and
historical license/navigation paths remain. There are no compiling old-crate
facades. The CLI's compiled source closure exports the complete base SDK and
rejects unknown files, changed bytes and links. Current guides, rustdoc, sample
inventory, build-source binding and release automation use the two-package map.
Historical document edits are limited to navigation to moved files; original
results, source/tag/run IDs, support scopes and old payloads remain intact.

| Affected check | Result and scope |
|---|---|
| Native SDKs | All-feature/all-target compilation PASS;169 base tests accepted (168 initial passes plus the repaired environment-only OpenSSL gate);18 Browser tests PASS;9 existing external-service cases remain ignored |
| Examples | Root all-target compilation and192 tests PASS; all8 isolated consumers compile with supported selected features; incompatible diagnostic features are checked separately, not combined |
| WASM | Two SDKs with all features PASS; six root browser consumers plus Browser Tasks, Physics Lab and Scene Island compile PASS |
| Documentation |11 SDK doctests PASS; current docs/internal links/Markdown/DAG PASS (75 documents,211 backlog items) |
| Moved SDK examples |21 tests PASS; actual listener-free Config/provider and Settings-backed default-off rate-limit startup checks PASS |
| Real local HTTP | Newly built Hello World, REST, Memory Session cart and Auth Notes smoke PASS, including the existing cleanup checks |
| CLI |88 distinct unit cases accepted after focused corrections; actual browser creation, selected service/settings/Vite generator export/compile and remaining Linux integration targets PASS; existing tool/external cases remain ignored |
| Normal packaging | Exactly two final archives verify in dependency order; Browser stages the normally verified final base archive/checksum. No bootstrap or disabled verification |
| Archive consumer | External registry-shaped directory-source consumer runs natively and compiles for WASM with broad selected features; cross-package View identity compiles; native Auth import on WASM is correctly refused |
| Dependency isolation | Core-only and default HTML normal graphs contain only `yetlet`; broad WASM union has no native HTTP, Session, crypto, storage, settings or rate-limit dependency stack |
| Existing dependency identities | Root, isolated consumers and CLI locks introduce zero external registry version/source/checksum identities beyond the pre-migration lock union |
| Closed release cohorts | Schema1/sixteen compatibility retained; schema2/consolidated rejects arbitrary names and legacy continuation; two-subject provenance, attachment names, predecessor ordering and token-free exact read URLs tested locally |

Retained local evidence is under `target/yt211-single-package-20261030/` (the
original folder name is preserved). It includes original source copies, move and
migration maps, raw stdout/stderr, commands/results, both normal packaging input
generations, final-source hashes and independent review. The first package proof
predates final guide/example changes and is not represented as the final bytes.

Failures remain recorded: initial contract P2 omission; rejected serde alias
probe; compile/fixture/old-golden corrections; initially missing generated `.mjs`
path (including the mistaken `.js` correction); stale generator lock/mutation
assumptions; explicit diagnostic feature conflict; stale workflow-test position;
OpenSSL host3.5.5 versus required3.5.9; and early smokes racing the rebuild.
The latter four stale-binary observations are not migration proof; the accepted
smokes ran after the native build finished. The OpenSSL gate uses a task-local
pinned3.5.9 prefix, without changing the system or weakening the assertion.
Moved fixture paths and example feature requirements are fixed and exercised.
No old failure or ignored case is counted as a pass.

The registry's read-only name checks returned404 for both new names at this
checkpoint. That observation does not reserve a name or authorize publication.
No fresh Windows, live browser/device/CSP qualification, Azure topology, external
Valkey/IdP, long workload, hosted signing or registry publication is claimed.
Prior evidence remains usable only at its original source and scope. The next
release must select new immutable inputs and follow the existing owner gate;
alpha.4's signed artifacts cannot attest to alpha.5.

### Independent final review followup

The first final review returned FAIL with two P2 findings and confirmed all441
source hashes/Git status unchanged during review. Its original result is retained.
The Auth authentication integration target now additionally requires `server-axum`;
`auth` itself still enables only its existing native Axum dependency. Auth-only
eligible-target compilation and the corrected Auth/server target compile PASS
without replaying their previously accepted runtime cases. Current bridge, TLS
and GPU guide/rustdoc feature names now match `auth-session`,
`session-redis-tls`, `wgpu-browser`; Auth-only and bridge-enabled Auth doctests
both PASS. Independent followup returns PASS with zero issues and required followups; all441
reviewed hashes and Git status remain unchanged throughout review. Combined
with the full implementation review, it explicitly accepts YT-211 Done at this
bounded scope.

Normal archive consumers and package proofs retain their recorded input hashes.
Subsequent changes are Cargo test-target eligibility, English guide/rustdoc and
the Auth guide example's feature guard, with no runtime/resource/API body change.
They are migration evidence, not selected or frozen release artifacts. A future
release must package its own reviewed immutable source; no prior archive is
silently promoted or represented as those later documentation bytes.
