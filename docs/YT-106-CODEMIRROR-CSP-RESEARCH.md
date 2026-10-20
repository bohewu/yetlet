# YT-106 — CodeMirror CSP feasibility

Date: 2026-10-20. Inspected checkout: `5b7bde7`, which commits the independently
reviewed [Chart.js slice](YT-106-CHART-VERIFICATION.md). The user selected this
research separately and explicitly deferred implementation. No application,
manifest, settings, Rust dependency or npm lock changed. Package artifacts were
read outside the repository, not installed or executed. Implementation acceptance
is not implied by the generic browser experiment below. Independent Reviewer
`/root/yt106_codemirror_csp_research_review_oct20` PASS has zero issues or required
followups; this research is complete at the stated scope.

## Recommendation

**A small nonce-based trial is feasible on the inspected source and observed
Chrome mechanisms. CodeMirror itself is not yet qualified in Yetlet.** The
existing policy alone is insufficient: CodeMirror generates a stylesheet and
`style-src 'self'` does not authorize that inline block. Recommend a fresh,
response-local **style nonce on the selected editor page only**, using
`EditorView.cspNonce`; keep native textarea/editor links and the current script
policy. Do not enable `unsafe-inline`, add JavaScript `unsafe-eval`, fork
CodeMirror, or build a core nonce/provider framework for this one application.

The earlier [interoperability study](YT-106-FRONTEND-INTEROP-RESEARCH.md) correctly
left stylesheet and attribute behavior separate. This study narrows that open
question; the actual editor's rendered behavior remains a later gate.

## What the published packages do

Read on 2026-10-20 from npm's publisher artifacts, with versions, tarball
integrities and selected-file SHA-256 identities in external `source-inputs.json`:

| Published input | Inspected behavior relevant to this decision |
| --- | --- |
| [@codemirror/view 6.43.15](https://registry.npmjs.org/@codemirror/view/6.43.15) | `dist/index.d.ts` exposes `EditorView.cspNonce`; `dist/index.js` `mountStyles` passes it to `StyleModule.mount`. `updateAttrs` supplies tab size through the style helper's `cssText` path; copy/paste fallback textareas also use `cssText`. Layout/selection code uses individual style properties |
| [style-mod 4.1.4](https://registry.npmjs.org/style-mod/4.1.4) | Satisfies view's declared `^4.1.0` range. For a normal Document root, `src/style-mod.js` creates a `<style>`, applies the supplied nonce before insertion and fills its text. The root retains a shared style set; ShadowRoot/adopted-stylesheet behavior is a separate path |
| [@codemirror/state 6.7.6](https://registry.npmjs.org/@codemirror/state/6.7.6) | Compatible with view's declared state range; state alone does not provide the view's stylesheet authorization |
| [@codemirror/commands 6.11.1](https://registry.npmjs.org/@codemirror/commands/6.11.1) | Candidate for ordinary history/keymaps in a later plain-text trial. It also declares language/Lezer dependencies; this is not a measured bundle or a promise that only two packages suffice |

These four packages declare MIT. This is a selected CSP-path inspection, not a
complete dependency audit or resolved installation. In particular, style-mod
4.1.4 is a compatible inspected candidate, not a version in the Admin lock.
The current CodeMirror rendered reference returned HTTP 403; the published
declarations/code above are the version-specific evidence. GitHead source URLs
returned 404, so the report does not substitute unverified main-branch code for
these artifacts.

`cspNonce` authorizes the generated **stylesheet**, not every style operation.
It cannot authorize a `style="..."` attribute. The package's attribute helpers
special-case style through CSSOM rather than blindly using `setAttribute`.
Thus finding the word `style` in emitted DOM is not, by itself, proof of a CSP
violation. App-supplied attributes, widgets and later extensions still need review.

## Resolve the CSSOM uncertainty with a bounded experiment

[MDN's style-src page](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/style-src)
contains both an example describing `cssText` as blocked and a note that browsers
do not enforce the proposed CSSOM `unsafe-eval` restriction. Its
[maintainer issue #11697](https://github.com/mdn/content/issues/11697) records the
same discrepancy. The [CSP draft's CSSOM discussion](https://www.w3.org/TR/CSP3/#directive-style-src)
also has an unresolved explanatory note. Do not infer a required policy
weakening from those examples or claim all browsers behave identically.

One temporary, dependency-free HTTP fixture served an external same-origin JS
file and CSS file. Existing Playwright with system Chrome 155.0.8059.84 on Ubuntu
inspected computed values and `securitypolicyviolation` events. Browser plugin
was unavailable. No CodeMirror code or Yetlet server ran in this experiment.

The two policies were the current triage CSP and the same policy with only this
proposed style change (placeholder, not a usable/static nonce):

```text
style-src 'self' 'nonce-<fresh-response-value>'; style-src-attr 'none'
```

All other directives, including `script-src 'self' 'wasm-unsafe-eval'`, stayed
the same. The fixture used a random 16-byte nonce for each HTML response.

| Operation | Current policy | Candidate style policy |
| --- | --- | --- |
| Same-origin external CSS | Applied | Applied |
| HTML style attribute | Blocked | Blocked |
| `setAttribute('style', ...)` | Blocked | Blocked |
| The same attribute with a matching nonce on its element | Blocked | Blocked |
| `style.cssText`, including tab size | Applied | Applied |
| Individual `style.color` / `style.setProperty` | Applied | Applied |
| Dynamic `<style>` without nonce / with wrong nonce | Blocked | Blocked |
| Dynamic `<style>` with the response's matching nonce | Blocked | Applied |

The current/candidate cases produced exactly six/five intentional inline-style
violations respectively, with matching CSP console messages, no other console
error and no warning. These are expected negative controls, **not** a zero-CSP-
error editor qualification. Both cases applied the CSSOM values, including
`tab-size: 8`. Browser and HTTP fixture were closed afterward.

External evidence is `/tmp/yetlet-yt106-codemirror-csp-20261020-3al18m_w/`:
`csp-mechanisms.mjs`, `csp-mechanisms-result.json`, `source-inputs.json` and
`research-evidence-files.json`. The experiment command was
`node /tmp/yetlet-yt106-codemirror-csp-20261020-3al18m_w/csp-mechanisms.mjs`
(exit 0, cleanup true). It is a direct mechanism probe, not new repository test
infrastructure. No app build/runtime suites were rerun for this document-only
research; the committed Chart.js evidence remains separately scoped.

## Minimum later application change

These are recommendations for a separately authorized trial, not changes made
by this research:

1. Generate at least 128 unpredictable bits per selected HTML response using
   the existing Board `getrandom` dependency. Reuse its generation/encoding
   idiom, never existing login codes, CSRF/session values, secrets or a settings
   constant. The [CSP nonce requirement](https://www.w3.org/TR/CSP3/#security-nonces)
   requires fresh, unpredictable values. Nonces are per response/document, not
   per process or per widget mount; remounts in that document reuse its policy.
2. Keep one app-private value for HTML and header construction. The triage
   response already carries an extension selected by the Board security
   middleware; use that existing seam to avoid its final CSP assignment
   overwriting a handler-only header. Preserve the existing no-store HTML policy
   and fresh values on subsequent responses. Random-source failure must retain
   the native path, not emit a permissive policy.
3. Supply the value through one trusted head carrier, for example Vite's
   `<meta property="csp-nonce" nonce="...">` convention, read using `.nonce`.
   Keep it out of ordinary data attributes, user content, settings and shared
   hydration DTOs. The [nonce-hiding rationale](https://www.w3.org/TR/CSP3/#security-nonce-exfiltration)
   explains why the DOM nonce property is preferable to an ordinary readable
   content attribute. An implementation must check this carrier and header
   match before enhancing the native field.
4. Pass it to `EditorView.cspNonce.of(value)` in a private TS factory, using a
   normal Document-root host outside keyed rows. Preserve the native description
   until successful editor startup. Keep editor DOM owned by CodeMirror, text
   state/Save owned by Rust, and update/destroy/retirement guards app-local like
   the Chart.js adapter. Do not treat a DOM `style` attribute's presence as a
   failure; inspect applied values and actual CSP events.

The [triage view](../examples/lab-operations-board/src/triage.rs) and the Board's
[security/page functions](../examples/lab-operations-board/src/lib.rs) are the
relevant server seams. They already inject the selected module and choose the
route's policy. The existing core document renderer needs no general nonce API.
The [frontend entry](../examples/admin-dashboard/frontend/app.ts) should retain
its reviewed acyclic lazy-import arrangement; CLI/settings and asset delivery
need no new contract. Preserve the existing asset bounds and measure the actual
new package/lock and raw/gzip delta only when a trial is built.

### Why the alternatives are not the minimum

| Alternative | Tradeoff |
| --- | --- |
| Keep policy unchanged | Keep native textarea; cannot assume the generated editor stylesheet will work |
| Precompute/externalize CodeMirror CSS or stylesheet hashes | Adds generated-class ordering/style-set coordination and still does not remove runtime layout updates. No supported static-CSS replacement was established in the inspected API |
| Shadow DOM, iframe or adopted-stylesheet route | Adds a different styling/lifetime boundary without proving the selected editor's CSP behavior; not needed for a normal app-owned host |
| Broad `unsafe-inline`, editor fork or a generic nonce service | Expands authority or maintenance scope without evidence that the response-local nonce approach is insufficient |

## Necessary acceptance if implementation is selected

| Later check | Smallest useful evidence |
| --- | --- |
| Policy and nonce | Distinct fresh HTML response values, matching header/carrier/editor stylesheet; wrong/missing nonce and inline attributes stay blocked; ordinary pages retain their existing CSP |
| Real editor styles | Build the selected locked packages; observe its generated style sheets and CSSOM during mount, resize/scroll, selection and clipboard fallback if reachable. No unexplained CSP event or console error; do not add unsafe-inline/eval to obtain a pass |
| Data and editing | Existing plain-text 4,000-character description boundary, correct selected-item callback, ordinary typing/undo/redo, stated Reset/switch history boundary and pending-item read-only state; local edits do not POST |
| Lifetime and fallback | Late import/callback cannot reach a retired runtime; destroy/reload mounts once. JS/package/nonce failure leaves native textarea or editor links. Shared document style-mod sheets may outlive a view; destroy must release view resources without deleting another instance's styles |
| Existing execution | CLI build/run/stop, same-origin manifest/MIME and owned cleanup; native label/keyboard usability. Use the bounded Ubuntu browser scope rather than opening a new platform or IME/AT matrix |

The recommendation is complete at research scope after independent review.
Actual CodeMirror package installation, adapter, nonce/header changes, editor
browser qualification and bundle measurement remain unimplemented. The next
unblocked decision is whether to authorize that single plain-text Admin trial;
retaining the current textarea is a valid practical outcome.
