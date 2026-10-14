# YT-075 — Ubuntu Firefox Browser Scene Island Qualification

**Date:** 2026-10-14. **Selected environment:** Ubuntu 26.04.1 LTS, kernel
`7.0.0-38-generic`, XFCE 4.20.4 on X11 `:0`, Firefox 158.0 Stable
(Snap), geckodriver 0.37.1, Intel UHD Graphics 770 (Raptor Lake-S).
**Selected browser execution:** **PASS**, running the actual Yetlet YT-075
Rust/wgpu-generated WebAssembly bundle, not a JavaScript scene recreation.
**Independent reviewer gate:** Fresh [review](YT-075-INDEPENDENT-REVIEW.md)
**PASS**, zero issues and zero required followups. This qualification
does not assert broad Linux/Chromium/NVIDIA support.

## Source and evidence identity

The source is the original Ubuntu Yetlet checkout, branch `master`, baseline`76be7aa1107e08e3c7850abc8c68f53301cf0f7c`, with YT-075 uncommitted
changes retained. The application-owned fixture is
`examples/webgpu-scene-island`, isolated from the root Cargo dependency
graph and not a promoted `yetlet-wgpu` crate. Browser source pins wgpu
`=27.0.1` with the `std`, `webgpu`, `wgsl` features, and
`wasm-bindgen =0.2.129`. The selected release build uses the existing
independently hashed scene digest `5eef120113321637` and counter
digest `81c388455a88d98a`; the scene WASM has 233,528 bytes.
The original native SSR server served the copied asset manifest and exact
bundle through loopback HTTP. Neither the root Cargo.lock nor the separate
Todo/Search artifacts was rebuilt or modified during this browser probe.

**Exact successful focused browser command:**

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_firefox.py
```

The completed immutable local receipt and logs are in
`target/m11-gpu/yt075-20261014/firefox-158-x11-window-owner/`:
`result.json`, `geckodriver.log`, `server.log` and the actual
`actual-canvas.png`. The result JSON SHA-256 is
`965471fa1b1eaf9deada9a94f92016f7f56c20251166b6931a5f585122617424`;
the PNG SHA-256 is
`53b92c2f7238b84791bf09d00e5fceb022324c67f362414b3d06dceaa48b67ad`.
No reported GPU state or pixel count is inferred from source alone.

Firefox Stable Linux requires the experimental `dom.webgpu.enabled=true`
preference. It was enabled **only** in an isolated disposable WebDriver
profile, not the user's regular browser or OS. No GPU blocklist override,
unsafe browser launch or sandbox bypass was used. Headless Firefox first
demonstrated real WebGPU command submission but a blank screenshot while
its log identified Software WebRender; using the existing X11 hardware
compositor avoided that presentation issue. Those earlier failed
evidence directories remain untouched. There was no Firefox Nightly
installation, host driver change or session replacement.

## Actual hardware and pixels

The geckodriver graphics log independently selected
`Intel(R) Graphics (RPL-S)`, `device_type: IntegratedGpu`, PCI
`0000:00:02.0`, `backend: Vulkan` and Mesa driver information.
The navigator request reported a non-fallback adapter. The browser ran
`RenderPass::draw 6 120 0 0` and completed queue submissions. The
GPU initializer reached `ready` only after the asynchronous validation
scope; no real scene validation error was observed on this selected pass.

The WebDriver screenshot of the actual `600 × 400` canvas was decoded:
**120/120 synthetic records** had the expected exact red/green/blue
center pixels, zero mismatches, with two independent black-background
points inside the surface. Firefox's element screenshot included a
one-pixel **white capture seam on row zero**, whereas row one and all
interior background points were black. The report retains the white
edge `[255,255,255]`; it must not be relabeled a renderer clear failure
or silently cropped away. The test asserted actual interior background
and all 120 record centers.

## SSR, security and independent sibling

The real application route `/scene` returned HTTP 200, `lang="en"`,
strict CSP without `unsafe-inline`, escaped/synthetic descriptions,
search, record links, pagination and meaningful HTML fallback.
An inert page did not fetch scene JS/WASM before explicit activation;
the scene had its own hashed loader and was independent of the counter
entry. The counter remained usable after GPU activation and after
injected GPU failures. No generic core/DOM GPU dependency, raw browser
event forwarding or required JS functionality was added to the default
SSR experience.

## Ownership and negative browser checks

The same browser receipt reports these actually executed checks:

| Check | Selected result |
| --- | --- |
| Explicit opt-in activation and single owner | PASS |
| Repeated activation while current owner exists | PASS, no duplicate starts |
| Owned disposal and fresh remount | PASS |
| Held acquired real device; retire old owner during pending initialization | PASS |
| Late device release destroys retired owner resource | PASS |
| Replacement owner status unaffected by old completion | PASS |
| Independent counter survives activation, retirement and failure | PASS |
| Injected absent `navigator.gpu` | PASS, semantic HTML fallback |
| Injected null adapter | PASS, semantic HTML fallback |
| Injected rejected device request | PASS, semantic HTML fallback |
| Injected initialization validation error | PASS, semantic HTML fallback |
| Missing scene WASM in disposable asset copy | PASS, server omits scene loader |
| Invalid manifest in disposable asset copy | PASS, strict-CSP semantic SSR |

The injected cases are explicitly **simulated negative paths**, not a
claim of actual device loss on this hardware. The delayed-owner test
used a genuinely acquired GPU device to verify late destruction. The
harness checked the independent counter and active owner after
completion. Full loss/recovery, DPR and dynamic resizing remain
**YT-076**, and quantitative Canvas2D comparisons remain **YT-077**.

## Cleanup and prior failure history

The successful test exited 0, explicitly deleted its WebDriver session,
reaped the geckodriver process (exit 0), stopped its owned server
(signal 15), and removed its private temporary profile. Post-test
process inspection reported **no remaining owned YT-075 processes**.
The source checkout, user's active browser profile and display login
were preserved. This was a single-user-local qualification and makes
no production-availability guarantee.

Earlier attempts must remain historical rather than be overwritten:
seven Chrome-based Ubuntu configurations could not satisfy actual
drawing, including NVIDIA Vulkan shared-image texture failures,
Intel headless null adapters, and NVIDIA nested Weston Wayland; two
Windows Chrome/Edge **standalone JavaScript** WebGPU baselines passed
120/120 centers but did not run Yetlet WASM. Firefox's own previous
runs also exposed a WebDriver null-value handler defect, headless
Software WebRender presentation limits, a screenshot capture seam,
and delayed-device injection scope errors. Each was retained under a
separate local ignored evidence folder. The successful receipt and
actual Yetlet WASM now establish a working **selected Linux browser**
rather than support for those failed configurations.

## Qualification boundary

The selected browser **runtime gate is PASS** and a fresh
[independent read-only reviewer](YT-075-INDEPENDENT-REVIEW.md) returned
**PASS** against the source and receipts. **YT-075 is Done only at this
selected experimental browser scope.** This record does not
automatically start YT-076, qualify loss/recovery or dynamic performance, promote
a public GPU crate, authorize release, or justify commit/push.
