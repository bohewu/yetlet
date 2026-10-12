# ADR 0008 — Product Readiness and an Optional Scene Renderer

- **Status:** Proposed; planning only. No implementation or API promotion accepted.
- **Date:** 2026-10-12.
- **Task:** YT-069 / M11.
- **Related:** [M11 plan](../M11-PLAN.md), [product research](../M11-PRODUCT-READINESS-RESEARCH.md), [GPU plan](../M11-WGPU-PLAN.md), [ADR 0003](0003-renderers-and-loading.md), [ADR 0005](0005-opt-in-capability-modules.md), [ADR 0007](0007-bounded-keyed-dom-and-local-graph.md).

## Context

Five bounded studies are complete, and a new product-readiness handoff asks for useful full workflows, existing CSS reuse and precise XSS responsibilities. The native wgpu/Vulkan study proves colored instanced rendering, not browser portability or supported scene APIs. The owner requests planning for both product readiness and `yetlet-wgpu`.

## Proposed decision

Keep two independent M11 lines. Product readiness is established by one meaningful persisted, authorized, usable SSR-first application within an explicit operating envelope. Browser GPU work is an optional scene surface with its own lifecycle, performance, accessibility and support evidence. Neither is a prerequisite for publishing the original nine candidates, and GPU is not a prerequisite for the product.

Use the smallest layer that owns each concern. Safe View/HTML behavior remains a framework correctness contract. Selected live form semantics may justify bounded renderer helpers; existing CSS can remain an application asset choice, with a tiny additive server stylesheet helper only after repeated proof. Domain authorization, SQL, rich-content policy and deployment operations do not become mandatory core subsystems.

The proposed `yetlet-wgpu` candidate is first-party maintained, explicitly linked/initialized and Experimental. It does not enter core/HTML/default server/simple DOM dependency graphs. A shared HTML View is not forced to describe GPU scenes. Browser-first proof, async lifecycle/loss handling, dynamic matched measurements and semantic alternatives precede any crate extraction. A future general scene crate requires another materially different consumer.

## Alternatives not selected

- Built-in GPU for ordinary DOM/CSS: wrong semantics and unnecessary dependency/startup cost.
- Promoting all five studies into core: evidence does not justify the additional public contracts.
- A broad form/VDOM/hydration/head/CSS framework: exceeds the selected product needs.
- Requiring every native graphics backend, production HA, backups or public publication before a browser-only/private product proof: confuses target, application and release responsibilities.
- Declaring an official crate by moving the native benchmark into `crates/`: skips browser consumers, lifetime, fallback and API evidence.

## Consequences and acceptance

Optional integration increases maintenance and qualification combinations, so supported surfaces/targets must be explicit. Default no-JS SSR, local events, typed remote commands, CSP/escaping and unrelated browser assets are regression gates. Controls/text/IME/accessibility stay in semantic HTML rather than a newly built canvas UI toolkit.

This ADR remains Proposed until the owner authorizes a selected design and the relevant implementation/evidence passes the declared review gates. Documentation self-review is not independent implementation or security assurance. Existing YT-023 and M7/M10 results keep their original scope; YT-020 and the nine-candidate release inventory remain unchanged.
