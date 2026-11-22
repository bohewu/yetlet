# M28 release preparation — SDK 0.9.0-alpha.9 and CLI 0.4.2

## Release decisions — 2026-11-22

The accepted Experimental distributions and Reader Edition are SDK
`0.9.0-alpha.8` and CLI `0.4.1`. Public index and release readback still showed
that pair at this preparation checkpoint. [SDK publication](ALPHA8-HOSTED-RECORD.md)
and [CLI publication](CLI-041-HOSTED-RECORD.md).

| Track | Candidate scope | Readiness at pre-review checkpoint |
| --- | --- | --- |
| SDK (YT-296) | `yetlet` and `yetlet-browser` at `0.9.0-alpha.9`, with Rust 1.99. The new alpha identifies breaking defaults, features and imports without reusing the published alpha.8 identity. Browser requires Core `=0.9.0-alpha.9` with defaults disabled. | **Not ready for publication.** Local archives and independent archive consumers passed. Integrated review, frozen source, candidate CI and authorized build/signing/publication are open. Browser's standalone public-index dry-run requires the published Core row. |
| CLI (YT-295) | Independent `yetlet-cli 0.4.2` patch ships the M28 SDK-aware scaffolds. The exact build SDK and embedded snapshot stay at published alpha.8, with `sdk-latest-v2` and its existing alpha.8 checksum binding. | **Not ready for publication.** Local archive, dry-run and installed binary comparison passed. Integrated review, frozen source, candidate CI, authorized build/signing/publication and public installed-consumer readback are open. The CLI can ship on the published SDK. |

## SDK release notes and migration

- Core now has empty defaults and built-in standard-library composition.
  `Host`, `HostGroup` and `ResourceGroup` coordinate readiness, completion and
  ordered release, including application-owned hosts. Enable `view` for typed
  View and tags, or `html` for View plus pure HTML rendering.
  [Core guide](../crates/yetlet/guides/core.md).
- Select native `web` for HTTP/API and `web,html` for SSR. Migrate the
  `server-axum` feature and `yetlet::server_axum` imports to `web` and
  `yetlet::web`. Direct `App::serve` and Axum/Tower interoperability remain.
  `App::into_host` opts into managed `WebHost` with caller-runtime or owned-thread
  drivers. `web::prelude` gates View and document helpers on `html`.
  [Web guide](../crates/yetlet/guides/server-axum.md).
- Select `native-winit` for the caller-thread NativeHost and its windows,
  `udp` for the caller-driven endpoint or managed UdpHost, `command-bridge`
  for bounded ingress and snapshots, and `renderer-wgpu` for the independent
  native Vulkan renderer. Import the adapters from Core's feature modules.
  The temporary `yetlet-hosts` staging crate was retired.
  [API reference](API-REFERENCE.md) and [capability map](CAPABILITIES.md).
- `actions` uses an executor-associated error. Move HTTP error/envelope imports
  to `yetlet::actions_http` and enable `actions-http` for the transport protocol.
  Reactive and provider features remain opt-in. Browser defaults to `dom`.
  `dom` and `dom-keyed` forward Core View, `actions` forwards
  `actions-http`, and the GPU options are `wgpu` and explicit `wgpu-browser`.
  [Compatibility](COMPATIBILITY.md#current-checkout--web-and-http-protocol-ownership)
  and [Browser feature guide](../crates/yetlet-browser/README.md).

## CLI release notes and compatibility

The comparison used the crates.io-installed CLI 0.4.1 and CLI 0.4.2 installed
from its extracted archive. For alpha.9, the CLI read seeded offline index rows
derived from the actual archives, and Cargo used a checksummed directory source
with crates.io source IDs in the locks. A separate fresh public index lookup
returned `SDK_VERSION_NOT_FOUND` for alpha.9.

| Alpha.9 template | Published CLI 0.4.1 | Archive-installed CLI 0.4.2 |
| --- | --- | --- |
| Native | Explicit and default `new` returned `SDK_INCOMPATIBLE` for missing `server-axum`, without creating a destination or falling back. | Explicit and default `new`, `inspect`, `build`, HTTP `run` and Watchexec `watch` passed. Native uses `web,html` and `web-app-serve`. Settings adds `config` and `web-configured`, with creation, inspection and build also verified. |
| Service | Explicit and default `new --service` returned the same failure. | Creation, inspection, build, HTTP run, health endpoints and watch passed with `web,html` and `web-service`. |
| Browser | Creation, inspection, build, Chrome preview and watch passed. Companion `dom,dom-keyed` supplies Core View transitively. | The same commands passed. The scaffold names Core `reactive,view` directly and pairs the exact Browser peer with `dom,dom-keyed`. The startup profile is `browser-wasm`. |

Both CLIs passed `doctor` and correctly refused offline creation with an empty
metadata cache. Generated manifests, locks and receipts were checked against the
selected identities. The CLI 0.4.2 public alpha.8 control passed explicit and
default creation for Native, Service and Browser, plus locked fetch, build and
HTTP runtime for Native and Service. Users still on CLI 0.4.1 can pin alpha.8
with `--sdk-version 0.9.0-alpha.8` for Native and Service, or upgrade to 0.4.2
after its verified publication. M28 deployment generator support remains
outside this scope. [CLI guide](CLI-GUIDE.md).

## Pre-review qualification evidence

The preparation branch `codex/m28-sdk-release-prep` was at
`74172315d05114843f1d49c906974be7172eef55` (master after PR #102), with
candidate changes in the working tree. MSRV, Stable native/samples/Chrome and
Rust CodeQL passed at that exact base. SDK runtime, sample and asset source,
smoke code and the Rust workflow match the base on the compared paths. Those
hosted results cover the base only. The candidate's independent integrated
review, commit, PR and CI had yet to occur at this checkpoint.
[Base PR](https://github.com/bohewu/yetlet/pull/102),
[MSRV](https://github.com/bohewu/yetlet/actions/runs/45015858531/job/137187905261),
[Stable and Chrome](https://github.com/bohewu/yetlet/actions/runs/45015858531/job/137187905557)
and [CodeQL](https://github.com/bohewu/yetlet/actions/runs/45015858556/job/137187938239).

| Normally verified pre-freeze archive | SHA-256 |
| --- | --- |
| `yetlet-0.9.0-alpha.9.crate` | `d9c2728ca4ba5a5926ae0d18ce7bf33971a4c2263d34935886d72de0683f71b2` |
| `yetlet-browser-0.9.0-alpha.9.crate` | `caa8d6b4e9a9b3da97a8200ac3bd410205fe9eea57008bd787a7a737ec0840c6` |
| `yetlet-cli-0.4.2.crate` | `3f148d05de0c627c0d14c92312d30e4e85e39986c8d9e728bfef60c525c9253d` |

The archives embed the base SHA with `dirty: true`. The SDK input diff digest
was `aa0abf79df91f309a2eab790e0337123654746892755550781906a47bcd11af7`.
Freeze and rebind clean-source archives, their checksums and dependent proofs
to the approved source. Reconcile any input drift before reusing results.
YT-294's local alpha.8 archives and the published alpha.8 archives have
different checksums from each other and are historical inputs.
[YT-294 archive qualification](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#22-yt-294--normal-archives-and-external-consumer-qualification).

- SDK source formatting, locked check and strict Clippy passed, with 229
  selected tests passing. Six archive consumer families passed 21 actions,
  14 dependency graph checks and 22 integration tests. Coverage includes
  Linux Core, View/HTML, managed Web/SQLite, Native/UDP loopback, and Wasm
  Browser feature isolation and Todo binding generation. Windows x64 MSVC
  cross-target checks from Linux passed. Windows linking and execution,
  visible Native windows, GPU presentation and physical LAN were outside this
  candidate qualification.
- CLI passed 148 package tests (three intentionally ignored), release verifier
  tests and Linux installed command checks. Native and Service returned escaped
  SSR with restrictive CSP headers and observed watch restarts. Real Chrome
  exercised Wasm, keyed task interactions and watch reload with changed Wasm
  bytes and CSP. This run covers Linux Chrome and the generated templates.
  Windows CLI and browser execution and a deliberate inline-event CSP
  negative probe were outside the run.
- Core and CLI public-index dry-runs passed with upload aborted. Initial
  offline SDK dry-runs required HTTP, and Browser's online attempt stopped at
  the missing exact Core row. Cargo did verify Browser locally against that
  Core archive. Other retained failures include the old CLI feature rejection,
  an initial CLI extraction caught by workspace discovery, missing template
  dependencies in the offline Settings cache, the runtime harness's immediate
  port rebind assumption and a shared-target Wasm collision. The scoped
  extraction, cache, reusable listener check and isolated-target replays
  passed with the same candidate inputs.

### RustSec scope

`cargo-audit 0.22.2` with advisory DB
`d1fe6ac3eba823a3515e849045f36dc34543885d` found zero vulnerabilities in the
Core and Browser packaged locks and the independent CLI lock with an empty
ignore set. Core and CLI reported the informational unsoundness warning
`RUSTSEC-2026-0486` for `rustix 1.1.5`, reached through optional Native
dependencies and CLI `tempfile`, respectively. The scoped source inspection
found no calls to the affected xattr interfaces on those paths. Browser
reported no advisories, with a separate yank-check diagnostic for unpublished
Core. Record warning disposition and recheck against the frozen release inputs.

Unfiltered exact-base and candidate workspace audits both reported
`RUSTSEC-2023-0071` through `rsa` in the vendored OpenID Connect dependency of
the source-only Lab Operations Board, plus the informational
`RUSTSEC-2026-0543` (`chrono`) and `RUSTSEC-2026-0486` (`rustix`) warnings.
The configured workspace scan passed under the unchanged, sole OIDC
public-verification applicability exception and still reported both warnings.
The SDK package graphs exclude that OIDC path. [CI security review](CI-SECURITY-REVIEW.md).

## Publication order and owner gates

1. Complete independent review and the release documentation gate. With
   approval, deliver guarded source and verify exact PR-head and master CI.
   For each release, prepare its versioned workflow bindings, select its
   immutable source and tag, and regenerate its archive records. Obtain
   separate authorization for hosted build and custody readback, the frozen
   signer inputs and independently verified signing result, and publication.
   [Release workflow](RELEASING.md) and [publication contract](RELEASE-PUBLICATION.md).
2. Publish **CLI 0.4.2 first** after its gates clear. Verify GitHub and crates.io
   archive identity, install from the public registry and verify generated
   Native, Service and Browser consumers on published alpha.8. Keep the CLI
   release decision and receipt independent of the SDK.
3. Use the approved schema-2 SDK publisher to publish the frozen **Core alpha.9**
   archive before **Browser alpha.9**. The publisher checks Core's public
   index checksum and downloaded bytes before uploading Browser, then checks
   Browser and runs a fresh registry-only SDK consumer. A standalone Browser
   public-index dry-run becomes runnable once Core's exact public row is
   available. Verify installed CLI Native, Service and Browser consumers
   against the accepted pair, including exact lock and receipt identities.
   [Ordered registry publication](RELEASE-PUBLICATION.md).
4. Close out the Reader Edition **after each verified CLI or SDK publication**.
   First verify alpha.8/0.4.2 following CLI publication, then alpha.9/0.4.2
   after the verified Core/Browser cohort. Reconcile published records,
   current README and edition/registry inputs. Verify the first-app path,
   native/Wasm and static references, independently review the update, obtain
   CI and authorized Pages deployment, and read back the live files before
   closing that release. [Required Reader Edition closeout](RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication).

## Milestone documentation follow-through

- Review the three completed M27 DX records first. Keep the original
  gap report, audit plan and prioritization plan at verified immutable
  references once the tracker links are mapped. The [CLI guide](CLI-GUIDE.md),
  [First App](FIRST-APP.md) and generated Service README own the live guidance.
- After release preparation, map the M28 original study, E promotion, F0A
  contract and F0C decision to immutable history. Follow with the F2 ADR
  boundary, public contract, Core/Web/UDP/Native/WGPU adapter records,
  public three-Host consumer and package compatibility record.
  Preserve failed attempts, scoped approvals and all cited heading links.
  The [architecture](ARCHITECTURE.md), [ADR 0013](adr/0013-composable-application-and-hosts.md)
  and [ADR 0014](adr/0014-layered-host-resource-lifecycle.md) own the decisions.
  Package guides and the [finite sample catalog](SAMPLES.md) own current usage.
- Keep the M28 Host lifecycle design, multi-host delivery plan and external
  consumer/Audit Host record in place while reconciling their active links.
  Move durable usage to the module guides and current consumers. Keep dated
  physical UDP readiness, private application acceptance and SDK qualification
  distinct in the [qualification matrix](QUALIFICATION-MATRIX.md),
  [roadmap](ROADMAP.md) and [backlog](BACKLOG.md).
- Keep [M7g](M7G-VERIFICATION.md) and [M8c](M8C-VERIFICATION.md) at their
  required paths until a separate link and frozen-snapshot reconciliation.
  Keep [MVP](MVP.md) as the project specification. Each future batch needs a
  checked inbound link and anchor map before moving a record.
