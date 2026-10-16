# YT-075 — Browser Scene Island Verification

**Date:** 2026-10-14. **Current browser gate:** **PASS** on a selected
Ubuntu Firefox 158 X11 experimental-opt-in configuration, using actual
Rust/wgpu 27.0.1 scene Wasm, 120/120 real colored canvas pixels, semantic
SSR, counter isolation, pending-owner retirement, negative fallback and
owned cleanup. **Overall YT-075: DONE at the selected scope**, after a fresh
[independent reviewer PASS](YT-075-INDEPENDENT-REVIEW.md) with zero issues and
zero required followups. The earlier reviewer FAIL and seven failed Ubuntu Chrome configurations
remain valid historical evidence; they are not the current Firefox result.
[Selected Firefox qualification](YT-075-UBUNTU-BROWSER-VERIFICATION.md)
records source, platform, exact receipt and limitations.

## Historical pre-Firefox browser outcomes and continuation boundary

The owner additionally authorized continuing the dependent M11-G items only if
YT-075 is feasible, with reviewer-role review after each item. That conditional
scope does not waive YT-075's actual browser-draw requirement. YT-076 onward has
not started. YT-074, the YT-081 dependency and YT-020 remain unchanged.

The same original checkout remains on master at `76be7aa`, with uncommitted
fixture/documentation changes and an empty index. The leaf generalist Job
`wc_job_lB0DZOKr2b4oS7SJ` completed with native exit 0. Its exact Codex thread
was `01a13910-da0e-75f1-8993-3891cc41ae5e`; saved/observed model
`gpt-6.1-sol` and effective `high` matched. This is a role-directed Job result,
not native managed-role identity. Parent changes and browser results below are
separate from that implementation/build result.

Three bounded Chrome 155.0.8059.79 attempts completed with native exit 1; none
remains active or uncertain. All used isolated headless Chrome, a uniquely
returned page target and an owned loopback server/profile. No GPU blocklist,
sandbox bypass, driver installation or host-setting change was used.

| Retained directory under target/m11-gpu/yt075-20261014 | Selected configuration and observation | Result |
| --- | --- | --- |
| browser-initial | No DISPLAY; enable-gpu and ANGLE Vulkan. RTX 4090 graphics renderer, Chromium Vulkan feature off; native navigator adapter null. | No drawing; application unavailable state with HTML records retained. |
| browser | Same configuration plus Chromium Vulkan feature. Native NVIDIA/Lovelace adapter, fallback false; actual scene reached ready. Chromium rejected external texture import: allocation 1011712 bytes versus required 1245184 bytes, followed by invalid texture/command buffer. | First rectangle-center pixel assertion failed; no correct drawing claim. |
| browser-x11 | Newly built validation-error correction; existing owner X11 DISPLAY :0 and enable-gpu/ozone-platform=x11. ANGLE selected Intel/Mesa RPL-S; native navigator adapter null. | No drawing; unavailable state with HTML records retained. No JS exception or failed HTTP response recorded. |

Each attempt reached the no-JS semantic document check and retained a no-JS
capture. Those checks and unavailable text do not establish the later injected
failure/lifetime/sibling assertions: the scripts stopped at the real-draw gate,
before those assertions. The second attempt's ready status was not success.
A direct screenshot-image retrieval was rejected by the artifact policy; the
image was not accessed through another path or tool. Visual inspection remains
unverified; actual test failures and safe text diagnostics are retained.

### Subsequent owner-authorized NVIDIA-only ICD diagnostic — 2026-10-14

One separately authorized, bounded Chrome attempt tested **process-only** GPU
selection after the three earlier terminal runs. The opt-in
`YT075_NVIDIA_ICD_DIAGNOSTIC=1` profile used X11 `:0`, the NVIDIA
`VK_DRIVER_FILES=/usr/share/vulkan/icd.d/nvidia_icd.json` override,
`__NV_PRIME_RENDER_OFFLOAD=1`, `__VK_LAYER_NV_optimus=NVIDIA_only`,
`--use-angle=vulkan` and `--enable-features=Vulkan`. Chromium sandbox and
existing display/server policies remained intact. This mode preserves the
older evidence and writes to
`target/m11-gpu/yt075-20261014/browser-nvidia-icd/`.

**FAIL, native exit 1:** Chrome reported the NVIDIA RTX 4090 ANGLE/Vulkan
1.4.329 renderer and enabled WebGPU, but `navigator.gpu.requestAdapter()`
returned `null`. The owned Chrome log recorded `vkCreateInstance() failed: -7`
and a failed Vulkan initialization. Vulkan code -7 means
`VK_ERROR_EXTENSION_NOT_PRESENT`; Chrome's exact requested extension was not
captured. A separate read-only Vulkan loader extension enumeration returned
27 instance extensions with the ordinary ICD discovery and 26 with NVIDIA
alone. The sole difference, `VK_EXT_headless_surface`, appeared only with
ordinary discovery. This is a **plausible** missing-extension clue, not proof
that Chrome requested it or that restoring it resolves the earlier NVIDIA
external-memory texture-import failure.

No correct GPU pixels or completed fault injections were obtained. No-JS SSR
stayed readable; the counter remained at baseline 2, with zero observed
JavaScript exceptions and HTTP failures. Owned Chrome exited 0, the server
terminated by signal 15, the private profile was removed, and cleanup reported
zero errors. All three prior result directories remain intact. No sudo,
installation, global host settings, additional browser run, commit or push
was involved. The earlier independent reviewer verdict remains FAIL;
YT-075 is still Blocked and YT-076 remains unstarted.

### Owner-authorized PRIME Offload with default ICD discovery — 2026-10-14

A further **single bounded browser execution** followed the NVIDIA-only
diagnostic. The explicit `YT075_PRIME_OFFLOAD_DIAGNOSTIC=1` profile kept
normal Vulkan loader ICD discovery (cleared `VK_DRIVER_FILES` and
`VK_ICD_FILENAMES` in the owned Chrome environment), selected NVIDIA via
`__NV_PRIME_RENDER_OFFLOAD=1`, `__VK_LAYER_NV_optimus=NVIDIA_only`, and
`__GLX_VENDOR_LIBRARY_NAME=nvidia`, and retained the existing X11 `:0`,
`--use-angle=vulkan`, `--enable-features=Vulkan` and normal Chromium
sandbox. Earlier evidence was preserved in separate directories; the new
ignored evidence is `target/m11-gpu/yt075-20261014/browser-prime-offload/`.

**FAIL, native exit 1, but adapter acquisition improved:** native
`navigator.gpu.requestAdapter()` reported NVIDIA / Lovelace and
`fallback=false`. ANGLE identified the RTX 4090 with Vulkan 1.4.329;
Chromium reported Vulkan/WebGPU enabled. The preceding NVIDIA-only
`vkCreateInstance -7` error did not recur. However, the application GPU
initialization returned `unavailable` and Chrome logged
`Error creating wgpu::Texture`. No completed GPU pixels were observed.
Unlike the earlier Vulkan allocation failure, the new Chrome log did not
include an allocation-size pair, so their precise failure mechanisms
cannot be claimed identical. The retained error-scope fix surfaced this
failure rather than allowing a false successful drawing verdict.

No-JS SSR remained usable; the counter stayed at baseline 2; HTTP
failures and JavaScript exceptions were zero. The later device-failure,
owned-retirement/replacement and sibling-interaction injections were
not reached (zero completed fault cases), and independent acceptance
review remains unmet. The owned Chrome process exited 0, the test
server ended via signal 15, the private profile was removed, and
cleanup errors were empty. The result SHA-256 is
`d5ffc8f6e4a4d7dd32c047037878b9895341765c8669fa072018f8d7cace3ec0`.
No admin rights, driver/config changes, test repetition, push or commit
were used. The earlier independent reviewer FAIL remains; YT-075 is
Blocked and no YT-076 work is activated.

### Owner-authorized Intel Gen12 Chrome diagnostic — 2026-10-14

The user asked whether Firefox or the integrated GPU could work. The
installed Firefox is 158.0 Stable (Snap), and no Firefox Nightly installation
was found. The GPUWeb [implementation status](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)
still reports Linux Firefox WebGPU in Nightly rather than Stable/Beta at
this date. No Firefox browser session, preference override or installation
was attempted.

Chrome 144 introduced a supported Linux WebGPU path for Intel Gen12+ GPUs
([Chromium announcement](https://developer.chrome.com/blog/new-in-webgpu-144)).
The installed Intel UHD 770 (PCI 8086:a780, kernel i915) meets this
hardware generation, and installed Chrome is 155.0.8059.79. One isolated
experiment chose the integrated GPU in the **owned browser process only**
using `YT075_INTEL_GEN12_DIAGNOSTIC=1`, `DRI_PRIME=0` and
`MESA_VK_DEVICE_SELECT=8086:a780!`. The default Vulkan ICD discovery was
preserved; prior NVIDIA offload variables were omitted; Chrome used X11
`:0`, `--enable-gpu` and `--use-angle=gl` without forcing the Vulkan
compositor. Normal Chromium sandbox and strict application CSP remained.

**FAIL, native exit 1, no WebGPU drawing.** Chrome's ANGLE renderer reported
Intel/Mesa RPL-S OpenGL 4.6 (Mesa 26.0.8), and
`webgpu_on_vk_via_gl_interop` and WebGPU were shown enabled. However,
`navigator.gpu.requestAdapter()` returned `null`, and the scene correctly
entered `unavailable`. The GPU inventory still mentioned both PCI GPUs;
no assertion of exclusive device visibility or successful Vulkan rendering
is made. No PNG pixel assertion or fault-injection/lifetime scenario was
reached. The log contained no additional Vulkan/texture errors in this
run. The single result is not proof that Intel hardware lacks WebGPU
support, only that this **actual Chrome/headless/driver configuration**
failed to acquire the adapter.

The no-JS searchable/paginated SSR content remained intact, with zero
observed HTTP failures/JavaScript exceptions and the independent counter
at baseline 2. Browser exit was 0, the owned server exited by signal 15,
the private profile was removed, and cleanup reported no errors.
Evidence is retained separately under
`target/m11-gpu/yt075-20261014/browser-intel-gen12/`;
`result.json` SHA-256 is
`b6f56177d74cf6dde085f70b1aa738ae04646937935ec3fbece68c44e942ac1c`.
All five prior result folders remain intact. No admin action, host setting
change, Firefox installation, second browser attempt, commit or push
was performed. YT-075 remains Blocked; the earlier reviewer FAIL and
YT-076 dependency gate remain unchanged.

### Owner-installed Weston: nested X11 Wayland and NVIDIA browser proof — 2026-10-14

The owner installed Weston 14.0.2. The original XFCE/X11 login remained
active; no reboot, session replacement, host GPU configuration or privileged
operation was performed by this run. A first short, owned preflight launched
Weston with the X11 backend, GL renderer and a unique Wayland socket.
The socket came up, the GL renderer was Intel/Mesa, and the compositor
exited cleanly, removing its socket. This establishes the feasibility of
a nested Wayland compositor but does not qualify GPU rendering.

For the one actual browser attempt, the parent launched another isolated
Weston X11 backend with process-local NVIDIA PRIME offload variables,
`__NV_PRIME_RENDER_OFFLOAD=1`,
`__GLX_VENDOR_LIBRARY_NAME=nvidia` and
`__VK_LAYER_NV_optimus=NVIDIA_only`. The Weston log confirmed
**EGL vendor NVIDIA**, **GL vendor NVIDIA Corporation**, and
**GL renderer NVIDIA GeForce RTX 4090/PCIe/SSE2**. An owned headless
Chrome 155 process then used that unique socket
`wayland-yt075-weston-browser` via `--ozone-platform=wayland`,
with normal Vulkan ICD discovery, `--use-angle=vulkan` and
`--enable-features=Vulkan`. The normal Chromium sandbox and application
CSP were preserved. The separate browser harness opt-in was
`YT075_NESTED_WESTON_DIAGNOSTIC=1`; no global Wayland session was created.

**Browser result: FAIL, native harness exit 1.** Chrome's renderer was
ANGLE/NVIDIA Vulkan 1.4.329, and
`navigator.gpu.requestAdapter()` returned actual
NVIDIA/Lovelace metadata with `fallback=false`. Nevertheless,
the application initialization returned `unavailable`; the Chrome
GPU-service log recorded `Error creating wgpu::Texture`.
No correct scene draw or pixel check was obtained. The error is
consistent in broad failure stage with the previous NVIDIA/PRIME
texture failures, but this log does not give a matching allocation-size
pair or prove an identical underlying cause. The no-JS HTML alternative
passed; the standalone counter remained 2; there were zero observed
HTTP failures or JavaScript exceptions. Subsequent fault-injection,
asynchronous retirement and sibling-interaction assertions were not
reached, so browser lifecycle acceptance remains open.

The owned Chrome process exited 0, the HTTP fixture server was stopped
by signal 15, its browser profile was removed without cleanup errors,
the Weston process exited 0 and its Wayland socket disappeared. Results
were recorded in separate ignored directories:
`target/m11-gpu/yt075-20261014/weston-x11-preflight/`,
`target/m11-gpu/yt075-20261014/weston-x11-browser-launch/`, and
`target/m11-gpu/yt075-20261014/browser-weston-x11/`.
The browser result JSON SHA-256 is
`de4455af65dc9102a0888afc4a5e029ef078cdcdc22f48635e334d7fe5a08990`.
Prior evidence was preserved. **Nested Weston using the X11 backend
is not the same as an actual NVIDIA Wayland desktop/DRM compositor
session**, so this test does not establish the support status of that
different environment. There was one Chrome attempt and no automatic
retry; no installer/driver or source bundle change, commit or push occurred.
The earlier independent review FAIL and YT-075 Blocked gate remain.

### Windows Chrome/Edge hardware WebGPU baseline — 2026-10-14

The owner requested cross-platform qualification following seven failed
Linux Chrome scene-island attempts. WebCodex reached the existing Windows
runner project `agent:ncut-bohe-c420c5de1cac4bd4:yetlet-360e2631`,
in `C:\\repos\\yetlet` on a **clean** historical
`maint/m10-qualification-20261010` checkout at
`3128d23591feee15ce976d7b83a56dcd587048d7`. This checkout does
**not** contain the current uncommitted Ubuntu YT-075 fixture, and the
Windows Rust installation was 1.88 with only
`x86_64-pc-windows-gnu` installed, no wasm32 target/wasm-bindgen CLI.
The generated Ubuntu artifact path was rejected by WebCodex's project
artifact policy. No policy workaround, source transfer, tool install,
checkout/branch change, commit or push was attempted.

A separate **browser-capability baseline** was therefore executed on
Windows using Python 3.14's standard library and the existing WebCodex
repo's CDP transport. The temporary localhost HTTP server enforced
external script loading and strict CSP, with an accessible text fallback.
Its JavaScript uses the **exact YT-075 WGSL shader bytes** (SHA-256
`bc57bafa70fd2f835f33c101128f637764f3aa49016aa5b7fe2c09a24263ef3f`)
and the same 120 synthetic records, packed as two float32x4 instance
attributes with the same positions, sizes and red/green/blue statuses.
It creates a WebGPU device, GPU vertex buffer, shader, render pipeline,
canvas surface and submit; awaits queue completion, checks the GPU
validation error scope, and decodes a real CDP PNG screenshot to check
all **120 colored rectangle centers and the black background**.

| Windows browser | Hardware adapter | Actual GPU draw / pixel result | Browser test |
| --- | --- | --- | --- |
| Google Chrome 155.0.8059.40 | Intel / Gen12LP, fallback false | 120/120 colored centers; black margin | **PASS** |
| Microsoft Edge 155.0.4283.45 | Intel / Gen12LP, fallback false | 120/120 colored centers; black margin | **PASS** |

Windows reported Intel UHD Graphics 770 driver `32.0.101.6632`.
The browser graphics-service inventory identified ANGLE/D3D11; it did
**not** independently attest which API WebGPU used internally, so no
D3D12 backend claim is made. Both screenshots had the same actual PNG
SHA-256:
`161ca96968285e8184015fc98470e46e16576fe7ebc05cb0cfea986ff65eed6f`.
Both runs reported zero network failures and JavaScript exceptions.
Each browser had an independent profile/server, and browser cleanup
was restricted to its own process subtree; the server and profile were
removed. A process inspection found zero remaining owned profile
processes. The Windows Git tree remained clean.

Local ignored Windows evidence is under
`C:\\repos\\yetlet\\target\\yt075-windows-webgpu-baseline\\`:
`probe.py`, `chrome.json`, `edge.json`, and the two genuine canvas PNG
captures. No screenshot file was exported through a prohibited Project
artifact path. The HTML/CSP and JS tests do **not** execute the current
Yetlet Rust/wgpu-generated Wasm bundle, real Yetlet SSR server, opt-in
asset loader, device failure/retirement, replacement or sibling-island
workflow. Therefore **Windows GPU/scene-algorithm feasibility is PASS,
but YT-075 integration acceptance remains Blocked**, with earlier
independent reviewer FAIL and all uncompleted browser gates intact.
The next valid step requires a separately authorized/available exact
fixture and source-bound integration acceptance; do not infer it from
this browser baseline or proceed automatically to YT-076.

### Material correction and evidence limits

The texture-import failure exposed an application issue: WebGPU's asynchronous
validation errors did not stop the initial ready report. Parent code now pushes
a validation scope around configuration/pipeline/initial drawing, awaits its
result and checks lifetime again before retaining the renderer. Rejection drops
owned resources and returns the semantic unavailable path. No unsafe workaround
or unverified driver repair was added. The selected X11 run could not obtain an
adapter, so this new rejection/late-completion path remains compile-verified but
not browser-exercised; its explicit injected regression is present but Not run.

The parent correction passes rustfmt check, Python syntax, rebuilt release WASM
and targeted wasm32 strict Clippy. Logs: wasm-build-validation-fix.log and
wasm-clippy-validation-fix.log. Current scene digest is `5eef120113321637`, WASM
233528 bytes; counter digest remains `81c388455a88d98a`. Old asset identities
below are historical build-stage results, not the patched browser input.
At that browser checkpoint native source/tests, root lock and all eight
Todo/Search assets were unchanged. The later review-driven native language fix
is recorded below; it does not alter GPU assets. parent-asset-check.json and
browser-x11-input-hashes.json retain their original checkpoint identities.

The leaf generalist used one of its two product repair rounds. The parent
separately made one validation-error correction. Browser setup handling changed
the Vulkan feature once, then used the verified existing X11 configuration after
the concrete texture-import failure. Browser/GPU execution stopped at that
boundary. The subsequent language fix below is an independently identified,
parent-admitted bounded correction, not another generalist run or renewed GPU
execution authority. The
separate parent asset-check script's schema KeyError was corrected once by
comparing the actual live manifest; it required no build or product change.

All three results record browser exit 0, server termination -15, removed private
profiles and zero cleanup errors. Their recorded server/debug ports were checked
closed, including the final ports 60693 and 60913. The generalist PID is absent.
Raw JSON/text evidence and source hashes remain local and ignored. No staging,
commit, fetch, push, YT-074 repetition, Windows/cloud work or release occurred.

### Independent reviewer and bounded disposition

Read-only reviewer Job `wc_job_DRn3svlxj-v0LC4J` completed with native exit 0;
its review verdict is **FAIL**, not a failed dispatch. Exact thread
`01a1392d-45c4-7e52-b529-83187f8e1610` observed saved model `gpt-6-astra` and
effective `high`, matching the resolved ordinary ad-hoc review. The role JSON
was valid, all 17 reviewed file hashes were unchanged during review and the
reviewer PID was absent afterwards. This is an independent role-directed Job,
not native agent_type or formal assurance. reviewer-result.json SHA-256 is
`cf65bcb43c3e211c9a62237fef148e4f66def27862b582a942a08484b113e66e`;
reviewer-observation.json and reviewer-source-hashes.json retain identity.

The reviewer identified three P2 issues. One concrete logic issue was wrong
root language: English HTML used the existing renderer's zh-Hant default,
confirmed in all three retained no_js_html responses. Under AGENTS.md's
accessibility requirement, the parent admitted the smallest repair: explicitly
select LanguageTag en through the existing DocumentMetadata API, without changing
the core default, and assert the actual routed HTML response's root language.
That selected test passed (1 executed, 4 filtered), native strict Clippy passed
and the native binary was rebuilt. Logs: reviewer-language-test.log,
reviewer-language-clippy.log and reviewer-language-build.log. No new browser
run, dependency, API or validator was introduced. This post-review delta has
parent verification only, not a second independent reviewer PASS.

The two remaining issues are required evidence: no correct displayed drawing,
and unexecuted acquisition/failure/async-retirement/replacement/sibling browser
checks, including the validation-error regression. The reviewer did not equate
environment failure with a current framework defect. Its original FAIL is
retained; language correction does not waive those gates or reopen old failures.

Keep YT-075 Blocked unless real drawing, the remaining declared browser assertions
and independent acceptance review pass on a selected configuration. Any renewed
execution must preserve these attempts and use a concrete bounded strategy;
this record does not authorize another loop or a bypass of artifact policy.
The owner-authorized dependent chain remains conditional and unstarted.

## Historical leaf implementation/build checkpoint

At the leaf handoff, browser acceptance and independent review were Not run.
The following records describe that earlier checkpoint, not the current result.

## Scope and source identity

The owner activated only the [M11-G browser-first slice](M11-WGPU-PLAN.md).
The admitted continuation stayed in the original Ubuntu Yetlet checkout at initial HEAD`76be7aa1107e08e3c7850abc8c68f53301cf0f7c`; no commit was created. The earlier
three dirty activation documents were reconciled, preserving their preflight
failure history in [activation](YT-075-ACTIVATION.md). YT-074 files/evidence and
the historical native [YT-023 research](YT-023-WEBGPU-RESEARCH.md) were untouched.

Changed paths: root `Cargo.toml` (one nested-workspace exclusion);
`examples/webgpu-scene-island/{Cargo.toml,Cargo.lock,README.md,build.py,verify_browser.py,scene-loader.js,counter-loader.js}`;
`examples/webgpu-scene-island/src/{lib.rs,server.rs,main.rs,browser.rs,scene.wgsl}`;
`docs/{BACKLOG.md,ROADMAP.md,YT-075-ACTIVATION.md,YT-075-VERIFICATION.md}`.
No public crate, shared scene/core/View API or default GPU dependency was added.

The SSR page labels all 120 records synthetic, summarizes three states, and has
bounded search, pagination, record-detail GET links and reset. The optional
fixed-size canvas owns one local device; activation guards pending initialization,
retirement invalidates first, and late acquired devices are explicitly destroyed.
Only owner-current completions may update status. The existing counter is an
independent WASM entry. Missing/invalid manifest or missing files omit the affected
loader while retaining semantic HTML. No raw HTML sink, remote event forwarding,
inline script/style, eval permission or native graphics backend is introduced.

## Exact dependency and asset isolation

The nested workspace uses cached/offline wgpu `=27.0.1`, wasm-bindgen `=0.2.129`,
wasm-bindgen-futures `=0.4.79`, web-sys `=0.3.106`, with matching installed
`wasm-bindgen 0.2.129`. Exact cached wgpu source `Cargo.toml` and `webgpu.rs` were
inspected: `webgpu` selects browser bindings, `wgsl` gates `ShaderSource::Wgsl`,
and null adapters/device promise rejection map to errors. Defaults are disabled;
only `std`, `webgpu`, `wgsl` are selected for wasm32. Feature-tree evidence shows
no enabled Vulkan/Metal/DX12/GLES/WebGL backend. The native package tree contains
no wgpu. Browser-only transitive native packages may appear in the lockfile but
are absent from the selected target build graph. The root lockfile is unchanged.

Before changing any source/lock input, `python3 scripts/build_browser.py --release`
completed against current source, building Todo and Search independently. The
same command completed afterwards: digest **`27597bad9c53b93e`** before and after,
exact manifest equality and all eight unrelated file SHA-256 hashes equal.
Ignored evidence under `target/m11-gpu/yt075-20261014` includes
`baseline-build.log`, `baseline-hashes.json`, `post-build.log`, `post-hashes.json`,
`wasm-features.txt` and `native-tree.txt`. This is current-source evidence, not an
old dist manifest. GPU and counter entries were built independently by `build.py`:
scene digest `bf62f2be723b2c10`, WASM 223488 bytes; counter digest
`81c388455a88d98a`, WASM 151613 bytes. These are raw artifact sizes, not startup or
network/performance measurements.

## Focused checks and failure history

All final commands below exited 0:

| Command | Actual result / ignored log |
| --- | --- |
| `cargo test --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | 5 passed, 0 failed, 0 ignored, 0 filtered; `cpu-final.log` |
| `cargo build --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | Native binary built; `native-build.log` |
| `python3 examples/webgpu-scene-island/build.py` | Release wasm32 scene and separate counter built and bindgen completed; `wasm-build-repair1.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --all-targets -- -D warnings` | Native targeted lint passed; `native-clippy-repair1.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --target wasm32-unknown-unknown --lib -- -D warnings` | Browser targeted lint passed; `wasm-clippy.log` |
| `python3 scripts/build_browser.py --release` | Both pre/post builds passed and exact unrelated bytes matched |
| `git diff --exit-code -- Cargo.lock` | Root lock unchanged |
| Python `ast.parse` on `build.py` and `verify_browser.py` | Syntax passed; browser script was not executed |
| `node --input-type=module --check < examples/webgpu-scene-island/scene-loader.js` and the same check for `counter-loader.js` | Both module syntax checks passed without execution |
| `python3 scripts/check_docs.py` | Documentation, references, structure, whitespace and DAG passed |

The first WASM build exited 1 through the build script (Cargo 101):
`E0599: ShaderSource::Wgsl` unavailable with the initial feature list. The first
native clippy command exited 101 on `chunks_exact_to_as_chunks` in a CPU test.
Both are classified `product_failure` in the assigned build/lint surface. One
bounded repair round added the required browser shader feature and used the
Rust 1.99 array-chunk test pattern. Focused reruns passed; original
`wasm-build.log` and `native-clippy.log` are retained. Product repair rounds used
**1/2**; operational retries **0/2**; harness corrections after failed execution
**0**. No operational or reasoning-recovery dispatch occurred.

## Parent-owned browser and review gates

The app-specific `verify_browser.py` reuses only the existing CDP transport and
owned-process cleanup. It is prepared for parent inspection, **not executed**.
Its single attempt creates a unique returned page target, captures fresh
document/state before and after input, records actual navigator adapter and
system GPU diagnostics separately, checks real screenshot pixels at all 120
record centers plus a black margin, and saves the actual canvas screenshot.
It uses no sandbox/GPU-security bypass, software adapter forcing or fabricated
canvas output. Missing required GPU capability stops acceptance.

The bounded script includes explicit labeled injections for absent WebGPU, null
adapter, device rejection, missing WASM, absent/invalid manifest, and a held real
device completion after owner retirement/replacement. These assert named HTML
fallback, single pending initialization, late device destruction, replacement
status isolation and independent counter behavior. They remain **Not run**;
source/build evidence cannot be transferred into those browser assertions.
Cached wgpu browser `get_info()` returns largely empty adapter metadata, so the
script also records actual navigator adapter metadata; system diagnostics alone
are not selected-adapter proof. No hardware/performance/support claim is made.

Canonical next command, after parent inspection and capability admission:

```sh
python3 examples/webgpu-scene-island/verify_browser.py
```

Then obtain independent read-only review before accepting YT-075. YT-076 is the
next dependent backlog item only after YT-075 browser/review gates pass and the
parent applies the owner's continuation authority. Resize/DPR/loss recovery,
picking, dynamic/performance work and public extraction remain later items.

## Resource boundary

This implementation assignment started no server, browser, watcher, persistent Node process
or GPU workload. All bounded compiler/build commands completed; temporary bindgen
directories were removed by their context managers. No owned background resource
remains. The unexecuted browser check tears down only its own target, sockets,
server/browser process group and temporary profile, recording cleanup even on a
failed gate. No staging, commit, push, fetch, install, host or external environment
change was performed.
