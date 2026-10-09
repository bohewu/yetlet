# ADR 0007 — Bounded keyed DOM and explicitly owned local graph

- **Status:** Accepted and locally qualified at experimental prototype scope; reviewer-found lifetime repair and repeat M7 review passed
- **Date:** 2026-10-09
- **Related:** [ADR 0003](0003-renderers-and-loading.md), [Architecture](../ARCHITECTURE.md), [M7j verification](../M7J-VERIFICATION.md)

## Context

Yetlet originally used one small shared `View` model and whole owned-subtree replacement in the browser. M7a measured that baseline against a positional `View` patch, keyed direct DOM and runtime static slots. M7b–M7d then proved one fixed keyed authoring shape, explicit local graph ownership and cross-browser form-state/lifecycle behavior. M7e froze those results without selecting a general renderer.

The evidence supports preserving semantic keyed identity and explicit ownership for frequently updated local collections, but it does not justify promoting the fixed authoring API, a framework-wide VDOM, a template compiler or a dependency-tracked signals system. The architecture must also preserve HTML-first SSR, explicit typed HTTP actions, strict CSP and pay-for-what-you-use browser dependencies.

## Decision

### 1. Promote only renderer-neutral binding vocabulary into `yetlet-core`

`yetlet-core::binding` owns validated component/instance identity, semantic keys, slot names, inert attribute names and slot descriptors. These types define a trust-boundary vocabulary shared by SSR and browser bindings; they do not select a DOM algorithm, scheduler, macro DSL or hydration protocol.

### 2. Keep the local graph optional and explicitly owned

`yetlet-reactive` provides one synchronous local message graph with:

- one non-cloneable strong owner;
- weak dispatch handles and subscriptions;
- FIFO batches with one successful commit/notification per pass;
- bounded later-pass re-entrancy;
- recoverable reducer failures and faulted commit/pass-limit failures;
- deterministic disposal and capture release.

It is an opt-in message/reducer/commit primitive, not field-level dependency tracking, automatic effects, resources or an async scheduler.

### 3. Keep keyed DOM ownership in a separate optional crate

`yetlet-dom-keyed` owns:

- complete desired-key validation before structural mutation;
- semantic key-to-element/state ownership;
- retained identity across insert/remove/reorder;
- explicit browser-listener ownership and disposal;
- documented non-transactional callback/final-DOM mutation boundaries.

It remains separate from the simple `yetlet-dom` mount crate. Earlier attempts to place or feature-gate the keyed implementation inside `yetlet-dom` changed unrelated production bundle bytes before adoption. An independent crate preserves bundle isolation until an application explicitly opts in.

### 4. Adopt the primitives only for the production Todo local list

The `/app` Todo local list preflights the existing shared-view DOM, reuses matching keyed rows and dispatches typed `TodoCommand` values through one route-owned local graph. The existing `ActionExecutor<TodoCommand>` boundary is retained through a small graph-backed adapter.

The remote Create action remains an explicit `HttpExecutor` POST and renders into its separate owned host. Search Island and remote Todo result updates retain bounded subtree replacement. Renderer selection remains per owned surface rather than framework-global.

### 5. Make runtime lifetime explicit

The Todo browser route owns the graph, keyed entry state and local/remote listener owners. Each runtime also owns a shared lifetime token. Disposal or replacement invalidates that token before the graph/listeners are dropped: a queued action exits before starting, and an HTTP action that already yielded cannot update retired DOM after it completes. The current DOM remains readable; remount creates a fresh token and exactly one local and one remote action path. The external module bootstrap exposes only a frozen internal mount/dispose bridge for lifecycle control and tests.

### 6. Build browser entries independently

Todo Browser and Search Island are built in separate locked Cargo invocations before wasm-bindgen. This prevents Cargo feature unification from one browser application from silently changing another application's generated assets.

## Consequences

### Positive

- Local keyed rows preserve identity instead of being replaced on every update.
- Ownership and cleanup are deterministic and independently testable.
- Local messages remain local; remote work still crosses an explicit typed HTTP boundary.
- Search Island remains dependency- and byte-isolated from the Todo keyed graph.
- Simple pages and bounded subtrees can continue using the smaller direct mount path.

### Costs and limits

- The repaired production Todo Browser runtime grows by 82,726 raw bytes and 20,042 local-gzip bytes relative to the M7e release baseline.
- The keyed reconciler cannot transactionally roll back arbitrary caller mutations or unexpected DOM API failures in its final structural phase.
- The synchronous graph is single-threaded and has no async effects or automatic dependency tracking.
- The fixed `KeyedListTemplate`, current controlled-text-input API, focus restoration policy and conditional-slot shape remain experimental examples.
- The decision does not establish a stable public API or production-release qualification.

## Explicitly not selected

M7 does not select or implement:

- a general VDOM or positional tree patch as the default renderer;
- a public static-template authoring surface or `ui!` procedural macro;
- a general Signals/Effects/Resources framework;
- a generic forms runtime;
- full hydration or resumability;
- a default server-driven event circuit;
- WebGPU integration for ordinary DOM/CSS rendering.

These require separate evidence and bounded tasks rather than being inferred from the M7 benchmark.

## Verification basis

The decision is based on M7a–M7j evidence, including 91 workspace tests, 5 doctests, six wasm32 package checks, Chrome 155 and Firefox 157 debug/release regressions, the full retained render-strategy workload, isolated keyed lifecycle tests, dependency graph checks, exact bundle comparison and a repeated local release baseline. See [M7j verification](../M7J-VERIFICATION.md) for commands, measurements and limitations.
