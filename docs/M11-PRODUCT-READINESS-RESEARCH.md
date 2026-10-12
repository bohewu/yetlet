# M11 — Product Readiness Research

**Date:** 2026-10-12. **Task:** YT-069, documentation and design only.
**Inspected source:** `c7ecc49eeb667e145cfb7e764c10b72ba104bbee`, `master`, current Ubuntu checkout; clean before this documentation work. The research branch described in the preceding conversation has since been merged. Local remote-tracking state is not a fresh hosted-CI or remote-settings check.

**Input:** the owner's complete `Yetlet-product-readiness-research-handoff-20261012.md`, SHA-256 `a7c41198d85ec87fa86a3d6785e1411553b1a205c8956674a3bcddf27f05baea`. Its workstreams A–E and exclusions are retained below. The subsequent conversation authorizes repository documentation and review, but not implementation, installations, new product builds/tests, credentials, deployment, commit, push or publication. [M11 plan](M11-PLAN.md) owns sequencing; [wgpu plan](M11-WGPU-PLAN.md) is an independent optional track, not a prerequisite for this product.

## 1. Decision summary and evidence vocabulary

**Recommendation:** proceed toward one complete, small SSR-first **Lab Operations Board**, using an existing locally hosted CSS system and application-owned persistence/authorization. Yetlet has credible building blocks, but the inspected source and historical tests do not qualify that not-yet-built product. Preserve the tiny framework; promote only behavior required by real consumers.

Finding labels used here:

- **S — source-confirmed behavior:** observed implementation or test source at the inspected commit; not a newly executed test.
- **E — existing measured evidence:** the linked historical run, with its original source, environment and limitations.
- **P — proposed experiment/design:** future work requiring the relevant authorization; all proposed numeric targets are assumptions.
- **U — unknown:** not established by the available source/evidence.
- **O — outside selected scope:** deliberately excluded, not an implementation defect.

Only the three gaps in section 6 are priority-ranked. Other matrices define their coverage and the separate GPU gates, not additional top-priority projects. No new observed exploitable vulnerability or runtime regression is asserted by this static review.

### Source register

| Ref | Inspected source or retained record | Relevant boundary |
|---|---|---|
| S1 | [core](../crates/yetlet-core/src/lib.rs), lines 8–49 | Text/Element only; static names and string-valued attributes |
| S2 | [HTML renderer](../crates/yetlet-html/src/lib.rs), lines 41–129 and 137–187 | Escaping, name/tag/attribute policy, selected URL schemes, test source |
| S3 | [DOM renderer](../crates/yetlet-dom/src/lib.rs), lines 24–118 and 125–154 | Narrow preflight allowlist, DOM creation, owned-subtree replacement |
| S4 | [server renderer](../crates/yetlet-server-axum/src/lib.rs), lines 30, 41–91; [Page](../crates/yetlet-server-axum/src/response.rs), lines 53–61; [metadata](../crates/yetlet-server-axum/src/document.rs), lines 54–87 | CSP response paths versus string helpers; title/language only |
| S5 | [keyed guide](../crates/yetlet-dom-keyed/README.md), lines 9–26; [fixed binding example](../examples/keyed-bindings/src/lib.rs), lines 1018–1027 and 1109–1167 | Caller-owned callbacks/properties; model-aware text input updates |
| S6 | [browser build](../scripts/build_browser.py), lines 69–79 and 97–147; [Todo server](../examples/todo-server/src/lib.rs), lines 75–111 and 136 | Separate builds, hashed CSS/JS/WASM, validated application head insertion |
| S7 | [dashboard source](../examples/multi-island-dashboard/src/lib.rs), lines 136–205 and 212–251; [dashboard guide](../examples/multi-island-dashboard/README.md) | Asset existence/path checks, per-island fallback and application response policy |
| S8 | [SQLite source](../examples/sqlite-crud/src/lib.rs), lines 1–78; [Modular Shop](../examples/modular-shop/README.md) | Direct SQL, bounded blocking work, explicit version and transaction concepts |
| E1 | [M7d](M7D-VERIFICATION.md), [M7j](M7J-VERIFICATION.md) | Bounded text-input/keyed lifecycle and repaired late-action evidence |
| E2 | [M9j](M9J-VERIFICATION.md), [qualification matrix](QUALIFICATION-MATRIX.md) | Historical application/native/browser tests; optional real IME/AT/mobile gaps |
| E3 | [Redis b3](YT-066B3-VERIFICATION.md), lines 51–72 | Single-host, single-primary, no-persistence semantics and login-loss boundary |
| E4 | [release readiness](RELEASE-READINESS.md), [API classification](API-CONTRACTS.md) | Experimental candidates, separate release/source identities |

## 2. Workstream A — HTML, forms and browser behavior

### Capability matrix

Entries describe S unless explicitly marked E or U. SSR acceptance means serialization, not complete HTML content-model, live-control, accessibility or browser qualification.

| Capability | Native SSR | Generic browser mount | Keyed/application-owned DOM | Existing evidence and remaining need |
|---|---|---|---|---|
| Semantic layout/headings/lists | Valid names other than the forbidden set serialize; application chooses correct structure | Only `main section div article header h1 h2 p span ul li button strong input label` | Caller can create other elements; correctness belongs to that caller | S1–S3; E2 covers particular samples, not every tag/nesting combination |
| Links/images/navigation | `a`, `img`, `href`, `src` serialize with selected scheme rejection; no general origin policy | `a`/`img` and URL attributes are rejected | Explicit `web-sys` creation is possible; no inherited sanitizer or URL policy | S2/S3/S5; P: selected links/images, alt text, keyboard navigation and parsed destinations |
| Tables | Table-related names serialize; structure is application-owned | Table tags are rejected | Caller can maintain rows under explicit ownership | P: captions, header associations, explicit `tbody`, overflow and keyed row identity |
| Forms/input types/labels/buttons | Form/select/option/textarea/input and ordinary fields serialize; type semantics are left to browser/app | Label/button/input exist, but no form/select/textarea; `type` is allowed without a typed input-kind contract | Caller owns submission, DOM control types and updates | Forms sample/E2 is native SSR; not generic client-form coverage |
| `name`, `value`, `checked`, `selected`, `disabled`, `required` | Valid attribute names/string values serialize; false boolean values must be omitted by caller | These names are rejected | Property setters/attribute removal can be used deliberately | S1/S2/S5; P: boolean presence, default versus live state and reset |
| `aria-*`, `data-*`, `title`, `lang`, focus metadata | Valid names serialize; that does not establish semantic correctness | Only the explicit S3 list: `id class type data-id data-completed aria-live aria-pressed for placeholder maxlength autocomplete` | Binding vocabulary permits selected inert attributes; callbacks retain responsibility | P: required labels/descriptions/errors/focus attributes, reserved namespace checks; no blanket acceptance of all names |
| Initial value versus live user state | Initial markup only; textarea content differs from input value attributes | New-node construction uses `set_attribute`; every mount replaces the owned subtree | Fixed text-input example compares model value before `set_value`; not a generic select/checkbox API | S3/S5 and E1; P: dirty inputs, caret, selection, select options, radio groups, reset and model/user conflicts |
| Events/submit/validation/errors | Native forms and server validation work in selected examples | `mount` does not install a general form/event runtime; listeners must outlive replacement appropriately | `DomEventListener` owns cleanup; application chooses event/validation policy | E1/E2; P: successful controls, Enter, cancellation, 422 field errors, PRG and duplicate submission |
| Reorder/focus/disposal | SSR has no live browser lifecycle | Replacement does not promise keyed identity/focus preservation | Keyed owner preserves entries; callbacks/final DOM mutation are not transactional | E1 is bounded; P: complete selected form lifecycle and stale completions |
| Chinese IME | Text can serialize; no live composition claim | No demonstrated generic composition lifecycle | Fixed input callback reads `input.value`; no general composition policy is established | U: real OS candidate selection/composition; synthetic Unicode/key input is not IME PASS |
| SVG/namespaces | Valid-looking names can serialize, but this is not a namespace-aware renderer | No SVG/createElementNS path | An app may own a separate explicit SVG surface | O: generic SVG support; P only if the selected product needs it |
| Canvas/GPU island host | `canvas` is not forbidden by the SSR serializer | `canvas` is not in S3's allowlist | A GPU adapter can own an SSR canvas without routing it through generic mount | Separate [GPU track](M11-WGPU-PLAN.md); not a reason to widen all DOM policy |

### Corrections and implications

**S:** the handoff's narrow DOM observation is still correct. It limits the standard shared-View mount, not the ability to ship native forms or write bounded application-owned `web-sys` code. Whole-page CSR parity is not required for an SSR-first product.

**S/P:** string-valued attributes are not a boolean-control abstraction. The HTML standard makes presence true and omission false; a string `disabled="false"` is not a way to enable a control. Default `value`/`checked` attributes also differ from dirty live properties. These are reasons for typed behavior/tests, not proof that an existing application currently emits that mistake. See primary references H1/H2 below.

**P:** prefer native POST/redirect/read and a small local enhancement first. Keep user-edited fields outside broad subtree replacement. A useful minimal enhancement covers a text field, textarea, checkbox, select and submit/reset with deliberate property semantics; do not build a generic forms framework, VDOM or hydration engine.

## 3. Workstream B — Existing CSS and UI design

### Comparison: three approaches, no installation or purchase

| Approach | Concrete candidate and source | Cost/compatibility | Recommendation |
|---|---|---|---|
| Locally hosted precompiled CSS | Bootstrap 5.3.8 reference distribution; MIT code, preserve notices (C1/C2) | No browser framework or Node requirement for consuming compiled CSS. Embedded SVG data URLs in several controls conflict with the current restrictive image policy (C3). An existing theme is not automatically licensed with Bootstrap. | **P: preferred first proof**, CSS-only, native controls and a small external application stylesheet |
| Build-time utility CSS | Tailwind CSS v4 CLI; MIT framework code (C4–C6) | Scan explicit Rust/UI source paths as text, use complete literal class alternatives, include bounded safelists, pin the compiler and retain generated CSS. Dynamic concatenated class names are not discoverable automatically. Standalone CLI is possible; Node is not inherently a runtime requirement. | P: alternative if utility authoring justifies an extra pinned build stage; not selected merely from prior preferences |
| CSS plus JS widgets | Selective Bootstrap JS, under the same code license (C1/C7) | Adds instance/listener/DOM ownership, keyboard/focus, asynchronous transitions and disposal obligations. DOM mutations/inline style use must be inspected per selected widget. | O for first slice; later opt-in only for a concrete interaction that native controls cannot meet economically |

The recommendation selects an integration direction for review, not a new dependency. Before YT-070 acquisition, approve exact artifact/version and license inventory. Free framework licenses do not grant rights to paid admin templates, Tailwind commercial assets, photographs, icon sets or fonts. No commercial template is selected.

**P — asset and CSP contract:** serve one application-owned hashed CSS bundle and all required images from the same origin. Preserve upstream notices and original source digest; record any deterministic transform and the resulting digest. For Bootstrap's embedded SVGs, replace only needed data-URL images with vetted same-origin files, or use native control styling that does not request them. Keep a finite manifest of replacements and test the actual controls. Do not add `data:`, `unsafe-inline` or `unsafe-eval` merely to make the demo look right. If the chosen subset cannot meet that contract cheaply, stop and compare the Tailwind alternative rather than broadening CSP silently. C3 confirms the compatibility risk; no Yetlet browser compatibility has been executed here.

**P:** use system fonts initially. An optional local font requires its own license/digest and an explicit same-origin `font-src` decision; the current `default-src 'none'` does not imply permission to fetch fonts. External CDNs, remote fonts, telemetry and icon data URLs are absent from the proposed first product.

**S:** S6/S7 already generate/serve CSS and insert validated link/script tags into a complete document. `DocumentMetadata` itself remains title/language only. **P:** first prove a typed application wrapper with ordered/deduplicated same-origin stylesheet references, independent of browser JS. Promote a tiny additive server-adapter stylesheet value only if two consumers demonstrate the same safety/lifecycle need. It must preserve old `Page` signatures and no-asset output, not create raw head markup, a head tree, arbitrary scripts, a CSP DSL or a core CSS engine.

**P:** content-addressed assets may get long-lived immutable caching after digest verification; HTML/manifest switching must not strand older referenced assets. A missing optional enhancement keeps semantic SSR; a product build missing its required CSS should fail its package/readiness check rather than be advertised as visually complete. Runtime missing-CSS behavior must remain readable and report the failure. Never claim an existing cache policy merely from hashed filenames.

### Small visual-design brief: Lab Operations Board

This is a planning brief, not a rendered mockup or UI PASS.

| Area | Proposed decision |
|---|---|
| Audience/job | A small noncritical lab/project team tracking work items and their actual persisted status; no school production systems, customer data or remote machine control |
| Information architecture | Overview, Work items, item detail/edit, item History; access-denied/not-found pages. One persistent navigation region and one main task surface |
| Primary reading order | Page title and current scope; actionable stale/error notices; filter/sort controls; actual records; pagination; contextual detail/history. Overview counts come from persisted records, not decorative animation |
| Navigation/state | Native links/forms, query-backed filters/page/sort, copyable detail URLs, browser back/refresh. Selection may enhance locally but the URL identifies the record |
| Page skeleton | Header with identity/environment, compact sidebar on wider screens, one-column main area with clear form sections; narrow screens use visible navigation links or native disclosure rather than a required JS hamburger |
| Typography/spacing | System UI plus installed CJK fallback; proposed 1rem body, 1.5 line height, 1.25/1.75rem section/page headings; spacing tokens 4/8/12/16/24/32px. These are design assumptions, not existing Yetlet tokens |
| Responsive behavior | Proposed under 48rem: stacked filters/forms, key columns plus detail links; 48–64rem: compact two-column summaries; above 64rem: side navigation. Keep table headers/labels and deliberate bounded table overflow; never shrink text to fit |
| Themes/visual roles | One neutral surface scale, one action accent, separate success/warning/error roles plus text/icons; external CSS variables/state classes. Start with one light theme; optional dark mode needs independent contrast checks |
| Interaction states | Loading: reserved area and named progress; empty: scope-aware explanation and create action; validation: summary plus field associations; denied: no hidden privileged action; stale/conflict: preserve submitted draft; success: PRG/status confirmation; disabled: true semantics and reason |
| Accessibility/quality | Visible keyboard focus, labels/help/error relationships, live status only for meaningful changes, no color-only state; 200%/400% reflow, Chinese IME and spoken assistive-technology journeys are explicit proposed coverage |

A future JS widget owns exactly one designated subtree, its instances/listeners and all teardown. Yetlet must not reconcile the widget's children concurrently. Mount/remount, transition completion after disposal, disabled state and keyboard/IME behavior need dedicated evidence. Successful CSS rendering alone does not prove widget compatibility.

## 4. Workstream C — XSS and untrusted content

### Precise responsibility statement

**S:** Yetlet provides contextual text/quoted-attribute escaping in its HTML serializer, selected tag/attribute rejection, a restricted generic DOM construction path, and CSP on specified response paths. It does **not** provide a vetted rich-HTML sanitizer, a general URL/origin trust policy or an assurance that all application HTML/JS callbacks are safe. Its `View` has no public RawHtml variant. Keyed callbacks and raw Axum `Html` are application escape boundaries, not protected automatically by the safe View path (S1–S5).

**S:** `Page::into_response` reapplies the default CSP even for its rendering-error response. `render_page_with_metadata` adds it on success; its returned `HtmlPageError` converted directly is not the same CSP-enforcing `Page` wrapper. `render_document*` and `render_to_string` return strings and cannot attach headers. An application can construct or overwrite response headers; `DEFAULT_CSP` is not universal middleware enforcement. Tests for each selected success/error path are therefore necessary.

**S:** URL filtering covers the listed names `href`, `src`, `action`, `formaction`, `xlink:href`, strips ASCII whitespace/control characters and lowercases before rejecting `javascript:`, `vbscript:` and `data:`. However, name validation rejects the colon in `xlink:href`; the entry is not evidence of usable SVG support. The filter does not validate origins, redirects, every URL-bearing attribute, compound values such as `srcset`, or browser parser behavior. Static developer-selected names are still important; do not map arbitrary imported element/attribute names into Views and call that sanitization.

**P:** for the first product, use plain bounded text and application-generated relative routes. HTML/attribute escaping remains at output time; do not pre-escape stored text and then double-encode it. Authentication, authorization, CSRF, URL navigation policy and XSS are separate checks. Session cookies or a WAF do not replace those checks. OWASP's primary guidance distinguishes safe sinks, output encoding and HTML sanitization (X1).

### Data-path decisions

| Path | Current observation | Proposed rule / unresolved boundary |
|---|---|---|
| Input → validation → SQL → detail/list/history SSR | S: escaped text/attributes and parameterized SQL patterns exist separately | P: store plain text, bound lengths, render through safe Views; test create/edit/save/read/error-preview together, including audit text |
| Input → browser update | S: generic renderer uses createTextNode/createElement/setAttribute, not HTML parsing | P: preserve fixed safe names and live-value properties; every application callback gets its own sink review |
| URL input → attribute/navigation | S: selected unsafe-scheme filter only | P: first product routes are application-generated. For future user links, approve explicit scheme/origin policy and test actual parsed destinations; encoding is not URL validation |
| Versioned state → HTML attribute → JSON decode | S/E: bounded island patterns exist | P: explicit public schema/size/version, attribute escaping, mismatched-version fallback; no tokens, repository handles or server capabilities |
| Plain text/structured blocks/controlled Markdown | S: no general importer is selected | P: prefer validated blocks or a Markdown AST mapped to safe Views; reject/ignore raw-HTML AST nodes explicitly and validate link/image destinations |
| Imported rich HTML | O for first product | P only on a later approved need: compare Rust Ammonia (HTML-parser-based server sanitization, X2) and browser DOMPurify (X3). Pin policy/version and define server/browser parity, not a regex sanitizer |
| Sanitized content → third-party editor → preview/save/read | U: no adopted editor path | P: sanitization can be invalidated by later mutation; sanitize/validate at the authoritative boundary, constrain preview sinks and keep server verification. Browser-only sanitization cannot protect other clients or SSR |

A later sanitizer would be optional application integration, not an automatic core dependency or a new unconstrained raw-HTML escape hatch. A structured-block implementation is distinct from automatically trusting an editor's exported HTML.

### Bounded proposed negative-test matrix

All rows are **P**, except the explicitly named existing test sources. No new full security scan is requested.

| Case | Assertion before acceptance | Existing basis / required owner |
|---|---|---|
| Text/quote payloads through create/edit/read/preview/history | Literal `<`, `>`, `&`, quotes survive as text; no extra nodes/attributes or stored execution | S2 `escapes_dynamic_text_and_attributes`; app adds complete persisted journey |
| Forbidden elements and event/style/srcdoc attributes | Safe View paths reject before generic DOM mutation; previous DOM remains intact on validation failure | S2/S3 negative-test source; framework/security reviewer |
| Mixed-case/control/whitespace script/data URLs | Declared URL policy rejects executable destinations; valid same-origin links still navigate | S2 `rejects_script_urls_and_inline_styles`; new HTML + actual-browser coverage |
| Entity-like and percent-encoded URL variants | Assert browser-parsed destination and application policy, not a blanket assumption that repeated percent-decoding is safe or required | U currently; strings containing literal entities are escaped in SSR; targeted parser/security review |
| Protocol-relative, external-origin, backslash and compound URL values | Match an explicit allow/reject policy; no open redirect or hidden secondary interpretation | App owns allowed destinations; omit unsupported compound/namespace cases from new API |
| Attribute booleans/live controls | False omits a boolean; checked/selected/value/reset follow the selected contract without caret or composition loss | H1/H2 plus S5; framework/browser reviewer |
| State payload breakout/oversize/version mismatch | No inline execution, no secret fields, bounded parse, readable SSR on rejection | E2 island tests retained; app adds its exact schema |
| Page/error/raw response header paths | CSP present on selected success/4xx/5xx routes; wrapper cannot silently drop policy; no unsafe-inline/eval | S4; app owns raw Html/header overrides |
| Stylesheet/manifest injection/missing asset | Reject wrong-origin/traversal/unexpected manifest refs; CSS dependencies obey CSP; SSR remains readable | S6/S7; asset contract reviewer and real-browser enforcement |
| Retired callbacks/widget mutation | No writes after disposal/remount; no shared-subtree ownership or unreviewed HTML parsing sink | E1/S5; selected app/GPU adapter review |
| Rich HTML/Markdown import, only if later selected | Stored and DOM XSS tests across parser → sanitizer/AST → storage → preview/read; sanitizer policy skew and post-mutation cases | Separate dependency/security approval; otherwise explicitly not implemented |
| Auth/CSRF/IDOR controls | Viewer POST denied; foreign-project resource denied; stale/absent CSRF rejected; no private cache leakage | Separate from XSS; app security reviewer and selected identity evidence |

Independent security-focused review is required before accepting broadened HTML/URL policy, live-property helpers, rich-content ingestion or an externally reachable authenticated product. This document's self-review cannot satisfy those later gates.

## 5. Workstream D — Compare complete products

These are candidates, not permission to build all of them. Relative effort/risk is qualitative, not a delivery estimate.

| Candidate/order | User and smallest complete workflow | Framework/UI exercised | Security/session/deployment | Exclusions, expected evidence, relative risk |
|---|---|---|---|---|
| **1. Lab Operations Board** | Small lab team: list/filter/sort/page → create → detail/edit → status transition → history; preserve draft on version conflict | SSR tables/forms, CSS shell, direct SQL, narrow optional status island/SSE; real persisted counts | Viewer/Operator plus project membership; server authorization and CSRF; one app + SQLite initially; Memory session may require re-login after restart | No remote execution, scheduler, school systems or decorative fake metrics. E2E write/read/denial/conflict/stale/restart evidence; medium scope, highest breadth of useful evidence |
| **2. Blog/light CMS** | Author: draft → preview → publish; visitor: tags/pagination/post | SSR/public cache versus private draft policy, metadata/assets, controlled content pipeline | Author permissions and CSRF; no Session on public reading unless needed; same simple persistence option | No arbitrary HTML/comments/uploads by default. Proves publishing/public-private content and stored-XSS paths the Board does not; low-medium with plain text, higher with Markdown |
| **3. Small shop** | Shopper: listing/detail/search → cart → authoritative price/quantity check → simulated order | Builds on Modular Shop rather than duplicating it; transaction, Session and idempotency behavior | Synthetic customers, explicit Session choice; DB is price/inventory authority, bounded stock conflict | No charges/payment account, tax/shipping/marketplace. Evidence: duplicate submit, changed price, insufficient inventory and restart; medium-high domain risk |
| **4. Limited knowledge workspace** | Team: hierarchy/search → edit a few typed blocks → save/conflict | Nested browser ownership, authoring surface, structured-content validation | Per-page permissions, CSRF, safe rendering and explicit version conflict | No collaborative CRDT/OT, offline sync, full editor engine. Compare an established editor with small structured controls only after a requirement; high ownership/IME/content risk |
| **Deployment axis, not another visual product** | Operator runs the selected app with only its needed DB/session service | Config, health/readiness, graceful shutdown, logs, restart and rollback | Separate edge/IdP/secrets/backup responsibilities; opt-in Redis only if shared sessions are needed | No Kubernetes, service mesh, distributed transactions or HA platform. One bounded container/process scenario; risk driven by selected services |

A CMS may supply a tiny static CSS/SSR fixture inside YT-070, but must not become a second complete product before the Board. Later products require separate activation. Neither the Board nor CMS depends on GPU rendering.

### First product contract

**P:** `WorkItem` has an ID, project ID, plain-text title/description, status, priority, assignee reference, version and timestamps. Approved status transitions append a persisted history event in the same transaction. This is a work-record system, not permission to execute shell commands or control WebCodex Jobs.

**P:** Viewer can read permitted projects but cannot mutate. Operator can create/update only within assigned projects. Enforce both (1) viewer mutation denial and (2) operator cross-project object denial at each server endpoint. Do not trust hidden buttons, client role fields or object IDs as authorization.

**P:** successful writes use POST/Redirect/GET. Reject stale expected versions with a visible 409 conflict and retained draft; validate fields with 422; bound pagination and sort identifiers. Use server-owned identity and parameterized SQL. Status notifications carry bounded authorized invalidations/versions; reload the authoritative snapshot after reconnect or a replay gap. A transactional history row is the durable source; missed transient SSE delivery must not lose the authoritative update. Keep last-known data visible with updated-at/stale/offline/error state, bounded reconnect/backoff and no automatic replay of mutations.

**P:** initial sample authentication may reuse a clearly labeled loopback-only synthetic fixture. That can prove authorization behavior, not real OIDC or production identity. Any externally reachable qualification requires a separately authorized real test IdP integration, server-validated identity and approved secret/config custody; no credential creation is authorized now.

**P:** Memory session is sufficient for the proposed single-process demonstrator if re-login on restart is accepted. Persist work items/history in SQLite; do not put durable business data in Session. Shared login across replicas would require selecting a separate provider and testing its actual failure/cookie contract. E3 explicitly permits login loss under extreme response reordering and loses state on no-persistence Redis restart; do not reinterpret it as durable HA.

## 6. Exactly three highest-priority gaps

| Gap | Impact and minimum change boundary | Alternatives | Acceptance and owner decision |
|---|---|---|---|
| **G1 — Safe browser/form semantics are too narrow for ordinary shared-View product controls** | Links/tables/select/textarea and common fields cannot use generic mount; adding strings alone misses live properties and ownership. Prove selected HTML controls, boolean omission, URL policy and owned live updates before promoting only reusable primitives; retain full SSR | Keep forms native and use bounded app-owned web-sys; no need to make all SSR tags mountable | Positive/negative S2/S3 policy, property/reset, focused reorder, real IME and disposal tests; security-focused independent review. Owner approves the bounded interactive-control scope, not a general forms runtime |
| **G2 — CSS works, but reusable document/asset integration and actual theme/CSP compatibility are not established** | Product setup repeats head insertion and manifest policy. First add a typed app wrapper and one CSS proof; only then an additive ordered same-origin stylesheet helper in server adapter if two consumers converge | Retain application wrapper; Tailwind build-time CSS rather than a CSP-incompatible theme. No core UI kit/head DSL | No-JS styled page, hostile/missing manifest, real CSP enforcement, CSS image dependency inventory, unchanged legacy Page behavior and unaffected bundles. Owner approves exact CSS artifact/license and any framework extraction |
| **G3 — Complete product trust and failure journeys are not qualified end to end** | Separate samples are not proof of one persisted authorized usable product. Compose Board workflows, content/permission boundaries and a realistic operating envelope; changes mainly belong to the application and tests | Keep localhost prototype claims; do not add built-in auth/ORM/HA to hide missing application evidence | Two-user authorization, CSRF, stored-text XSS, transaction/version conflicts, notifications/reconnect, dependency failures/restart and chosen accessibility/load coverage. Owner selects identity/session loss policy and the operating envelope before any deployment claim |

G1/G2 can close with a justified decision to retain application code rather than a new crate/API. G3 cannot close by merely relabeling existing samples. The separate GPU line does not outrank these product gaps or force its dependencies into them.

## 7. Workstream E — Bounded qualification, not universal certification

### Proposed operating envelope — assumptions requiring approval

| Dimension | Initial proposed bound |
|---|---|
| Audience/data | Noncritical lab data; 25 concurrent active users; 10,000 work items and 100,000 history rows; page size 50; title 160 characters, description 4,000 characters with a separate byte/body bound |
| Request mix | 80% authorized read requests and 20% validated writes; up to 10 long-lived status-stream clients measured separately from request percentages |
| Runtime/storage | One application process, SQLite on local persistent storage, bounded SQL worker admission; no HA/durability guarantee beyond the selected tested disk/restart scenario |
| Latency/errors | Candidate targets: read p95 below 250 ms, write p95 below 500 ms, p99 below 1 s, unexpected 5xx below 0.1%; exclude deliberate fault injections but report them separately |
| Sessions/identity | Explicit re-login after Memory restart is acceptable for the demonstrator; real authentication and any shared/durable Session policy need separate selection |
| Browsers/accessibility | Propose current exact Chrome/Edge and Firefox desktop builds plus real Chinese IME and one spoken AT journey; narrow-screen/touch, Safari/mobile and zoom coverage selected by actual audience, not assumed from historical tests |

**P — measurement plan:** build the approved immutable candidate outside timing. Record source/lock/toolchain, hardware, DB state, network/proxy path and exact browser versions. Warm up for two minutes, then run three separate ten-minute steady-state measurements with declared arrivals, concurrency and seeds; follow with one thirty-minute soak. These durations are initial experiment assumptions, not proof of production readiness. Record achieved throughput, p50/p95/p99 with sample counts, errors, CPU, process memory trend, queue depth, active connections/tasks and cleanup. Separate cold start from steady state and report variability; do not compare an optimized GPU release with a debug CPU baseline.

**P — resilience plan:** exercise graceful stop with active request/SSE/SQL work, restart with persisted records and accepted Session loss, database busy/read-only/unavailable responses, cancelled/deadline requests, an already-committed write with lost response, stale form retries, dependency recovery, and rollback to an explicitly compatible schema/source. Stop on unknown write outcomes; do not automatically repeat mutations. A fixed-duration flat memory trace is bounded evidence, not a proof of no leaks.

| Owner | Responsibilities |
|---|---|
| Framework | Safe documented renderer/asset/property contracts, lifetime/error semantics, dependency isolation and honest supported configurations |
| Application | Domain validation, routes, authorization, CSRF use, direct SQL/transactions/migrations, content/URL policy, idempotency/conflict handling, UI/accessibility and selected identity/provider integration |
| Operator/release owner | TLS/trusted proxy, credentials/ACL, storage/backup/restore and chosen RPO/RTO, resource limits, monitoring/rollout/rollback; separate immutable package/provenance/signing/registry decisions |

**E/U:** original nine candidates are still experimental and unpublished, and YT-020 remains Blocked. Historical signed source `8e557e7` does not attest inspected `c7ecc49` or this documentation overlay. Publication/signatures are not runtime correctness, and lack of a public registry package does not prevent a separately approved private deployment from an immutable source. A scoped production statement needs actual selected product/identity/browser/load/failure evidence, accepted residual risks, an operational owner and an approved rollout—not every possible framework feature or platform.

## 8. Staging, stop gates and completion

| Stage | Inputs → outputs | Stop / approval boundary |
|---|---|---|
| Research/design, current YT-069 | Handoff + current code/retained evidence + primary references → this matrix, three gaps, product/CSS recommendation, GPU companion, task graph and review | Documentation only; self-review permitted by owner, not independent assurance |
| Isolated proof, YT-070 / independently YT-075 | Approved exact slice/dependencies → real form/CSS or browser-GPU proof with current evidence | New source, dependency acquisition/builds/browser actions need later authority; no broad policy changes |
| Reviewed framework changes, YT-071/072 or YT-079 | Proof plus real consumers → minimal reusable contract, or reasoned no-promotion outcome | Independent implementation/security review and compatibility/isolation evidence; promotion cannot be inferred from a benchmark |
| Complete sample, YT-073 | Accepted primitives or safe app wrappers → persisted Board journeys with meaningful states | No production credentials/network exposure by default; real identity integration is separately approved |
| Bounded qualification, YT-074 / independently YT-080 | Approved envelope/source/targets → actual measurements, failure/accessibility record, cleanup and residual-risk acceptance | No unsupported platform/SLA/publication claim; missing selected gates remain open |

**Research completion criterion:** the owner can review a concrete first slice, CSS choice, three bounded changes and explicit unknowns without another open-ended discovery round. The unresolved decisions are finite: first product/domain, exact CSS asset acquisition, identity/session semantics, operating envelope, and independent GPU target/promotion scope. Details settled by source remain settled; do not ask the owner whether CSS or escaping exists.

### Handoff coverage

A is covered by section 2; B by section 3; C by section 4; D by section 5; E by section 7. Section 6 contains exactly the requested three priority gaps. Section 8 and [M11 plan](M11-PLAN.md) give stages, IDs and gates; [review](M11-PLANNING-REVIEW.md) records actual documentation checks and review limits. Unknowns remain explicit: real IME/AT/mobile behavior, selected CSS compatibility, full persisted product security/resilience, browser GPU behavior and current-source release attestation. No proposed product or experiment has been run by this planning work.

## Primary external references

Consulted 2026-10-12; these support external facts, not Yetlet execution. Exact dependency pins and artifact licenses must be rechecked at later acquisition.

- H1: [WHATWG boolean attributes](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#boolean-attributes).
- H2: [WHATWG input live/default state and reset](https://html.spec.whatwg.org/multipage/input.html).
- C1: [Bootstrap 5.3 license](https://getbootstrap.com/docs/5.3/about/license/).
- C2: [Bootstrap compiled distribution reference, 5.3.8 shown](https://getbootstrap.com/docs/5.3/getting-started/introduction/).
- C3: [Bootstrap embedded SVG and CSP guidance](https://getbootstrap.com/docs/5.3/customize/overview/).
- C4: [Tailwind source detection and complete class names](https://tailwindcss.com/docs/detecting-classes-in-source-files).
- C5: [Tailwind CLI](https://tailwindcss.com/docs/installation/tailwind-cli).
- C6: [Tailwind framework license](https://github.com/tailwindlabs/tailwindcss/blob/main/LICENSE).
- C7: [Bootstrap JS lifecycle/API](https://getbootstrap.com/docs/5.3/getting-started/javascript/).
- C8: [Bootstrap color modes](https://getbootstrap.com/docs/5.3/customize/color-modes/).
- X1: [OWASP XSS prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html).
- X2: [Ammonia primary documentation](https://docs.rs/ammonia/latest/ammonia/).
- X3: [DOMPurify primary README and post-sanitization warning](https://github.com/cure53/DOMPurify).
