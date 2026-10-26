# M21 — Opt-in server capability roadmap (YT-113–YT-123)

**Registered:** 2026-10-22. **Planning base:** `86c7bbe89af18d3c822b154e268f85c4029f86f2` (YT-111/112 feature branch). **Registration status:** all 11 follow-ups were Todo. **Current status:** [YT-113 contract research](YT-113-AUTHENTICATION-CONTRACT.md) is Done after independent Reviewer PASS; [YT-114 foundation](YT-114-IMPLEMENTATION.md) is Done after its separate independent Reviewer PASS; YT-115 is Done at Memory bridge scope and YT-116/119/120 are Done at independently reviewed research scopes; [YT-117 rate limiting](YT-117-RATE-LIMIT.md) is Done at the separately selected per-process scope after independent Reviewer PASS; [YT-118 Audit research](YT-118-AUDIT-STUDY.md) is Done after independent Reviewer PASS; [YT-121 Cache research](YT-121-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-122 Output Cache research](YT-122-OUTPUT-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-123 Tracing/Correlation research](YT-123-TRACING-CORRELATION-STUDY.md) is Done after independent Reviewer PASS. This file is a roadmap and contract-selection proposal; implementation evidence is recorded separately, with no release claim.

## Why this batch exists

[ADR 0005](adr/0005-opt-in-capability-modules.md) already approves a pay-for-what-you-use server capability boundary. `yetlet-request-id` and bounded `yetlet-session` (synchronous memory) already exist, and an experimental separately selected asynchronous Redis Session provider has narrowly qualified evidence. [Auth Notes](../examples/auth-notes/README.md) exercises synthetic login, CSRF/session rotation, and per-resource access; the [Admin OIDC sample](YT-094-OIDC.md) uses a mature OIDC library, externally verified issuer/subject mapping and app-owned permission checks. Neither proves a reusable authentication crate today. Existing [SecretProvider](SECRET-PROVIDERS.md), [Threat Model](THREAT-MODEL.md) and Axum/Tower [extension hooks](EXTENSIONS.md) should be reused rather than recreated.

The next realistic consumers could include unrelated administration products (such as a standalone reverse-proxy/WAF **control-plane application**). They are **motivation, not a Yetlet feature dependency**: NGINX/WAF policy compilation, PostgreSQL tables, node agents, deployment/reconciliation, credentials and privileged host operations must remain outside Yetlet core and these items.

## Accepted planning priorities

| Order | Item | Status | Deliverable type | Earliest selection |
|---|---|---|---|---|
| 1 | YT-113 — Authentication contract review | Done (research only) | [Reviewed documentation-only contract](YT-113-AUTHENTICATION-CONTRACT.md), independent Reviewer PASS | Complete; YT-114 separately selected |
| 2 | YT-114 — `yetlet-auth` minimal principal and authorization seam | Done (bounded foundation) | [Owner-selected foundation](YT-114-IMPLEMENTATION.md), independent Reviewer PASS | Complete; YT-115 separately selected |
| 3 | YT-115 — Optional Session/Cookie auth bridge | Done (Memory bridge) | [Selected Memory bridge](YT-115-SESSION-AUTH.md), independent Reviewer PASS | Complete |
| 4 | YT-116 — OIDC adapter study and bounded reuse | Done (study) | Study first; optional implementation after selection | After YT-114/115 |
| 5 | YT-117 — Optional rate-limiting capability | Done (per-process memory) | [Selected per-process contract and implementation](YT-117-RATE-LIMIT.md) | Complete; independent Reviewer PASS |
| 6 | YT-118 — Generic audit event/sink study | Done (study) | [Source-backed decision](YT-118-AUDIT-STUDY.md), independent Reviewer PASS | Complete; no audit API/runtime selected |
| Later | YT-119 — Bearer API authentication adapter | Done (study) | Study first; implementation deferred | Real bearer consumer needed |
| Later | YT-120 — Cookie primitives review | Done (study) | Research only | After Session bridge identifies duplication |
| Later | YT-121 — Application data-cache provider study | Done (study) | [Source-backed study](YT-121-CACHE-STUDY.md), independent Reviewer PASS | Complete; no cache API/runtime selected |
| Later | YT-122 — Auth-aware HTTP/output-cache policy study | Done (study) | [Source-backed study](YT-122-OUTPUT-CACHE-STUDY.md), independent Reviewer PASS | Complete; no output-cache API/runtime selected |
| Later | YT-123 — Tracing/correlation integration study | Done (study) | [Source-backed study](YT-123-TRACING-CORRELATION-STUDY.md), independent Reviewer PASS | Complete; no tracing API/runtime selected |

**Order is a recommendation, not a release or automated execution queue.** YT-116, YT-117, and YT-118 do not automatically block one another. ID order does not require doing YT-119 before other independent tasks.

## Separately selected follow-up — YT-124

The original YT-113–YT-123 registration remains eleven items. The owner later
selected [YT-124 multi-instance Auth/Valkey Session contract research](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md),
Done at research scope after independent Reviewer PASS. It reuses the already
selected Valkey application/provider evidence and historical Redis/rolling
proof, defines the missing async Auth integration contract and does not select
implementation or repeat passed qualification.

The owner subsequently selected [YT-125 minimum application integration](YT-125-VALKEY-AUTH-INTEGRATION.md),
Done after focused new verification and independent Reviewer PASS. It is
separate from the original registration and does not promote a generic bridge.

The later owner-selected [YT-126 Admin adoption](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md)
is Done after focused application verification and independent
Reviewer PASS. It reuses the resolver pattern in the real Admin consumer with
explicit opt-in selection and current shared local policy. The original eleven
items, framework/provider interfaces and release gates are unchanged; strict
cross-store business-write ordering remains a separate decision.

## YT-113 — Authentication contract review

Inventory existing Auth Notes, Admin/Lab Operations OIDC, native handlers, `yetlet-session`, CSRF, `yetlet-secrets`, Axum extractors and policy checks. Produce a reviewed contract for a server-only authenticated principal: stable issuer+subject (plus scheme, and optional validated tenant when relevant), provenance/trust, request-local propagation, and the distinction between 401 unauthenticated versus 403 authenticated-but-denied. Compare scoped middleware and typed extractor APIs against idiomatic Axum/Tower; preserve unauthenticated handlers and the possibility of multiple independently selected providers. Specify safe result/errors, no SSR serialization, and revocation/freshness limits. **No implementation** in this research item. The [contract study](YT-113-AUTHENTICATION-CONTRACT.md) is Done at documentation-only research scope after independent Reviewer `/root/yt113_contract_review_20261022` PASS, zero issues/required followups. YT-114 and YT-115 were separately authorized; the latter is tracked in its [Memory bridge record](YT-115-SESSION-AUTH.md).

## YT-114 — Minimal opt-in authentication and authorization

The owner-selected [foundation implementation and evidence](YT-114-IMPLEMENTATION.md)
is Done at the bounded native scope after independent Reviewer
`/root/yt114_foundation_review_20261022` PASS, zero issues/required followups;
Session integration is outside that foundation slice; the subsequent [YT-115 Memory bridge](YT-115-SESSION-AUTH.md) is separately selected.

Implement **only after** the YT-113 contract is selected. Candidate `yetlet-auth` is a server-only opt-in crate or smaller feature as justified by that review, never a new field in `RequestContext`, default middleware, DI container, controller stack or all-schemes registry. A trusted server-side provider may install a typed `Principal` and optional/required authenticated extraction; minimal policy guard(s) should allow explicit application-owned resource/tenant authorization and fail closed by default. No roles from browser headers, user profile, email or unsigned claims; authenticated identity itself grants no business permission. Prove route isolation, forged-identity rejection, 401/403, no body consumption, unchanged SSR/anonymous samples and absence from baseline/WASM dependency graphs. Independent Reviewer PASS.

## YT-115 — Optional Session and Cookie authentication bridge

Re-use existing `yetlet-session` identity, CSRF, rotation/revocation, cookie protections and state ownership. Keep **authentication independent of session**: login verification and account policy remain application/provider responsibilities, and anonymous sessions remain valid without Auth. Preserve synchronous memory-session ergonomics; any distributed/async bridge is separately selected, not a mandatory async rewrite. Test fresh anonymous-to-authenticated session rotation, CSRF for credentialed mutation/logout, fixation/replay/retired-ID denial, expiry and revocation, shared state ordering, denied or disabled account, scoped routes and redacted responses. No new cookie storage engine or homegrown password/MFA/IdP system. Independent Reviewer PASS.

## YT-116 — Optional OIDC adapter study and selected integration

The owner-selected [study and extraction decision](YT-116-OIDC-ADAPTER-STUDY.md) is Done after independent Reviewer PASS;
no adapter implementation is selected.

Compare the accepted [Admin OIDC consumer](YT-094-OIDC.md) against the new Principal/Session boundary. Re-use maintained OIDC/OAuth client libraries; never implement discovery, PKCE, nonce or signature algorithms inside Yetlet. Decide whether application-owned configuration and direct Axum integration suffice or whether a small opt-in adapter provides demonstrable reuse. Retain exact issuer/audience/signature/expiry/nonce/state/PKCE and redirect/origin checks, callback transaction one-time use, replay denial, SecretProvider separation, only explicit issuer/subject-to-account mapping, fail-closed key changes and scoped CSP. The existing security/advisory applicability review must be revisited if dependencies change. Independent Reviewer of the **study** first; a separate owner-selected bounded implementation and real-provider verification is needed before claiming an adapter Done. No auto-provisioned roles, global auth registry or production OIDC guarantee.

## YT-117 — Opt-in request rate limiting

The owner selected the [per-process endpoint/group contract](YT-117-RATE-LIMIT.md)
before minimal implementation. Reusable Policy configuration and shared Budget
accounting are distinct; application-owned startup settings default to disabled
and omit limiter construction and request work while off. Benchmarking and live
configuration reload remain future work. The selected implementation is Done
after focused new verification and independent Reviewer PASS.

First select a narrow admission contract using existing Tower/Axum layers: route/group scoping, explicit keying and trustworthy clock/accounting, bounded memory, correct 429 response (and Retry-After only when well-defined), and cancellation/cleanup. A single-node in-memory implementation is acceptable only with its explicit per-process limits; Redis/distributed enforcement is a separate opt-in decision. Unauthenticated IP limits must not trust arbitrary client-supplied `X-Forwarded-For`; authenticated keys come only from a verified Principal and must never expose secret identifiers. Test legitimate traffic, burst/expiry, concurrent keys, scope isolation, memory bounds and fail-closed invalid configuration. This does not replace edge DDoS/WAF controls or promise cluster-wide enforcement. Independent Reviewer PASS before Done.

## YT-118 — Generic audit event and sink feasibility (research only)

Compare at least Auth Notes plus Admin operation/audit behavior before selecting an API. Consider an allowlisted actor/context/outcome/time/correlation event structure with explicit application-created resource/action fields, independent sink composition, bounded delivery failure and redaction rules. Persistence/retention, PostgreSQL schema, tamper-evident storage, async durability and business audit content stay application/operator owned. `yetlet-audit` is a **candidate name, not an authorized crate**. Only design research and independent review in this item. The owner-selected [study](YT-118-AUDIT-STUDY.md) compares Auth Notes and Admin/Board, with supporting SQLite history and diagnostic tracing evidence; it recommends retaining application-owned composition without extracting a new API. Independent Reviewer `/root/yt118_audit_study_independent_review_20261022` PASS has zero issues or required followups; Done at research scope.

## YT-119 — Bearer authentication adapter feasibility (research first)

The owner-selected [study and extraction decision](YT-119-BEARER-STUDY.md) is Done after independent Reviewer PASS;
no adapter implementation is selected.

Requires a real typed API/bearer-token consumer beyond browser Session/OIDC. Compare opaque-token validation with JWT/JWKS using maintained libraries; explicit key/audience/issuer/clock-skew/revocation and token provenance rules. No insecure fallback to cookie or principal, no token in logs, and no homemade JWT signing/validation. Determine whether an adapter is needed; implementation requires a separately selected slice, independent review and real negative cases. Not a prerequisite for YT-114/115/116.

## YT-120 — Cookie primitives gap review (research only)

The owner-selected [study and extraction decision](YT-120-COOKIE-GAP-STUDY.md) is Done after independent Reviewer PASS;
no adapter implementation is selected.

Compare existing `cookie` handling in `yetlet-session` and native Axum extraction with real second consumer glue before proposing `yetlet-cookies`. Cover strict duplicate/invalid cookie parsing, Set-Cookie protection flags, response lifecycle, signed/encrypted-cookie key rotation if truly required, and CSRF separation. Avoid duplicating Session identity/revocation or introducing an ambient cookie jar. No cookie crate extraction unless the review demonstrates a recurring need and selects a bounded contract.

## YT-121 — Application data-cache provider study (research only)

Compare two consumers before selecting a typed application-key/value/cache-store seam. Distinguish data caches from Session state, business database persistence and HTTP response caching; bound key namespace, TTL, memory/capacity, invalidation/failure and privacy. Existing generic Rust caches/Redis clients remain candidate dependencies of application-selected adapters, not Yetlet core. No `yetlet-cache` or Redis mandatory dependency/implementation authorized through this study.

The owner-selected [YT-121 study](YT-121-CACHE-STUDY.md) compares Starter issues and Shop products plus supporting SQL reads/asset snapshots. None demonstrates duplicate application data caches; retain application-owned reads and native cache options, separate authoritative Session/stock/policy from disposable values, and defer output caching to YT-122. Research is Done after independent Reviewer PASS; no cache API/runtime is selected.

## YT-122 — Private-safe output-cache policy study (research only)

Model public versus authenticated SSR/API responses, `Cache-Control`/`Vary`/`Set-Cookie`, authorization/resource/tenant changes, CSRF, error responses, content hashes and per-user isolation. Default to **not caching** private/credentialed results; never infer safety from route names or a bare authenticated flag. Study the requirements before considering `yetlet-output-cache`. Fail closed if a privacy-safe key and invalidation model cannot be demonstrated; no implementation selected here.

The owner-selected [YT-122 study](YT-122-OUTPUT-CACHE-STUDY.md) compares actual public/static and private SSR/API responses, preserves no-store and lifecycle/tenant isolation, and defines public eligibility plus commit/build/multi-instance invalidation boundaries. Retain app-owned header/static delivery; no repeated generic response-cache implementation justifies extraction. Research is Done after independent Reviewer PASS; no cache API/runtime is selected.

## YT-123 — Tracing/correlation adapter study (research only)

Compare existing `yetlet-request-id`, native Rust `tracing` and Tower/Axum observability for SSR/API/SSE and background job contexts. Request ID is not an authenticated actor, distributed trace ID, secret or audit event. Study bounded span propagation, redaction and operation correlation only if real consumers duplicate boilerplate; leave exporter/sdk/cloud telemetry to applications. No new default span, tracing subscriber, background agent or crate is selected.

The owner-selected [YT-123 study](YT-123-TRACING-CORRELATION-STUDY.md) compares actual RequestIds/SSR/API/Starter diagnostics and bounded task/stream source with native tracing/Tower. Retain local correlation and application-owned field/trust/lifetime policy; no repeated generic propagation adapter or durable job consumer is demonstrated. Research is Done after independent Reviewer PASS; no tracing API/runtime/subscriber/exporter is selected.

## Cross-cutting gates

- **No auth, session, rate limiting, cookies, cache, audit or tracing enabled by default.** Tiny native SSR remains functional; no optional HTTP dependency may leak into browser WASM or serializable View state.
- Keep existing Axum/Tower `Layer`/`Service`, native extractors, scoped installation, typed `RequestContext` extensions and ordinary `App::configure`. No separate middleware engine or service locator.
- Auth vs authorization vs Session vs CSRF are distinct: verified identity never implies business permission; explicit denial, local revocation/freshness and 401/403 semantics need consumer proof.
- Keep platform/key secrets in protected server providers. Never expose passwords, bearer tokens, private headers or provider handles through errors, logs, SSR or hydration. A deliberately rendered, session-bound CSRF token in a protected server-generated native form is allowed and required for existing no-JavaScript credentialed mutations; validate it on POST and never log it or automatically copy it into generic hydration/browser state. Preserve scoped cache-control and cookie protections.
- Each implementation item requires its own owner-selected contract and focused native/browser boundary/negative/security verification, plus independent Reviewer PASS. Mark **Done** only at demonstrated scope, not at registration. Avoid automatic stable API, Windows/cluster/production promises.
- Preserve accepted YT-094 OIDC, YT-066 Session/Redis and YT-111/112 DX evidence. Do not silently widen them or alter the original YT-096 immutable candidate, YT-020 release gate, publication, registry inventory or global installation.
- This M21 roadmap is independent of any reverse-proxy/WAF product. That application's CRUD, configuration compiler, PostgreSQL schema, node agent, rollout or secrets management **must not be built into Yetlet**.

## Registration review (2026-10-22)

Independent read-only Reviewer Job `wc_job_NoeHIpcvlO4v706D` initially returned
FAIL for one P2 contradiction: a blanket SSR CSRF-token prohibition would break
existing native forms. The single cross-cutting sentence was corrected to allow
deliberate session-bound tokens in protected SSR forms, without weakening the
server-secret, logging or generic-hydration restrictions. No runtime code changed.

Independent follow-up Reviewer Job `wc_job_1u2DDZmf0DOsZaMR` completed with exit 0,
`overall_status: pass`, zero issues and zero required followups. This was an
independent role-directed Codex Job, not native managed `agent_type` dispatch or
self-review. The six-document scope and the exact CSRF correction were checked;
observed `gpt-6-astra` / `high` runtime metadata was retained. Original failure
and follow-up evidence remain preserved in local ignored artifacts.

The review accepted **registration and planning only**. All eleven items were
Todo at that checkpoint; YT-113's actual authentication-contract research was
still the next work item. Its later study and review are recorded separately. No implementation, commit, push, release or global installation is implied.
