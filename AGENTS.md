# Yetlet — repository agent instructions

## Intent and scope

Build a **small, HTML-first Rust universal UI framework**. The guiding contract is in `README.md` and `docs/MVP.md`; architecture, proposal status, milestone gates and task dependencies are in `docs/ARCHITECTURE.md`, `docs/adr/`, `docs/ROADMAP.md` and `docs/BACKLOG.md`.

This repository is currently in the design / pre-MVP phase. Do not assume undocumented crate interfaces exist.

## Documentation language

Write **all repository documentation in English**, including the README, ADRs, architecture, roadmap, backlog, validation records, and future Markdown files. Preserve the original technical meaning, acceptance evidence, links, IDs, and decision status when editing. Run `python3 scripts/check_docs.py` to verify language, references, and backlog structure.

## Work sequence

1. Read this file, the relevant ADR(s), the targeted backlog item(s), and any higher-priority workspace policies before making changes.
2. Implement one bounded vertical slice in the **current checkout** by default; do not create a detached worktree unless expressly requested.
3. Keep server HTTP routing, UI component semantics, HTML/DOM renderers, browser loader, and command transport separable.
4. Add tests that demonstrate the acceptance criteria for the item; report compile / browser / CSP verification evidence separately.
5. Only mark backlog items Done when evidence exists. Update roadmap gates based on completed work, not intentions.
6. Do not commit, push, deploy or alter external environments without explicit instruction. Preserve unrelated work.
7. When changing a fundamental architectural decision, update the relevant ADR and explain tradeoffs.

## Technical guardrails

- Default SSR must work without JS/WASM and without a persistent server-side event circuit.
- Client events stay local; remote behavior crosses a typed HTTP/action boundary unless an optional custom transport is installed.
- Do not evaluate code or compile Wasm on each request; serve build-time versioned assets.
- Strict CSP, HTML/state escaping, semantic HTML and accessibility are correctness requirements.
- Do not put server-only secrets or capabilities into browser builds or serialized hydration state.
- WebGPU is a separate optional renderer, **not** an accelerator for normal DOM/CSS.
- Do not replace a mature HTTP parser/server implementation with an in-house transport stack.

## Work reporting

Include: task IDs, actual changed files, validation commands/results, known limitations, open decisions, and the next unblockable backlog item. If a toolchain or dispatch mechanism is unavailable, explicitly separate completed repository work from validation that was not run.
