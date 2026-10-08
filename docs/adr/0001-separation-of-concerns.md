# ADR 0001 — Separate execution, rendering, interactivity, and transport

- **Status:** Proposed for MVP
- **Date:** 2026-10-08
- **Owners:** Project maintainers
- **Related:** [Architecture](../ARCHITECTURE.md), [MVP](../MVP.md)

## Context

Many full-stack frameworks couple component rendering, browser hydration, client/server communication, and a specific HTTP server. Yetlet needs a shared Rust component model for native SSR and browser UI without forcing every page to pay for an unnecessary client runtime or persistent remote event circuit.

## Proposal

Keep four independent dimensions: **execution target** (native or WASM), **renderer** (HTML, DOM, or scene), **interactivity** (none, island, or full browser), and **action executor** (local, HTTP, or custom). Validate supported combinations against target capabilities rather than accepting impossible runtime configurations.

Use an existing HTTP router and middleware pipeline for APIs. SSR may call native domain capabilities directly, without loopback HTTP calls. The framework must not establish a persistent session/circuit or proprietary event WebSocket by default.

For the MVP, validate three presets:

1. Native static SSR without a browser runtime.
2. Native SSR with an optional interactive browser island.
3. Full browser UI backed by an explicit HTTP data API.

## Expected benefits

- Static pages have zero browser application-runtime cost.
- SSR and CSR reuse component semantics instead of duplicating UI definitions.
- Event/action transport can change independently of the UI renderer.
- A future Canvas/WebGPU renderer can be added without affecting normal HTML/DOM behavior.

## Alternatives considered

- **Blazor Server/LiveView-like remote event circuit:** Convenient programming model, but requiring network round-trips for routine interaction conflicts with the local-event-first principle.
- **CSR-only:** Simple architecture, but poor alignment with immediately usable HTML and minimum-runtime goals.
- **Static SSR-only:** Very small client footprint but does not support the required stateful browser applications.
- **One universal runtime binary:** Native and browser capabilities differ substantially; this approach risks introducing server-only dependencies into browser bundles.

## Risks and validation

The combination matrix could grow without bounds. The MVP must expose only the three validated presets, each with a smoke test. Do not advertise configurations without evidence. If separating all four dimensions produces excessive adapter boilerplate, simplify the public configuration without coupling internal responsibilities.

## Consequences

Developers explicitly select interactivity and remote action boundaries. Internal traits/modules must stay small and well-defined. This ADR remains a **proposal**, not a finalized public API contract.
