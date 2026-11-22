# M28-F2 / YT-284 — Public source-package and compatibility gate

- **Date:** 2026-11-21.
- **Status:** YT-284 **Done at public-source/local package compatibility scope**.
  Independent R1 `wc_job_CFmqc67Ec3Z4MEWU` terminal exit0, actual JSON
  **PASS**, `issues=[]`, `required_followups=[]`. **Not a registry-published
  two-crate release.**
- **Basis:** current Yetlet root checkout at `363dde2`; the accepted YT-282
  public Composition/adapter and YT-283 real external Native integration remain
  source-identical, except for this task's separate compatibility consumer.
- **Scope:** the publicly importable Rust source APIs and selected local Cargo
  source/package contents. Hosted publication, version selection, tags, signing,
  CLI template updates, Windows, GPU replay and physical LAN are not part of
  YT-284.

## Selected independent consumer and exact commands

The small independent Cargo project
[`yt284-public-compat`](../examples/external-consumers/yt284-public-compat/Cargo.toml)
uses its own `[workspace]`, `Cargo.lock` and ordinary public
`yetlet`/`yetlet-hosts` path dependencies. It does not import research
modules, `include!` the repository source or copy a private Supervisor.
The headless test installs a genuine caller-owned `Rc<Cell<u64>>`, attaches
a borrowed custom Audit that records revisions 1,2,3, then proves exact
Composition Complete/Stopped and its original close callback. The Native-only
test creates an uninstantiated `WindowSlot` and actually binds, polls and
closes an owned UDP loopback socket. The Pure Web test obtains an ordinary
Axum Router through `yetlet::server_axum::App::into_router()`.

| External selected feature | Focused test | Normal dependency proof |
|---|---|---|
| `pure-web` | 1/1 PASS | Axum/Tokio present; no Winit/wgpu |
| `local-headless` | 1/1 PASS | only `yetlet`; no Axum/Tokio/Winit/wgpu |
| `native-udp` | 1/1 PASS | Winit/UDP present; no Axum/Tokio/wgpu |
| `pure-web,local-headless,native-udp` | 3/3 PASS | all modes coexist; no mandatory renderer |
| default-off | strict Clippy PASS | no extra caller-side features |
| all three modes | strict all-target Clippy, Rustfmt | both PASS |

All selected tests use `--manifest-path` and `--offline --locked
--no-default-features`; no dependency was fetched online. Separate Cargo
normal-dependency trees, tests, stderr, exact command receipts and captured
manifest/lock/source SHA are preserved in
`.pipeline-output/m28-f2-yt284-20261121/`.

The existing external
[`yt283-revision-audit`](../examples/external-consumers/yt283-revision-audit/Cargo.toml)
already provides the actual three-way Winit + chosen WGPU renderer + Web
worker + UDP Host + original custom RevisionAuditHost proof. Its source-bound
Ubuntu X11 RTX4090 Vulkan run exited0, observed real Window/key/mouse/focus,
11,631 returned Presents, exact Audit/C1 Revision11630 and original Web
JoinedSuccess/Complete/Stopped. Reuse its independent R1 PASS and existing
focused tests **without** restarting an OS Window or repeating those tests.

## Package candidates and provenance limits

| Selected candidate | Evidence | What it proves |
|---|---|---|
| `yetlet` development source `0.9.0-alpha.8` | `cargo package --list` 90 files, including `src/composition`, tests, guides, README and both license files; `cargo package --allow-dirty --offline --locked -p yetlet` exit0, independent packaged-source compile PASS | A **local** 90-file Rust `.crate` candidate was created and verified |
| `yetlet-hosts` development source `0.9.0-alpha.8` | `cargo package --list` 16 files (feature-gated public adapters and examples); direct external path consumers all compile/test | The selected companion's **local source/manifest** is available and feature-isolated |
| `yetlet-hosts` **registry/archive publication** | `publish = false`. An experimental `cargo package --no-verify` failed because its local `yetlet` dependency has no version requirement; a temporary exact version pin then also failed because the existing crates.io `yetlet 0.9.0-alpha.8` does **not** provide this new `composition` feature. The temporary pins were reverted, preserving the intentionally source-only manifest. | **NOT qualified** as a standalone registry-resolvable `yetlet-hosts` archive or published dependency |

**Do not conflate** the locally verified `yetlet-0.9.0-alpha.8.crate`
with the already released public registry copy of the same version. The local
dev sources add `composition` after that published version. This is a
source/package-compatibility gate, **not a release gate**. A future **separately
authorized** SDK version selection/publication must resolve package version
identity and compatible `yetlet-hosts` dependency metadata before either
crate can be claimed registry-installable. Do not weaken Cargo resolution,
synthesize a registry package or silently change `publish = false`.
The real failed packaging receipts are retained, not relabeled PASS.

## Working consumer recipes and constraints

- **Direct Pure Web/SSR:** import `yetlet::server_axum::App`; use
  `App::new().get("/status", handler).into_router()` with the caller's
  ordinary Axum/Tokio runtime. No Composition is mandatory.
- **No Web / caller-local:** select only `yetlet/composition`; attach
  the existing `Rc` domain using `Builder::required_local`, optionally
  `borrowed` for a mutable custom Audit, and complete via
  `complete_until`. This requires no Tokio, Window or `Send + Sync`
  for the local authority.
- **Native + Network without HTTP:** opt in `yetlet-hosts/native-winit`
  and `yetlet-hosts/udp`. The caller owns Winit
  `EventLoop`/`ApplicationHandler` and real `WindowEvent`; a
  `WindowSlot` only tracks its own Window/Redraw lifecycle. The caller
  explicitly drives `UdpEndpoint::poll_bounded` and closes it. There is
  no implicit renderer, scheduler, HTTP worker or domain.
- **Three-way + custom Host:** select Web, UDP, Winit and WGPU only in the
  external application's Cargo features, as in YT-283. The consumer owns
  the sole C1 and its input/clock/codec; the public Composition API
  coordinates required local/foreground/owned-worker resources and caller
  custom Audit. Foreground EventLoop stays on its supported calling thread.
  Close Web command admission first, then the original domain/UDP/Native
  resources, signal worker shutdown, and observe the **same** original Web
  Join. A Pending result retains resources and is not Complete.

## Exact acceptance and non-goals

The bounded YT-284 **source compatibility** gate accepts the independent
external Cargo consumers, feature/normal-dependency isolation, caller-local
types, public extension with real operation and shutdown, current local
candidate contents and truthful release limitation above. The existing
source-reviewed public API is not changed. The sample's source-compatible
coverage does **not** imply `yetlet-hosts` is publishable now, nor does it
requalify untested platforms, a new native engine, physical LAN,
YTP1/Replica/Reconnect, precise frame pacing, Browser/Windows matrix or
Hosted release.

Independent Reviewer R1 `wc_job_CFmqc67Ec3Z4MEWU` inspected the actual
external source/tests, normal dependency trees, local 90-file verified Core
archive, 16-file Host source listing, both retained Host packaging failures,
and accepted YT-283 three-way evidence. It returned terminal exit0, actual
JSON PASS with no blocking issues or required followups. Its non-blocking notes
explicitly retain the registry-qualified Host archive gap, distinguish source
from publication, and require a separate Strict/Builtin staged Guard before
any Git Commit. This closes YT-284 **source/local** acceptance, not the future
registry/version gate.

Overall M28-F2 **Git delivery** is separate: retain all current dirty evidence
until independent review, exact-source comparison, Strict/Builtin Commit Guard
and a user-authorized local Commit succeed. No Push/PR/Merge/Publish is
included in YT-284.
