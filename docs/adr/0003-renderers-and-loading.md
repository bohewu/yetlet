# ADR 0003 — Dual-target renderers and opt-in browser loading

- **Status:** M2/M3 render modes validated; M7 freezes a bounded keyed-DOM/local-graph decision in ADR 0007, with final read-only review pending. No general VDOM, template IR, dependency-tracked signals/forms API or resumability design is selected.
- **Date:** 2026-10-09
- **Related:** [Architecture](../ARCHITECTURE.md), [MVP](../MVP.md)

## Context

Yetlet aims to reuse Rust UI logic across SSR and CSR while minimizing browser payload for server-rendered pages. Native HTML serialization, browser DOM APIs, and WebGPU scene rendering require different capabilities. Forcing all of them through a single rendering implementation would invite unnecessary abstraction.

## Proposal

- Share **component semantics and source**, not necessarily binaries. Generate native executables, browser WASM, external bootstrap scripts, and manifests as separate build artifacts.
- **Server:** A native HTML renderer emits complete HTML. Pages without interactive components ship no JS or WASM.
- **SSR + islands:** Preserve the server-rendered markup. A tiny external JS loader resolves a build-time asset reference and mounts/activates only the selected island on interaction. Full resumability is not required for MVP.
- **Browser:** A more complete Rust/WASM runtime creates and updates DOM nodes directly.
- **Bootstrap:** Use the smallest practical implementation, including plain JS. Making every bootstrap byte Rust/WASM is not itself a goal.
- Generate/hash all code and assets at build/deploy time; never compile new code in the HTTP request path.
- A future **GPU scene renderer** is separate from HTML/DOM; WebGPU is not advertised as a general DOM/CSS accelerator.

## Unresolved design: UI representation

### Option A — Minimal shared `View` tree

Represent a small set of nodes/values to validate both native HTML and browser DOM rendering. This is inexpensive to implement but may introduce allocation and tree-update overhead.

### Option B — Compile-time static templates with dynamic slots

A future `ui!` macro could extract fixed HTML/templates and dynamic slots. Native rendering would serialize these slots; the browser would clone templates and update bound values, avoiding a full VDOM diff. However, macro complexity, event captures, and hydration identity are substantial design risks.

**Decision gate:** M7a measured replacement, positional `View` patch, keyed direct DOM and runtime static slots. M7b–M7d prove one fixed authoring/reactivity/lifecycle shape; M7e freezes the evidence without selecting a general renderer. M7f–M7h then promote only renderer-neutral identity/slot contracts, the explicitly owned synchronous graph and keyed DOM/listener ownership. M7i adopts those primitives for the production Todo local list while retaining subtree replacement for remote/Search paths. M7j freezes this bounded choice in [ADR 0007](0007-bounded-keyed-dom-and-local-graph.md). It is not a stable template macro, general VDOM or signals/forms selection. See [M7a](../M7A-VERIFICATION.md), [M7e](../M7E-VERIFICATION.md) and [M7j verification](../M7J-VERIFICATION.md).

## Hydration and state

- SSR HTML and browser code require stable, verifiable component/instance identities. Hydration must not take over unrelated DOM.
- Escape serialized state correctly and include schema/asset versions. Incompatible versions must explicitly fail or fall back.
- Browser state must not contain secrets, repository objects, or server handles.
- Avoid placing raw JSON in inline script blocks; tests must cover CSP and XSS boundaries.

## Alternatives considered

- **VDOM everywhere:** Uniform design but can add runtime costs; requires measurement.
- **Server-streamed UI updates:** Conflicts with local-first and optional transport goals.
- **Full resumability from day one:** Would require constrained Rust closures/captures and stable handler IDs; defer as research.
- **Canvas for every HTML element:** Would amount to rebuilding CSS layout, selection, IME, and accessibility infrastructure.

## Verification and current evidence

**M2 — 2026-10-08:** The same shared `View` source renders through native HTML and browser WASM/direct-DOM implementations. A headless Chrome test confirmed byte-identical initial TodoList markup, local state updates, and no WebSocket. Browser mode uses a tiny JS bootstrap and content-addressed assets. DOM updates currently replace one owned subtree rather than performing fine-grained subscriptions. See [M2 verification](../M2-VERIFICATION.md).

**M3 — 2026-10-08:** Native `/search` returns complete SSR HTML. An external loader waits for an input/focus event to load Search WASM. Headless Chrome confirmed initial DOM preservation, local search filtering, and no event circuit. YT-017 added an allowlisted, size-bounded `SearchStateV1` JSON payload and verified fail-closed behavior for incompatible versions. This is **not** generic resumability. See [M3 verification](../M3-VERIFICATION.md).

**M7a — 2026-10-09:** The isolated Chrome 155 benchmark used identical seeded operations with rotating strategy order. The retained `2026-10-09T02:02:22+00:00` report recorded median round totals of 213.500 ms (subtree replacement), 41.700 ms (positional View patch), 2.000 ms (keyed direct DOM) and 1.900 ms (static slots). All final states matched; production Browser/Search asset hashes were unchanged. The benchmark does not cover layout/paint, heap, realistic conditional component trees or other frameworks.

**M7b — 2026-10-09:** The isolated keyed/static-slot crate renders the same typed template to native SSR and binds it in Chrome without replacing matching SSR nodes. Instance/key/slot mismatches fail closed before listeners are attached; local typed events, one conditional slot, keyed reverse/append/remove, focus preservation and explicit listener disposal passed. Todo Browser/Search dependencies and asset hashes remain unchanged. The prototype is deliberately fixed-shape and is not a general renderer.

**M7c — 2026-10-09:** The isolated crate now drives that binding through one synchronous local graph. Native tests cover one-commit batching, weak ownership, deterministic capture release, later-pass re-entrancy, pass limits, recoverable reducer errors and faulted renderer errors. Chrome confirms the same behavior against real DOM with no network circuit.

**M7d — 2026-10-09:** The fixed row adds an optional controlled text input. Model-aware property updates and focused-input restoration preserve value, focus and selection through keyed moves. Chrome and Firefox verify removal/disposal listener cleanup and no API event circuit.

**M7e — 2026-10-09:** Complete native/wasm32/Chrome/Firefox regressions, the M7a workload and isolated M7b/M7c/M7d release fixtures pass. Production Browser/Search assets remain byte-identical at digest `3f3c5f074478c525`, and their dependency graphs exclude both experimental crates. Validated identity, slot, keyed-reconciliation, explicit-disposal and strong-owner/weak-handle patterns are candidates for later bounded extraction, but the current authoring, input, graph, focus and conditional-slot APIs remain experimental. This is a no-decision evidence freeze, not a renderer selection.

**M7f–M7h — 2026-10-09:** Renderer-neutral identity/key/slot safety contracts move to `yetlet-core`; the exact tested local graph moves to optional `yetlet-reactive`; keyed order, entry state and listener ownership move to independent `yetlet-dom-keyed`. The fixed template/input/focus/conditional API remains experimental. Independent crates preserve pre-adoption Browser/Search bytes.

**M7i — 2026-10-09:** Todo Browser adopts the graph and keyed owner for its local list. An explicit runtime lifetime now invalidates queued actions before they start and suppresses retired HTTP completions after disposal/remount. Chrome and Firefox preserve row/list identity and retain the typed HTTP/CSP/Search regressions. The repaired Browser runtime digest is `faebb0709dccd863`, with +82,726 raw / +20,042 local-gzip bytes relative to M7e; separately built Search JS/WASM remain byte-identical. Remote and Search result subtrees are not migrated.

**M7j — 2026-10-09:** Replacement validation after the first final review passes 91 workspace tests, 5 doctests, six wasm32 package checks, debug/release Chrome 155 and Firefox 157 production regressions, queued/in-flight lifetime races, the full M7a workload, isolated keyed lifecycle suites and a repeated release baseline. Median local observations are 0.037 ms SSR first byte, 49.0 ms Chrome app ready and 3.5 ms Search ready. Production digest is `faebb0709dccd863`; Search assets remain byte-identical. Repeat reviewer `wc_job_XZzdXEOIx6owj1o5` returned PASS with no issues or follow-ups.

M5 retains production compressed-WASM/SSR/mount baselines. M7a–M7e provide the research record; M7f–M7j make, adopt and freeze a bounded primitive choice. No benchmark winner, general renderer, unmeasured cross-framework comparison or production performance claim is implied. M7 is locally complete at experimental prototype scope.
