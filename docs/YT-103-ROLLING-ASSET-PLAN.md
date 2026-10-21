# YT-103 — Rolling deployment and retained asset qualification

Date: 2026-10-18. Status: Done at bounded local qualification scope;
plan and final evidence review passed. Actual outcomes are in the
[qualification record](YT-103-ROLLING-ASSET-VERIFICATION.md).
Source baseline: `9850820f4ea5e35f5b09e521d8c24e00aa937d89`.

The owner authorized the reviewed CLI/watch commit and continuation to the next
item, YT-103. This slice implements the backlog's required comparison and a
disposable local qualification. It introduces no mandatory framework module,
provider, CLI command, production deployment or release support promise.

## Existing boundary

The [asset contract](ASSETS.md#cache-and-retention) and
[YT-101 evidence](YT-101-VERIFICATION.md) qualify one image's startup snapshots.
The app-owned Vite adapter serves `/static/` from memory with `no-store`; replacing
that image does not retain files referenced by old pages. The accepted Kubernetes
generator provides health probes and a termination budget, without proving
rolling availability or a hard draining deadline. Those historical gates remain
unchanged. This work uses separate application/deployment-owned lab files and
does not widen the generator's accepted source representation.

Vite documents that deleting a previous deployment's chunks can break dynamic
imports in an already-open page. Refreshing HTML does not make that old page's
pending imports disappear. Keeping the referenced bytes available is the
selected compatibility mechanism. See [Vite load error handling](https://vite.dev/guide/build.html#load-error-handling).

## Options and selected lab arrangement

| Option | Strength | Cost and failure boundary | Decision for this slice |
| --- | --- | --- | --- |
| Versioned object storage, optionally behind a CDN | Asset lifetime is independent of app Pods; suitable for many releases | Provider credentials, consistency, origin/CDN behavior, costs and deletion policy need separate qualification | Keep as an interchangeable deployment option; no provider SDK or service is required here |
| Independent static asset service | Existing HTTP server, same-origin routing, independent asset retention and rollback | Another service must have replicas, verified publication and its own retention policy | Select for the local proof using existing Nginx |
| Multiple releases bundled in every app image | Few services; inspectable, fixed image bytes | Larger images; every rollout and rollback image must include every still-supported graph; old rollback images can lose newer clients' chunks | Useful bounded alternative, but does not simplify bidirectional rollback retention here |

The lab's Nginx Deployment serves a verified public-only union of v1 and v2
assets under the existing `/static/` URLs. Other routes proxy to the Rust Service.
The gateway/static Deployment stays unchanged while Rust rolls forward or back.
There is no object-store dependency, runtime Node process, new core crate or
request-time Rust compilation. This selects a qualification arrangement, not a
universal production hosting recommendation.

Nginx maps only the supplied public asset tree. Missing files return 404, never
an HTML fallback. Dotfiles, build manifests, settings and source files are absent
from that tree. MIME types and `nosniff` remain explicit; asset and HTML responses
retain `no-store`. We do not infer immutable caching or cryptographic identity
from Vite's short filename hashes. See [Nginx file mapping](https://nginx.org/en/docs/http/ngx_http_core_module.html#root)
and [proxy routing](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_pass).

## Publication, identity and rollback

1. Produce two actual frozen Rust/Vite builds. Record source inventories, locks,
   image IDs and SHA-256 of the complete public graphs, including lazy JS/CSS.
2. Construct the union from verified public files. The same URL with different
   bytes is a publication error; never overwrite it silently. Retain full hashes
   independently of filenames. Reject private files and escaping paths.
3. Publish the union to every static-service replica and verify HTTP bytes before
   starting v2 HTML. Publication failure leaves v1 selected; application startup
   must not compensate by downloading or installing dependencies.
4. Roll Rust from v1 to v2. Both versions may issue HTML during overlap, so both
   complete graphs remain reachable from every public serving path.
5. Roll Rust back to v1 while keeping the same union. A tab that already received
   v2 must still load its lazy graph. Restoring old HTML alone is insufficient.

This graph policy covers these public read-only sample pages. Compatibility of
old frontend code with changed APIs, schemas, authenticated state or actions is
an application release contract and is not established by retained bytes.

## Retention and garbage collection

The operator chooses the supported client lifetime `L`, rollback pins and a
propagation/clock margin `E`. A release's retirement time `R` is the last possible
issuance of its HTML, after all serving replicas have stopped issuing it.
Deletion is eligible only after `R + L + E`, with neither active nor rollback pin.
Serving the release again resets retirement; a rollback resets this calculation.
Cached HTML or offline support would need an additional bounded lifetime contract.

Production values are deliberately unspecified. The lab uses a compressed
30-second client lifetime and 5-second margin measured from the final retirement
of v1 after rollback and another forward rollout. This exercises ordering, not a
day-scale durability claim. Before eligibility, keep the entire union. After
eligibility and explicit removal of the v1 rollback pin, publish a verified
v2-only tree to every static replica. Preserve shared files still reachable from
v2. Record the expected 404 for an expired v1-only chunk separately from failures
inside the promised compatibility window. Never use an access-log lull alone as
proof that all older tabs are gone.

## Local availability and draining assumptions

Read-only preflight observed three existing Multipass K3s nodes Ready, all
`v1.36.4+k3s1`. Reuse them with one uniquely owned namespace; do not reconfigure,
restart or delete VMs, existing namespaces, OpenBao or Valkey resources.

| Parameter | Local qualification choice |
| --- | --- |
| Rust replicas | 3; initially one per node, with actual placement recorded for every revision |
| Gateway/static replicas | 2, on distinct nodes |
| Rust update strategy | `maxUnavailable: 0`, `maxSurge: 1`, `minReadySeconds: 2` |
| Probes | Generated live/ready handlers; one-second readiness sampling in this lab |
| Termination | 30 seconds total; native kubelet preStop sleep of 5 seconds, then generated SIGTERM/Axum graceful shutdown |
| In-flight request | One app-only fixed eight-second handler, observed starting before its Pod is deleted |
| Disruption | Delete one identified app Pod with ordinary grace, without altering other workloads |
| Network | Same-origin HTTP through the lab gateway Service; no public Internet/TLS or external load-balancer claim |

Readiness, termination and endpoint propagation are asynchronous. The five-second
delay is a measured lab allowance, not a propagation guarantee. The grace budget
includes the hook; remaining in-flight work must finish before forced termination.
The chosen replica/update settings preserve desired capacity only under the
declared scheduling and resource assumptions. See [Deployment strategy](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/#strategy),
[Pod termination](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-termination-flow)
and [lifecycle hooks](https://kubernetes.io/docs/concepts/containers/container-lifecycle-hooks/#hook-handler-execution).

Direct Pod deletion does not demonstrate PodDisruptionBudget enforcement. Any
future PDB claim needs the Eviction API and its own evidence; PDBs also do not
prevent every involuntary failure. See [Kubernetes disruptions](https://kubernetes.io/docs/concepts/workloads/pods/disruptions/).

The initial plan anticipated three-node placement after each rollout. Native
execution observed the first v2 revision on two nodes: two replicas on one and
one on another. The chosen spread selector counts overlapping revisions and
does not establish per-revision placement. Preserve that observation and report
the original single-Pod disruption criterion under the actual placement; the
advisory placement assumption does not create a new node-failure test or justify
another rollout. Node-failure availability remains unqualified.

## Required observations

| Stage | Evidence and acceptance |
| --- | --- |
| v1 baseline | Actual SSR, health, CSP and public graph bytes; old browser tab has not requested its lazy chunk |
| v1 to v2 | Continuous HTTP observation; new pages identify v2; held v1 tab loads v1 lazy JS/CSS without document reload |
| Rollback to v1 | New pages identify v1; a held v2 tab still loads v2 lazy JS/CSS |
| Single-Pod disruption | Concurrent traffic remains observed through termination/replacement; selected eight-second request completes; replacement becomes Ready |
| Protected retention window | All required v1/v2 graph bytes remain available, including late imports |
| Deliberate expiry | After deadline and pin removal, v2 graph stays valid and a v1-only missing file returns the documented expected 404 |
| Cleanup | Owned namespace/resources, task image tags and temporary uploads removed; gateway listener closed; original nodes/workloads preserved |

For each rolling/disruption phase, sample at least ten HTTP requests per second
throughout the operation and ten seconds beyond readiness recovery, with a
two-second request timeout. Retain each request's monotonic start, duration,
status/error and release identity. Report request counts, 404, 5xx, connection
errors, timeouts and latency distributions by phase. Do not hide retries in the
measurement client. Zero observed failures qualifies only the measured run;
otherwise retain the failures and do not claim zero downtime. An expected expiry
404 is a separate policy test, never subtracted from an unlabelled total.

Browser evidence must show real delayed imports and computed lazy CSS from held
tabs, not a new navigation or warm module-cache replay. Direct graph-byte checks
complement that browser evidence. A synthetic missing-file request verifies that
an HTML fallback cannot disguise a missing JS chunk.

## Execution and reporting boundary

Preparation may create only task-owned build files, local image tags and temporary
containers. Lab mutation is limited to its isolated namespace, exact task image
imports and owned uploads. No cluster-wide policy, node lifecycle, registry push,
cloud provisioning, production rollout or global tool install is part of this
slice. Record resource identities before mutation and verify deletion afterward;
do not use global pruning or delete pre-existing resources.

The final record must distinguish the reviewed plan, actual execution, failures,
browser evidence and cleanup. YT-103 remains open until those required observations
and independent review pass; the linked execution record now satisfies those
conditions at the bounded local scope. This does not unblock YT-096/YT-020 or assert node,
host, zone, database, storage, network or production-service availability.
