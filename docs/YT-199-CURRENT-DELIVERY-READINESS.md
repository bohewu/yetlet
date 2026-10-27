# YT-199 — Current delivery scope and evidence gaps

Date: 2026-10-26 (Asia/Taipei). Status: **Done at documentation-planning scope
after independent Reviewer PASS, zero issues and required followups.** This is
source-based planning, not product qualification.

## Authority and selected planning contract

The owner approved a current-version delivery-scope and evidence-gap review:
identify the package/application boundaries, reconcile retained evidence with
the current source, record remaining decisions and update Backlog/Roadmap.
Work is documentation only, with independent review before Done. Do not select
a new release candidate, add features/samples, replay accepted verification,
build/test/scan the product, allocate resources, sign, publish, Commit or Push.
The repository-required documentation check validates this new delivery only.

The inspected source is clean commit
`64a6d1c408a0b7a6234620576b4d4820394d28a4` on
`docs/yt113-123-opt-in-capability-plan`. It is a **comparison snapshot**, not a
new candidate. The selected immutable candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, tree
`0c5fa71179741f7b52dead078ff6d8a9efc30241`, original nine experimental 0.1.0
libraries, Rust MSRV 1.99 and qualification pin 1.99.0. See the
[candidate contract](YT-096-CANDIDATE-QUALIFICATION.md).

Keep [YT-096](YT-096-TRUST-DISCLOSURE-DECISIONS.md) In progress with D1–D6
Pending and YT-020 Blocked. The original candidate review remains
[FAIL / release-ready NO](YT-096-SECURITY-RELEASE-REVIEW.md), with the later
[owner support/inventory disposition](YT-096-FINAL-RECONCILIATION.md#owner-support-and-inventory-acceptance-2026-10-20)
preserved. Completed backlog items establish their recorded scopes, not current
whole-product certification. Source-checkout adoption, package publication and
an application's production deployment require distinct evidence and decisions.

## Delivery classes and recommendation

These classes describe actual source and recommend a future boundary. They do
not select a distribution, promote an API classification or promise SemVer.
Use the [current API map](API-REFERENCE.md), [compatibility policy](COMPATIBILITY.md),
[naming/migrations](NAMING.md) and [opt-in guide](OPT-IN-QUICK-REFERENCE.md).

| Class | Current contents | Recommended disposition and limit |
|---|---|---|
| Original package candidates | Core, HTML, Axum adapter, DOM, keyed DOM, actions, reactive graph, request ID, Memory Session | Keep the selected nine-package candidate intact. If delivering later source, select a separate explicit revision and reconcile its affected evidence; do not transfer old archive digests, signatures or support claims |
| Later optional packages | Auth, rate limit, config, secrets, signing, Redis-protocol Session provider, wgpu | Retain explicit source-checkout use and `publish = false`. Consider each for distribution only after a separate scope/version/dependency decision; the API map's sixteen entries are not sixteen approved releases |
| Application-owned integrations | Board/Admin write authorization, request/worker tracing and Valkey/Auth; Shop display-data cache; Starter deployment/admission/recovery references | Reusable reference code with typed opt-in startup, not framework-wide Audit/Cache/Tracing crates or mandatory Settings. Application data, authorization, capacity and deployment choices retain their owners |
| Browser assets and tools | CLI/templates, common pre-Wasm shell, Tasks/islands; experimental physics Lab and its private renderer/Rapier scenes | Independently identify source/assets/tool requirements. Loader assets are opt-in application inputs, not a new runtime crate. Lab and estimated FPS do not establish general GPU/browser or production support |

All sixteen packages remain unpublished and experimental. The historical
[API classifications](API-CONTRACTS.md) govern their recorded scope, not an
automatic current promotion. In particular, actions/reactivity/browser APIs and
metadata conveniences retain their stated limits. The original nine manifests
have no package feature table at this snapshot; target-conditioned browser
dependencies still matter. Compiling a feature makes an optional capability
available; explicit startup options install it. Settings loading stays optional.

## Original nine: source delta and reusable evidence

The comparison below uses Git bytes from ce56e08 to 64a6d1c, not executed checks.
It distinguishes production code, test-only changes and unchanged source. It
does not establish archive equality: READMEs/tests, root workspace membership,
Cargo.lock, target features and transitive resolution can change package/build
inputs even when a library's production source is unchanged.

| Package | Inspected delta | Retained evidence and current-source limit |
|---|---|---|
| `yetlet-core` | `src/lib.rs` adds only a cfg-test; production implementation and manifest unchanged | [YT-152](YT-152-FIRST-CHILD-CAPACITY.md) rejected the measured one-slot policy and restored production code. Its archived experimental variant is not the default implementation. The final test-bearing checkout was not separately rebuilt there |
| `yetlet-html` | Rendering rules/errors and form/textarea handling changed in YT-110; later private escape appends and streaming URL checks | [YT-110](YT-110-DOM-CAPABILITIES.md), [YT-147](YT-147-PRIVATE-ESCAPE-INTO.md) and [YT-149](YT-149-PRIVATE-URL-PREFIX.md) provide reviewed bounded evidence. YT-110 includes new `RenderError` variants and input rejection, so this is not only a performance delta from the old candidate |
| `yetlet-server-axum` | Document body plus suffix capacity reservation, with explicit body release | [YT-154](YT-154-DOCUMENT-CAPACITY-COMPARISON.md) and later [HTTP SSR comparison](YT-156-HTTP-SSR-COMPARISON.md) retain source/workload-specific evidence; no blanket host/SLO or current archive qualification |
| `yetlet-dom` | YT-110 finite vocabulary, validation, native controls and navigation; manifest adds the `web-sys` Url feature | [YT-110](YT-110-DOM-CAPABILITIES.md) retains native/Wasm/browser evidence and failure history. Old ce56e08 browser qualification does not qualify the expanded policy/dependency feature |
| `yetlet-actions` | Rust source and package manifest unchanged | Retain candidate and selected consumer evidence; unchanged library source alone does not prove current build/archive closure |
| `yetlet-reactive` | Rust source and package manifest unchanged | Retain candidate weak-handle snapshot/local ownership evidence; changed consumers do not expand graph guarantees |
| `yetlet-dom-keyed` | Rust source and package manifest unchanged | Retain bounded keyed/listener lifetime evidence; no generic DOM identity or transactional rollback promise |
| `yetlet-request-id` | Rust source and package manifest unchanged | Retain scoped process-local counter evidence; later tracing propagation does not make IDs distributed trusted identities |
| `yetlet-session` | Rust source and package manifest unchanged | Retain bounded Memory Session/CSRF evidence; this is neither durable/distributed storage nor authentication |

Root Cargo.toml and Cargo.lock have changed since the selected candidate,
including Auth/rate-limit workspace membership. Do not infer current resolved
dependency equality or server-dependency absence from this file comparison;
existing feature/graph evidence retains its own recorded inputs. Exact current
package inputs, packaged consumer resolution and inventory are **Not verified
by YT-199**. There is no newly identified runtime defect from this planning work.

## Seven later packages: actual opt-in boundaries

All seven explicitly declare `publish = false`. Auth/rate limit are root
workspace members at inherited version 0.1.0; the other five use isolated
workspaces. Versions below are manifest values, not release decisions.

| Package | Compile selection / version | Existing evidence and remaining distribution boundary |
|---|---|---|
| `yetlet-auth` | Native-only; default features empty; `session` selects the optional Memory bridge; 0.1.0 | [YT-114](YT-114-IMPLEMENTATION.md)/[YT-115](YT-115-SESSION-AUTH.md), naming migration and resolver consumers. Principal is verified identity input, not business authorization. No generic OIDC/Bearer protocol implementation is selected |
| `yetlet-rate-limit` | Native dependency; no package feature table; 0.1.0 | [YT-117](YT-117-RATE-LIMIT.md) and [naming](NAMING.md). Policy reuse and shared Budget accounting are distinct; runtime enable/policy settings belong to the consuming app, with the documented example default off. No distributed quota |
| `yetlet-config` | Explicit dependency; optional `secret-provider`; 0.0.0 | [Configuration](CONFIGURATION.md), protected SecretBinding and [generator source-closure repair](YT-102-GENERATOR-SOURCE-CLOSURE.md). No ambient mandatory Settings service/live reload; release version and package closure remain separate decisions |
| `yetlet-secrets` | Explicit native dependency; no package feature table; 0.0.0 | [Provider contract](SECRET-PROVIDERS.md), missing/access/unavailable distinctions and explicit Settings snapshot replacement. No automatically installed vault/rotation or browser serialization |
| `yetlet-signing` | Explicit native dependency; no package feature table; 0.1.0 | [Current API/source](API-REFERENCE.md) and [public-key naming migration](NAMING.md). Key material membership is not trust. Local backend evidence does not activate release signing or imply general PKI support |
| `yetlet-session-redis` | Default features empty; explicit native `tls`; 0.0.0 | [Provider guide](../crates/yetlet-session-redis/README.md) and [YT-184](YT-184-AZURE-AUTH-SESSION-RESTART.md). Async Redis protocol/Valkey reference is not interchangeable with synchronous Memory Session. Selected cross-host/persistence restart evidence is not HA/failover qualification |
| `yetlet-wgpu` | CPU-only defaults; explicit WASM `browser`; 0.1.0 | [Bounded support](../crates/yetlet-wgpu/SUPPORT.md). Its Rust source/manifest/lock are unchanged against ce56e08, but it was outside that nine-package candidate. The Lab uses an application-private renderer; its 3D success does not promote the flat-scene crate |

OIDC/Bearer/Cookies, Audit, data/output cache and tracing research Done statuses
do not imply that corresponding generic crates exist. Current application-owned
write fencing, tracing and Shop cache are separately implemented and reviewed;
see [capabilities](CAPABILITIES.md) and [YT-185 options](YT-185-OPT-IN-OPTIONS.md).
Server identity, provider handles, secrets and worker capabilities must stay out
of browser dependencies and serialized hydration state (ADR [0005](adr/0005-opt-in-capability-modules.md)).

## Evidence gaps and smallest future actions

These are source-bound planning dispositions, not newly executed tests, detected
vulnerabilities or an instruction to replay old qualification. The existing
contracts below determine a later selected delivery's requirements. Unselected
packages/platforms do not create new blockers for the frozen nine-package scope.

| Boundary / authority | What is already available | Missing claim or decision; minimum next action if selected |
|---|---|---|
| Source, versions and API scope — candidate/compatibility contracts | Frozen nine-package ce56e08 candidate; current sixteen-package API map; explicit naming migrations | Owner has not selected 64a6d1c or all sixteen for release. Recommend a separate later revision with the original nine as the initial scope; evaluate additions individually. Keep old candidate/evidence intact and record changed rejection/error APIs and migration obligations |
| Packaging, dependencies, licenses and packaged consumers — release supply-chain contract | Historical nine-archive clean-consumer, target inventory and rebuild records; individual later consumer checks | No consolidated archive/consumer/inventory result is established here for the comparison snapshot. After scope selection, compare exact package inputs, relevant locks/features and retained evidence; perform only missing selected-input packaging/consumer checks. Publishing later path-only dependencies would need separate manifest/version work |
| Native/browser support — candidate contract and selected feature boundaries | Recorded Ubuntu/Windows/Wasm/Firefox candidate proof; YT-110 changes; [YT-196 startup](YT-196-WASM-STARTUP-SHELL.md) and real Tasks BFCache continuation; [YT-198 Lab](YT-198-PHYSICS-LAB-SLOPE.md) Firefox/private-Weston observations | No blanket current Windows/Chrome/WebKit/mobile/GPU qualification. Historical Chrome lacks independent candidate-wide binding. Map selected source/assets/features to exact evidence first; qualify only uncovered selected behavior/targets or explicitly choose a narrower supported scope |
| Security source/coverage and findings — release security-acceptance contract | Original [YT-067 scan/fix](M10H-VERIFICATION.md), [candidate security review](YT-096-SECURITY-RELEASE-REVIEW.md), six-file [Auth scan](YT-115-SESSION-AUTH.md#auth-group-security-scan--2026-10-22), later item-specific reviews | No whole-current-tree security-scan claim. The old Auth scan is not blanket coverage of later naming, Valkey resolver, TLS, write-authority, worker/cache or loader changes. Establish exact prior input bindings and uncovered changed trust boundaries; recommend a bounded security-delta assessment for selected later delivery. No new scan or findings disposition occurs now |
| Reader documentation and adoption — accepted publication-scope contract | [First app](FIRST-APP.md), [troubleshooting](TROUBLESHOOTING.md), API/rustdoc ownership and [selected manuscript inputs](DOCUMENTATION-SCOPE.md) | No generated rustdoc/site or complete publication/privacy qualification is established here. For an actual reader release select exact pages/assets, check their complete contents/links and generate target/feature-specific API output only when authorized. Do not export all docs/history as a website |
| Trust, registry and public activation — existing YT-096/YT-020 gates | Reviewed proposal, explicit metadata disclosure questions, accepted bounded support/inventory disposition | D1–D6 remain Pending; formal trust, actual registry proof, public reporting/activation and final approval remain open. Future planning cannot substitute for explicit decisions and actual evidence. No signing, workflow execution, registry access or publication is authorized |

The old source-scoped security findings and scan coverage are retained as-is;
independent implementation reviews are useful adjacent evidence, not a renewed
scan. No old PASS/FAIL, waiver or missing evidence is rewritten. Likewise,
accepted performance/soak results describe their recorded workload and topology,
not arbitrary application capacity or current-version latency guarantees.

For GPU startup specifically, this host's recent default-Chrome page opening
did not demonstrate a working 3D Run. The accepted Lab path is the recorded
isolated Firefox/private-Weston configuration. Neither this failed preview nor
the selected success establishes that all Chrome installations require Weston.

## Framework readiness versus deployment qualification

The [production profiles](PRODUCTION-DEPLOYMENT-PROFILES.md),
[Phase 2 matrix](PRODUCTION-READINESS-PHASE2.md#final-production-readiness-matrix),
write fencing and cross-host restart references provide concrete integration
choices. Keep typed application options, existing limits, zero added automatic
retry/queue promises and authoritative Session/authorization/SQL boundaries.
An adopter still selects business data, hardware, SLO, proxy/IdP, persistence,
HA/failover, backup/restore, retention and operational responsibility. Those are
not universal framework source-package completion gates. Add topology-specific
verification only for an actual selected deployment need. This task establishes
no public-service support commitment, production certification or 1.0 stability.

## Acceptance, decisions and next action

The recommendation is **current source preview with explicitly bounded
capabilities**, while keeping the existing release candidate and experimental
status. Next unblockable action is an owner decision on the intended delivery:
continue source-checkout adoption, or select a separate later revision and exact
package/target scope for release preparation. If later source is chosen, close
its concrete evidence differences before carrying support/security claims. No
additional Lab feature, benchmark campaign or universal operations matrix is
required by this review. No replacement candidate or YT-200 is selected here.

Four changed documents are planned: this report, Backlog, Roadmap and a current
planning pointer in Release Readiness. Verification consists of source/manifest/
Git inspection, `cargo xtask check docs`, tracked/new-file whitespace, direct
new-document links/fences/EOF checks and independent read-only review. No product
test/build/rustdoc/browser/CSP/benchmark/provider/cloud/security scan is run.
The required-doc count printed by the existing checker is not whole-corpus
link/structure coverage; this new report receives separate direct checks.

Planning evidence is retained outside the checkout at
`/tmp/yetlet-yt199-delivery-20261026-npsjwmgi/`. The source inventory and preflight
record both Git identities, clean starting state and the sixteen manifest/delta
observations. Documentation and independent review results are recorded below
at closeout; only completion metadata may change after Reviewer PASS.

### Documentation verification and independent review

`cargo xtask check docs` passed: 75 fixed required documents, 199 backlog items,
English corpus and the existing link/structure/whitespace/DAG checks. Direct
new-report local links, fences, EOF and whitespace passed separately. Tracked
`git diff --check` passed; the new-report no-index whitespace check produced no
diagnostics, with exit 1 for the expected content difference against /dev/null.
No product qualification command or external resource was used.

Independent Reviewer `/root/yt199_delivery_scope_independent_review_20261026`
returned PASS with zero issues/required followups. It inspected the four actual
documents, relevant manifests/Git deltas, candidate/security/disclosure contracts
and bounded app/tool references without changes or test replay. All four source
and four evidence hashes, HEAD/status and staged-zero state matched before and
after review. Native trace verified the registered reviewer and saved model
binding, with effective effort matching the shared resolver; no conflict or
degradation. Acceptance is documentation planning only. Subsequent edits are
limited to this closeout and the three navigation/status summaries. Original
candidate, D1–D6, YT-020, source/runtime and historical evidence stay unchanged.
