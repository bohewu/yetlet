# Browser application DX research

Status: maintainer research, 2026-11-07. Inspected Yetlet source:
`bebdd03660ca7dc33fdd1ed4c7e56b5680e055c1`, with published SDK
`0.9.0-alpha.6` and CLI `0.2.0`. The owner-selected followup is one documentation
batch: connect the existing browser starter, tool preparation, local interaction
and static deployment paths in [Build a browser-only Yetlet app](BROWSER-APP.md).
The observed gaps concern discoverability and explanations of existing features;
this study has not established a required new API. SDK/CLI implementation and
desktop-host integration are outside this delivery.

## Evidence and scope

This document reviews Yetlet's own CLI browser template, DOM/event APIs, current
guides and published qualification records. The sources below establish the
existing capabilities and their limits. The selected changes improve navigation
and explain a connected application workflow; they do not qualify a new runtime
or platform. No product checks are replayed for this documentation batch.

## Existing capabilities to reuse

| Need | Existing source | DX conclusion |
|---|---|---|
| A browser app without an SDK checkout or native HTTP server | [CLI creation choices](CLI-GUIDE.md#creation-choices-and-settings) and [browser template setup](../tools/yetlet-cli/templates/browser/README.md) | `yetlet new NAME --template browser` already selects the browser target and published packages. A new generator is not needed to establish this path. |
| WASM build, static assets and development loop | [Browser build helper](../tools/yetlet-cli/templates/browser/build-wasm.mjs), [Vite configuration](../tools/yetlet-cli/templates/browser/vite.config.mjs) and template setup | Existing build/run/watch behavior, bindgen version checks, relative asset paths and `frontend/dist` ownership should be explained together. Vite preview is not a native package or production server. |
| Loading, failure and page lifetime | [Browser entry](../tools/yetlet-cli/templates/browser/app.ts) and [startup shell](../assets/wasm-startup.mjs) | Reuse the existing startup shell and disposal handling rather than introducing another loader. |
| Local state, Rust events and stable collections | [Browser Rust template](../tools/yetlet-cli/templates/browser/lib.rs) and [dom-keyed guide](../crates/yetlet-browser/guides/dom-keyed.md) | The template already owns a local graph, keyed rows and `DomEventListener` handles. There is no demonstrated need for a new event registry or remote event circuit. |
| Explicit ordinary DOM listener lifetime | [DOM guide](../crates/yetlet-browser/guides/dom.md) and [documentation search caller](../examples/docs-site/src/browser.rs) | A small caller-owned Rust closure is also an existing option. Explain replacement, rebinding, disposal and focus tradeoffs. |
| Build cache and evidence separation | [Development artifact guide](DEV-ARTIFACTS.md) | Maintainer cache/evidence tooling already exists; generated app assets and their own ignored directories remain application-owned. Do not add a second cleanup mechanism. |

[Static HTML export](FIRST-APP.md#next-export-a-static-site) and the browser-only
target serve different needs. The former emits readable build-time HTML; the
latter starts an interactive app after JavaScript/WASM loads. A browser-first
guide should make that choice early and link the existing first-app paths.

## Selected documentation gaps

### A connected browser-first entry guide

The browser workflow is implemented, but readers must join the CLI guide,
generated README, DOM guide and lifecycle examples. The public first-app guide
starts with native SSR. A short browser-first walkthrough should connect the
existing creation, lock/cache preparation, build, preview, local interaction and
ship-directory steps. It should retain the matching bindgen requirement and
separate native, browser and static-export targets. This is the smallest useful
followup and does not require an SDK or generator change. The new
[browser walkthrough](BROWSER-APP.md) provides this connection in the current
repository; it does not select another live documentation-site chapter.

### Explicit preparation and output ownership

The CLI/template already documents matching Rust/bindgen tools, separate lock
preparation, frozen builds and a frontend ship directory. The walkthrough puts
those steps in execution order, identifies generated file ownership and links
safe stage-specific recovery. This addresses an integration-documentation gap,
not a missing installer, build stage or cleanup API.

### State, events and DOM lifetime in one flow

The existing sample owns a local graph, typed commands, keyed rows and listener
handles. The DOM guide already distinguishes subtree replacement from keyed
identity and application-owned input/focus handling. The walkthrough connects
these sources, with a small caller-owned closure as an alternative when a graph
is unnecessary. No new event registry or state abstraction is selected.

## Delivery and evidence reuse

The current batch adds the browser walkthrough and links it from the existing
CLI, first-app, DOM, opt-in and troubleshooting guides. It also corrects the
keyed DOM guide's current distribution description while retaining its historical
delivery text. These are documentation changes, not new capability qualification.

Retain the source/limits of [browser starter evidence](YT-104-BROWSER-STARTER.md),
[Linux watch acceptance](YT-112-IMPLEMENTATION.md) and the
[published CLI consumer](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02).
Older statements in dated records keep their original checkpoint meaning.
Only affected document checks and independent review are selected for this batch;
no accepted build, browser, watch or platform check is replayed.

Further SDK API work should start from a concrete, separately scoped limitation.
The current batch changes documentation and selects no runtime, dependency,
scaffolding target, policy relaxation or generalized desktop support.

This study uses Yetlet repository inspection and linked public evidence at the
stated source. It adds no backlog Done item, tool installation, product execution
or new platform qualification.
