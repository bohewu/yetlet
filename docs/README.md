# Developer Documentation

Start with the [native SSR quickstart](../README.md#start-with-native-ssr), then
choose a [runnable sample](SAMPLES.md). The two-package
[SDK `0.9.0-alpha.8`](ALPHA8-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-12)
and independent [CLI `0.4.0`](CLI-040-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-12)
are published on crates.io with matching Experimental GitHub pre-releases.
The first-app instructions and selected reader sources use the edition pinned
in [the site manifest](../site/pages.toml). The live reader labels its deployed edition and
source; local guide changes do not deploy it.
Earlier release layouts and registry outcomes remain historical. Current task
status belongs to [Backlog](BACKLOG.md), milestone gates to [Roadmap](ROADMAP.md),
and distribution versus qualification to [Release readiness](RELEASE-READINESS.md).

## Reader entry points

- [Current publication and API documentation boundaries](RELEASING.md#release-documentation-gate):
  package READMEs and versioned module guides must match the independently
  published SDK/CLI, not a later source-only checkout; the original YT-244
  package/link audit and repaired CLI heading failures remain in local Git
  `ad564ad1ef07e5fca063ae88587c0f21e6edb5b1`, distinct from later hosted release qualification.
- [Optional application-owned Rust CSS](CLI-GUIDE.md#optional-application-owned-rust-css-yt-239-working-tree)
  covers Native Style and Browser Style templates and build-time stylesheet
  generation. CLI `0.3.0` introduced Style authoring. SDK `0.9.0-alpha.7`
  introduced typed Tag Builders. See [CLI and SDK availability](API-REFERENCE.md#cli-style-authoring-and-checkout-showcase),
  the [asset contract](ASSETS.md) and the native-first
  [Style Showcase checkout sample](../examples/style-showcase/README.md).

- [Troubleshooting common problems](TROUBLESHOOTING.md): symptom → possible cause → correction → existing guide.
- [Build your first Yetlet app](FIRST-APP.md): CLI0.4.0 → protected standalone project → SDK0.9.0-alpha.8 → app lock/cache → first build/run with `App::serve()`. The optional
  [static-export path](FIRST-APP.md#next-export-a-static-site) uses a separate example.
- [Build a browser-only Yetlet app](BROWSER-APP.md): published CLI/SDK → explicit tool and lock preparation → local state/events → build/preview/watch → static ship directory.
- [Write Views and reusable components](VIEW-COMPONENTS.md): typed data → functions → lists/conditions → existing SSR handler.
- [Opt-in integration quick reference](OPT-IN-QUICK-REFERENCE.md): package/feature → startup API/options → consumer settings/defaults.
- [Settings, options and logging](SETTINGS-OPTIONS-LOGGING.md): startup loading → validation → typed options → native state and application-owned diagnostics, using existing examples.
- [Integration guide template](GUIDE-TEMPLATE.md): prerequisites → minimal integration → expected result → configuration/errors → API reference.
- [Current API reference](API-REFERENCE.md): package entry points, feature/target
  selection, application options, a Rust API reading guide and local rustdoc; read guides and signatures
  at the same revision.
- [Documentation sources and publication scope](DOCUMENTATION-SCOPE.md): selected
  reader sources, separate maintainer/evidence material and no automatic export
  of `docs/**` or linked records.
- [Native SSR quickstart](../README.md#start-with-native-ssr),
  [existing sample catalog](SAMPLES.md) and [Board startup](BOARD-START.md).
- [HTTP pipeline](HTTP-PIPELINE.md), [opt-in composition](EXTENSIONS.md),
  [configuration](CONFIGURATION.md), [naming](NAMING.md) and
  [current capabilities](CAPABILITIES.md).
- [Current CLI guide](CLI-GUIDE.md#current-command-and-capability-inventory).

- [WebAssembly startup and loading UI](WASM-STARTUP.md): optional shared JS/CSS shell, customization, full-page/SSR-region integration and caller-owned lifetime.

- [Public developer guide](https://bohewu.github.io/yetlet/): eight selected chapters,
  navigation and search, labeled with SDK/CLI edition and source commit.
- [Public documentation plan](PUBLIC-DOCS-PLAN.md): source/API boundaries,
  doc-comment gaps and deferred Wiki.

## Maintainer entry points

- [YT-250 CLI 0.3.1 release](CLI-031-RELEASE-PREPARATION.md): published
  Registry latest SemVer correction, retained exact app locking/provenance and
  Windows Browser Style shutdown verification. The Reader Edition adopted
  CLI 0.3.1 at that release.

- [M26 Application Startup & Sample Experience](ROADMAP.md#m26--application-startup--sample-experience)
  delivered the [SDK alpha.8 startup API and prelude](API-REFERENCE.md#published-application-startup--sdk-alpha8) and
  [CLI 0.4.0 SDK-aware scaffolding](CLI-GUIDE.md#generated-startup-profiles).
  The accepted API decisions are in [ADR 0011](adr/0011-application-startup-api.md)
  and [ADR 0012](adr/0012-server-axum-prelude.md).
- [YT-247 SDK resolution contract](CLI-GUIDE.md#sdk-resolution-yt-247),
  [ADR 0010](adr/0010-dynamic-sdk-version-resolution.md) and
  [Phase A handoff](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-247-PHASE-A-HANDOFF.md): Owner-accepted version/cache/locking
  boundary. The [Phase B record](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence)
  reports the source implementation, tests and platform limitations.
- [YT-247 Windows supplement](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence): actual MSVC
  Resolver/new/lock/offline verification, scoped test correction and retained
  evidence with Owner-authorized temporary build/cache cleanup.
- [YT-248 CLI 0.3.0 Style release](CLI-030-RELEASE-PREPARATION.md): preserved
  reviewed release plan and [actual publication](CLI-030-HOSTED-RECORD.md), including
  the original failed publisher and successful standalone registry consumer.
- [CLI 0.4.0 publication](CLI-040-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-12)
  records the YT-259 release archive, six hosted generation cases and installed
  first-app verification. The [candidate preparation](CLI-040-RELEASE-PREPARATION.md)
  retains the prepublication qualification scope.


- [Published SDK alpha.7 source and delivery evidence](ALPHA7-HOSTED-RECORD.md):
  the original two-package YT-245 local SDK candidate and MSVC proof preceded
  separately owner-authorized signing/publication. The independent
  [CLI 0.3.0 release](CLI-030-HOSTED-RECORD.md) later shipped Style templates;
  neither candidate preparation alone constituted publication. Original
  source-only checkpoints remain in local Git `ad564ad1ef07e5fca063ae88587c0f21e6edb5b1`.
- [Documentation scope and dated YT-242 audit](DOCUMENTATION-SCOPE.md#yt-242-inventory-and-semantic-audit-boundary--2026-11-08):
  the original 2026-11-08 516-file inventory inspected 54 entry surfaces,
  corrected current/published claims and passed a corrective independent
  Reviewer after an earlier P2 finding. Its historical counts and SDK
  alpha.6/CLI 0.2.0 edition are **not** present counts or current releases.
  YT-243 separately completed source/Linux and local Windows-MSVC
  qualification at its own scope; no complete corpus/production certification.
- [Browser application DX research](BROWSER-DX-RESEARCH.md): selected
  documentation gaps, reuse of the browser starter and event ownership, and
  source-bound Yetlet evidence without new runtime or API changes.

- [Development caches and verification evidence — YT-233](DEV-ARTIFACTS.md):
  separate new cache/evidence roots, inventory logical sizes and preview cleanup;
  preserve failures and unknown legacy output, with no deletion operation.

- [Next public guide edition — YT-232](DOCUMENTATION-SCOPE.md#historical-reader-source-and-rollover-evidence): selected
  SDKalpha.6/CLI0.2.0 sources, registry consumers, static introduction and bounded
  local verification; live Pages promotion remains separate.

- [HTML DOM coverage and optional URL/image policies](../crates/yetlet-browser/guides/dom.md):
  first published in SDK `0.9.0-alpha.6`, with 89 default elements and an
  explicitly opted-in image element. The [dated standards inventory](API-CONTRACTS.md#historical-html-element-and-attribute-inventory)
  remains the Typed Tag Builder selection reference. Current URL validation,
  resource/security limits and browser ownership belong to the DOM guide.
- [Static HTML export](CLI-GUIDE.md#static-html-build): first published in the
  independent CLI `0.2.0`; includes bounded public pages/assets, project
  prefixes, create-only output and private receipts. The [retained protocol](CLI-CONTRACT.md#historical-static-export-protocol-and-guardrails)
  is the technical contract; the [standalone example](../examples/static-site/README.md)
  provides the current consumer workflow.
- [M25 delivered outcomes](ROADMAP.md#m25--broader-html-dom-support-and-static-site-delivery):
  first publication, verification boundaries and Reader Edition closeout.
- [Documentation consolidation plan](DOCUMENTATION-CONSOLIDATION-PLAN.md):
  functional-document ownership, actual YT file deletion, evidence recovery,
  review gates and the next topic-based batches.

- [Yetlet Reader guide](../examples/docs-site/README.md): application-owned
  eight-chapter selection, escaped Markdown, optional search, protected local
  and public artifact preparation with separate non-deployment receipts.
- [GitHub Pages publication and Reader cutover](GITHUB-PAGES-PUBLICATION.md#authorized-yetlet-publication--2026-11-01):
  owner-approved source `0ae6b93`, the accepted 15-file public artifact, HTTPS
  byte/MIME verification, master-only deployment and original limited browser
  observations. The original YT-224–226 pilot/hosted/reader failures remain
  recoverable from fixed Git history; public Pages qualification is distinct
  from local preview and build-only CI.

- [Historical mdBook reader preview](../site/README.md): pinned reference builder,
  exact chapter selection and loopback-only usage; use the
  [Yetlet reader](../examples/docs-site/README.md) for the current build path.
  [YT-217 completion](https://github.com/bohewu/yetlet/blob/4792565e6cff7a94c8c2c860d1858f840f122cbe/docs/YT-217-DOCS-PREVIEW.md)
  records API comments and independently accepted local reader checks with retained
  failures; the separate YT-218 record covers actual public hosting.

- [Completed CLI0.1.1 release](COMPATIBILITY.md#historical-first-cli-releases-and-consumer-failures):
  accepted output, owner-approved publication and installed consumer; the
  [CLI0.1.0 failure](COMPATIBILITY.md#historical-first-cli-releases-and-consumer-failures)
  remains preserved.

- [Alpha.5 two-package release plan](ALPHA5-RELEASE-PLAN.md): committed migration
  input, intended package/platform scope, current gaps and later execution gates.

- [Alpha.4 hosted checkpoint](ALPHA4-HOSTED-RECORD.md): fixed C/tag, original
  successful build/readback, preserved failures/custody, distinct signer delivery
  and final publication boundary.


- [Alpha.4 local version preparation](ALPHA4-VERSION-PREFLIGHT.md): sixteen
  coordinated package versions, unchanged third-party dependencies and bounded
  affected verification; no candidate or execution authority.

- [Development and PR workflow](DEVELOPMENT.md): bounded changes, focused checks,
  Git authority, strict/builtin Guard, CI and merge reconciliation.
- [New-version release workflow](RELEASING.md): one-time setup versus each version,
  immutable C/S, protected jobs, exact approvals, independent verification and
  publication; distinguishes installed tooling from Pending hosted qualification.
- [Release SOP and version authority](RELEASING.md#version-authorities-and-documentation-synchronization):
  the 2026-11-01 YT-227 plan proposed SDK alpha.6 then independent CLI 0.2.0,
  with a separate **unadopted** registry OIDC option. Those versions
  subsequently shipped under independent accepted release/consumer evidence;
  the original proposal does not authorize current publishing.
- [Completed SDKalpha.6 release](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01):
  qualification, source-bound release stages, owner-approved GitHub/crates.io
  publication and registry consumer; registry OIDC deferred.
- [Completed CLI0.2.0 release](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02):
  SDKalpha.6 pin/resources, preserved historical bindings and actual registry
  install/version/help/new; no SDK republishing.
- [Single-gate release automation and registry promotion](RELEASE-PUBLICATION.md):
  automated build/sign/verification, final owner-approved GitHub then crates.io
  publication, local implementation and one-time rollout limits.
- [Historical sixteen-name crates.io inventory](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-020-REGISTRY-READINESS.md):
  2026-10-30 not-found observations, public account without proven publishing
  rights, and original dependency/bootstrap limits. These are not current
  registry availability or permission; use the [published SDK and CLI records](RELEASE-READINESS.md)
  for actual distribution and [release SOP](RELEASING.md) for future actions.
- [Config/Secrets first-publication contract](RELEASE-PUBLICATION.md#dated-local-evidence-and-remaining-provenance):
  unchanged manifests, local normal publish dry-runs and their artifact/checksum
  limits; registry activation remains separate.
- [Formal registry publication and artifact binding](RELEASE-PUBLICATION.md#first-registry-publication-and-immutable-artifact-contract):
  selected dependency correction/order, sixteen frozen artifacts, exact-byte API
  promotion and bounded isolated prototype results; actual upload remains gated.
- [Alpha.3 hosted record and prepare scope](ALPHA3-HOSTED-RECORD.md): actual C,
  build/readback/retention results and the owner-approved batch ending at prepare.
- [Alpha.3 version/package preflight](ALPHA3-VERSION-PREFLIGHT.md): retained
  coordinated migration/custody correction, accepted checks and harness limits.
- [Alpha.2 retained build/custody failure](ALPHA2-HOSTED-RECORD.md): immutable
  unpromoted tag, original outputs and correction rationale; its
  [earlier local package preflight](ALPHA2-VERSION-PREFLIGHT.md) keeps its scope.

These are repository-maintenance guides, not application prerequisites or
default reader-site exports. Use them for procedures; use dated records below
for evidence rather than replaying the first release's troubleshooting history.

This repository index includes both reader guidance and maintainer history;
it is not a documentation-site export. Current APIs belong to the reference
and package guides above. Dated qualification below retains its original
source, status and scope; old pending/uncommitted wording does not describe the
current checkout. Current task status remains in [Backlog](BACKLOG.md).

<details>
<summary>Existing repository navigation, delivery records and historical evidence</summary>


## Historical release-candidate qualification

[YT-096 candidate selection and package preflight](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) records the owner-approved `ce56e08` source, original nine experimental `0.1.0` libraries, Rust 1.99 and selected support targets. Nine source packages and a clean packaged-input native consumer pass on Ubuntu 26.04.1. The [selected dependency/API delta](RELEASE-SUPPLY-CHAIN.md#historical-candidate-inventory-and-technical-consolidation) records 23 SBOMs and license/source checks, nine passing RustSec package-lock scans, API/threat reconciliation and an unsigned checksum bundle with explicit coverage limits. The selected Ubuntu 24.04 baseline and remaining platform/build-bound inventory/rebuild/formal review/trust gates are explicit. YT-096 is In progress and YT-020 remains Blocked. Historical accepted evidence is preserved; no publication or production support is activated.

The [partial platform followup](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) adds Ubuntu
24.04.5 container userspace tests/doctests, five packaged wasm builds and a
source-reconciled Firefox workflow. It retains the shared-kernel/Chrome evidence
limits and the historical Windows cache failures. The resumed standalone
archive lane passes all nine packages on Windows MSVC: 83 native tests and
four doctests. The [two-host archive rebuild](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-096-TWO-HOST-REBUILD.md) now
matches all nine Windows archives byte-for-byte to the Linux originals.
Independent builder control and later release gates remain open. The earlier
[SSH failures](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) are preserved. The
[Azure Run Command continuation](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) now passes on an
actual Ubuntu 24.04.5 guest with an empty cache: nine identical archives,
83 native tests/four doctests and five wasm libraries. All temporary resources
are deleted; aggregate Cargo observations retain explicit SBOM/dev-only limits.

The [GitHub-hosted candidate rebuild](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) reuses the
existing manual workflow at the exact same candidate. All nine returned archives
match complete primary bytes with matched source/lock/tool inputs and verified
unsigned bindings. This adds separately operated execution evidence; formal
trust and release approval remain open. The owner subsequently accepted only
the bounded inventory/platform disposition described below.

The [final inventory/platform evidence reconciliation](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence)
joins nine archives to 23 target SBOMs and retained Cargo observations, with
133 browser-source/eight asset identities checked and no runtime reruns.
The [owner's support/inventory decision](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence)
accepts the experimental nine-source-package scope and known differences.
Exact-candidate Chrome qualification and complete compiled/system inventory
remain Not verified; no formal release gate is waived.

The [final candidate security/release review](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints)
is complete. Bounded technical/security evidence is accepted with limits and no
new material source defect was identified; the overall release gate is FAIL,
release-ready NO. Subsequent owner support/inventory acceptance resolves one
disposition; formal integrity/trust, actual registry proof, public activation
controls and concrete maintainer release approval remain open.
No runtime rerun, signing or publication occurred.

The historical [minimum formal trust/provenance proposal](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-FORMAL-TRUST-PROPOSAL.md#historical-nine-package-proposal--2026-10-20)
and [2026-10-23 decision brief](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-TRUST-DISCLOSURE-DECISIONS.md#historical-decision-brief--2026-10-23) retain their
original nine-package scope, pending decisions and independent review PASS.
Those reviews apply to the original documentation, not the new implementation.

The [D1–D5 release supply-chain policy](RELEASE-SUPPLY-CHAIN.md#adopted-release-trust-and-disclosure-policy--yt-096) is
**Adopted (policy) as of 2026-10-27; D6 remains Pending**. It covers GitHub workload
OIDC/public Sigstore disclosure, closed sixteen-subject public provenance/DSSE
and complete private evidence, sixteen experimental `0.1.0` packages with
`118f3d0` as technical baseline and future co-located `C_release`/release workflow
with proposed `v0.1.0`, Cosign3.1.4/reference roots subject to actual bootstrap,
validity and compatibility checks, and source-package-only claims with acyclic
bindings. No actual release SHA, tag, final output/payload, authenticated Cosign
installation or new build result is established by adoption.

The owner authorizes build/verify-only [Release preparation workflow implementation](RELEASING.md#existing-workflow-map-and-limits),
with new changes uncommitted/unpushed. Hosted build/verify execution needs separate
explicit authority and may be authorized while D6 remains Pending. D6 holds
identity issuance/signing; publication needs separate explicit approval.
YT-096 remains In progress, YT-020 Blocked and YT-206 Done.

**Form/save feedback (YT-105/107–109):** The accepted Admin/Todo slices
keep field validation, explicit save/recovery state, editor feedback and
narrow native/DOM `hidden` consistency application-owned. Follow the
[current View/form guidance](VIEW-COMPONENTS.md#configuration-and-errors),
[HTTP pipeline](HTTP-PIPELINE.md), [forms sample](../examples/forms-validation/README.md)
and [retained bounded DOM contract](BROWSER-APP.md#historical-native-dom-controls-and-navigation-boundaries). The original
reviews and browser/native/negative test results remain in immutable local
Git source `b610acc7e098020b6b59dfa0d1ab2b75b059e656`; this is not a public production/browser promise.

## Architecture and contracts

- [Independent signer contract](RELEASE-SUPPLY-CHAIN.md#two-payloads-complete-assembly-and-independent-verification) and [ADR 0009](adr/0009-release-build-and-signer-identities.md) define separate fixed-SHA build/signer identities and two-payload freeze. [Alpha.1 D6 acceptance](RELEASE-READINESS.md#alpha1-frozen-inputs-signatures-and-acceptance--2026-10-29) retains the actual historical signing result. Current version execution follows the [publication policy](RELEASE-PUBLICATION.md).

The fourteen original YT-020/YT-093/YT-096 signing, custody and registry Markdown
sources are recoverable from local Git `66de5b5794ee9587dfbfe5e5c629c572e1af7b8a`.
Their immutable GitHub links preserve dated proof headings. Remote reachability
of that unpublished history is unverified. Current normative sections are in the
release, secret-provider and qualification guides. Original raw signatures,
receipts, encrypted custody and ignored/external evidence remain separately
retained at their recorded stores. Git recovery of Markdown does not establish
continued availability of those raw bytes.

- [Architecture](ARCHITECTURE.md) and [MVP contract](MVP.md): responsibilities, trust boundaries and original acceptance criteria.
- [DOM capability and authoring review](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-109-DOM-CAPABILITY-REVIEW.md): current renderer/consumer inventory, property and ownership boundaries, and staged proposals; research only, not runtime implementation.
- [Bounded DOM capability implementation](BROWSER-APP.md#historical-native-dom-controls-and-navigation-boundaries): finite View vocabulary, duplicate/void consistency, native control defaults/reset, same-origin navigation and actual verification; no general binding runtime.
- [HTTP pipeline](HTTP-PIPELINE.md): native Axum/Tower handlers, middleware, request state and response semantics.
- [Extensions](EXTENSIONS.md): explicit opt-in setup and provider boundaries.
- [Naming guide](NAMING.md): current Rust/CLI/settings vocabulary, one canonical pre-public Rust API and explicit source migrations. [Audit response](NAMING-AUDIT-20261022.md) records N01–N08 and focused evidence; [the earlier compatibility checkpoint](NAMING-COMPATIBILITY.md) retains its original accepted results and superseded alias decision.
- [Current capability and API carrier map](CAPABILITIES.md): implemented packages/features, application integrations, research decisions and release inclusion.
- [M9 API classification and historical supplements](API-CONTRACTS.md) and [Compatibility](COMPATIBILITY.md): the original classification and pre-public stability limits; later packages have separate contracts in the current map.
- [CLI contract](CLI-CONTRACT.md) and [current CLI guide](CLI-GUIDE.md#current-command-and-capability-inventory):
  published CLI `0.4.0`, settings-first grammar, finite SDK resolution/startup
  policy and explicit `inspect`, `run/watch` and generator boundaries.
- [Browser-only app walkthrough](BROWSER-APP.md) and
  [Browser DX evidence boundaries](BROWSER-DX-RESEARCH.md): project-owned
  Rust/Wasm build/reload, Vite frontend changes and static preview. Earlier
  YT-104 and YT-111/112 source-only implementations and their failed tests
  remain recoverable from local Git `ed7a082b1f9f72430f06866f7d550b0ed9ecbae9` (not verified publicly reachable);
  Windows browser-watch is not generalized by accepted Linux tests.
- [YT-115 optional Memory Session/Auth bridge](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection): explicit feature, current account mapping, native-form CSRF and real loopback HTTP lifecycle evidence; independent Reviewer PASS.
- [YT-117 optional rate limiting](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection): selected endpoint/group policies, separate configuration reuse and shared per-process budgets, and default-off startup installation; Done at per-process scope after independent Reviewer PASS.
- **Research-only Audit, Cache, Output Cache and Tracing (YT-118/121/122/123):**
  the independently reviewed studies retained application-owned business
  history, disposable data reads, strict no-store for private HTTP output and
  native Request ID/correlation boundaries. **No generic audit, cache, output
  cache or tracing framework module was selected.** See the
  [current capability matrix](CAPABILITIES.md#research-decisions-and-unselected-candidates)
  and [extension boundaries](EXTENSIONS.md); original reviews and negative cases
  remain in fixed local Git `4ad0978f8a70b36c307f5e538aa1a3aa496a7403`.
- [YT-134 optional release/storage capacity reference](QUALIFICATION-MATRIX.md#historical-operations-capacity-and-recovery-cohorts): predeclared direct-loopback measurements for the exact accepted release binary and disposable schema-2/ext4 fixture, retaining passing and failed load points, write reconciliation and cleanup; Done at bounded optional reference scope after independent Reviewer PASS, zero issues/required followups. Consumer hardware/SLO and production qualification remain separate. A separately authorized [ephemeral Azure supplement](PRODUCTION-DEPLOYMENT-PROFILES.md#bounded-private-and-azure-operations) completed its workload, reconciliation, ordinary reboot/readback and full cleanup, with separate independent Reviewer PASS, zero issues/required followups; failed cloud points and measurement limits remain explicit.
- [YT-135 current-schema recovery reference](PRODUCTION-READINESS-PHASE2.md#current-schema-recovery-and-starter-ownership): separately activated release/schema-2 backup, fresh restore, explicit two-acknowledgment loss accounting, missing-index refusal and forward-restart readback; isolated proof/cleanup complete with independent Reviewer PASS, zero issues/required followups; Done at bounded optional reference scope.
- **Optional telemetry and write-consistency studies (YT-136–138):**
  [Starter telemetry collection commands](../examples/production-starter/deployment/telemetry/README.md)
  demonstrate private, bounded metrics/log snapshots and fail-closed
  last-good handling; no production collector/alert/SLO was activated.
  Source-bound write-outcome research preserves **unknown late commits**
  and no automatic mutation retries. Authorization-to-commit research
  retained the accepted Admin pre-SQL revocation race; a future fence
  requires a separately authorized business write contract.
  See [deployment guidance](PRODUCTION-DEPLOYMENT-PROFILES.md#operational-decisions-versus-research-only-candidates)
  and fixed local original Git `be337ff02826978a9baa1b88ddf85696cb4406b4`.
- [YT-139 multi-instance authority/recovery reference](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority): selected local two-Admin business/Session/current-policy authority and recovery contract; independent contract Reviewer PASS, zero issues/required followups; bounded new stale-Session restoration/re-login proof, complete business/policy reconciliation and owned cleanup complete; final independent Reviewer PASS, zero issues/required followups; Done at bounded optional local reference scope. No mandatory database, HA or stronger write-ordering guarantee.
- [YT-140 OIDC proxy/replica integration](PROVIDER-OPERATIONS.md#oidc-cross-instance-origin-and-callback): contract-first minimum integration for maintained-client callback, explicit trusted origin/proxy and identity freshness boundaries; reuse accepted IdP and local authority evidence. Done at bounded optional local integration scope; initial independent contract Reviewer PASS, zero issues/required followups; minimum origin integration, three new exact tests, static checks and owned cleanup complete; final independent Reviewer PASS, zero issues/required followups. New proof uses two Board listeners/managers in one process, without successful token exchange or cross-process Admin OIDC qualification. No new OIDC protocol or mandatory identity product.
- [YT-141 reproducible performance-validation reference](QUALIFICATION-MATRIX.md#historical-operations-capacity-and-recovery-cohorts): independent contract PASS, minimal k6 reference and new controlled-arrival read-only API measurement/cleanup complete; 14,006 measured valid-200 responses, zero drops/errors, p50/p95 1/2 ms and p99 2–4 ms. Twenty rows/schema/index/sequence unchanged; Done at bounded optional local reference scope after final independent Reviewer PASS, zero issues/required followups; eight hashes matched. Existing YT-134 passing/failed points retain their original scope; no universal throughput or consumer SLO/CI gate.
- **Historical API/saturation comparisons (YT-142–144):** The initial
  1,000/s SQL-admission cohort failed with five 503 responses, while the
  250/s recovery passed. Native Axum/Yetlet and PostgreSQL comparison pairs
  had explicit failed/harness-corrected and noncomparable cases. See the
  [performance evidence matrix](QUALIFICATION-MATRIX.md#historical-performance-and-capacity-evidence-bounded)
  and [Axum reference sample](../examples/axum-workload-comparison/README.md).
- **SSR allocation and latency attribution (YT-145–160):** Fixed 8/128/1,024
  row pages, counted vs ordinary timing, private escaping/URL improvements,
  one rejected first-child optimization, and unresolved full-HTTP/frequency
  causes. See [SSR sample commands](../examples/ssr-performance-baselines/README.md)
  and the [qualification scope](QUALIFICATION-MATRIX.md#historical-performance-and-capacity-evidence-bounded).
- **Benchmark isolation and private capacity (YT-161–169):** Do not generalize
  SSH/CPU/VM comparisons. The bounded 6k/s sustained point **failed** with
  dropped arrivals; separate 5k/s 300s and 1,800s cohorts passed with
  controlled-source caveats. See the [Axum fixture](../examples/axum-workload-comparison/README.md)
  and [historical performance evidence](QUALIFICATION-MATRIX.md#historical-performance-and-capacity-evidence-bounded).
- **Synthetic Auth/Session/PostgreSQL (YT-170–174):** Source-bound fixture
  workloads, Busy/recovery and passive trace observations are not real OIDC
  authentication, distributed admission, or durable-write performance.
  Current run/profile selection is in the
  [Axum workload guide](../examples/axum-workload-comparison/README.md);
  original faults and measured populations remain recoverable in local Git history.
- [YT-175 Session admission burst and overload policy](PROVIDER-OPERATIONS.md#provider-overload-and-quota-distinctions): Done at document-only contract scope; independent Reviewer PASS, zero issues/required followups; preserve current limits and safe failure/reconciliation semantics. Optional early HTTP admission and rate quotas remain application choices; no implementation or old proof replay.
- [YT-176 Opt-in fail-fast HTTP admission](HTTP-PIPELINE.md#opt-in-application-http-admission-contract): Done at bounded local Auth-reference integration scope; contract PASS, six new composition tests and feature-off check PASS. Shared early private 503 with zero added queue/retry, permit held through finalization, unchanged Session/lock/default graph. Final independent Reviewer PASS, zero issues/required followups; no old proof replay or new-change Commit/Push.
- [YT-177 Isolated HTTP admission=4 workload](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts): Done at bounded policy-behavior scope; final independent Reviewer PASS, zero issues/required followups. 12452 valid/98 checked pre-Session 503 out of12550, zero provider/SQL errors/drops; more rejections than historical baseline, no capacity improvement. Recovery and exact-owned cleanup PASS; source/Session/zero-queue/no-new-retry unchanged, old evidence preserved.
- [YT-178 Isolated HTTP admission=8 comparison](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts): Done at bounded comparison scope; final independent Reviewer PASS, zero issues/required followups.12475valid/74checked failures out of12549, versus cap4 12452/98 out of12550; no provider pressure, but longer burst/client tails. Same binary/caller, unchanged Session/queue/retry; recoveries and exact-owned cleanup PASS, all failures preserved.
- [YT-179 Isolated HTTP admission=16 comparison](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts): Done after final independent Reviewer PASS, zero issues/followups; 12552 selected/12540 valid/12 checked503 (7 outer, 5 provider Busy). Session4/8 and zero added queue/retry unchanged; retained cap4/8 comparison and all failures stay explicit.
- [YT-180 Example HTTP admission capacity selection guide](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection): Done at documentation-only scope after independent Reviewer PASS, zero issues/followups; application-owned opt-in 4/8/16 choices, retained failure/latency tradeoffs and per-process/Session/rate-quota boundaries. No universal default, new runtime or replayed tests.
- **YT-181–184 actual app follow-ups:** the original scheduling/feasibility
  proposal is historical, **not** a cloud execution record. The separate
  [write contract](PROVIDER-OPERATIONS.md#commit-time-authority-and-revocation),
  [worker trace contract](PROVIDER-OPERATIONS.md#request-tracing-and-owned-worker-spans),
  [Shop cache contract](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183) and
  [actual Azure Auth/Session restart](PRODUCTION-DEPLOYMENT-PROFILES.md#historical-shared-authority-and-azure-persistence-qualification)
  retain their individually authorized/verified boundaries. See
  [deployment guidance](PRODUCTION-DEPLOYMENT-PROFILES.md#operational-decisions-versus-research-only-candidates).
- [YT-181 Minimal sensitive-write/revocation contract](PROVIDER-OPERATIONS.md#commit-time-authority-and-revocation): accepted contract; selected SQL-owned account/project write authority and generation-checked item/history commit. Separate contract/implementation acceptance, native middleware without generic AOP, existing Session/file-policy race preserved; independent contract Reviewer PASS, zero issues/followups; later implementation/proof tracked separately below.
- [YT-181 Opt-in commit-time authorization implementation](PROVIDER-OPERATIONS.md#commit-time-authority-and-revocation): Done after independent implementation Reviewer PASS, zero issues/followups; application-only feature/Settings, generation-fenced item/history and conservative v2 reopen, six new Board cases/eight total new test executions, no prior test/benchmark replay or generic AOP.
- [YT-182 Opt-in tracing and safe worker contract](PROVIDER-OPERATIONS.md#request-tracing-and-owned-worker-spans): Contract Reviewer PASS, zero issues/followups; Board/Admin native middleware, generated local IDs, safe worker/Dispatch propagation and separate lifecycle facts. Default-off feature/Settings, no generic AOP/exporter or old proof replay.
- [YT-126 opt-in Admin Valkey/Auth adoption](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence): real application adoption with fresh shared local policy and actual two-Admin-process qualification; Done after independent Reviewer PASS, retaining existing pre-SQL ordering limits.
- [YT-125 minimal Valkey/Auth integration](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence): standalone application resolver and actual two-process qualification, Done after independent Reviewer PASS; no generic Auth feature or Admin migration.
- [YT-124 multi-instance Auth/Valkey Session contract](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority): owner-selected research follow-up, Done after independent Reviewer PASS; later Valkey two-app evidence is reused alongside historical Redis and rolling-asset proof. No async Auth bridge implementation is selected.
- **Auth provider research (YT-116/119/120):** independently reviewed
  app-owned OIDC issuer/subject mapping, opaque Bearer guards and mature
  Cookie/Session composition; no automatic OIDC/JWT/Cookie adapters.
  See [current Auth and capability boundaries](CAPABILITIES.md#application-owned-integrations-and-tools)
  and retained [OIDC reference evidence](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence).
- [YT-182 Opt-in tracing/worker implementation](PROVIDER-OPERATIONS.md#request-tracing-and-owned-worker-spans): Done after independent implementation Reviewer PASS, zero issues/followups; default-off Board/Admin feature and Settings, scoped HTTP/worker Dispatch, separate lifecycle outcomes, seven successful new test executions, original compile/drop failures retained and old proof not replayed.

- [M21 opt-in server capability plan](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection):
  all eleven registered tasks are now Done at distinct contract, native
  Auth/Memory/rate-limit or research-only scopes. Actual runtime contracts are
  [YT-113 Auth](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority),
  [YT-115 Memory bridge](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) and
  [YT-117 rate limiter](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection), with retained
  [YT-114 foundation evidence](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection). Research-only
  OIDC/Bearer/Cookie/Audit/Cache/Tracing decisions do **not** install new
  default providers or expand production support.
- [Windows CLI exit-125 follow-up](WINDOWS-CLI-125-VERIFICATION.md): Cargo executable ACL diagnosis, safe stage diagnostics and source-matched native build/run/Ctrl-C cleanup with protected task-local tools.
- [Development HTTPS](DEV-HTTPS-VERIFICATION.md): existing settings, externally prepared mkcert certificates, optional Rust TLS, Vite HTTPS/WSS and native run/stop/build evidence.
- [Opt-in secret providers](SECRET-PROVIDERS.md): asynchronous named string lookup, explicit settings snapshots and constructor/State injection with replaceable providers.
- [Settings from SecretProvider](SETTINGS-SECRET-PROVIDER.md): opt-in startup resolution into selected protected fields, narrow module settings and fail-closed errors.
- [Shared SQLite reference](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority): two Compose app replicas accessing one local business-data directory, application conflicts and restart/error behavior.
- [CLI diagnosis and developer-loop scope](CLI-GUIDE.md#historical-cli-and-developer-loop-qualification):
  doctor is environment-scoped, inspect is project-scoped, frontend dotenv
  and generated assets are confined, and Rust Watch/restart is distinct from
  Vite's app-owned HMR and source-matched Windows browser checks.
- [Retained Linux completion safety](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence) and
  [native Windows Vite pair verification](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence):
  process-tree, console and bounded natural-exit evidence with no universal
  platform or production performance claim.
- [Rolling static assets](ASSETS.md#cache-and-retention) and
  [local YT-103 verification](ASSETS.md#historical-single-host-rolling-asset-qualification):
  application/deployment-owned source-hashed graphs, two-version coexistence,
  rollback and cleanup, not a framework HA guarantee.
- ADRs [0001](adr/0001-separation-of-concerns.md), [0002](adr/0002-events-and-actions.md), [0003](adr/0003-renderers-and-loading.md), [0004](adr/0004-request-context-and-middleware.md), [0005](adr/0005-opt-in-capability-modules.md), [0006](adr/0006-handler-first-mvc-optional.md) and [0007](adr/0007-bounded-keyed-dom-and-local-graph.md): retained decisions and tradeoffs.

## M12–M15 — Rust tooling, CLI and generated deployment artifacts

The [current CLI guide](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries)
and [Rust development guide](DEVELOPMENT.md) own executable commands.
The [M12 migration ledger](DEVELOPMENT.md#rust-tooling-migration-contracts-and-historical-inventory) records the original
Python-to-xtask migration and its bounded parity, with unrun runtime targets
still unverified. M13 initially accepted only unpublished Linux CLI
`new`/`doctor`/`build`/`dev`; a separate Windows GNU/MSVC run verified
native compilation/fail-closed behavior, followed by independent M14
positive Windows new/build/dev parity. [CLI platform and original qualification limits](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries)
and [CLI contract](CLI-CONTRACT.md) distinguish those stages from the
subsequent published CLI, whose exact version is in the release records.

M15 selected opt-in Dockerfile/Compose/Kubernetes **config generation**
rather than runtime coupling. Original Linux Docker/Compose and isolated
K3s Pod/Service/HTTP/cleanup, with Windows generated-file parity,
were bounded historical observations and not permission for persistent
cluster deployment, cloud publication or general production support.
See [current CLI generator scope](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries),
[asset guide](ASSETS.md) and [deployment profiles](PRODUCTION-DEPLOYMENT-PROFILES.md).
Complete original closed task evidence is recoverable at fixed local Git
`b610acc7e098020b6b59dfa0d1ab2b75b059e656`; GitHub reachability remains unverified until approved delivery.

## M11 — historical planning and selected product/renderer delivery

The original [M11 plan](ARCHITECTURE.md#m11-product-and-renderer-decisions), [product-readiness research](ARCHITECTURE.md#m11-product-and-renderer-decisions), [GPU plan](ARCHITECTURE.md#m11-product-and-renderer-decisions) and [planning review](ARCHITECTURE.md#m11-product-and-renderer-decisions) were documentation-only at the YT-069 checkpoint. **Subsequent owner authorizations and independent reviewers** accepted YT-070–YT-080 at separately selected bounds. [ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) now accepts only the bounded Experimental opt-in architecture; it does not grant stable or broad browser support. See the [M11 reconciliation](https://github.com/bohewu/yetlet/blob/983653adadcedb4ec196016b75281d39e2ed0acf/docs/YT-081-M11-CLOSEOUT.md) and [API consistency audit](API-CONTRACTS.md#cross-crate-api-differences-and-scoped-gpu-refinement); historical planning statements retain their original date and evidence.

### Activated non-GPU implementation

The owner committed planning as `d96f7af` and [activated YT-070–YT-074](PRODUCTION-READINESS-PHASE2.md#m11-board-acceptance-and-original-blocked-journeys); M11-P implementation is committed at `f00ec597`. [Form/asset evidence](ASSETS.md#m11-native-css-proof-and-retained-wrapper-decision) records YT-070–YT-072; YT-073 is locally complete. [YT-074 qualification](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence) is Done at the owner-amended local scope, with retained real Chewing IME, repaired Firefox/Orca speech and final Chrome/Firefox PASS. The new owner-authorized full 62-minute mixed-load execution passed with 45,000 measured requests and zero errors; fresh independent review also passed, and the earlier synchronized-burst failure is retained. Historical repair/test records retain their original source and commit boundaries. See [desktop acceptance](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence), [spoken transcript](YT-074-SPOKEN-AT-EVIDENCE.json), [IME screenshots](screenshots/yt074-ime/README.md), [startup](BOARD-START.md) and [handoff](PRODUCTION-READINESS-PHASE2.md#m11-board-acceptance-and-original-blocked-journeys). No GPU or public-release change is claimed.

### Experimental GPU and cross-crate API evidence

The [GPU support guide](../crates/yetlet-browser/guides/wgpu-support.md)
owns the current explicitly opt-in browser contract and exact Firefox
158/X11/Intel Vulkan runtime limits; [ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md)
preserves the Experimental separation from ordinary SSR/DOM. The selected
[Firefox Canvas2D/WebGPU measurement](QUALIFICATION-MATRIX.md#experimental-gpu-browser-and-physics-lab-evidence) found **no proven
useful WebGPU speedup**. GPU-absent default Firefox, controlled device loss,
picking/HTML alternatives, delayed borrows and retirement have separate
historical accepted cases; browser/physical hardware coverage is not
generalized. The retained [API contract audit](API-CONTRACTS.md#cross-crate-api-differences-and-scoped-gpu-refinement)
separates CPU typed errors, Wasm-facing errors and first logical retirement
from physical GPU completion, and distinguishes Memory/Redis providers.

The original M11 product and GPU delivery passed independently reviewed
local slices. Full source, failed attempts and reviewer evidence are
recoverable at fixed **local Git** `983653adadcedb4ec196016b75281d39e2ed0acf`;
GitHub public reachability has not been verified. Current SDK distribution
and publication are separate from those dated experimental snapshots.

## Production-oriented reference application

The separately accepted YT-104 browser task prototype reused CLI
`new --template browser`, Rust/Wasm local state and static-only build output;
its Linux Chrome/static host and source-matched Windows GNU scope were
finite, not production hosting or a native backend. The
[current browser-only walkthrough](BROWSER-APP.md) owns commands for
the published CLI and SDK; original YT-104 reviews and failures remain
recoverable at the fixed local pre-consolidation Git source.
The [Admin Dashboard](../examples/admin-dashboard/README.md) demonstrates
the YT-094 application-owned SSR, Board storage, native form/CSRF, opt-in
OIDC and separately selected Valkey/SQLite integrations. Its first local
Linux/Chrome, Windows native MSVC and later Windows Rust/WASM evidence were
**different qualification slices**, not one universal production matrix.
The accepted closeout and original staged API/triage results remain at fixed
local Git `4ad0978f8a70b36c307f5e538aa1a3aa496a7403`; current operation belongs to the app README.
The optional triage workspace uses tab-local Rust/WASM drafts, keyed reorder
and one explicit authenticated/version-checked HTTP save; queued-save
lifetime failure and corrected zero-POST regression remain historical.
Windows OS IME/AT attestations and ongoing TLS/deployment/HA limitations
are source-specific and retained in [Windows WASM verification](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence).
The historical form-value/event, state-read and HTTP-feedback studies
accepted **app-owned** native and graph draft handling, body-token CSRF,
and explicit save/validation feedback rather than new public auto-binding
APIs. The [state-read proposal and settled contract](ARCHITECTURE.md#weak-handle-published-state-snapshot-and-explicit-first-render)
and [weak-handle graph snapshot](ARCHITECTURE.md#weak-handle-published-state-snapshot-and-explicit-first-render) preserve
source-specific semantics. Use [current View/form guidance](VIEW-COMPONENTS.md#configuration-and-errors)
and [reactive lifetime guide](../crates/yetlet/guides/reactive.md).
Original YT-105 source studies and failed cases are recoverable from
local Git `b610acc7e098020b6b59dfa0d1ab2b75b059e656`; no later browser/production qualification is implied.
Packaging is covered by YT-104 above.
**Frontend component interop (YT-106):** Existing Admin-dashboard
[instructions](../examples/admin-dashboard/README.md) own the application-only
Chart.js bars and CodeMirror plain-text editor, Rust/TS/Wasm interop, strict
response-local CSP style nonce, explicit disposal and permanent native fallback.
The original research and actual Chrome/TypeScript acceptance are recoverable
at the fixed local Git source `983653adadcedb4ec196016b75281d39e2ed0acf`.
No framework-level editor, general plugin system, unsafe-inline exception or
cross-platform support follows from this bounded trial.
The [Windows WASM follow-up](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence) separates
completed native/browser checks from the returned operator-attested native
IME/keyboard/spoken-AT journeys, exit-0 images and broader open qualification.
The separately retained [Windows WASM qualification](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence)
reconciles source-matched browser output, manual Windows input/assistive
technology observations and cleanup. Earlier setup-only handoff details
remain recoverable at the fixed local original Git source.
The [opt-in OIDC slice](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence) records existing settings/SecretProvider
composition and its separate real-provider qualification boundary.
The [Valkey shared-session slice](PROVIDER-OPERATIONS.md#redis-session-cas-operation-outcome-and-cookie-ordering) records explicit
provider/settings selection and two-container login/logout, restart and outage
verification. The later [shared-SQLite slice](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority) adds
separately reviewed two-app business storage on a single local volume; neither
slice qualifies production HA.


The standalone [Issue Tracker starter](../examples/production-starter/README.md) exercises
YT-092 with native SSR forms, a command JSON API, SQLite restart persistence,
existing settings/CLI, health/readiness, correlated JSON logs and graceful shutdown.
Its [verification record](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-092-STARTER-VERIFICATION.md) separates native platform
evidence from browser/hosting qualification. This opt-in reference application is
independent of the historical twelve-member sample catalog and does not change
framework defaults or generator source contracts.

The standalone [local signing reference](../examples/signing/README.md) exercises
YT-093's opt-in [provider contract](SECRET-PROVIDERS.md#local-signing-and-replaceable-provider-contract--yt-093), multiple named
keys/certificates, public-only verification and confirmed key CLI rotation with
automatic profile settings updates. Its [verification record](QUALIFICATION-MATRIX.md#local-signing-key-cli-and-azure-evidence--2026-10-18)
separates local, native Windows and disposable Azure experiment evidence.

The [native Rust composition guidance](ARCHITECTURE.md#research-only-handler-resumption-and-native-composition) inspects
Starter, signing, Admin and existing secret/settings consumers. It recommends
constructors and native State/FromRef, with explicit generic, trait-object and
finite-enum boundaries; no new container, provider contract or implementation
is selected. Independent Reviewer PASS has zero issues or required followups;
YT-095 is Done at research-only scope.

## Library guides

The early developer workflow audit selected the existing CLI, Cargo, Watchexec
and Vite rather than another general dev-command system. Its only bounded
process-latency improvement skips a natural-exit grace after a narrow Linux
child/lifetime proof; ordinary cancellation and cleanup still apply.
See [current CLI guidance](CLI-GUIDE.md#historical-cli-and-developer-loop-qualification),
[Linux completion evidence](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence) and the
[preserved CLI contract audit](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence).
The original audit and research/verification sequence is recoverable at fixed
local Git `ed7a082b1f9f72430f06866f7d550b0ed9ecbae9`; no product runtime was requalified by document cleanup.
These module guides describe the consolidated SDK and its practical limits.
Use source/rustdoc matching the version you consume and Rust 1.99 or newer;
native SSR needs no browser build. The [API reference](API-REFERENCE.md) covers
additional modules and explicitly labels later source-only additions.

| Area | Package guide |
|---|---|
| Shared View and binding vocabulary | [yetlet::view](../crates/yetlet/guides/core.md) |
| Native escaped HTML | [yetlet::html](../crates/yetlet/guides/html.md) |
| Browser subtree mount | [yetlet_browser::dom](../crates/yetlet-browser/guides/dom.md) |
| Optional keyed children/listeners | [yetlet_browser::dom_keyed](../crates/yetlet-browser/guides/dom-keyed.md) |
| Optional owned local update graph | [yetlet::reactive](../crates/yetlet/guides/reactive.md) |
| Explicit typed actions | [yetlet::actions](../crates/yetlet/guides/actions.md) |
| Native handlers, routing and Page responses | [yetlet::server_axum](../crates/yetlet/guides/server-axum.md) |
| Optional request correlation | [yetlet::request_id](../crates/yetlet/guides/request-id.md) |
| Optional Memory Session | [yetlet::session](../crates/yetlet/guides/session.md) |

The optional [Redis provider module](../crates/yetlet/guides/session-redis.md) is distributed in the consolidated SDK. Its bounded qualification does not confer durable or production Session guarantees.

## Completed research prototypes

All five authorized studies are Done at their recorded bounded scopes. Their fixtures are outside the thirteen-application catalog and do not establish supported core APIs, public release qualification or a benchmark winner. The linked functional decisions and fixtures identify original commands, measurements and source boundaries; full dated research records remain recoverable in local Git history, not necessarily through public GitHub links.

| Study | Current decision or evidence scope | Existing fixture |
|---|---|---|
| Opt-in HTTP deadlines | [YT-029](HTTP-PIPELINE.md#application-owned-deadlines-streaming-and-admission-timing): retain explicit app-owned middleware; general provider/core promotion remains deferred | [Middleware Lab](../examples/middleware-lab/README.md) |
| Incremental SSR | [YT-021](HTTP-PIPELINE.md#application-owned-deadlines-streaming-and-admission-timing): retain the full-versus-streaming comparison; reject core streaming/suspense promotion | [Streaming SSR research](../examples/streaming-ssr-research/README.md) |
| Custom typed transport | [YT-024](EXTENSIONS.md#application-owned-action-transport): adopt one explicit app-owned TCP adapter pattern; reject a core transport framework | [Custom action transport](../examples/custom-action-transport/README.md) |
| Handler/capture descriptors | [architecture decision](ARCHITECTURE.md#research-only-handler-resumption-and-native-composition): retain finite versioned handlers/numeric captures; reject generic/core resumability | [Resumability research](../examples/resumability-research/README.md) |
| Independent scene rendering | [YT-023](ARCHITECTURE.md#experimental-scene-browser-gpu-and-physics-lab-boundaries): retain opt-in native wgpu/Vulkan draw/readback and frame/allocation observations; reject core/browser-portability promotion | [Scene research](../examples/webgpu-scene-research/README.md) |

Generic hydration/resumability, a template macro, dependency-tracked signals/forms and a stable general-purpose GPU renderer remain unselected. The separately approved Experimental opt-in candidate is documented above and does not change those stable/core boundaries. Default SSR, local browser events, typed remote boundaries and the existing ADR decisions remain intact.

## Historical qualification and Private preparation

[Qualification matrix](QUALIFICATION-MATRIX.md), [provider operations](PROVIDER-OPERATIONS.md) and [threat model](THREAT-MODEL.md) distinguish implemented behavior, retained evidence and unverified coverage. YT-030, YT-065, YT-066 and YT-067 retain their established bounded Done status; YT-020 remains Blocked.

The [Private preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12), [integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12), [hosted rebuilding](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-020-HOSTED-REBUILD.md), [trust policy](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures), [signing verification](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) and [supply-chain design](RELEASE-SUPPLY-CHAIN.md) preserve their exact evidence identities. Signed source `8e557e760407a3d4b3f5a8b8f833748946be1387` differs from policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and subsequent changes; historical signatures do not attest changed package contents. Formal release activation remains open.

## Historical documentation delivery status — YT-068

The entry refresh, nine package guides and current/history navigation are reconciled after the five completed research studies. T08 has reconciled the [final Private source/package boundary](RELEASE-SUPPLY-CHAIN.md#current-preparation-source-boundary-2026-10-12). YT-068 is **Done** at documentation scope; the independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate. YT-020 remains **Blocked**. At that pre-M11 checkpoint, all 68 backlog items were accounted for: 67 Done, one Blocked and zero In progress or Deferred, each at its stated scope. M11 adds separately filed planning/implementation proposals above; consult Backlog/Kanban for current counts. Established YT-030/YT-065/YT-066/YT-067 completion limits and optional unverified IME, speech, WebKit/mobile and zoom coverage remain unchanged.

Documentation-stage verification uses `cargo xtask check docs`, `cargo xtask check samples` and `git diff --check`, plus candidate package file listing. The code snippets in the root/server guide, core guide and request-ID guide are copied from existing source/test/rustdoc; this phase does not claim fresh compilation or execute historical qualification suites.

Candidate README additions are new package inputs: Cargo discovers the conventional `README.md` without a manifest change. Prior signed archives at source `8e557e760407a3d4b3f5a8b8f833748946be1387` do not bind these candidate bytes. Policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and the current dirty checkout remain separate boundaries. The changed root manifest/lock and research also differ from the signed source. No archive/signature is regenerated or relabeled. A separately approved immutable candidate needs current-source package preflight, rebuild/provenance, selected SBOM/license and signing verification under separate authority; a later local commit alone does not activate release.

## Historical evidence and maintainer notes

The original plans, ADRs, verification documents and sample catalog references remain at their existing paths. Their dates, source identities, reviewer IDs, repair histories and measurement limits remain authoritative for those historical claims. The following expandable notes retain the former README information and detailed commands; they are historical context, not an instruction to rerun every suite or a replacement for current status above.

<details>
<summary>Retained pre-refresh README notes and evidence navigation</summary>

### Yetlet

**Yetlet** is an experimental, HTML-first Rust universal UI runtime. A shared Rust `View`/component source supports native SSR and browser WASM/DOM without a default remote event circuit.

> **Status:** M0–M7 are implemented and locally verified at **experimental prototype** scope. M7 closes with renderer-neutral binding contracts, optional local graph/keyed-DOM ownership and one production Todo local-list adoption. A first final review found a queued/in-flight action disposal race; the lifetime repair, replacement full validation and repeat read-only reviewer now pass. No general VDOM, template macro, dependency-tracked signals/forms framework, resumability or WebGPU renderer is selected.

#### Design principles

- **HTML first:** Pure SSR requires no client JS/WASM. Server and browser targets produce separate build outputs.
- **Local events first:** DOM events remain in the browser. Remote work uses explicit typed HTTP commands instead of transmitting raw events.
- **Independent rendering and transport:** No default SignalR/LiveView-like circuit, custom HTTP/TLS stack, or CSS engine.
- **Strict CSP:** Escaped SSR output and an allowlisted browser DOM subset. The `/app` CSP allows `'wasm-unsafe-eval'` where necessary, but not `'unsafe-eval'` or `'unsafe-inline'`.
- **Modular, opt-in capabilities:** `configure`/`try_configure` use ordinary setup functions, not a plugin manager. `yetlet-request-id` proves independent scoped/global installation; Session, Cookie Auth and Cache remain future providers. See [Extensions](EXTENSIONS.md) and [ADR 0005](adr/0005-opt-in-capability-modules.md).
- **Handler-first, Component-first, MVC optional (approved direction):** Ordinary Rust handlers and shared Views, typed HTTP results with minimal scaffolding. Full Controllers, model binders, DI and a mandatory MVC file layout are not required. See [ADR 0006](adr/0006-handler-first-mvc-optional.md).
- **Bounded keyed updates, not a global renderer mandate:** Applications opt into explicit local graph and keyed ownership for proven surfaces; simple/independent subtrees may retain direct mount replacement. See [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md).

#### Implemented and planned functionality

| Mode / route | Status | Behavior |
|---|---|---|
| `GET /` | Implemented (M1) | Native SSR TodoList; zero browser runtime |
| `GET /app` | Implemented (M2/M7i) | Rust/WASM mount; keyed local Todo graph plus separate typed HTTP action |
| `GET /healthz` | Implemented | Health check |
| `GET /api/ping` | Implemented | Standalone JSON API endpoint |
| `GET /search` | Implemented (M3) | Complete SSR search page; Search WASM loads only after interaction and filters locally |
| `GET/POST /api/todos` | Implemented (M4) | Typed snapshot API and validated semantic Create/Toggle commands, with structured errors |
| Canvas/WebGPU | Deferred research | Independent scene renderer, not a DOM replacement |

Server and browser targets compile the same `TodoList` source from `examples/todo-ui`. On `/app`, the local toggle dispatches a typed command through an explicitly owned local graph and keyed DOM binding without a request; the separate server-create button uses `HttpExecutor` to POST a typed Todo command, then renders the server snapshot in its own host. `/search` continues to render HTML immediately and activate Search WASM lazily from escaped versioned state. **DOM events are never forwarded as remote events.**

#### HTTP middleware and request context

`yetlet-server-axum` now provides an owning `RequestContext<S>` with typed request-local extensions and separate application state. Middleware uses native Axum `Next`/`from_fn_with_state`, nested Router scopes and Tower layers. `with_global` adds an explicit pre-routing boundary around a completed Router. API and SSR routes share this pipeline; browser/WASM code does not depend on it.

See [HTTP pipeline usage and limits](HTTP-PIPELINE.md) and [ADR 0004](adr/0004-request-context-and-middleware.md). The existing anonymous API remains a prototype; request context is not authentication, a DI container or a session subsystem.

#### M6 — Minimal handlers, results and opt-in composition

`App<S>` remains a thin native Axum Router facade. A complete minimal SSR/JSON application needs no Controller, mandatory RequestContext, DI container or optional module:

```rust
use yetlet_core::View;
use yetlet_server_axum::{App, response::Json};

async fn home() -> View { View::text("Hello") }
async fn status() -> Json<&'static str> { Json("ready") }

let app: axum::Router = App::new()
    .page("/", "Home", home)
    .group("/api", |api| api.get("/status", status))
    .into_router();
```

For typed parameters or application errors, use `.get` with native `Path`, `Query`, `State`, `Json` extractors and return `Page::new(title, view)` or `Result<Page, E>`. `App::page` is specifically a no-argument async View-handler convenience; no duplicate model binder is introduced. `Page` retains escaped SSR, strict CSP and generic 500 rendering errors. Json, Redirect and status/header tuples retain Axum semantics.

Pages needing an explicit root language use the bounded metadata API without changing the default path:

```rust
use yetlet_core::View;
use yetlet_server_axum::{DocumentMetadata, LanguageTag, Page};

let page = Page::with_metadata(
    DocumentMetadata::with_language(
        "Hello",
        LanguageTag::new("en").expect("static language tag"),
    ),
    View::text("Welcome"),
);
```

The metadata contract is deliberately only title + language. Charset/viewport stay fixed and CSP remains server response policy; Yetlet does not add an arbitrary head manager or DSL. See [Compatibility Policy](COMPATIBILITY.md) and [M10c plan](COMPATIBILITY.md#m10-metadata-compatibility-evidence).

`configure`/`try_configure` execute ordinary module setup functions once at application construction. Providers compose native scoped middleware and typed request/application state. The Todo server explicitly selects `yetlet-request-id`; the M8c cart explicitly selects `yetlet-session` only for `/cart`. A minimal app depends on neither. See [Extensions](EXTENSIONS.md), [complete M6 evidence](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements) and [M8c evidence](M8C-VERIFICATION.md).

A runnable provider-free example is available with `cargo run -p yetlet-server-axum --example minimal`.

#### Run the examples

Requirements: Rust 1.99 or newer; `rust-toolchain.toml` pins development to Rust/Cargo 1.99.0. Building browser samples additionally needs the WASM target and wasm-bindgen 0.2.129, but the adopted Rust `cargo xtask build` commands need no Python. The Rust verification/qualification candidate is recorded in [YT-086 verification](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-086-VERIFICATION.md); Firefox verification also requires Firefox and geckodriver. Native-only applications need no browser toolchain. The maintainer selected this unified baseline before the first experimental release. Historical Rust 1.88, Chrome 155 and Firefox 157 results retain their recorded sources; [integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) and [hosted rebuilding](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-020-HOSTED-REBUILD.md) record current checks, the completed protected-check migration and the unsigned Private preparation boundary.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked \
  --root "$HOME/.local/share/yetlet-tools"

cargo xtask build browser
cargo run -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/` (SSR), `http://127.0.0.1:3000/app` (browser/WASM), and `http://127.0.0.1:3000/search` (SSR plus lazy Search Island). The Git-ignored `dist/` directory contains build-time generated assets and the content-hash manifest. The server never compiles WASM on request; `/app` explicitly returns HTTP 503 when browser assets have not been built.

User-facing runnable applications are listed in `examples/samples.toml` and [the sample guide](SAMPLES.md). Validate the catalog with `cargo xtask check samples`; internal browser entry crates and research fixtures are deliberately not presented as standalone application samples.

A minimal native-only Hello World application has no optional providers or browser toolchain:

```bash
cargo run -p yetlet-hello-world
```

It listens on `http://127.0.0.1:3003` by default, with a script-free SSR `/` and a safely escaped `/hello/{name}` route. See [M9a evidence](SAMPLES.md#hello-world-acceptance); independent reviewer `wc_job_Htk-Quo_aFExMJs3` PASS.

The native-only REST sample requires no browser toolchain:

```bash
cargo run -p yetlet-rest-api
```

It listens on `http://127.0.0.1:3001` by default and exposes `/healthz`, `/api/books` and `/api/books/{id}`.

The native SSR Session cart also requires no browser toolchain:

```bash
cargo run -p yetlet-cart-server
```

It listens on `http://127.0.0.1:3002` by default and rejects non-loopback listeners while using its explicit insecure-localhost cookie override. `/healthz` stays outside Session scope; `/cart` uses strict-CSP forms, a server-side cart and a session-bound CSRF token.

The native Middleware Lab demonstrates request/context scope and streaming without new Yetlet infrastructure:

```bash
cargo run -p yetlet-middleware-lab
```

It listens on `http://127.0.0.1:3004` by default; inspect `/outside`, `/lab/plain`, `/lab/guarded`, `/lab/stream` and `/lab/broken-stream` for global/group/method-specific middleware, reverse unwind, 404/405 and streaming failure semantics. See [M9b evidence](SAMPLES.md#middleware-lab-acceptance); independent reviewer `wc_job_UnKflDaA8ZQHXCce` PASS.

The optional, **stateless** SSR Forms Validation sample demonstrates accessible field errors and no-side-effect PRG without JavaScript or Session:

```bash
cargo run -p yetlet-forms-validation
```

It listens on `http://127.0.0.1:3005` by default. Routes are `/form`, POST `/form/submit`, and `/form/success`. A valid POST redirects with HTTP 303, but **stores nothing**; this is not a claim of credentialed CSRF defense. See [M9c evidence](SAMPLES.md#forms-validation-acceptance); final independent reviewer `wc_job_lGtk_ZIoAvK4F6PW` PASS.

The native File Server passed M9d independent acceptance review:

```bash
cargo run -p yetlet-file-server
```

It requires loopback (default port 3006), exposes a fixed static fixture and token-gated uploads/downloads, and stores only bounded temporary text files. See the [File Server README](../examples/file-server/README.md) for the private local development token and curl workflow, and [M9d evidence](SAMPLES.md#file-server-acceptance) for verification status. It is not a public file-hosting or identity service.

The SQL-first SQLite CRUD sample passed M9e independent repeat review:

```bash
cargo run -p yetlet-sqlite-crud
```

It defaults to loopback port 3007 and `target/yetlet-sqlite-crud.sqlite3`. It uses application-local direct `rusqlite`, explicit SQL migrations/transactions and bounded `spawn_blocking`; no Yetlet ORM or mandatory data layer is introduced. Reviewer `wc_job_Q3pBfUqPYGYNSIrD` found and prompted a fail-closed schema-column repair; repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS. See the [comparison](SAMPLES.md#sql-first-data-access-and-sqlite-acceptance) and [M9e evidence](SAMPLES.md#sql-first-data-access-and-sqlite-acceptance).

The native SSE Notifications sample passed M9f independent repeat review:

```bash
cargo run -p yetlet-sse-notifications
```

It listens on loopback port 3008 by default. `/events` uses bounded replay, heartbeat comments and `Last-Event-ID`; `/notifications` is a loopback command-oriented JSON publisher. No browser runtime, WebSocket or framework-global event circuit is introduced. First reviewer `wc_job_LF-i2IsQxFXO-obd` found control-character validation occurred after trim; raw-input validation fixed it and repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS. See [M9f evidence](SAMPLES.md#sse-notifications-acceptance).

The isolated Multi-Island Dashboard passed M9g final independent review:

```bash
cargo xtask build multi-island-dashboard && cargo run -p yetlet-multi-island-dashboard
```

It listens on loopback port 3009 and keeps generated Counter/Filter bundles under `target/multi-island-dashboard-dist/`, outside production `dist/`. Complete SSR remains when either bundle or the whole manifest is missing/invalid. Final reviewer `wc_job_OMstvGrtDiKZ5p6C` returned PASS after retained Chrome/Firefox fallback evidence was supplied. See [M9g evidence](SAMPLES.md#multi-island-dashboard-acceptance).

The loopback-only Auth Notes fixture passed M9h security-focused review:

```bash
cargo run -p yetlet-auth-notes
```

It generates synthetic Alice/Bob login codes in the local terminal, uses opt-in Session rotation/CSRF, and rechecks note ownership on every request. Reviewer `wc_job_EgSLng_VafIvs3ju` returned PASS. It is not a password, OIDC, MFA or production identity provider. See [M9h evidence](SAMPLES.md#auth-notes-acceptance).

The larger Modular Shop reference application passed M9i repeat independent review:

```bash
cargo run -p yetlet-modular-shop
```

It listens on loopback port 3008, composes synthetic identity, SQL catalog, Session cart and transactional orders through explicit Rust modules, and can disable Orders with `YETLET_SHOP_DISABLE_ORDERS=1`. First reviewer `wc_job_a_WnhFSftzNoYYgf` found and prompted smoke readiness cleanup; repeat reviewer `wc_job_zbWS7OSbUtaKzYcn` returned PASS. It is not a payment or production commerce system. See [M9i evidence](SAMPLES.md#modular-shop-acceptance).

`cargo xtask build browser --release` produces an optimized browser bundle. Do not confuse debug WASM bytes with production transfer sizes or performance.

#### Verify

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo check --locked \
  -p yetlet-todo-browser \
  -p yetlet-search-island \
  -p yetlet-reactive \
  -p yetlet-dom-keyed \
  -p yetlet-render-bench \
  -p yetlet-keyed-bindings \
  --target wasm32-unknown-unknown

cargo xtask check docs
cargo build --locked -p yetlet-todo-server
cargo xtask smoke browser-chrome
cargo test --locked -p yetlet-session -p yetlet-cart-server --all-targets
cargo build --locked -p yetlet-cart-server
cargo xtask smoke cart
```

For the **release-mode** informational baseline and a separate Firefox E2E check:

```bash
cargo build --release --locked -p yetlet-todo-server
cargo xtask build browser --release
cargo xtask smoke browser-firefox --profile release
cargo xtask measure baseline --profile release --samples 30 \
  --browser-runs 5 --out target/m5-baseline-release.json
```

The isolated M7a render benchmark is built and measured separately; it does not modify production `dist/`:

```bash
cargo xtask build browser --release
cargo xtask measure render-strategies \
  --capture-production-before target/m7a-production-before.json
cargo xtask build render-bench --release
cargo xtask measure render-strategies --profile release \
  --rows 200 --updates 500 --rounds 10 --seed 7 \
  --production-before target/m7a-production-before.json \
  --out target/m7a-render-strategies.json
```

The combined M7b/M7c/M7d authoring, local-reactivity and lifecycle prototype is also isolated from production routes/assets. M7e reruns both this fixture and the production regressions before freezing the evidence:

```bash
cargo xtask build keyed-bindings --release
cargo xtask smoke keyed-bindings-chrome --profile release
cargo xtask smoke keyed-bindings-firefox --profile release
```

Do not mix debug and release asset builds. Results are recorded in [M5 verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements); size estimates refer to local gzip artifacts, not actual transfer bytes.

Browser smoke tests use isolated headless profiles and temporary localhost servers, stopping only their own processes. M2 checks shared DOM/local clicks; M3 covers lazy islands and fail-closed state; M4 covers the explicit typed HTTP boundary. M7i additionally checks stable keyed row/list identity, explicit graph/listener disposal, readable detached DOM and duplicate-free remount in Chrome and Firefox. See [M7i verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

#### Workspace structure

- `yetlet-core` — Renderer-neutral `View` model plus validated component/instance/key/slot and inert-attribute binding contracts.
- `yetlet-reactive` — Optional strong-owner/weak-handle synchronous local message, batching and commit graph.
- `yetlet-html` — HTML serializer with escaping and a safe element/attribute policy.
- `yetlet-dom` — Direct browser DOM renderer with a restricted HTML subset.
- `yetlet-dom-keyed` — Optional keyed child reconciliation, entry-state ownership and disposable browser listeners.
- `yetlet-actions` — Typed commands and injectable LocalExecutor/HttpExecutor contracts.
- `yetlet-server-axum` — Thin App Builder, Page response, explicit setup composition, RequestContext and native Tower middleware.
- `yetlet-request-id` — Optional process-local numeric request correlation; the core does not depend on it.
- `yetlet-session` — Optional process-local server-side Session, expiry, CSRF and cookie lifecycle provider.
- `examples/todo-ui` — Shared TodoList, local transitions, and versioned search state.
- `examples/todo-browser` — Rust/WASM entrypoint with a keyed local graph and separate typed HTTP action path.
- `examples/search-island` — Lazy Search WASM, SSR DOM preservation, and local input handling.
- `examples/render-bench` — Isolated M7a strategy fixture; not a production route or public renderer API.
- `examples/keyed-bindings` — Isolated fixed-shape authoring and cross-browser lifecycle/form-state prototype; its reactive module re-exports optional `yetlet-reactive`.
- `examples/todo-server` — SSR/browser routes and in-memory typed GET/POST Todo API.
- `examples/rest-api` — Native-only typed Book CRUD with RequestContext policy and no browser/WASM dependency.
- `examples/cart-server` — Native-only SSR cart with explicitly scoped `yetlet-session` and no browser/WASM dependency.
- `cargo xtask build browser` — Content-addressed browser asset pipeline with isolated Todo/Search Cargo builds.

**Known limitations:** Only the production Todo local list uses the promoted keyed graph; remote Todo results and Search result updates retain bounded subtree replacement. The fixed keyed/static-slot authoring shape, text-input/focus policy and conditional slot remain experimental. Nested/general component trees, async effects, IME/mobile/WebKit coverage and generic hydration/resumability remain absent. The Todo API and Session cart are process-local/in-memory; the Session proof is anonymous, single-node and not production auth/CSRF qualification. Durable persistence, stable API, full IME/a11y matrix and cross-framework benchmarks remain absent. See [M7i verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence), [M8c verification](M8C-VERIFICATION.md) and [release readiness](RELEASE-READINESS.md).

#### Governance, security, and license

Yetlet is a maintainer-directed project. External pull requests are not accepted; repository settings restrict pull-request creation to collaborators so maintainers can still use PRs and required CI internally. Reproducible bugs, compatibility reports, documentation feedback, and focused design proposals are welcome through Issues. Security reports must follow [SECURITY.md](../SECURITY.md) and must not be posted publicly. See [CONTRIBUTING.md](../CONTRIBUTING.md) for the complete policy.

Yetlet is dual-licensed under your choice of [MIT](../LICENSE-MIT) or [Apache License 2.0](../LICENSE-APACHE). Workspace packages carry the SPDX expression `MIT OR Apache-2.0`; example and application packages remain `publish = false`. The license permits forks and independent modifications but does not require the upstream project to accept external code contributions.

#### Qualification status

[YT-020 Private release preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) selects experimental `0.1.0` and the original nine library candidates. The prepared policy preserves API behavior and the selected Rust 1.99 minimum in `0.1.x` patches, with best-effort maintenance and no SLA. [Integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) retains earlier Ubuntu/Windows experiments; later [hosted rebuilding](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-020-HOSTED-REBUILD.md) compares the same immutable source on separately operated builders. [Private trust policy](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) selects Cosign maintainer public-key preparation and future crates.io distribution. [Actual Private signing](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) records a dedicated key/fingerprint, three signatures and separate public-only verification. Formal OIDC, certificate/chain, transparency/inclusion and builder/source guarantees remain open and unchanged; registry activation is unverified. No package has been published, no public support policy is activated, and the isolated Redis provider remains nonpublishable.

M10 release-supply-chain design/preflight is independently reviewed. Four local Firefox release/keyboard suites, real local Session/SQLite operations and the YT-066a three-instance Docker/Memory proof passed with owned resource cleanup. The scheduled/manual qualification workflow was merged and hosted run `38043565238` passed Linux, Windows MSVC, macOS ARM64 and Linux Firefox lanes; separate local Windows GNU and offline Docker evidence also passed. YT-065 is complete at that bounded automated scope. Real IME, spoken screen-reader output, WebKit/mobile and zoom remain explicitly Not verified optional coverage. YT-066a demonstrates stateless multi-instance serving and current Memory Session limitations only; its final independent review passed, while the separately opt-in Redis provider and [three-application fixture](QUALIFICATION-MATRIX.md#historical-session-auth-oidc-and-cross-host-evidence) now have passing bounded single-host/no-persistence execution, cleanup and independent b3 review; YT-066 is Done at single-host Docker multi-instance and selected Redis-provider scope. Production TLS/ACL policy, backups, business RPO/RTO and operational ownership are application responsibilities; broader deployment durability/recovery remains unverified. No result establishes a supported-device range, accessibility conformance, production deployment or publication approval.

#### Documentation

- [Opt-in typed application configuration](CONFIGURATION.md)
- [YT-090 configuration scope and verification](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-090-VERIFICATION.md)

- [Architecture and module boundaries](ARCHITECTURE.md)
- [Full MVP specification](MVP.md)
- [Roadmap](ROADMAP.md)
- [Backlog](BACKLOG.md)
- [SSR, browser and handler architecture](ARCHITECTURE.md#ssr-browser-and-handler-architecture-boundaries)
- [Early SSR/handler measurements and limits](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements)
- [Keyed ownership and runtime retirement](BROWSER-APP.md#keyed-ownership-and-runtime-retirement)
- [Sample-specific acceptance and failures](SAMPLES.md#sample-acceptance-and-source-boundaries)
- [SQL-first application data access; no Yetlet ORM](SAMPLES.md#sql-first-data-access-and-sqlite-acceptance)
- [Original API inventory and review limits](API-CONTRACTS.md#original-inventory-acceptance-and-review-limits)
- [M7g optional synchronous local graph](M7G-VERIFICATION.md)
- [Sample application contract and catalog](SAMPLES.md)
- [M8c Session and SSR cart verification](M8C-VERIFICATION.md)
- [Public API contract classification](API-CONTRACTS.md)
- [Post-M9 CI and security review](CI-SECURITY-REVIEW.md)
- [Pre-public compatibility policy](COMPATIBILITY.md)
- [M10c compatibility/document metadata plan](COMPATIBILITY.md#m10-metadata-compatibility-evidence)
- [M10c planning reviewer evidence](COMPATIBILITY.md#m10-metadata-compatibility-evidence)
- [M10c implementation verification](COMPATIBILITY.md#m10-metadata-compatibility-evidence)
- [M10d production edge/authentication threat-model plan](THREAT-MODEL.md#m10-threat-model-acceptance-and-corrected-boundaries)
- [M10d planning reviewer evidence](THREAT-MODEL.md#m10-threat-model-acceptance-and-corrected-boundaries)
- [Production edge and authentication threat model](THREAT-MODEL.md)
- [M10d threat-model verification](THREAT-MODEL.md#m10-threat-model-acceptance-and-corrected-boundaries)
- [M10a license/security-policy verification](CI-SECURITY-REVIEW.md#m10-policy-governance-and-scan-evidence)
- [M10b repository-governance verification](CI-SECURITY-REVIEW.md#m10-policy-governance-and-scan-evidence)
- [Remaining M10 qualification plan](QUALIFICATION-MATRIX.md#m10-native-browser-and-delivery-reconciliation)
- [Release supply-chain design and preflight](RELEASE-SUPPLY-CHAIN.md)
- [M10e release verification](RELEASE-SUPPLY-CHAIN.md#m10-unsigned-supply-chain-preflight-evidence)
- [Platform/browser/accessibility matrix](QUALIFICATION-MATRIX.md)
- [M10f automated platform/browser qualification evidence](QUALIFICATION-MATRIX.md#m10-native-browser-and-delivery-reconciliation)
- [Provider operations and local limitations](PROVIDER-OPERATIONS.md)
- [M10g local operational evidence](PROVIDER-OPERATIONS.md#m10-local-operations-and-bounded-framework-closeout)
- [Optional future external-assessment template](EXTERNAL-SECURITY-REVIEW.md)
- [M10h internal security-scan status](CI-SECURITY-REVIEW.md#m10-policy-governance-and-scan-evidence)
- [M10 CI, Windows and Docker delivery verification](QUALIFICATION-MATRIX.md#m10-native-browser-and-delivery-reconciliation)
- [Contribution policy](../CONTRIBUTING.md)
- [Security policy](../SECURITY.md)
- [Module composition and optional providers](EXTENSIONS.md)
- [Prototype release readiness and blockers](RELEASE-READINESS.md)
- [Private experimental 0.1.0 preparation and remaining activation gates](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12)
- [Private public-key trust and future crates.io policy](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures)
- [Actual Private maintainer-key signing and independent verification](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures)
- [ADR 0001](adr/0001-separation-of-concerns.md)
- [ADR 0002](adr/0002-events-and-actions.md)
- [ADR 0003](adr/0003-renderers-and-loading.md)
- [ADR 0004 — RequestContext and middleware](adr/0004-request-context-and-middleware.md)
- [ADR 0005 — Optional capability modules](adr/0005-opt-in-capability-modules.md)
- [ADR 0006 — Handler-first and optional MVC](adr/0006-handler-first-mvc-optional.md)
- [ADR 0007 — Bounded keyed DOM and local graph](adr/0007-bounded-keyed-dom-and-local-graph.md)
- [ADR 0011 — Simple application startup and explicit native control](adr/0011-application-startup-api.md)

M0–M9 are locally complete at prototype scope. M8 freezes the runnable sample contract; M9 adds twelve independently reviewed applications, a 170-export compatibility classification and complete native/browser/provider evidence freeze. The post-M9 CI/security round patches `RUSTSEC-2026-0009`, raises and verifies the Rust 1.88 MSRV, and hardens hosted CI. GitHub Actions run `38024582602` completed successfully for commit `86c30a42114ca74f4197f0389707861308a844a7`, including both the Rust 1.88 job and the stable native/samples/Chrome job. M10 adds the bounded document contract, threat model, dual-license/security policy and maintainer-only governance configuration. Repeat reviewers and green CI do not make Yetlet SemVer-stable or production-ready. SQL access remains application opt-in and Yetlet has no built-in ORM or mandatory DB layer. Public-release readiness (`YT-020`) is **Blocked** for actual immutable-source/registry, trusted release integrity, public reporting/scanner activation and concrete maintainer approval. Private preparation selects experimental `0.1.0`, the original nine libraries and bounded maintenance/qualification policy; completed framework documentation/qualification is not an open blocker. Application deployment policy, backups, business RPO/RTO and on-call ownership are not framework qualification prerequisites. The Codex Security Standard source scan reviewed 227/227 regular tracked files and found one Low issue; its tiny local correction passed comprehensive regressions and independent reviewer. No external audit, production certification or post-patch complete rescan is claimed. Paid external assessment is not a current requirement. Current code is not a production release.

</details>

- [YT-183 Optional catalog-data cache contract](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183): Done at application-reference scope after independent Reviewer PASS; actual Shop display consumer, default-off native cache, bounded source age and commit-generation invalidation; authoritative Session/checkout preserved; zero review issues/required followups.

- [YT-183 Optional catalog-data cache implementation](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183): Done after independent Reviewer PASS, zero issues/followups; actual Shop display reuse, two-second source age, commit-generation invalidation and unchanged Session/checkout authority; nine new test executions PASS without old qualification replay.

- [YT-184 Azure cross-host Auth/Session and persistent restart](PRODUCTION-DEPLOYMENT-PROFILES.md#historical-shared-authority-and-azure-persistence-qualification): Done at bounded reference scope after final independent Reviewer PASS0: native opt-in TLS/unit/compile proof and preallocation review pass; first cloud bootstrap fails before live Auth/restart, smallest Path fix locally checked and all owned resources/credentials removed. Limited first correction/cleanup review PASS0. Authorized generation2 reaches both apps READY/wrong-name TLS refusal, then fails the SQL observer before Auth/restart; observer corrected locally, both failed generations fully deleted. Full integration pending; maintainer explicitly authorizes generation3 with the reviewed observer fix and cumulative US$5/90min including cleanup bounds; generation3 passes private SQL/HTTPS checks then first auth form fails; missing direct-TIME ACL correction now locally verified by one new named-user Session provider regression (6oldcases filtered); local named-app/Session independent Reviewer PASS0, previous invalid probes retained, new local resources removed; first three Azure generations fully deleted; fourth passes cross-host Auth/policy/logout/app-restart then fails first Valkey recovery snapshot; new local persistent read-only readiness case and independent fourth-results/correction review PASS0; first four allocations deleted; fifth completes remaining authority recovery/VM/SQL-policy/new-epoch proof with native exit0; all five groups deleted, final Reviewer PASS0 accepts composed reference proof; no sixth authorized.

- **M23 onboarding and documentation (YT-185–191):** the accepted Board/Admin
  and Shop startup options separate Cargo compile-time features from explicit
  application typed runtime selection; no generic Core capability registry
  or implicit settings provider was added. The docs-only sequence rebuilt
  task-oriented entry points, ten five-part integration guides, a source-backed
  native first-app walkthrough, 27 troubleshooting cases across six groups,
  and current CLI/View/opt-in navigation. Refer to
  [First App](FIRST-APP.md), [Guide Template](GUIDE-TEMPLATE.md),
  [Troubleshooting](TROUBLESHOOTING.md), [View authoring](VIEW-COMPONENTS.md),
  [Opt-in quick reference](OPT-IN-QUICK-REFERENCE.md), [CLI guide](CLI-GUIDE.md)
  and [Documentation Scope](DOCUMENTATION-SCOPE.md). Original reviewer
  failures and corrections (including YT-187 browser prerequisites and
  YT-188/189 strict Guard handoff) are recoverable from fixed **local Git**
  `4792565e6cff7a94c8c2c860d1858f840f122cbe`; these docs-only checks did not replay runtime/browser tests,
  publish a guide or validate novice usage. Public GitHub SHA reachability
  remains unverified until separately approved delivery.
- [YT-192 3D physics lab minimum contract and feasibility](ARCHITECTURE.md#experimental-scene-browser-gpu-and-physics-lab-boundaries): Done at documentation/source-level feasibility scope after independent Reviewer PASS0; browser/scene/fixed-step/GPU-failure contract, no implemented 3D API/sample or compile/runtime qualification.

- **YT-193–198 — Physics Lab implementation:** The
  [current application guide](../examples/physics-lab/README.md) owns the
  opt-in Rapier/WebGPU experiment, first explicit Run, Pause/Step/Reset,
  bounded scenarios including slope friction, estimated submission FPS
  and safe native SSR/HTML fallback. [Wasm startup guidance](WASM-STARTUP.md)
  owns pre-Wasm loading, explicit recovery and stale/disposed owner handling.
  The original isolated Firefox/Wayland/NVIDIA test scope, failures and
  review outcomes remain in fixed local Git history; neither hardware
  portability nor physics correctness/SLO or public release is certified.
</details>
