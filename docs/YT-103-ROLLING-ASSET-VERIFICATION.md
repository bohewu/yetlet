# YT-103 — Local rolling asset qualification

Date: 2026-10-18. Status: Done at bounded local qualification scope;
independent final Reviewer PASS, zero issues and required followups.
Source baseline: `9850820f4ea5e35f5b09e521d8c24e00aa937d89`.
Evidence root: `artifacts/yt103-rolling-20261018/`.

The owner authorized committing the independently reviewed CLI/watch changes
and continuing to YT-103. The [reviewed plan](YT-103-ROLLING-ASSET-PLAN.md) selects
an independent Nginx static/gateway Deployment with verified v1/v2 public assets
before changing Rust HTML. This is an application/deployment-owned local proof;
no core, CLI, generator, provider SDK or production default changes are included.

## Build identity and preparation

Two actual apps were generated with the current `new --vite --service` command,
explicit local Cargo dependency overrides, offline lock preparation and frozen
Release builds. Pinned offline frontend installs, typechecks and production
builds passed. The generated `/static/` route is retained. Each release has
seven reachable graph files, including late JS/CSS; the verified union contains
eleven. Same-URL/different-byte publication fails instead of overwriting content.

| Image | Frozen Docker index identity |
| --- | --- |
| App v1 | `sha256:8979bb88285e0f69bfafc14958f909761061e3108da6a2f8cc52c6dd41ae44a1` |
| App v2 | `sha256:c0d5b75a367289e8db532602d8e860adaa7fe217b1f4fe7a4d1046ed917d49c0` |
| Static/gateway union | `sha256:7d8ff4549fa7a4a80d5a7e6f7dcde07385333c3723c5de0827f8752e81481a63` |
| Static/gateway v2-only | `sha256:e4d7ab7381a9d71e8ae37de137a77f316f26343444bc527863089112946fa987` |

The final Docker archive is 31,918,080 bytes, SHA-256
`06d0f7a21f13d6df2bfd29c478c74f23720ed2653eb2f1f70136ea9169dbffd8`.
Complete source/image/graph receipts and commands are in `builds/`:
`image-receipts.json`, `ownership-receipt.json`, `asset-summary.json`,
`v1-SHA256SUMS`, `v2-SHA256SUMS` and `YT-103-BUILDS.md`.

These are index identities, not the platform manifest or container config
identities. `builds/archive-runtime-digest-map.json` records all four
index-to-linux/amd64-manifest-to-config relationships. K3s reports the config
digest as `imageID`; for example, app v1 reports `sha256:aec5192713d7e63ccd7131748343fde83864282555e8be7ab36debca59e2635e`
and the union gateway reports `sha256:d159eff7d7b91b8b8519140f23ed226a0f52c63ac69a6e696b899767c12e5a49`.
Both match their frozen archive entries.

The test app adds only release/Pod observation headers and a fixed eight-second
handler that logs start/completion. A signal-reception log observes the generated
Axum shutdown path without changing its semantics. Runtime credentials are
synthetic and supplied only to the task app. Public asset images contain only
verified graph bytes, with exact URL mappings, real missing/private-file 404s,
explicit MIME/nosniff and `no-store`; no immutable-cache claim is made.

App images use the compiled executable and its required runtime libraries.
The existing Nginx image is Alpine/musl while the host is glibc, so the app does
not silently assume binary compatibility with that image. Both app images were
actually exercised non-root before transfer. Local smoke verified SSR, health,
all graph bytes, headers and in-flight completion after SIGTERM, with exit 0.
Static smoke verified both trees and gateway routing. All smoke containers were
removed before lab execution.

## Retained preparation failures

Two real image-packaging failures were corrected within the same task outputs:
public TOML copies retained private source permissions and were unreadable by
the non-root app; Nginx could not create its temporary cache directory as its
runtime user. Only the image's public TOML permissions and cache ownership were
changed. Source permissions, secrets and framework sources were preserved.
Both initial logs remain in `builds/`; final focused checks passed.

A fixture assertion that rejected the text `lazy-` in HTML matched existing DOM
IDs rather than lazy asset requests. It was corrected to inspect actual asset
references, without changing product behavior. Browser setup also retained two
capability limits: native Cua reported no browsers and no available IAB; the
REPL's direct ESM wrapper import failed, then its existing CommonJS package was
loaded through `createRequire`. No browser dependency was installed.

## Native lab and browser observation

The existing three Multipass K3s nodes were Ready at `v1.36.4+k3s1`. Preflight
recorded an absent `yt103-qa-20261018` namespace, unused NodePort 31553, absent task
image references and thirty pre-existing resource identities. The one-off
`lab/yt103-lab.py` performs only explicit phases, records owned imports/uploads
before effects and preserves the pre-existing namespaces and VMs.

The application uses three replicas and the static/gateway uses two. Initially
each app replica occupied a different node; the first v2 revision placed two on
node02 and one on node01. The spread selector counts overlapping revisions, so
it did not retain the initial three-node distribution. This observation is
preserved without adding another rollout or claiming node-failure tolerance.
Application updates use zero unavailable, one surge and two seconds minimum
readiness. Termination has a thirty-second total budget including a five-second
native kubelet sleep hook. Gateway readiness is local to Nginx, independent of
Rust readiness. This is a single physical host with three VMs, not independent
host/zone fault domains.

Browser observation uses the project's already installed Playwright package and
system Chrome **155.0.8059.84**, headless, at 1280 by 800. Its scope is delayed
production asset loading across update/rollback; it does not retest Vite HMR.

## Rolling availability and draining results

The native phase commands use the same one-off script:

```text
python3 artifacts/yt103-rolling-20261018/lab/yt103-lab.py install
python3 artifacts/yt103-rolling-20261018/lab/yt103-lab.py forward-v2
python3 artifacts/yt103-rolling-20261018/lab/yt103-lab.py rollback-v1
python3 artifacts/yt103-rolling-20261018/lab/yt103-lab.py disrupt
python3 artifacts/yt103-rolling-20261018/lab/yt103-lab.py forward-v2-final
```

All five commands exited 0. Installation verified the baseline graph through
NodePort and both gateway Pod addresses. Each traffic phase samples SSR `/`
at ten requests per second, with a two-second timeout, no measurement retries
and Nginx upstream retries disabled. Observation continues ten seconds beyond
the operation's recovery; the final phase also includes the retention window.

| Phase | SSR requests | Observation (s) | v1 / v2 responses | 404 / 5xx / connection errors or timeouts | p50 / p95 / max (ms) |
| --- | ---: | ---: | ---: | --- | --- |
| v1 to v2 | 537 | 53.602 | 215 / 322 | 0 / 0 / 0 | 1.357 / 2.040 / 9.751 |
| Rollback to v1 | 627 | 62.791 | 378 / 249 | 0 / 0 / 0 | 1.391 / 2.114 / 9.745 |
| Single graceful Pod deletion | 261 | 26.009 | 261 / 0 | 0 / 0 / 0 | 1.359 / 2.094 / 9.595 |
| Final v2 rollout and retention | 939 | 93.835 | 257 / 682 | 0 / 0 / 0 | 1.320 / 1.947 / 9.043 |

All **2,364 measured SSR requests returned HTTP 200**. Raw `*-requests.json`
retain each request, timestamps, duration, release and Pod identity;
`*-timeline.json` retain Pods and EndpointSlices. `*-summary.json` contains
the phase aggregates. These totals exclude deliberate missing-file probes,
browser requests and the separate slow handler. They do not claim availability
outside the measured intervals or during the later static-service GC rollout.

The victim was `yt103-app-765b8b9dd4-5fb2v`. Its direct eight-second request
returned 200 in 8.014 seconds. `disrupt-pod.log` records start at
`15:33:58.149289975+08:00`, actual signal reception at
`15:34:04.163282787+08:00`, and completion at `15:34:06.150421181+08:00`.
The request therefore finished about 1.987 seconds after SIGTERM, rather than
only finishing inside the preStop delay. A replacement became Ready, restoring
three app replicas. This tests ordinary graceful Pod deletion, not forced
termination, node failure or PodDisruptionBudget eviction enforcement.

At every completed phase, both complete seven-file graphs matched frozen SHA-256
values through NodePort and each gateway Pod. Gateway UIDs and the union image
remained unchanged throughout Rust rollout, rollback and disruption. Initial
three-node app placement became two-node placement after rollout/rollback, as
described above; no three-node fault-tolerance conclusion is drawn.

## Held-browser compatibility

`browser/browser-result.json` and four PNGs preserve actual browser state:

| Held page | Server after change | Actual delayed result |
| --- | --- | --- |
| v1 HTML, lazy JS/CSS never previously requested | v2 after all v1 Pods retired | Both v1 lazy assets returned 200; v1 module executed; computed CSS was `rgb(126, 34, 206)` |
| v2 HTML, lazy JS/CSS never previously requested | v1 after all v2 Pods retired during rollback | Both v2 lazy assets returned 200; v2 module executed; computed CSS was `rgb(5, 105, 74)` |

Both pages retained their original heading and `performance.timeOrigin`, proving
no document reload. New v2 HTML was also observed before rollback. Resource
entries before interaction exclude the lazy URLs; response records after
interaction show their real 200 responses, rather than a warm module-cache replay.

Screenshots are `v1-held-before-update.png`, `held-v1-lazy-after-v2.png`,
`v2-held-before-rollback.png` and `held-v2-lazy-after-rollback.png` under
`browser/`. There were zero uncaught application page errors. One optional
`favicon.ico` 404 produced a console error and is retained with Resource Timing
status 404; it is not counted as a lazy-asset or SSR measurement failure. The
browser and context were closed after the second compatibility observation.

## Retention window

After the final forward rollout, the last v1 Pod was confirmed absent at
monotonic `364734.535741053`. This conservative retirement upper bound starts
the lab's 30-second lifetime plus five-second margin; eligibility is
`364769.535741053`. The observation ended at `364769.890321591`, with the union
and rollback pin still intact.

`lab/protected-retention.json` retains 344 dedicated v1/v2 lazy JS/CSS requests,
all HTTP 200 and SHA-256 matched. Their first and last starts are
`364735.281531172` and `364769.489924531`. Full graph checks bracket this window;
general SSR observation continues until `364779.899850226`. This is a compressed
local ordering proof, not a production client-lifetime recommendation.

The separate command `python3 artifacts/yt103-rolling-20261018/lab/yt103-lab.py expire`
exited 0. The operator removed the v1 rollback pin at `364868.314188425`,
98.778 seconds after eligibility, then selected the frozen v2-only static image.
Every union gateway Pod retired before the expiry assertions. All seven v2
assets still matched their SHA-256 values through NodePort and each of the two
replacement gateways; current SSR remained v2 with CSP, nosniff and no-store.
Shared files required by v2 were preserved.

The v1-only lazy JS and CSS returned expected HTTP 404 at
`364899.587128171` and `364899.587990938`, recorded separately in
`lab/expected-expiry404.json`. `expired-v2-graphs.json` and
`expired-v2-per-gateway-graphs.json` retain the surviving graph evidence.
No continuous SSR availability measurement is claimed for this separate
static-service GC rollout.

## Cleanup and preservation

`python3 artifacts/yt103-rolling-20261018/lab/yt103-lab.py cleanup` exited 0;
`lab/cleanup-command.json` retains the native command receipt. Cleanup removed
the owned namespace, twelve imported task image references and twelve temporary
archive uploads across the three nodes. NodePort 31553 no longer accepted
connections. All three nodes remained Ready. Thirty pre-existing resource
identities and their specifications were unchanged between the saved preflight
and cleanup snapshots. All pre-existing node image references remained present
(35, 28 and 29 respectively). Task log followers and host phase processes ended.

`cleanup-result.json`, `cleanup-cluster.json`,
`cleanup-preexisting-images.json` and `cleanup-watchers.json` retain those checks.
No VM, existing namespace, OpenBao or Valkey lifecycle operation was performed.

Local Docker cleanup removed exactly five recorded task tags with
`docker image rm --no-prune`, all five native commands exiting 0. No task smoke
containers remained. Original Nginx and Rust image IDs and digest references
remained inspectable; restoration was unnecessary. No global prune or removal
by original image ID occurred; intermediate build cache was left alone.
`builds/local-image-cleanup-commands.json` and
`local-image-cleanup-preservation-receipt.json` preserve commands and before/after
inspection. The final archive, failed historical archive and original-image
backup retain their recorded hashes. Evidence archives and screenshots remain
local artifacts intentionally.

## Review and scope

The initial plan received independent Reviewer PASS with no issues or required
followups (`plan-review.json`). Final independent Reviewer
`/root/yt103_plan_review` returned PASS with zero issues and required followups
(`final-review.json`); native role, model and effective effort matched the
dispatch (`final-review-trace.json`). Review independently reproduced the
request aggregates, matched 72 source-inventory entries and both public trees,
checked archive digest mappings, held-browser chronology, retention ordering
and exact cleanup. Runtime
framework, CLI and generator sources are unchanged against the source baseline;
this slice changes `docs/YT-103-ROLLING-ASSET-PLAN.md`,
`docs/YT-103-ROLLING-ASSET-VERIFICATION.md`, `docs/ASSETS.md`, `docs/README.md`,
`docs/BACKLOG.md` and `docs/ROADMAP.md`. Earlier unrelated planning changes are
preserved; backlog/roadmap completion bookkeeping records this final PASS.
`cargo xtask check docs` passed with 75 documents and 103 backlog items;
`git diff --check` passed. Unchanged CLI/runtime sources did not require another
CLI regression run for this documentation and local-deployment slice.

This local proof establishes the measured update, rollback, ordinary graceful
Pod deletion and retained-byte behavior under the declared assumptions. It does
not establish node/host/zone failure availability, storage durability, external
load balancing/TLS/CDN behavior, long-lived/offline clients, old-code/new-API
compatibility or a production SLA. Default generated images still contain only
their own startup snapshot. No registry publication, production deployment,
mandatory service/provider/Node dependency or release gate is introduced.

The next independently unblocked application item is YT-092, the opt-in Production
Starter. YT-102's broader DX research remains separate; YT-096/YT-020 release
gates retain their existing status and owner-approval requirements.
