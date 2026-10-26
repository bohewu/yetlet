# WebAssembly startup and customizable loading UI

Status: Experimental opt-in JS/CSS helper and minimum consumer adoption under
YT-196, independently accepted after bounded lifecycle repair. No new Rust
crate/Core API, mandatory
native asset inclusion or WebGPU dependency. Original YT-195 contract and
historical qualification remain scoped to their recorded source.

## Prerequisites

Loading feedback must exist in the initial HTML, with its stylesheet linked
from that HTML. It cannot depend on downloading, compiling or mounting the Wasm
that it describes. This covers the Wasm-specific waiting period after HTML is
available; it cannot remove network delay before the document itself arrives.

Ordinary DOM/Wasm applications need JavaScript, WebAssembly and correctly served
JS/CSS/Wasm assets with the application's CSP. They do not inherently require
WebGPU. A GPU consumer separately checks its secure context, WebGPU capability
and usable adapter/device. See [the physics lab guide](../examples/physics-lab/README.md)
for those additional conditions.

## Minimal integration contract

| Mode | Initial visible surface and start trigger |
|---|---|
| Browser-only Wasm application | A branded HTML shell, readable loading status and disabled dependent controls appear before Wasm. The explicitly selected browser entry may start preparation on page load |
| SSR with optional Wasm island | Keep readable SSR and unrelated actions usable. The initial island shell stays inert until its declared user interaction; loading feedback belongs to that island |
| Mostly-Wasm fullscreen application | The same pre-Wasm HTML shell may occupy the page. Its size/style is an application choice, not a mandatory framework overlay |

A site-owned HTML shape can contain a loading status, decorative indicator,
no-JS explanation and a static recovery instruction/link. Use external CSS and
an external bootstrap script under the existing CSP. A label still renders if
animation or CSS fails. No generated percentage is displayed without a measured
progress model.

The bootstrap owns these transitions:

1. Keep one pending initialization per owned root; publish a loading/busy state
   without replacing unrelated SSR or enabling dependent actions.
2. Load the selected build-time module, instantiate Wasm, then mount/bind the
   intended UI. Download or init completion alone is not application readiness.
3. Publish ready only after mount succeeds. Clear the busy state and remove or
   replace that root's loading decoration; avoid taking keyboard focus merely
   because loading finished.
4. On error, retain useful HTML, show a bounded problem/next-action message and
   offer the selected explicit retry/reload path. Never expose raw stack/URL data
   as a user-facing diagnostic. No automatic retry loop is implied.
5. Before retry/replacement/disposal, retire the prior owner and outstanding
   borrows. Ignore stale ready/error callbacks; late completion must not clear
   a newer failure or mount a second runtime.

The initial shell must also explain what to do if the bootstrap script itself
never runs. A catch block in that missing script cannot recover it, and
noscript only detects disabled JavaScript. Static recovery text remains useful
for a blocked/missing entry module. A timeout is a named failure, not a
fabricated completed percentage or permission to free an outstanding borrow.
Reload explicitly discards tab-local application state.

## Expected result and customization

Before Wasm is ready the visitor sees the application identity, a concise loading
message and useful HTML/recovery instructions rather than an empty mount root.
An optional spinner/skeleton is decoration; an accessible status and aria-busy
communicate the wait without continuous announcements. Respect reduced motion,
keep text contrast/focus usability and avoid animation as the only signal.

Applications own the initial HTML content, brand, text/localization, external
CSS/theme, indicator choice and page/island placement. The implemented `createStartupShell` receives explicit root/status/indicator
and optional owned fieldset/text messages. It imposes no brand, page layout,
CSS framework, WebGPU or mandatory Core feature. Canonical assets live at
[wasm-startup.mjs](../assets/wasm-startup.mjs) and
[wasm-startup.css](../assets/wasm-startup.css).

Do not replace meaningful SSR with a full-page splash by default. Where an app
intentionally has no functional UI until Wasm mounts, its page-level shell is
an explicit application choice under the same lifecycle contract.

## Configuration, errors and current coverage

[ADR0003](adr/0003-renderers-and-loading.md) already separates native SSR,
interaction-triggered islands and tiny external bootstrap JS. The current
[CLI browser template](../tools/yetlet-cli/templates/browser/index.html) and
[browser tasks HTML](../examples/browser-tasks/frontend/index.html) ship loading
text/disabled controls before Wasm; their Rust mount enables the workspace after
binding. YT-196 checks the changed startup seam; the prior Tasks business and CLI
generation matrices are not replayed.

YT-196 links both stylesheets from initial HTML, moves generated module/Wasm URL
imports inside the bootstrap try boundary and preserves the watch generation
rewrite for those dynamic imports. The Tasks entry checks page ownership after
async boundaries, mounts once and then marks ready. Its app-owned disposal
export retires listeners/graph; a late initialized runtime is disposed without
mounting. The template's manual recovery is reload; it offers no in-page retry
of a partially mounted runtime. The shared helper does not own these resources.
Only the new adoption/startup seam is checked, not the full prior CLI/Tasks matrix.

The physics lab adopts the regional loading/error pattern in
[YT-195](YT-195-BROWSER-STARTUP-GUIDANCE.md) and the same shared helper in
[YT-196](YT-196-WASM-STARTUP-SHELL.md). It remains an optional GPU consumer,
not the definition of all Wasm applications. More lab scenarios and visible FPS
are separately recorded owner feedback; neither is part of this startup slice.

## API reference

`createStartupShell({ root, status, indicator?, controls?, messages? })` creates
one presentation owner for an explicit root, without mutating its initial HTML
or loading anything. `controls` is an optional fieldset containing only the
Wasm-dependent controls; unrelated SSR controls must stay outside it. The
controller never queries a global selector or replaces DOM with innerHTML.

| Method/property | Contract |
|---|---|
| `begin(message?)` | Return an opaque ticket and show loading/busy; return null if another startup is pending or this controller was disposed |
| `ready(ticket, message?)` | Only the current loading ticket may mark ready; default empty ready text hides status, hides indicator and enables the supplied fieldset |
| `fail(ticket, message?)` | Current loading/ready ticket may show bounded text error and disable dependent controls; a later ready cannot erase it |
| `cancel(ticket, message?)` | Retire that ticket into idle; no underlying promise is canceled or resource freed |
| `dispose()` | Permanently invalidate this controller/tickets and release root ownership; later callbacks cannot affect an explicitly new controller |
| `state` | Read-only idle/loading/ready/error/disposed presentation state |

`messages` selects loading/ready/error text; initial HTML text is the default
loading message. Strings use textContent, never HTML parsing. Brand/logo/custom
markup stays in trusted initial HTML, independent of helper startup. Optional
CSS classes are `wasm-startup-panel` (a direct child of its owned root in the
default style) and `wasm-startup-indicator`. Accent/track custom properties
control colors, reduced motion disables rotation, and applications may replace
the indicator entirely. Keep its accessible status and initial recovery text.
The initial root may declare `data-startup-state="loading"` for a browser-only
page or idle for an interaction-triggered island. A full-page splash is an
explicit application CSS choice; native SSR never acquires one automatically.

The caller must await actual retirement before retrying or replacing Wasm/GPU
resources. Ticket invalidation only rejects stale UI updates; it cannot cancel
an arbitrary import or borrowed Wasm future. The Lab retains its existing
cleanupPending/retirement guards and permits explicit GPU Retry only when safe.
Do not call begin for a second runtime while the first still owns resources.

Current examples are the [browser entry](../tools/yetlet-cli/templates/browser/app.ts)
and [Lab entry](../examples/physics-lab/loader.js). Shared asset publication is
explicit in the [CLI emitter](../tools/yetlet-cli/src/browser.rs) and
[Lab builder](../examples/physics-lab/src/bin/build-assets.rs). Existing feature
boundaries, CSP, escaping, build-time assets and dependency separation apply.


## Recorded Back/Forward cache coverage

The ready Tasks consumer has additional actual local Firefox154 evidence for
cached Back and Forward restoration: persisted pageshow, unchanged document/
model, retained completion and usable controls. The
[YT-196 followup](YT-196-WASM-STARTUP-SHELL.md#actual-browser-backforward-cache-followup)
records the fixture, preserved failures and automation unload interference.
This is bounded ready-page evidence; browsers may decline or evict cached pages.
Pending initialization, long retention, other browsers and GPU consumer cache
restoration are outside that proof. Application loaders still own their actual
resource/session restoration policy; the helper does not promise browser caching.
