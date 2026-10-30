# Developer Documentation

Start with the [native SSR quickstart](../README.md#start-with-native-ssr), then
choose a [runnable sample](SAMPLES.md). The Experimental
[v0.9.0-alpha.1 source-package pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.1)
is published on GitHub; crates.io remains separate. Current task status belongs
to [Backlog](BACKLOG.md), milestone gates to [Roadmap](ROADMAP.md), and current
distribution versus historical qualification to [Release readiness](RELEASE-READINESS.md).

## Reader entry points

- [Troubleshooting common problems](TROUBLESHOOTING.md): symptom → possible cause → correction → existing guide.
- [Build your first Yetlet app](FIRST-APP.md): local CLI → protected standalone project → local dependencies → app lock/cache → first build/run.
- [Write Views and reusable components](VIEW-COMPONENTS.md): typed data → functions → lists/conditions → existing SSR handler.
- [Opt-in integration quick reference](OPT-IN-QUICK-REFERENCE.md): package/feature → startup API/options → consumer settings/defaults.
- [Settings, options and logging](SETTINGS-OPTIONS-LOGGING.md): startup loading → validation → typed options → native state and application-owned diagnostics, using existing examples.
- [Integration guide template](GUIDE-TEMPLATE.md): prerequisites → minimal integration → expected result → configuration/errors → API reference.
- [Current API reference](API-REFERENCE.md): package entry points, feature/target
  selection, application options and local rustdoc; read guides and signatures
  at the same revision.
- [Documentation sources and publication scope](DOCUMENTATION-SCOPE.md): selected
  reader sources, separate maintainer/evidence material and no automatic export
  of `docs/**` or linked records.
- [Native SSR quickstart](../README.md#start-with-native-ssr),
  [existing sample catalog](SAMPLES.md) and [Board startup](BOARD-START.md).
- [HTTP pipeline](HTTP-PIPELINE.md), [opt-in composition](EXTENSIONS.md),
  [configuration](CONFIGURATION.md), [naming](NAMING.md) and
  [current capabilities](CAPABILITIES.md).
- [Current CLI guide](../tools/yetlet-cli/README.md#current-command-and-capability-inventory).

- [WebAssembly startup and loading UI](WASM-STARTUP.md): optional shared JS/CSS shell, customization, full-page/SSR-region integration and caller-owned lifetime.

## Maintainer entry points

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
- [Single-gate release automation and registry promotion](RELEASE-PUBLICATION.md):
  automated build/sign/verification, final owner-approved GitHub then crates.io
  publication, local implementation and one-time rollout limits.
- [Sixteen-package crates.io readiness](YT-020-REGISTRY-READINESS.md): live name
  and public-account observations, dependency order, first-publication bootstrap
  and remaining registry-only evidence; no upload authorization.
- [Config/Secrets first-publication contract](YT-020-FIRST-PUBLISH-CONTRACT.md):
  unchanged manifests, local normal publish dry-runs and their artifact/checksum
  limits; registry activation remains separate.
- [Formal registry publication and artifact binding](YT-020-REGISTRY-PUBLICATION-CONTRACT.md):
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


## Current release-candidate qualification

[YT-096 candidate selection and package preflight](YT-096-CANDIDATE-QUALIFICATION.md) records the owner-approved `ce56e08` source, original nine experimental `0.1.0` libraries, Rust 1.99 and selected support targets. Nine source packages and a clean packaged-input native consumer pass on Ubuntu 26.04.1. The [selected dependency/API delta](YT-096-SUPPLY-CHAIN-DELTA.md) records 23 SBOMs and license/source checks, nine passing RustSec package-lock scans, API/threat reconciliation and an unsigned checksum bundle with explicit coverage limits. The selected Ubuntu 24.04 baseline and remaining platform/build-bound inventory/rebuild/formal review/trust gates are explicit. YT-096 is In progress and YT-020 remains Blocked. Historical accepted evidence is preserved; no publication or production support is activated.

The [partial platform followup](YT-096-PLATFORM-QUALIFICATION.md) adds Ubuntu
24.04.5 container userspace tests/doctests, five packaged wasm builds and a
source-reconciled Firefox workflow. It retains the shared-kernel/Chrome evidence
limits and the historical Windows cache failures. The resumed standalone
archive lane passes all nine packages on Windows MSVC: 83 native tests and
four doctests. The [two-host archive rebuild](YT-096-TWO-HOST-REBUILD.md) now
matches all nine Windows archives byte-for-byte to the Linux originals.
Independent builder control and later release gates remain open. The earlier
[SSH failures](YT-096-AZURE-CLEAN-BUILDER.md) are preserved. The
[Azure Run Command continuation](YT-096-AZURE-RUN-COMMAND.md) now passes on an
actual Ubuntu 24.04.5 guest with an empty cache: nine identical archives,
83 native tests/four doctests and five wasm libraries. All temporary resources
are deleted; aggregate Cargo observations retain explicit SBOM/dev-only limits.

The [GitHub-hosted candidate rebuild](YT-096-GITHUB-HOSTED-REBUILD.md) reuses the
existing manual workflow at the exact same candidate. All nine returned archives
match complete primary bytes with matched source/lock/tool inputs and verified
unsigned bindings. This adds separately operated execution evidence; formal
trust and release approval remain open. The owner subsequently accepted only
the bounded inventory/platform disposition described below.

The [final inventory/platform evidence reconciliation](YT-096-FINAL-RECONCILIATION.md)
joins nine archives to 23 target SBOMs and retained Cargo observations, with
133 browser-source/eight asset identities checked and no runtime reruns.
The [owner's support/inventory decision](YT-096-FINAL-RECONCILIATION.md#owner-support-and-inventory-acceptance-2026-10-20)
accepts the experimental nine-source-package scope and known differences.
Exact-candidate Chrome qualification and complete compiled/system inventory
remain Not verified; no formal release gate is waived.

The [final candidate security/release review](YT-096-SECURITY-RELEASE-REVIEW.md)
is complete. Bounded technical/security evidence is accepted with limits and no
new material source defect was identified; the overall release gate is FAIL,
release-ready NO. Subsequent owner support/inventory acceptance resolves one
disposition; formal integrity/trust, actual registry proof, public activation
controls and concrete maintainer release approval remain open.
No runtime rerun, signing or publication occurred.

The historical [minimum formal trust/provenance proposal](YT-096-FORMAL-TRUST-PROPOSAL.md)
and [2026-10-23 decision brief](YT-096-TRUST-DISCLOSURE-DECISIONS.md) retain their
original nine-package scope, pending decisions and independent review PASS.
Those reviews apply to the original documentation, not the new implementation.

The [D1–D5 release supply-chain policy](YT-096-D1-D5-RELEASE-TRUST-PLAN.md) is
**Adopted (policy) as of 2026-10-27; D6 remains Pending**. It covers GitHub workload
OIDC/public Sigstore disclosure, closed sixteen-subject public provenance/DSSE
and complete private evidence, sixteen experimental `0.1.0` packages with
`118f3d0` as technical baseline and future co-located `C_release`/release workflow
with proposed `v0.1.0`, Cosign3.1.4/reference roots subject to actual bootstrap,
validity and compatibility checks, and source-package-only claims with acyclic
bindings. No actual release SHA, tag, final output/payload, authenticated Cosign
installation or new build result is established by adoption.

The owner authorizes build/verify-only [Release preparation workflow implementation](YT-096-RELEASE-CI-INTEGRATION.md),
with new changes uncommitted/unpushed. Hosted build/verify execution needs separate
explicit authority and may be authorized while D6 remains Pending. D6 holds
identity issuance/signing; publication needs separate explicit approval.
YT-096 remains In progress, YT-020 Blocked and YT-206 Done.

[YT-107 form/save feedback](YT-107-FORM-FEEDBACK.md) is Done at the bounded
Admin/Todo scope, with independent review of all three slices: server validation to field messages,
explicit save state/recovery, and editor/native/Todo consistency. Its
[application authoring guide](YT-107-FORM-FEEDBACK.md#application-authoring-guide)
shows existing patterns, intentional Reset/HTTP differences and the actual
costs that remain app-owned.

## Architecture and contracts

- [YT-096 independent signer contract](YT-096-SIGNER-CONTRACT.md) and [ADR 0009](adr/0009-release-build-and-signer-identities.md): maintainer release-engineering design, separate fixed-SHA build/signer identities and two-payload freeze; D6 execution remains Pending.

- [Architecture](ARCHITECTURE.md) and [MVP contract](MVP.md): responsibilities, trust boundaries and original acceptance criteria.
- [DOM capability and authoring review](YT-109-DOM-CAPABILITY-REVIEW.md): current renderer/consumer inventory, property and ownership boundaries, and staged proposals; research only, not runtime implementation.
- [Bounded DOM capability implementation](YT-110-DOM-CAPABILITIES.md): finite View vocabulary, duplicate/void consistency, native control defaults/reset, same-origin navigation and actual verification; no general binding runtime.
- [HTTP pipeline](HTTP-PIPELINE.md): native Axum/Tower handlers, middleware, request state and response semantics.
- [Extensions](EXTENSIONS.md): explicit opt-in setup and provider boundaries.
- [Naming guide](NAMING.md): current Rust/CLI/settings vocabulary, one canonical pre-public Rust API and explicit source migrations. [Audit response](NAMING-AUDIT-20261022.md) records N01–N08 and focused evidence; [the earlier compatibility checkpoint](NAMING-COMPATIBILITY.md) retains its original accepted results and superseded alias decision.
- [Current capability and API carrier map](CAPABILITIES.md): implemented packages/features, application integrations, research decisions and release inclusion.
- [M9 API classification and historical supplements](API-CONTRACTS.md) and [Compatibility](COMPATIBILITY.md): the original classification and pre-public stability limits; later packages have separate contracts in the current map.
- [CLI contract](CLI-CONTRACT.md): accepted settings-first CLI naming, grammar, execution boundaries and intentional migration; see [current implementation evidence](YT-102-CLI-IMPLEMENTATION.md). The [current CLI guide](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) remains the executable reference; [YT-102 baseline verification](YT-102-CLI-CONTRACT-VERIFICATION.md) records this documentation slice without completing DX research.
- [YT-111 and YT-112 DX follow-ups](YT-111-112-DX-SCOPE.md): [read-only project inspection](YT-111-IMPLEMENTATION.md) and [browser-only Rust edit/build/reload](YT-112-IMPLEMENTATION.md) are Done at their selected scopes, with independent reviews and focused Linux runtime proof. Historical registration, failures and unverified Windows browser-watch scope remain explicit.

- [YT-115 optional Memory Session/Auth bridge](YT-115-SESSION-AUTH.md): explicit feature, current account mapping, native-form CSRF and real loopback HTTP lifecycle evidence; independent Reviewer PASS.
- [YT-117 optional rate limiting](YT-117-RATE-LIMIT.md): selected endpoint/group policies, separate configuration reuse and shared per-process budgets, and default-off startup installation; Done at per-process scope after independent Reviewer PASS.
- [YT-118 Audit study](YT-118-AUDIT-STUDY.md): existing Auth Notes/Admin behavior, transactional business history versus diagnostics, allowlisted event semantics and explicit extraction threshold; Done at docs-only research scope after independent Reviewer PASS; no audit crate/runtime selected.
- [YT-121 Cache study](YT-121-CACHE-STUDY.md): compares real business reads and asset snapshots, separates disposable data from authoritative state and HTTP output, and records native options/extraction thresholds; Done at docs-only research scope after independent Reviewer PASS; no cache crate/runtime selected.
- [YT-122 Output Cache study](YT-122-OUTPUT-CACHE-STUDY.md): public assets versus personalized SSR/API, final HTTP header/cookie policy, Session/tenant isolation and invalidation; private content stays uncached by default. Done at docs-only research scope after independent Reviewer PASS; no output-cache crate/runtime selected.
- [YT-123 Tracing/Correlation study](YT-123-TRACING-CORRELATION-STUDY.md): existing RequestIds/SSR/API diagnostics versus native tracing/Tower, explicit task/stream propagation and source-trust/redaction boundaries; Done at docs-only research scope after independent Reviewer PASS; no tracing crate/runtime/subscriber/exporter selected.
- [YT-134 optional release/storage capacity reference](YT-134-RELEASE-STORAGE-CAPACITY.md): predeclared direct-loopback measurements for the exact accepted release binary and disposable schema-2/ext4 fixture, retaining passing and failed load points, write reconciliation and cleanup; Done at bounded optional reference scope after independent Reviewer PASS, zero issues/required followups. Consumer hardware/SLO and production qualification remain separate. A separately authorized [ephemeral Azure supplement](YT-134-AZURE-REFERENCE.md) completed its workload, reconciliation, ordinary reboot/readback and full cleanup, with separate independent Reviewer PASS, zero issues/required followups; failed cloud points and measurement limits remain explicit.
- [YT-135 current-schema recovery reference](YT-135-CURRENT-SCHEMA-RECOVERY.md): separately activated release/schema-2 backup, fresh restore, explicit two-acknowledgment loss accounting, missing-index refusal and forward-restart readback; isolated proof/cleanup complete with independent Reviewer PASS, zero issues/required followups; Done at bounded optional reference scope.
- [YT-136 bounded telemetry collection reference](YT-136-TELEMETRY-COLLECTION.md): opt-in private snapshot/log collection and [reusable CLI guidance](../examples/production-starter/deployment/telemetry/README.md), reusing YT-128 observability and YT-133 access evidence. New collection/failure/staleness integration, bounded log projection, oversize rejection and cleanup complete; independent Reviewer PASS, zero issues/required followups; Done at bounded optional reference scope; no mandatory collector or production sink/alert operation.
- [YT-137 write-outcome/idempotency contract research](YT-137-WRITE-OUTCOME-IDEMPOTENCY.md): source-bound Starter/Board no-automatic-retry and reconciliation decision, modular-shop scoped SQLite order deduplication and conditional future replay-ledger boundaries; independent Reviewer PASS, zero issues/required followups; Done at research scope. Research only, with no new automatic retries, middleware, runtime or public API.
- [YT-138 authorization-to-commit ordering research](YT-138-AUTHORIZATION-COMMIT-ORDERING.md): retain the source-bound Admin/Board pre-SQL race; distinguish Session/policy snapshots, revocation, business commit and acknowledgment, with conditional shared-authority ordering/failure/recovery boundaries; independent Reviewer PASS, zero issues/required followups; Done at research scope. Research only; no stronger runtime ordering guarantee or automatic framework feature is selected.
- [YT-139 multi-instance authority/recovery reference](YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md): selected local two-Admin business/Session/current-policy authority and recovery contract; independent contract Reviewer PASS, zero issues/required followups; bounded new stale-Session restoration/re-login proof, complete business/policy reconciliation and owned cleanup complete; final independent Reviewer PASS, zero issues/required followups; Done at bounded optional local reference scope. No mandatory database, HA or stronger write-ordering guarantee.
- [YT-140 OIDC proxy/replica integration](YT-140-OIDC-PROXY-REPLICAS.md): contract-first minimum integration for maintained-client callback, explicit trusted origin/proxy and identity freshness boundaries; reuse accepted IdP and local authority evidence. Done at bounded optional local integration scope; initial independent contract Reviewer PASS, zero issues/required followups; minimum origin integration, three new exact tests, static checks and owned cleanup complete; final independent Reviewer PASS, zero issues/required followups. New proof uses two Board listeners/managers in one process, without successful token exchange or cross-process Admin OIDC qualification. No new OIDC protocol or mandatory identity product.
- [YT-141 reproducible performance-validation reference](YT-141-PERFORMANCE-VALIDATION.md): independent contract PASS, minimal k6 reference and new controlled-arrival read-only API measurement/cleanup complete; 14,006 measured valid-200 responses, zero drops/errors, p50/p95 1/2 ms and p99 2–4 ms. Twenty rows/schema/index/sequence unchanged; Done at bounded optional local reference scope after final independent Reviewer PASS, zero issues/required followups; eight hashes matched. Existing YT-134 passing/failed points retain their original scope; no universal throughput or consumer SLO/CI gate.
- [YT-142 saturation and recovery reference](YT-142-SATURATION-RECOVERY.md): initial independent contract PASS; first 1,000/s point retained FAIL with five SQL-admission 503 outcomes among 10,001 requests (~0.05%), no CPU/generator saturation claim. Immediate 250/s recovery 7,501 valid HTTP 200 responsess, all 30 bins healthy; full 20-row/schema/index/sequence equality and cleanup complete. Done at bounded optional observation scope after final independent Reviewer PASS, zero issues/required followups; eight hashes matched. The load point remains FAIL. No product optimization or old passing-proof rerun.
- [YT-143 native Axum/Yetlet workload comparison](YT-143-AXUM-WORKLOAD-COMPARISON.md): Done at bounded comparison scope after independent final Reviewer PASS, zero issues and required followups; nine hashes matched. Minimal standalone sample and focused checks PASS; all fixed phases ran once. Original tmpfs/client-cap failures remain retained; an independently accepted ext4 correction ran only the previously unstarted second repetition. Five corrected pairs are comparable; SSR 4,000/s remains noncomparable after one native-Axum drop. Cleanup complete; no two-valid-repetition, stable-overhead or capacity claim. PostgreSQL is the next separate step.
- [YT-144 optional PostgreSQL workload comparison](YT-144-POSTGRES-WORKLOAD-COMPARISON.md): Done at bounded comparison scope after independent final Reviewer PASS, zero issues and required followups. Previously accepted optional SQLx implementation/build/focused PostgreSQL 18.6 checks are reused. Separately human-authorized observer correction and header-lookup harness correction completed only the eight never-started groups: 100,007 measured exact valid 200s, zero errors/drops/interruptions and four comparable pairs. Original observer failures, failed preload and final review FAIL remain retained. Full logical/schema equality and owned cleanup complete within the renewed fixed deadline. No stable-overhead/capacity claim or old passing-proof replay.
- [YT-145 small/medium/large SSR costs](YT-145-SSR-PERFORMANCE-BASELINES.md): Done at fixed local SSR baseline scope after independent final Reviewer PASS, zero issues and required followups; independent contract PASS, scoped sample/two focused tests and separate normal/counted builds complete. Fixed 8/128/1,024-row pages produced 1,200 measured timings, 240 warmups and 36 balanced allocation observations; six exact HTML hashes matched. Logical heap peaks and whole-process RSS remain separate; raw populations, limits and future gap candidates retained. Nine reviewed file hashes matched the independent acceptance.
- [YT-146 SSR serializer attribution](YT-146-SSR-SERIALIZER-ATTRIBUTION.md): Done at bounded attribution-study/proposal scope after independent final Reviewer PASS, zero issues and required followups; nine reviewed hashes matched. Contract PASS preceded four new focused tests, isolated builds and measurements: six output hashes match, six allocation operations reconcile to YT-145, and 80 component timings plus 20 warmups are retained. Escaping contributes 85.70% of document allocation calls; output-buffer request volume and standalone timing have separate limits. One private escaping follow-up is proposed without implementation, API/default-graph changes or replaying accepted proof.
- [YT-147 private escape-into implementation](YT-147-PRIVATE-ESCAPE-INTO.md): Done at bounded implementation/comparison scope after independent final Reviewer PASS, zero issues and required followups, dependent on Done YT-146. Independent contract review precedes implementation and a minimum matched SSR comparison; preserve public escaping/output/security semantics and unchanged textarea normalization. Reuse the source-derived temporary-String reference, accepted verification and all-size literal/hash oracles; retain failures, timing variation, finite bounds and cleanup. Independent final Reviewer PASS is required before Done; no numerical performance gate, public API/dependency/default-graph change, commit or push. Contract PASS now precedes the completed private helper, three new filtered tests and two offline locked builds. Twelve output hashes match, reference counts reconcile, and the fixed 160 timings/40 warmups plus 12 balanced count observations are retained. Allocation calls fall while held output/document peak remain unchanged; lower current medians are descriptive, with variation and control limitations. Nine reviewed file hashes matched; the independent verdict and source/result evidence are retained.
- [YT-148 URL-check allocation attribution](YT-148-URL-CHECK-ATTRIBUTION.md): Done at bounded attribution/proposal scope after independent final Reviewer /root/yt148_url_attribution_final_review_20261023 PASS, zero issues/required followups and eight reviewed hashes matching; contract PASS, two new filtered tests and both locked release builds. The fixed 40 timings/10 warmups and three balanced count operations are retained; URL events reconcile to accepted YT-146 evidence without replay. Product/API/security and historical bindings are unchanged. Standalone timing variation and source-derived limits remain explicit; one future prefix-streaming proposal only, no optimization implementation, commit or push.
- [YT-149 private streaming URL-prefix check](YT-149-PRIVATE-URL-PREFIX.md): Done at bounded implementation/comparison scope after independent final Reviewer /root/yt149_url_prefix_final_review_20261024 PASS, zero issues/required followups and nine reviewed hashes matching; contract PASS, the private function-body change, three new focused tests, both locked release builds and six literal/hash outputs. The fixed 80 timings/20 warmups and six counted operations remain retained; reference counts reconcile to YT-148 and the current copied URL probe observes zero logical allocation deltas. Security/API semantics, raw timing variation, failure history and evidence limits remain explicit; no public API/dependency change, commit or push.
- [YT-150 full SSR source-version comparison](YT-150-FULL-SSR-COMPARISON.md): Done at bounded full-SSR measurement scope after independent final Reviewer /root/yt150_full_ssr_final_review_20261024 PASS, zero issues/required followups and eight reviewed hashes matching; contract PASS, one focused test, four actual pinned-source release builds and twelve output hash/literal checks. The fixed 240 timings/60 warmups and 18 allocation operations are retained with reference reconciliation; document/combined observations improve locally while construction varies and held/peak memory stays unchanged. Provenance, preparation failures, login-session context and limits remain explicit; no other optimization, old-proof replay, new commit or push.
- [YT-151 View/metadata construction attribution](YT-151-VIEW-CONSTRUCTION-ATTRIBUTION.md): Done at bounded attribution/proposal scope after independent final Reviewer /root/yt151_view_attribution_final_review_20261024 PASS, zero issues/required followups. Exclusive formatting/String/Vec/metadata/inline-node creation-call buckets reconcile to accepted YT-150 global snapshots; separate ordinary format/prepared-assembly probes, structural/output bindings and explicit limits. Independent reviews, retained failures and one future proposal only; no product/API change, optimization implementation, commit or push.
- [YT-152 private first-child capacity comparison](YT-152-FIRST-CHILD-CAPACITY.md): Done at bounded comparison/rejected-candidate scope after independent final Reviewer /root/yt152_first_child_final_review_20261024 PASS, zero issues/required followups. Actual pinned libraries, single/multi-child and full-page construction/combined SSR. Candidate rejected after memory/reallocation/elapsed tradeoffs; default production Core restored, candidate evidence/new test retained; unchanged public API/output/security, fixed populations, retained allocation/time tradeoffs and failures, independent reviews before Done. No new commit or push.
- [YT-153 document buffer attribution](YT-153-DOCUMENT-BUFFER-ATTRIBUTION.md): Done at bounded attribution/proposal scope after independent final Reviewer /root/yt153_document_buffer_final_review_20261024 PASS, zero issues/required followups. Actual body API, source-derived envelope composition, finite growth/lifetime observations and separate ordinary append probes; historical reconciliation, unchanged product/API/security, independent reviews and one future proposal only. No optimization implementation, commit or push.
- [YT-154 document tail-capacity comparison](YT-154-DOCUMENT-CAPACITY-COMPARISON.md): Done at bounded implementation/comparison scope after independent final Reviewer /root/yt154_document_capacity_final_review_20261024 PASS, zero issues/required followups. Actual paired libraries, small/medium/large document and large combined SSR; explicit body drop before closing, unchanged API/output/security, retained tradeoffs/failures and independent reviews before Done. No new commit or push.
- [YT-155 small/medium latency stability](YT-155-SMALL-MEDIUM-LATENCY-STABILITY.md): Done at bounded research scope after independent final Reviewer /root/yt155_latency_stability_final_review_20261024 PASS, zero issues/required followups. Initial/declared-warm actual API observations and same-process alternating tail proxies, accepted YT-154 inputs, explicit uncertainty and accepted independent reviews. No product/API/security/optimization change, commit or push.
- [YT-156 full HTTP SSR comparison](YT-156-HTTP-SSR-COMPARISON.md): Done at bounded comparison scope after independent final Reviewer /root/yt156_http_ssr_final_review_20261024 PASS, zero issues/required followups. Actual before/current libraries, fresh small/medium/large HTTP Page routes, two offered points and two balanced repetitions; retained latency/resource/error populations and accepted independent reviews. No optimization, product/API/security change, commit or push.
- [YT-157 medium HTTP p95 attribution](YT-157-MEDIUM-HTTP-TAIL-ATTRIBUTION.md): Done at bounded attribution scope after independent final Reviewer /root/yt157_medium_tail_final_review_20261024 PASS, zero issues/required followups. Retained native components plus four new sampled pipeline/construction probes, bounded correlation/observer limits and accepted independent reviews. No product/API/output/security change, optimization, commit or push.
- [YT-158 render-history construction context](YT-158-RENDER-HISTORY-CONTEXT.md): Done at bounded attribution scope after independent final Reviewer /root/yt158_render_context_final_review_20261024 PASS, zero issues/required followups. Four fixed synchronous CPU/wall/history processes and preserved failures; current construction CPU slightly lower, small history sensitivity and original HTTP cause unresolved. No optimization, product/API/output/security change, commit or push.
- [YT-159 HTTP worker thread CPU](YT-159-HTTP-WORKER-THREAD-CPU.md): Done at bounded attribution scope after independent final Reviewer /root/yt159_http_worker_cpu_final_review_20261024 PASS, zero issues/required followups. Four new-feature HTTP populations, same-thread construction CPU/wall and empty overhead; CPU-visible difference established, unique original HTTP cause unresolved. No optimization, product/API/output/security change, commit or push.
- [YT-160 frequency and worker activity](YT-160-FREQUENCY-WORKER-DUTY.md): Done at bounded research scope after independent final Reviewer /root/yt160_frequency_duty_final_review_20261024 PASS, zero issues/required followups. Exact accepted artifacts plus four new read-only context populations; lower reported frequency associates with higher construction/empty CPU, unique historical cause unresolved. No runtime/host setting change, optimization, commit or push.
- [YT-161 controlled benchmark profile/admission](YT-161-BENCHMARK-EXECUTION-PROFILE.md): Done at profile/admission scope; Independent final Reviewer /root/yt161_benchmark_profile_final_review_20261024 PASS, zero issues/required followups. One metadata container verifies resource limits, exits0 and is removed; controlled-frequency Not admitted, suitable target Pending, no new SSR load. Retained wrapper failure/correction and126.94s aggregate versus120s bound miss; no product/host setting change, cloud provisioning, commit or push.
- [YT-162 benchmark target/platform comparison](YT-162-ISOLATED-PLATFORM-COMPARISON.md): Done at bounded platform-reference scope after Independent final Reviewer /root/yt162_azure_results_final_review_20261024 PASS, zero issues/required followups. Blob path restored; four populations retain39977valid200/27drops, only first pair comparable (current HTTPp95 lower18.48%, construction CPU median similar). All resources/credentials removed, raw failed point preserved; YT-163 records unstarted drop attribution. No general speedup/frequency claim, product change, commit or push.
- [YT-163 retained dropped-arrival attribution](YT-163-DROPPED-ARRIVAL-ATTRIBUTION.md): Done at bounded research scope; independent contract PASS, offline analysis locates all27drops in27.752147ms with64outstanding HTTP brackets at each drop. Waiting dominates; two short server probes cannot separate VM/SSH/client/pre-middleware causes. Independent final Reviewer /root/yt163_drop_results_final_review_20261024 PASS, zero issues/required followups; failed pair preserved, no live replay, product change, commit or push.
- [YT-164 private direct HTTP/SSH comparison](YT-164-PRIVATE-HTTP-SSH-COMPARISON.md): Done at bounded comparison scope; contract/mapping PASS,40004measured/1004warmupvalid200,zero drops/errors. Two balanced pairs observe directHTTPp95lower49.73/53.43percent; k6CPU similar, forwarding adds work, unique historical cause unresolved. All16owned resources/credentials removed; Independent final Reviewer /root/yt164_transport_results_final_review_20261024 PASS, zero issues/required followups. New changes no commit or push.
- [YT-165 private HTTP capacity/saturation/recovery](YT-165-PRIVATE-HTTP-CAPACITY-RECOVERY.md): Done at bounded profile-study scope; contract/mapping PASS,4kclean/8ksaturated with20450drops,5929.6validin-windowRPS,p95 63.445ms. Both same-PID1k recoveries have0errors/drops,p95below1ms; all16owned resources removed. Exact boundary/long-duration limits remain; Independent final Reviewer /root/yt165_capacity_results_final_review_20261024 PASS, zero issues/required followups, no new-change commit/push.
- [YT-166 private HTTP capacity refinement](YT-166-PRIVATE-HTTP-CAPACITY-REFINEMENT.md): Done at bounded refinement scope; contract/mapping PASS,5k/6k clean and7k pressure with9585drops,6015.7validin-windowRPS,p95 63.278ms.170418measured/753warmupvalid200,zero HTTP errors/mismatches; all16owned resources removed. New-host6k–7k bracket is finite, no old-point/recovery replay. Independent final Reviewer /root/yt166_refinement_results_final_review_20261024 PASS, zero issues/required followups; no new-change commit/push.
- [YT-167 sustained private HTTP capacity at6k](YT-167-PRIVATE-HTTP-SUSTAINED-CAPACITY.md): Done at bounded research scope; contract/mapping PASS, failed300s offered load retains1765706valid200/34302drops,5884.83in-windowvalidRPS,p95 63.333ms. All ten30s windows have pressure;0HTTP errors/mismatches. All16owned resources/credentials removed, full raw failure retained. No clean sustained6k or duration-only cause claim. Independent final Reviewer /root/yt167_sustained_results_final_review_20261024 PASS, zero issues/required followups; no new-change commit/push.
- [YT-168 sustained private HTTP capacity at5k](YT-168-PRIVATE-HTTP-SUSTAINED-5000.md): Done at bounded profile-observation scope; contract/mapping PASS, clean300s observation retains1500001valid200/0drops/errors,4999.983in-windowvalidRPS,p95 0.894ms. All ten30s windows clean;250warmupvalid200. All16owned resources/credentials removed, full raw/history retained. Finite new-target reference, no universal/same-host bracket claim. Independent final Reviewer /root/yt168_sustained5000_results_final_review_20261024 PASS, zero issues/required followups; no new-change commit/push.
- [YT-169 thirty-minute sustained private HTTP capacity](YT-169-PRIVATE-HTTP-SUSTAINED-1800.md): Done at bounded profile-observation scope; contract/mapping/final Reviewer PASS, zero issues/required followups.9000001valid200/0drops/errors,4999.996667in-windowvalidRPS,p95 0.954ms/p99 1.593ms; all60windows clean with localized tail retained. Full raw gzip and finite observer gap preserved, all16resources/credentials removed; new changes uncommitted/unpushed.
- [YT-170 Auth/Session/PostgreSQL functional workload](YT-170-AUTH-SESSION-POSTGRES-WORKLOAD.md): Done at bounded local functional-reference scope; contract/corrected mapping/final Reviewer PASS, zero issues/required followups. JSON2000/SSR2001valid200,0drops/errors,100in-windowRPS each,p95 2.116/2.112ms. New tenant/permission/revocation/PGoutage recovery and exact-owned cleanup pass; opt-in, synthetic fixture login, no capacity/realIdP claim or new-change commit/push.
- [YT-171 Auth/Session/PostgreSQL staged load and recovery](YT-171-AUTH-CAPACITY-RECOVERY.md): Done at bounded staged-research scope; contract/mapping/final independent Reviewer PASS, zero issues/required followups. JSON500/s one Session-busy503, SSR2000/s nineteen; failed points retained, no drops/SQL errors. Both same-PID100/s recoveries and exact-owned cleanup pass; bounded profile, no capacity ceiling/commit/push.
- [YT-172 Session admission bursts and queue attribution](YT-172-SESSION-ADMISSION-BURSTS.md): Done at bounded provider-research scope; contract/corrected mapping/final independent Reviewer PASS, zero issues/required followups.648reads yield552valid/96busy;8serial recovery reads pass. Fixed limits, bounded acquisition/reply observations; historical causation unresolved. Actual cleanup verified, all initial/glue failures retained; no new-change commit/push.
- [YT-173 Full HTTP and Session provider timeline attribution](YT-173-HTTP-SESSION-TIMELINE.md): Done at bounded correlation-research scope; contract/corrected mapping/final independent Reviewer PASS, zero issues/required followups.12550requests/37638complete records,12busy allprove4active+4pending fullgate; bursts/recovery valid,cleanup18.200s. Startup/reply-path and possibleobserver contexts retained; precise historicalcause unresolved. No limits/security change or new-change commit/push.
- [YT-174 Host-side passive resource-sampling comparison](YT-174-HOST-PASSIVE-SAMPLING.md): Done at bounded comparison scope; final independent Reviewer PASS, zero issues/required followups. Same binary/limits/security; zero Docker exec, 88.54% lower recorded sample cost. Nine busy remain (eight JSON burst/one SSR observe), all prove four active/four pending; no causal performance uplift. Cleanup PASS; old proofs/failures retained, new changes uncommitted/unpushed.
- [YT-175 Session admission burst and overload policy](YT-175-SESSION-BURST-OVERLOAD-POLICY.md): Done at document-only contract scope; independent Reviewer PASS, zero issues/required followups; preserve current limits and safe failure/reconciliation semantics. Optional early HTTP admission and rate quotas remain application choices; no implementation or old proof replay.
- [YT-176 Opt-in fail-fast HTTP admission](YT-176-HTTP-ADMISSION.md): Done at bounded local Auth-reference integration scope; contract PASS, six new composition tests and feature-off check PASS. Shared early private 503 with zero added queue/retry, permit held through finalization, unchanged Session/lock/default graph. Final independent Reviewer PASS, zero issues/required followups; no old proof replay or new-change Commit/Push.
- [YT-177 Isolated HTTP admission=4 workload](YT-177-HTTP-ADMISSION-WORKLOAD.md): Done at bounded policy-behavior scope; final independent Reviewer PASS, zero issues/required followups. 12452 valid/98 checked pre-Session 503 out of12550, zero provider/SQL errors/drops; more rejections than historical baseline, no capacity improvement. Recovery and exact-owned cleanup PASS; source/Session/zero-queue/no-new-retry unchanged, old evidence preserved.
- [YT-178 Isolated HTTP admission=8 comparison](YT-178-HTTP-ADMISSION-EIGHT.md): Done at bounded comparison scope; final independent Reviewer PASS, zero issues/required followups.12475valid/74checked failures out of12549, versus cap4 12452/98 out of12550; no provider pressure, but longer burst/client tails. Same binary/caller, unchanged Session/queue/retry; recoveries and exact-owned cleanup PASS, all failures preserved.
- [YT-179 Isolated HTTP admission=16 comparison](YT-179-HTTP-ADMISSION-SIXTEEN.md): Done after final independent Reviewer PASS, zero issues/followups; 12552 selected/12540 valid/12 checked503 (7 outer, 5 provider Busy). Session4/8 and zero added queue/retry unchanged; retained cap4/8 comparison and all failures stay explicit.
- [YT-180 Example HTTP admission capacity selection guide](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection): Done at documentation-only scope after independent Reviewer PASS, zero issues/followups; application-owned opt-in 4/8/16 choices, retained failure/latency tradeoffs and per-process/Session/rate-quota boundaries. No universal default, new runtime or replayed tests.
- [YT-181–YT-184 Application follow-up evaluation and schedule](YT-181-184-APPLICATION-FOLLOWUPS.md): registered follow-ups (YT-181 Done at bounded reference scope; YT-182 Done at reference scope; YT-183 Done at reference scope; YT-184 Done at bounded cross-host/restart reference scope) for sensitive writes, optional native tracing, consumer-gated data cache and isolated Azure cross-host recovery. Audit/public output cache retain their extraction gates. The original planning assessment activated no implementation or cloud work; separate records track subsequent explicitly authorized reference implementations and verification.
- [YT-181 Minimal sensitive-write/revocation contract](YT-181-SENSITIVE-WRITE-CONTRACT.md): accepted contract; selected SQL-owned account/project write authority and generation-checked item/history commit. Separate contract/implementation acceptance, native middleware without generic AOP, existing Session/file-policy race preserved; independent contract Reviewer PASS, zero issues/followups; later implementation/proof tracked separately below.
- [YT-181 Opt-in commit-time authorization implementation](YT-181-SENSITIVE-WRITE-IMPLEMENTATION.md): Done after independent implementation Reviewer PASS, zero issues/followups; application-only feature/Settings, generation-fenced item/history and conservative v2 reopen, six new Board cases/eight total new test executions, no prior test/benchmark replay or generic AOP.
- [YT-182 Opt-in tracing and safe worker contract](YT-182-TRACING-WORKER-CONTRACT.md): Contract Reviewer PASS, zero issues/followups; Board/Admin native middleware, generated local IDs, safe worker/Dispatch propagation and separate lifecycle facts. Default-off feature/Settings, no generic AOP/exporter or old proof replay.
- [YT-126 opt-in Admin Valkey/Auth adoption](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md): real application adoption with fresh shared local policy and actual two-Admin-process qualification; Done after independent Reviewer PASS, retaining existing pre-SQL ordering limits.
- [YT-125 minimal Valkey/Auth integration](YT-125-VALKEY-AUTH-INTEGRATION.md): standalone application resolver and actual two-process qualification, Done after independent Reviewer PASS; no generic Auth feature or Admin migration.
- [YT-124 multi-instance Auth/Valkey Session contract](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md): owner-selected research follow-up, Done after independent Reviewer PASS; later Valkey two-app evidence is reused alongside historical Redis and rolling-asset proof. No async Auth bridge implementation is selected.
- Auth provider studies: [YT-116 OIDC](YT-116-OIDC-ADAPTER-STUDY.md), [YT-119 Bearer](YT-119-BEARER-STUDY.md) and [YT-120 Cookies](YT-120-COOKIE-GAP-STUDY.md) record direct/native reuse decisions without new adapters; independent Reviewer PASS for each study.
- [YT-182 Opt-in tracing/worker implementation](YT-182-TRACING-WORKER-IMPLEMENTATION.md): Done after independent implementation Reviewer PASS, zero issues/followups; default-off Board/Admin feature and Settings, scoped HTTP/worker Dispatch, separate lifecycle outcomes, seven successful new test executions, original compile/drop failures retained and old proof not replayed.

- [Planned M21 opt-in server capabilities](YT-113-123-OPT-IN-CAPABILITIES.md): YT-113–YT-123 register a staged `yetlet-auth` contract/foundation, optional Session/OIDC, rate limiting, and research-gated Bearer/Audit/Cookies/Cache/Output Cache/Tracing; [YT-113 contract research](YT-113-AUTHENTICATION-CONTRACT.md) is Done after independent Reviewer PASS; [YT-114 foundation](YT-114-IMPLEMENTATION.md) is Done after its separate independent Reviewer PASS; YT-115 is Done at Memory bridge scope and YT-116/119/120 are Done at independently reviewed research scopes; [YT-117 rate limiting](YT-117-RATE-LIMIT.md) is Done at the separately selected per-process scope after independent Reviewer PASS; [YT-118 Audit research](YT-118-AUDIT-STUDY.md) is Done after independent Reviewer PASS and [YT-121 Cache research](YT-121-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-122 Output Cache research](YT-122-OUTPUT-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-123 Tracing/Correlation research](YT-123-TRACING-CORRELATION-STUDY.md) is Done after independent Reviewer PASS, with no new core/server dependency or release guarantee.
- [Windows CLI exit-125 follow-up](WINDOWS-CLI-125-VERIFICATION.md): Cargo executable ACL diagnosis, safe stage diagnostics and source-matched native build/run/Ctrl-C cleanup with protected task-local tools.
- [Development HTTPS](DEV-HTTPS-VERIFICATION.md): existing settings, externally prepared mkcert certificates, optional Rust TLS, Vite HTTPS/WSS and native run/stop/build evidence.
- [Opt-in secret providers](SECRET-PROVIDERS.md): asynchronous named string lookup, explicit settings snapshots and constructor/State injection with replaceable providers.
- [Settings from SecretProvider](SETTINGS-SECRET-PROVIDER.md): opt-in startup resolution into selected protected fields, narrow module settings and fail-closed errors.
- [Shared SQLite reference](YT-094-SHARED-SQLITE.md): two Compose app replicas accessing one local business-data directory, application conflicts and restart/error behavior.
- [Doctor follow-up](YT-102-DOCTOR-VERIFICATION.md): authorized Rustup permission repair and source-matched Linux/Windows verification; no change to doctor trust policy.
- [Windows Vite pair verification](YT-102-WINDOWS-VITE-VERIFICATION.md): source-matched native Rust/Vite lifecycle checks using existing installed tools, with separate execution and review evidence.
- [Windows browser CSS HMR verification](YT-102-WINDOWS-HMR-VERIFICATION.md): actual generated Rust SSR, rendered CSS updates with retained interaction state, restoration and owned browser/server cleanup.
- [Historical Windows JS/TS update verification](YT-102-WINDOWS-JS-TS-VERIFICATION.md): pre-implementation TypeScript full-document reload, unsupported plain-JS route, retained recovery limitation and fresh-process restoration.
- [Vite integration evidence](YT-102-STATEFUL-HMR-IMPLEMENTATION.md): configured CLI run/stop/build, frontend source confinement, current Linux follow-up and historical native Windows browser evidence.
- [Rust hot-reload research](YT-102-RUST-HOT-RELOAD-RESEARCH.md): existing watch/rebuild/restart tools versus in-process replacement, current CLI integration seams and a bounded future evaluation; no implementation.
- [Rust watch/restart experiment](YT-102-RUST-WATCH-POC.md): separately authorized Linux Watchexec trial of Rust edit, compile-error recovery and owned process cleanup; no native CLI integration.
- [Windows Rust watch/restart verification](YT-102-WINDOWS-RUST-WATCH-VERIFICATION.md): source-matched WebCodex trial with compilation-error recovery, actual console keyboard Ctrl-C and owned process cleanup.
- [Integrated Rust watch](YT-102-RUST-WATCH-IMPLEMENTATION.md): settings-controlled `run`, Watchexec file events, persistent Vite and Linux/Windows lifecycle verification.
- [Historical CLI generator compatibility — 2026-10-18](YT-102-GENERATOR-COMPATIBILITY.md): source-bound service/settings/frontend verification; see the [current generator source matrix](../tools/yetlet-cli/README.md#current-generator-source-matrix) for present acceptance and required source closure.
- [Generator source-closure repair — 2026-10-22](YT-102-GENERATOR-SOURCE-CLOSURE.md): exact opt-in Config/Secrets export, current profile outputs, negative-source tests and sanitized offline Cargo verification; separate from historical deployment qualification.
- [YT-103 rolling deployment and retained assets](YT-103-ROLLING-ASSET-PLAN.md): provider comparison, application/deployment-owned retention, rollback and draining assumptions; [local qualification](YT-103-ROLLING-ASSET-VERIFICATION.md) records actual build/runtime evidence and its limits.
- ADRs [0001](adr/0001-separation-of-concerns.md), [0002](adr/0002-events-and-actions.md), [0003](adr/0003-renderers-and-loading.md), [0004](adr/0004-request-context-and-middleware.md), [0005](adr/0005-opt-in-capability-modules.md), [0006](adr/0006-handler-first-mvc-optional.md) and [0007](adr/0007-bounded-keyed-dom-and-local-graph.md): retained decisions and tradeoffs.

## M12 — Rust repository toolchain plan

[M12 repository-toolchain plan](M12-RUST-TOOLCHAIN-PLAN.md) defines the ordered YT-082–YT-086 migration from active Python automation to isolated unpublished Rust xtask commands. The [complete migration ledger](M12-PYTHON-MIGRATION.md) maps all 52 tracked Python files, three workflows including inline private-rebuild code, and transitive Rust-test helpers. [YT-082 verification](YT-082-VERIFICATION.md) records the clean source baseline, directional review PASS and documentation-only checks. YT-082 passed separate independent document review; [YT-083 foundation](YT-083-VERIFICATION.md) is Done after corrected Linux/Windows MSVC tests and a separate independent Reviewer PASS; [YT-084 port evidence](YT-084-VERIFICATION.md) records seven builders and docs/catalog checks with 128 matched files and its separate independent Reviewer PASS; YT-085 is Done after its own Reviewer PASS; YT-086 is Done at bounded migration scope after separate independent Reviewer PASS; complete runtime graph and unavailable environment qualification remain unverified. Current build/catalog commands are adopted after parity; historical acceptance coverage is preserved. YT-087 is Done at bounded unpublished Linux developer CLI preview scope after separate independent 087a/087b/087c reviews; Windows support and release approval remain outside this acceptance. YT-020 remains Blocked.

## M13 — Yetlet CLI developer preview

[YT-087 CLI plan](YT-087-CLI-PLAN.md) defines an unpublished Rust developer preview using explicit local framework dependencies until release gates are met. [YT-087a native SSR verification](YT-087A-VERIFICATION.md) records independently accepted Linux `new`/`doctor` and its original external-app HTTP smoke. [YT-087b build/dev verification](YT-087B-VERIFICATION.md) records independently accepted Linux frozen build/foreground-dev implementation and retained real Ctrl-C/PTY/HTTP and resistant-SIGINT cleanup evidence. The isolated CLI [guide](../tools/yetlet-cli/README.md) and [YT-087c verification](YT-087C-VERIFICATION.md) document final validation and the separate independent Reviewer PASS. No registry publishing, global install, remote fetch or release activation is authorized.

The [additional native Windows verification](YT-087-WINDOWS-VERIFICATION.md) retains the first Blocked attempt, followed by maintainer-authorized toolchain provisioning and **PASS for native GNU/MSVC compilation, Windows-applicable tests and intentional fail-closed CLI behavior**. Windows `new/build/dev` functionality remains unsupported; this does not extend the Linux preview's accepted feature-support scope.

The [YT-088 cross-platform CLI parity plan](YT-088-CROSS-PLATFORM-CLI-PLAN.md) addresses that missing Windows functionality using thin, statically selected filesystem and process adapters. Actual Windows GNU/MSVC positive `new/build/dev` functionality has separately passed independent 088b/088c/088d reviews, including SSR HTTP, frozen builds, real console Ctrl-C/Break and private Job cleanup. The [YT-088e closeout](YT-088E-VERIFICATION.md) records the final Linux/GNU/MSVC source-bound regression and independent Reviewer `wc_job_IPC2lyaZyAz72KlT` PASS; YT-088 is Done at bounded unpublished Linux/Windows developer CLI feature scope. This does not alter YT-087 historical acceptance or unblock YT-020.

## M15 — Independently accepted opt-in Linux OCI and Kubernetes generators

The [independently reviewed YT-089 M15 plan](YT-089-CLOUD-NATIVE-GENERATORS-PLAN.md) (design Reviewer `wc_job_9IplTI1HHkyZK9hv` PASS, [YT-089a opt-in cloud runtime](YT-089A-VERIFICATION.md) accepted by independent Reviewer `wc_job_0dvTDNAV22Vbpc5j`) proposes local-only generated Dockerfile/Compose/Kubernetes manifests, using shared CLI validation and existing Linux/Windows safe-output platform adapters. The independently accepted opt-in 089a profile provides explicit app-owned bind, local-process health endpoints and graceful termination; unpublished Yetlet Cargo path dependencies need an explicit sanitized BuildKit source context. [YT-089b accepted evidence](YT-089B-VERIFICATION.md) records independent Reviewer `wc_job_IC92mEXoQMsG4nTg` PASS after actual three-context Docker image/HTTP/stop and identical native GNU/MSVC generator output. No Docker/K8s SDK dependency, implicit image build, registry access, persistent deployment or YT-020 release qualification is accepted. Compose and K8s generator slices have separately passed native/runtime reviews; the final [YT-089e review](YT-089E-VERIFICATION.md) also passed (`wc_job_ci3bjlyeh2KHRhta`), completing M15 only at the unpublished Linux-targeted configuration-generator preview scope.

[YT-089c accepted Compose verification](YT-089C-VERIFICATION.md) records independently accepted Linux Docker Compose HTTP/SIGTERM cleanup and identical Windows GNU/MSVC generated files. [YT-089d K8s verification](YT-089D-VERIFICATION.md) records accepted Linux-targeted manifests, real isolated Ubuntu Multipass K3s Pod/Service/health checks, safe namespace/image cleanup, and Windows native generator parity. These are bounded historical tests, not permission for persistent cluster action. [YT-089e final integration candidate](YT-089E-VERIFICATION.md) records latest-source Linux/GNU/MSVC checks and preserved real Docker/Compose/K3s runtime receipts. YT-089 is Done after final independent Reviewer `wc_job_ci3bjlyeh2KHRhta` PASS, while YT-020 remains Blocked.

## M11 — historical planning and selected product/renderer delivery

The original [M11 plan](M11-PLAN.md), [product-readiness research](M11-PRODUCT-READINESS-RESEARCH.md), [GPU plan](M11-WGPU-PLAN.md) and [planning review](M11-PLANNING-REVIEW.md) were documentation-only at the YT-069 checkpoint. **Subsequent owner authorizations and independent reviewers** accepted YT-070–YT-080 at separately selected bounds. [ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) now accepts only the bounded Experimental opt-in architecture; it does not grant stable or broad browser support. See the [M11 reconciliation](YT-081-M11-CLOSEOUT.md) and [API consistency audit](YT-081-API-CONTRACT-AUDIT.md); historical planning statements retain their original date and evidence.

### Activated non-GPU implementation

The owner committed planning as `d96f7af` and [activated YT-070–YT-074](M11-P-ACTIVATION.md); M11-P implementation is committed at `f00ec597`. [Form/asset evidence](M11-P-FORM-ASSET-VERIFICATION.md) records YT-070–YT-072; YT-073 is locally complete. [YT-074 qualification](YT-074-QUALIFICATION.md) is Done at the owner-amended local scope, with retained real Chewing IME, repaired Firefox/Orca speech and final Chrome/Firefox PASS. The new owner-authorized full 62-minute mixed-load execution passed with 45,000 measured requests and zero errors; fresh independent review also passed, and the earlier synchronized-burst failure is retained. Historical repair/test records retain their original source and commit boundaries. See [desktop acceptance](YT-074-DESKTOP-ACCEPTANCE.md), [spoken transcript](YT-074-SPOKEN-AT-EVIDENCE.json), [IME screenshots](screenshots/yt074-ime/README.md), [startup](BOARD-START.md) and [handoff](M11-P-IMPLEMENTATION-HANDOFF.md). No GPU or public-release change is claimed.

### Experimental opt-in WebGPU candidate

The owner separately approved [Experimental renderer extraction](M11-G-EXPERIMENTAL-PROMOTION-DECISION.md),
without a performance guarantee. [YT-078 picking](YT-078-VERIFICATION.md) and
[YT-079 candidate extraction](YT-079-VERIFICATION.md) have individual browser
and independent-review PASS at their selected scopes. Use the
[`yetlet-wgpu` guide](../crates/yetlet-wgpu/README.md), its
[support matrix](../crates/yetlet-wgpu/SUPPORT.md), and
[YT-080 qualification](YT-080-VERIFICATION.md) and its
[independent Reviewer PASS](YT-080-INDEPENDENT-REVIEW.md). YT-075–YT-080 are
complete at their declared scopes; [YT-081](YT-081-M11-CLOSEOUT.md) reconciles the final evidence, while publication remains excluded.
The candidate is CPU-only by default, explicitly browser-enabled, unpublished
and outside the original nine
release candidates. Future 3D is not implemented or guaranteed.

### M11 selected-scope reconciliation and API contract review

[YT-081 M11 closeout](YT-081-M11-CLOSEOUT.md) reconciles
the complete selected SSR-first product and Experimental GPU lines,
without upgrading their browser/host/production limitations into
general support. The separate
[cross-crate API audit](YT-081-API-CONTRACT-AUDIT.md) inventories
handler-first setup, domain errors, sync Memory versus async Redis
sessions, local/HTTP commands, lifecycle/disposal and opt-in GPU
features. The owner also selected a narrow [Experimental GPU API
refinement](YT-081-WGPU-API-REFINEMENT.md), which intentionally changes
CPU validation errors and the retirement return type. It does **not**
promote a Stable Candidate, publish a package or activate YT-020.
The changed source passed its [own independent reviewer](YT-081-WGPU-API-INDEPENDENT-REVIEW.md)
with zero issues and required followups. YT-081 is Done only at the
selected documented scope.

## Production-oriented reference application

The separate [browser-only task starter](YT-104-BROWSER-STARTER.md) exercises
YT-104 with settings-selected CLI creation/build/run/stop and static Rust/WASM
output. It requires JavaScript/WASM and serves without a native Yetlet backend;
SSR/island defaults remain unchanged. Its qualification record distinguishes
Linux browser/static-host evidence from native Windows checks.

The [SSR administration reference](../examples/admin-dashboard/README.md) closes
YT-094 at the selected reference-application scope: CLI-created
Vite/Tailwind/settings composition over the persisted Lab Operations Board, with
native no-JS forms and responsive screens. The [closeout matrix](YT-094-CLOSEOUT.md)
links existing evidence and accepted limitations; this is not production or
public-release qualification. The application guide retains the first-slice results.
The later [Rust/WASM triage slice](YT-094-WASM-TRIAGE.md) adds local multi-draft
interaction and explicit HTTP saves; [API observations](YT-094-API-OBSERVATIONS.md)
record the YT-105 follow-up. The first
[form-value/event comparison](YT-105-FORM-BINDING-RESEARCH.md) recommends existing
native-draft and graph-draft patterns before adding a public helper, with
independent Reviewer PASS. The separate
[state-read/initial-render study](YT-105-STATE-READ-INITIAL-RENDER-RESEARCH.md)
proposes a weak-handle snapshot and explicit application rendering for later
selection, with independent Reviewer PASS. The
[HTTP-feedback comparison](YT-105-HTTP-FEEDBACK-RESEARCH.md) retains existing
body-token CSRF and recommends application-owned field feedback before a shared
error API, with independent Reviewer PASS and zero issues/required followups.
YT-105 is Done at research-only scope. The separately authorized
[weak-handle snapshot slice](YT-105-GRAPH-SNAPSHOT.md) implements only that read
method with owner semantics; native tests, Wasm/rustdoc checks and independent
Reviewer PASS are recorded there. Its 2026-10-20 follow-up removes Admin/Todo
state mirrors and records native/Wasm/Chrome adoption checks and independent
consumer Reviewer PASS. Initial rendering, browser Tasks and other proposals remain
separate.
Packaging is covered by YT-104 above.
The separate [YT-106 package interoperability study](YT-106-FRONTEND-INTEROP-RESEARCH.md)
recommends app-owned TS/wasm-bindgen boundaries, Chart.js as the first proposed
trial and CodeMirror only after CSP feasibility. It compares Web Components
and React roots, records package cost limits and leaves implementation separate;
independent Reviewer PASS closes YT-106 at research-only scope.
The separately selected [Admin Chart.js trial](YT-106-CHART-VERIFICATION.md)
implements loaded draft-status bars with a permanent native summary and the
existing Vite/WASM/CLI. Independent implementation Reviewer PASS completes that
bounded trial; CodeMirror remains conditional on CSP feasibility.
The [CodeMirror CSP follow-up](YT-106-CODEMIRROR-CSP-RESEARCH.md) now finds a
response-local style nonce feasible at source/mechanism scope, preserving
attribute restrictions and native fallback. Independent research Reviewer PASS
has zero issues or required followups; no editor or application-policy change
is implemented.
The subsequent [Admin Description editor](YT-106-CODEMIRROR-VERIFICATION.md)
implements the selected nonce/TS/Rust trial with native fallback, bounded local
editing and existing CLI delivery. Targeted native/WASM/Chrome checks and
independent implementation Reviewer PASS complete this bounded trial, with zero
issues or required followups.
The [Windows WASM follow-up](YT-094-WINDOWS-WASM-VERIFICATION.md) separates
completed native/browser checks from the returned operator-attested native
IME/keyboard/spoken-AT journeys, exit-0 images and broader open qualification.
The [Windows desktop handoff](YT-094-WINDOWS-DESKTOP-HANDOFF.md) provides the
historical mint-game setup and manual verification steps; its passed results and
final cleanup are reconciled in the follow-up rather than rerun.
The [opt-in OIDC slice](YT-094-OIDC.md) records existing settings/SecretProvider
composition and its separate real-provider qualification boundary.
The [Valkey shared-session slice](YT-094-VALKEY-SESSIONS.md) records explicit
provider/settings selection and two-container login/logout, restart and outage
verification. The later [shared-SQLite slice](YT-094-SHARED-SQLITE.md) adds
separately reviewed two-app business storage on a single local volume; neither
slice qualifies production HA.


The standalone [Issue Tracker starter](../examples/production-starter/README.md) exercises
YT-092 with native SSR forms, a command JSON API, SQLite restart persistence,
existing settings/CLI, health/readiness, correlated JSON logs and graceful shutdown.
Its [verification record](YT-092-STARTER-VERIFICATION.md) separates native platform
evidence from browser/hosting qualification. This opt-in reference application is
independent of the historical twelve-member sample catalog and does not change
framework defaults or generator source contracts.

The standalone [local signing reference](../examples/signing/README.md) exercises
YT-093's opt-in [provider contract](YT-093-SIGNING-CONTRACT.md), multiple named
keys/certificates, public-only verification and confirmed key CLI rotation with
automatic profile settings updates. Its [verification record](YT-093-SIGNING-VERIFICATION.md)
separates local, native Windows and disposable Azure experiment evidence.

The [YT-095 composition comparison](YT-095-COMPOSITION-RESEARCH.md) inspects
Starter, signing, Admin and existing secret/settings consumers. It recommends
constructors and native State/FromRef, with explicit generic, trait-object and
finite-enum boundaries; no new container, provider contract or implementation
is selected. Independent Reviewer PASS has zero issues or required followups;
YT-095 is Done at research-only scope.

## Library guides

The [YT-102 developer workflow audit](YT-102-DX-AUDIT.md) reconciles current
CLI/settings/tool responsibilities and retained platform evidence. Bounded
Ubuntu measurements identify fixed process-completion waiting and onboarding
costs; the report proposes minimal improvements and a DX extension boundary
without changing implementation. Independent Reviewer PASS has zero issues or
required followups; YT-102 is Done at research-only scope.
The subsequent [Linux completion-latency slice](YT-102-COMPLETION-LATENCY.md)
implements one separately selected improvement with targeted process/CLI tests
and bounded warm-build/PTY observations. Independent implementation Reviewer
PASS has zero issues or required followups.

These nine unpublished candidate pages describe the implemented APIs and their practical limits. Use the checkout's Rust 1.99 toolchain; native SSR needs no browser build.

| Area | Package guide |
|---|---|
| Shared View and binding vocabulary | [yetlet-core](../crates/yetlet-core/README.md) |
| Native escaped HTML | [yetlet-html](../crates/yetlet-html/README.md) |
| Browser subtree mount | [yetlet-dom](../crates/yetlet-dom/README.md) |
| Optional keyed children/listeners | [yetlet-dom-keyed](../crates/yetlet-dom-keyed/README.md) |
| Optional owned local update graph | [yetlet-reactive](../crates/yetlet-reactive/README.md) |
| Explicit typed actions | [yetlet-actions](../crates/yetlet-actions/README.md) |
| Native handlers, routing and Page responses | [yetlet-server-axum](../crates/yetlet-server-axum/README.md) |
| Optional request correlation | [yetlet-request-id](../crates/yetlet-request-id/README.md) |
| Optional Memory Session | [yetlet-session](../crates/yetlet-session/README.md) |

The isolated [Redis provider](../crates/yetlet-session-redis/README.md) is outside the nine candidates and remains nonpublishable. Its bounded qualification does not confer durable or production Session guarantees.

## Completed research prototypes

All five authorized studies are Done at their recorded bounded scopes. Their fixtures are outside the twelve-application catalog and do not establish supported core APIs, public release qualification or a benchmark winner. The linked records retain actual commands, raw observations, source identities and unverified environment coverage.

| Study | Record and bounded decision | Existing fixture |
|---|---|---|
| Opt-in HTTP deadlines | [YT-029](YT-029-HTTP-DEADLINE-RESEARCH.md): retain explicit app-owned middleware; general provider/core promotion remains deferred | [Middleware Lab](../examples/middleware-lab/README.md) |
| Incremental SSR | [YT-021](YT-021-STREAMING-SSR-RESEARCH.md): retain the full-versus-streaming comparison; reject core streaming/suspense promotion | [Streaming SSR research](../examples/streaming-ssr-research/README.md) |
| Custom typed transport | [YT-024](YT-024-CUSTOM-TRANSPORT-RESEARCH.md): adopt one explicit app-owned TCP adapter pattern; reject a core transport framework | [Custom action transport](../examples/custom-action-transport/README.md) |
| Handler/capture descriptors | [YT-022](YT-022-RESUMABILITY-RESEARCH.md): retain finite versioned handlers/numeric captures; reject generic/core resumability | [Resumability research](../examples/resumability-research/README.md) |
| Independent scene rendering | [YT-023](YT-023-WEBGPU-RESEARCH.md): retain opt-in native wgpu/Vulkan draw/readback and frame/allocation observations; reject core/browser-portability promotion | [Scene research](../examples/webgpu-scene-research/README.md) |

Generic hydration/resumability, a template macro, dependency-tracked signals/forms and a stable general-purpose GPU renderer remain unselected. The separately approved Experimental opt-in candidate is documented above and does not change those stable/core boundaries. Default SSR, local browser events, typed remote boundaries and the existing ADR decisions remain intact.

## Qualification and Private preparation

[Qualification matrix](QUALIFICATION-MATRIX.md), [provider operations](PROVIDER-OPERATIONS.md) and [threat model](THREAT-MODEL.md) distinguish implemented behavior, retained evidence and unverified coverage. YT-030, YT-065, YT-066 and YT-067 retain their established bounded Done status; YT-020 remains Blocked.

The [Private preparation](YT-020-PRIVATE-RELEASE-PREP.md), [integrity preparation](YT-020-INTEGRITY-PREPARATION.md), [hosted rebuilding](YT-020-HOSTED-REBUILD.md), [trust policy](YT-020-PRIVATE-TRUST-POLICY.md), [signing verification](YT-020-PRIVATE-SIGNING-VERIFICATION.md) and [supply-chain design](RELEASE-SUPPLY-CHAIN.md) preserve their exact evidence identities. Signed source `8e557e760407a3d4b3f5a8b8f833748946be1387` differs from policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and subsequent changes; historical signatures do not attest changed package contents. Formal release activation remains open.

## Documentation delivery status — YT-068

The entry refresh, nine package guides and current/history navigation are reconciled after the five completed research studies. T08 has reconciled the [final Private source/package boundary](YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12). YT-068 is **Done** at documentation scope; the independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate. YT-020 remains **Blocked**. At that pre-M11 checkpoint, all 68 backlog items were accounted for: 67 Done, one Blocked and zero In progress or Deferred, each at its stated scope. M11 adds separately filed planning/implementation proposals above; consult Backlog/Kanban for current counts. Established YT-030/YT-065/YT-066/YT-067 completion limits and optional unverified IME, speech, WebKit/mobile and zoom coverage remain unchanged.

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

The metadata contract is deliberately only title + language. Charset/viewport stay fixed and CSP remains server response policy; Yetlet does not add an arbitrary head manager or DSL. See [Compatibility Policy](COMPATIBILITY.md) and [M10c plan](M10C-PLAN.md).

`configure`/`try_configure` execute ordinary module setup functions once at application construction. Providers compose native scoped middleware and typed request/application state. The Todo server explicitly selects `yetlet-request-id`; the M8c cart explicitly selects `yetlet-session` only for `/cart`. A minimal app depends on neither. See [Extensions](EXTENSIONS.md), [complete M6 evidence](M6-VERIFICATION.md) and [M8c evidence](M8C-VERIFICATION.md).

A runnable provider-free example is available with `cargo run -p yetlet-server-axum --example minimal`.

#### Run the examples

Requirements: Rust 1.99 or newer; `rust-toolchain.toml` pins development to Rust/Cargo 1.99.0. Building browser samples additionally needs the WASM target and wasm-bindgen 0.2.129, but the adopted Rust `cargo xtask build` commands need no Python. The Rust verification/qualification candidate is recorded in [YT-086 verification](YT-086-VERIFICATION.md); Firefox verification also requires Firefox and geckodriver. Native-only applications need no browser toolchain. The maintainer selected this unified baseline before the first experimental release. Historical Rust 1.88, Chrome 155 and Firefox 157 results retain their recorded sources; [integrity preparation](YT-020-INTEGRITY-PREPARATION.md) and [hosted rebuilding](YT-020-HOSTED-REBUILD.md) record current checks, the completed protected-check migration and the unsigned Private preparation boundary.

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

It listens on `http://127.0.0.1:3003` by default, with a script-free SSR `/` and a safely escaped `/hello/{name}` route. See [M9a evidence](M9A-VERIFICATION.md); independent reviewer `wc_job_Htk-Quo_aFExMJs3` PASS.

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

It listens on `http://127.0.0.1:3004` by default; inspect `/outside`, `/lab/plain`, `/lab/guarded`, `/lab/stream` and `/lab/broken-stream` for global/group/method-specific middleware, reverse unwind, 404/405 and streaming failure semantics. See [M9b evidence](M9B-VERIFICATION.md); independent reviewer `wc_job_UnKflDaA8ZQHXCce` PASS.

The optional, **stateless** SSR Forms Validation sample demonstrates accessible field errors and no-side-effect PRG without JavaScript or Session:

```bash
cargo run -p yetlet-forms-validation
```

It listens on `http://127.0.0.1:3005` by default. Routes are `/form`, POST `/form/submit`, and `/form/success`. A valid POST redirects with HTTP 303, but **stores nothing**; this is not a claim of credentialed CSRF defense. See [M9c evidence](M9C-VERIFICATION.md); final independent reviewer `wc_job_lGtk_ZIoAvK4F6PW` PASS.

The native File Server passed M9d independent acceptance review:

```bash
cargo run -p yetlet-file-server
```

It requires loopback (default port 3006), exposes a fixed static fixture and token-gated uploads/downloads, and stores only bounded temporary text files. See the [File Server README](../examples/file-server/README.md) for the private local development token and curl workflow, and [M9d evidence](M9D-VERIFICATION.md) for verification status. It is not a public file-hosting or identity service.

The SQL-first SQLite CRUD sample passed M9e independent repeat review:

```bash
cargo run -p yetlet-sqlite-crud
```

It defaults to loopback port 3007 and `target/yetlet-sqlite-crud.sqlite3`. It uses application-local direct `rusqlite`, explicit SQL migrations/transactions and bounded `spawn_blocking`; no Yetlet ORM or mandatory data layer is introduced. Reviewer `wc_job_Q3pBfUqPYGYNSIrD` found and prompted a fail-closed schema-column repair; repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS. See the [comparison](M9E-DATA-ACCESS-COMPARISON.md) and [M9e evidence](M9E-VERIFICATION.md).

The native SSE Notifications sample passed M9f independent repeat review:

```bash
cargo run -p yetlet-sse-notifications
```

It listens on loopback port 3008 by default. `/events` uses bounded replay, heartbeat comments and `Last-Event-ID`; `/notifications` is a loopback command-oriented JSON publisher. No browser runtime, WebSocket or framework-global event circuit is introduced. First reviewer `wc_job_LF-i2IsQxFXO-obd` found control-character validation occurred after trim; raw-input validation fixed it and repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS. See [M9f evidence](M9F-VERIFICATION.md).

The isolated Multi-Island Dashboard passed M9g final independent review:

```bash
cargo xtask build multi-island-dashboard && cargo run -p yetlet-multi-island-dashboard
```

It listens on loopback port 3009 and keeps generated Counter/Filter bundles under `target/multi-island-dashboard-dist/`, outside production `dist/`. Complete SSR remains when either bundle or the whole manifest is missing/invalid. Final reviewer `wc_job_OMstvGrtDiKZ5p6C` returned PASS after retained Chrome/Firefox fallback evidence was supplied. See [M9g evidence](M9G-VERIFICATION.md).

The loopback-only Auth Notes fixture passed M9h security-focused review:

```bash
cargo run -p yetlet-auth-notes
```

It generates synthetic Alice/Bob login codes in the local terminal, uses opt-in Session rotation/CSRF, and rechecks note ownership on every request. Reviewer `wc_job_EgSLng_VafIvs3ju` returned PASS. It is not a password, OIDC, MFA or production identity provider. See [M9h evidence](M9H-VERIFICATION.md).

The larger Modular Shop reference application passed M9i repeat independent review:

```bash
cargo run -p yetlet-modular-shop
```

It listens on loopback port 3008, composes synthetic identity, SQL catalog, Session cart and transactional orders through explicit Rust modules, and can disable Orders with `YETLET_SHOP_DISABLE_ORDERS=1`. First reviewer `wc_job_a_WnhFSftzNoYYgf` found and prompted smoke readiness cleanup; repeat reviewer `wc_job_zbWS7OSbUtaKzYcn` returned PASS. It is not a payment or production commerce system. See [M9i evidence](M9I-VERIFICATION.md).

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

Do not mix debug and release asset builds. Results are recorded in [M5 verification](M5-VERIFICATION.md); size estimates refer to local gzip artifacts, not actual transfer bytes.

Browser smoke tests use isolated headless profiles and temporary localhost servers, stopping only their own processes. M2 checks shared DOM/local clicks; M3 covers lazy islands and fail-closed state; M4 covers the explicit typed HTTP boundary. M7i additionally checks stable keyed row/list identity, explicit graph/listener disposal, readable detached DOM and duplicate-free remount in Chrome and Firefox. See [M7i verification](M7I-VERIFICATION.md).

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

**Known limitations:** Only the production Todo local list uses the promoted keyed graph; remote Todo results and Search result updates retain bounded subtree replacement. The fixed keyed/static-slot authoring shape, text-input/focus policy and conditional slot remain experimental. Nested/general component trees, async effects, IME/mobile/WebKit coverage and generic hydration/resumability remain absent. The Todo API and Session cart are process-local/in-memory; the Session proof is anonymous, single-node and not production auth/CSRF qualification. Durable persistence, stable API, full IME/a11y matrix and cross-framework benchmarks remain absent. See [M7i verification](M7I-VERIFICATION.md), [M8c verification](M8C-VERIFICATION.md) and [release readiness](RELEASE-READINESS.md).

#### Governance, security, and license

Yetlet is a maintainer-directed project. External pull requests are not accepted; repository settings restrict pull-request creation to collaborators so maintainers can still use PRs and required CI internally. Reproducible bugs, compatibility reports, documentation feedback, and focused design proposals are welcome through Issues. Security reports must follow [SECURITY.md](../SECURITY.md) and must not be posted publicly. See [CONTRIBUTING.md](../CONTRIBUTING.md) for the complete policy.

Yetlet is dual-licensed under your choice of [MIT](../LICENSE-MIT) or [Apache License 2.0](../LICENSE-APACHE). Workspace packages carry the SPDX expression `MIT OR Apache-2.0`; example and application packages remain `publish = false`. The license permits forks and independent modifications but does not require the upstream project to accept external code contributions.

#### Qualification status

[YT-020 Private release preparation](YT-020-PRIVATE-RELEASE-PREP.md) selects experimental `0.1.0` and the original nine library candidates. The prepared policy preserves API behavior and the selected Rust 1.99 minimum in `0.1.x` patches, with best-effort maintenance and no SLA. [Integrity preparation](YT-020-INTEGRITY-PREPARATION.md) retains earlier Ubuntu/Windows experiments; later [hosted rebuilding](YT-020-HOSTED-REBUILD.md) compares the same immutable source on separately operated builders. [Private trust policy](YT-020-PRIVATE-TRUST-POLICY.md) selects Cosign maintainer public-key preparation and future crates.io distribution. [Actual Private signing](YT-020-PRIVATE-SIGNING-VERIFICATION.md) records a dedicated key/fingerprint, three signatures and separate public-only verification. Formal OIDC, certificate/chain, transparency/inclusion and builder/source guarantees remain open and unchanged; registry activation is unverified. No package has been published, no public support policy is activated, and the isolated Redis provider remains nonpublishable.

M10 release-supply-chain design/preflight is independently reviewed. Four local Firefox release/keyboard suites, real local Session/SQLite operations and the YT-066a three-instance Docker/Memory proof passed with owned resource cleanup. The scheduled/manual qualification workflow was merged and hosted run `38043565238` passed Linux, Windows MSVC, macOS ARM64 and Linux Firefox lanes; separate local Windows GNU and offline Docker evidence also passed. YT-065 is complete at that bounded automated scope. Real IME, spoken screen-reader output, WebKit/mobile and zoom remain explicitly Not verified optional coverage. YT-066a demonstrates stateless multi-instance serving and current Memory Session limitations only; its final independent review passed, while the separately opt-in Redis provider and [three-application fixture](YT-066B3-VERIFICATION.md) now have passing bounded single-host/no-persistence execution, cleanup and independent b3 review; YT-066 is Done at single-host Docker multi-instance and selected Redis-provider scope. Production TLS/ACL policy, backups, business RPO/RTO and operational ownership are application responsibilities; broader deployment durability/recovery remains unverified. No result establishes a supported-device range, accessibility conformance, production deployment or publication approval.

#### Documentation

- [Opt-in typed application configuration](CONFIGURATION.md)
- [YT-090 configuration scope and verification](YT-090-VERIFICATION.md)

- [Architecture and module boundaries](ARCHITECTURE.md)
- [Full MVP specification](MVP.md)
- [Roadmap](ROADMAP.md)
- [Backlog](BACKLOG.md)
- [M2 verification record](M2-VERIFICATION.md)
- [M3 verification record](M3-VERIFICATION.md)
- [M4 typed actions/API verification](M4-VERIFICATION.md)
- [M5 acceptance and measured baseline](M5-VERIFICATION.md)
- [M6a App Builder verification](M6A-VERIFICATION.md)
- [Complete M6 verification and performance comparison](M6-VERIFICATION.md)
- [M7a render-strategy benchmark](M7A-VERIFICATION.md)
- [M7b keyed/static-slot authoring prototype](M7B-VERIFICATION.md)
- [M7c explicitly owned local-reactivity prototype](M7C-VERIFICATION.md)
- [M7d keyed lifecycle and form-state verification](M7D-VERIFICATION.md)
- [M7e renderer regression and no-decision evidence freeze](M7E-VERIFICATION.md)
- [M7f renderer-neutral binding contracts](M7F-VERIFICATION.md)
- [M7g optional synchronous local graph](M7G-VERIFICATION.md)
- [M7h optional keyed DOM ownership](M7H-VERIFICATION.md)
- [M7i production Todo keyed-graph adoption](M7I-VERIFICATION.md)
- [M7j final architecture and evidence freeze](M7J-VERIFICATION.md)
- [Sample application contract and catalog](SAMPLES.md)
- [M8a sample contract verification](M8A-VERIFICATION.md)
- [M8b API-only REST verification](M8B-VERIFICATION.md)
- [M8c Session and SSR cart verification](M8C-VERIFICATION.md)
- [M8d sample developer-experience audit](M8D-VERIFICATION.md)
- [M8e final sample architecture and evidence freeze](M8E-VERIFICATION.md)
- [M9 proposed sample expansion and API contract qualification plan](M9-PLAN.md)
- [M9 planning reviewer gate](M9-PLANNING-VERIFICATION.md)
- [M9e SQLite data-access comparison](M9E-DATA-ACCESS-COMPARISON.md)
- [M9e SQL-first SQLite CRUD verification](M9E-VERIFICATION.md)
- [M9f bounded SSE Notifications verification](M9F-VERIFICATION.md)
- [M9g independent multi-island dashboard verification](M9G-VERIFICATION.md)
- [M9h synthetic authentication and private-notes verification](M9H-VERIFICATION.md)
- [M9i modular shop integration verification](M9I-VERIFICATION.md)
- [Public API contract classification](API-CONTRACTS.md)
- [M9j final compatibility and evidence freeze](M9J-VERIFICATION.md)
- [Post-M9 CI and security review](CI-SECURITY-REVIEW.md)
- [Pre-public compatibility policy](COMPATIBILITY.md)
- [M10c compatibility/document metadata plan](M10C-PLAN.md)
- [M10c planning reviewer evidence](M10C-PLANNING-VERIFICATION.md)
- [M10c implementation verification](M10C-VERIFICATION.md)
- [M10d production edge/authentication threat-model plan](M10D-PLAN.md)
- [M10d planning reviewer evidence](M10D-PLANNING-VERIFICATION.md)
- [Production edge and authentication threat model](THREAT-MODEL.md)
- [M10d threat-model verification](M10D-VERIFICATION.md)
- [M10a license/security-policy verification](M10A-VERIFICATION.md)
- [M10b repository-governance verification](M10B-VERIFICATION.md)
- [Remaining M10 qualification plan](M10-REMAINING-PLAN.md)
- [Release supply-chain design and preflight](RELEASE-SUPPLY-CHAIN.md)
- [M10e release verification](M10E-VERIFICATION.md)
- [Platform/browser/accessibility matrix](QUALIFICATION-MATRIX.md)
- [M10f automated platform/browser qualification evidence](M10F-VERIFICATION.md)
- [Provider operations and local limitations](PROVIDER-OPERATIONS.md)
- [M10g local operational evidence](M10G-VERIFICATION.md)
- [Optional future external-assessment template](EXTERNAL-SECURITY-REVIEW.md)
- [M10h internal security-scan status](M10H-VERIFICATION.md)
- [M10 CI, Windows and Docker delivery verification](M10-DELIVERY-VERIFICATION.md)
- [Contribution policy](../CONTRIBUTING.md)
- [Security policy](../SECURITY.md)
- [Module composition and optional providers](EXTENSIONS.md)
- [Prototype release readiness and blockers](RELEASE-READINESS.md)
- [Private experimental 0.1.0 preparation and remaining activation gates](YT-020-PRIVATE-RELEASE-PREP.md)
- [Private public-key trust and future crates.io policy](YT-020-PRIVATE-TRUST-POLICY.md)
- [Actual Private maintainer-key signing and independent verification](YT-020-PRIVATE-SIGNING-VERIFICATION.md)
- [ADR 0001](adr/0001-separation-of-concerns.md)
- [ADR 0002](adr/0002-events-and-actions.md)
- [ADR 0003](adr/0003-renderers-and-loading.md)
- [ADR 0004 — RequestContext and middleware](adr/0004-request-context-and-middleware.md)
- [ADR 0005 — Optional capability modules](adr/0005-opt-in-capability-modules.md)
- [ADR 0006 — Handler-first and optional MVC](adr/0006-handler-first-mvc-optional.md)
- [ADR 0007 — Bounded keyed DOM and local graph](adr/0007-bounded-keyed-dom-and-local-graph.md)

M0–M9 are locally complete at prototype scope. M8 freezes the runnable sample contract; M9 adds twelve independently reviewed applications, a 170-export compatibility classification and complete native/browser/provider evidence freeze. The post-M9 CI/security round patches `RUSTSEC-2026-0009`, raises and verifies the Rust 1.88 MSRV, and hardens hosted CI. GitHub Actions run `38024582602` completed successfully for commit `86c30a42114ca74f4197f0389707861308a844a7`, including both the Rust 1.88 job and the stable native/samples/Chrome job. M10 adds the bounded document contract, threat model, dual-license/security policy and maintainer-only governance configuration. Repeat reviewers and green CI do not make Yetlet SemVer-stable or production-ready. SQL access remains application opt-in and Yetlet has no built-in ORM or mandatory DB layer. Public-release readiness (`YT-020`) is **Blocked** for actual immutable-source/registry, trusted release integrity, public reporting/scanner activation and concrete maintainer approval. Private preparation selects experimental `0.1.0`, the original nine libraries and bounded maintenance/qualification policy; completed framework documentation/qualification is not an open blocker. Application deployment policy, backups, business RPO/RTO and on-call ownership are not framework qualification prerequisites. The Codex Security Standard source scan reviewed 227/227 regular tracked files and found one Low issue; its tiny local correction passed comprehensive regressions and independent reviewer. No external audit, production certification or post-patch complete rescan is claimed. Paid external assessment is not a current requirement. Current code is not a production release.

</details>

- [YT-183 Optional catalog-data cache contract](YT-183-CATALOG-CACHE-CONTRACT.md): Done at application-reference scope after independent Reviewer PASS; actual Shop display consumer, default-off native cache, bounded source age and commit-generation invalidation; authoritative Session/checkout preserved; zero review issues/required followups.

- [YT-183 Optional catalog-data cache implementation](YT-183-CATALOG-CACHE-IMPLEMENTATION.md): Done after independent Reviewer PASS, zero issues/followups; actual Shop display reuse, two-second source age, commit-generation invalidation and unchanged Session/checkout authority; nine new test executions PASS without old qualification replay.

- [YT-184 Azure cross-host Auth/Session and persistent restart](YT-184-AZURE-AUTH-SESSION-RESTART.md): Done at bounded reference scope after final independent Reviewer PASS0: native opt-in TLS/unit/compile proof and preallocation review pass; first cloud bootstrap fails before live Auth/restart, smallest Path fix locally checked and all owned resources/credentials removed. Limited first correction/cleanup review PASS0. Authorized generation2 reaches both apps READY/wrong-name TLS refusal, then fails the SQL observer before Auth/restart; observer corrected locally, both failed generations fully deleted. Full integration pending; maintainer explicitly authorizes generation3 with the reviewed observer fix and cumulative US$5/90min including cleanup bounds; generation3 passes private SQL/HTTPS checks then first auth form fails; missing direct-TIME ACL correction now locally verified by one new named-user Session provider regression (6oldcases filtered); local named-app/Session independent Reviewer PASS0, previous invalid probes retained, new local resources removed; first three Azure generations fully deleted; fourth passes cross-host Auth/policy/logout/app-restart then fails first Valkey recovery snapshot; new local persistent read-only readiness case and independent fourth-results/correction review PASS0; first four allocations deleted; fifth completes remaining authority recovery/VM/SQL-policy/new-epoch proof with native exit0; all five groups deleted, final Reviewer PASS0 accepts composed reference proof; no sixth authorized.

- [YT-185 Application opt-in startup options](YT-185-OPT-IN-OPTIONS.md): Done after independent Reviewer PASS0; Board/Admin and Shop typed options, five former constructor migrations, ten successful focused test executions; existing security and feature/settings boundaries preserved.

- [YT-186 Current API/documentation preparation](YT-186-DOCUMENTATION-SCOPE.md): Done at documentation-preparation scope after independent Reviewer PASS0; publication classification precedes the minimum reference/navigation/stale-guidance edits; no new runtime, sample, qualification replay or publication.

- [YT-187 Product README and DX entry](YT-187-README-DX.md): Done after independent Reviewer follow-up PASS0; root196to108lines, browser prerequisites restored, prior README/first-P2 verdict retained; planning stays in Backlog/Roadmap.

- [YT-188 Consistent integration guides](YT-188-GUIDE-TEMPLATE.md): Done after independent Reviewer PASS0; ten existing framework guides adopt the common reader template, with original29code blocks/anchors/contracts/history preserved and no runtime qualification replay.

- [YT-189 First standalone application guide](YT-189-FIRST-APP-GUIDE.md): Done after independent Reviewer PASS0; connects existing native setup/local dependencies through first run/common failures; no generated project or runtime qualification replay.

- [YT-190 Common-error troubleshooting guide](YT-190-TROUBLESHOOTING-GUIDE.md): Done after independent Reviewer PASS0; 27 source-backed symptom/cause/correction/guide rows across six groups, no error reproduction or runtime qualification replay.

- [YT-191 View authoring, opt-in lookup and CLI status](YT-191-DX-GUIDES.md): Done after independent Reviewer PASS0; three selected documentation gaps closed with existing APIs/source and historical proof preserved, no runtime qualification replay.

- [YT-192 3D physics lab minimum contract and feasibility](YT-192-3D-PHYSICS-LAB-STUDY.md): Done at documentation/source-level feasibility scope after independent Reviewer PASS0; browser/scene/fixed-step/GPU-failure contract, no implemented 3D API/sample or compile/runtime qualification.

- [YT-193 Isolated physics-lab prototype](YT-193-3D-PHYSICS-LAB-PROTOTYPE.md): Done after whole-item Reviewer PASS0; separate native SSR/opt-in Rapier/private3D example, focused actual Firefox154/Wayland/NVIDIA interaction/recovery and cleanup proof; failures retained, no general support/performance/production claim.

- [YT-194 Physics laboratory controls](YT-194-PHYSICS-LAB-CONTROLS.md): Done after independent Reviewer PASS0; firstRun opt-in start, primary Run/Pause/Reset, Advanced Step, failure-only Retry and focused corrected-clock/control proof; failures and actual default-GPU telemetry limits retained.

- [YT-195 Browser startup and WebGPU guidance](YT-195-BROWSER-STARTUP-GUIDANCE.md): Done after independent Reviewer PASS0; ordinary-browser requirements, bounded actionable failures, static/noJS help, reduced-motion local loading and general pre-Wasm contract; reusable helper/template adoption is tracked in YT-196.

- [YT-196 Common pre-Wasm shell adoption](YT-196-WASM-STARTUP-SHELL.md): Done after independent acceptance and P2 re-review PASS0; optional customizable JS/CSS, Tasks/CLI full-page and Lab SSR integration,8new browser cases plus focused helper/emission/lifecycle checks. Raw failures retained; no old qualification replay or new-change Commit/Push.

- [YT-197 Lab scenes and estimated render FPS](YT-197-PHYSICS-LAB-SCENARIOS-FPS.md): Done after independent Reviewer PASS0; bounded stack/domino presets,staged selection/Reset and estimated running draw-submission HUD,7new automated tests and7browser observations; failed option fixture retained,no benchmark/support expansion.

- [YT-198 Slope comparison](YT-198-PHYSICS-LAB-SLOPE.md): Done after independent Reviewer PASS0; fixed ramp/shape/material lanes,active-world legend and staged Reset,newRust3 and actual browser proof; existing limits/FPS retained,no benchmark/Core expansion.

</details>
