# YT-089 — Opt-in Container and Kubernetes Manifest Generation Plan

**Date:** 2026-10-16. **State:** M15 architecture independently reviewed PASS after two bounded Docker context/source-closure corrections; [YT-089a cloud runtime](YT-089A-VERIFICATION.md) independently accepted by Reviewer `wc_job_0dvTDNAV22Vbpc5j` (0 issues/followups) after actual Linux early SIGINT/SIGTERM/HTTP/drain and GNU/MSVC app builds; [YT-089b Docker generator](YT-089B-VERIFICATION.md) also received independent Reviewer `wc_job_IC92mEXoQMsG4nTg` PASS, after a Linux image/runtime smoke and 8/8 GNU/MSVC output parity; 089c is independently accepted by Reviewer `wc_job_OCVGSYl5uAGET8w3` PASS; [089d Kubernetes](YT-089D-VERIFICATION.md) has independent Reviewer `wc_job_1SJSI37ZToF_od0U` PASS after source-bound GNU/MSVC config parity and isolated actual Linux Multipass K3s Pod/HTTP/cleanup; [089e final closeout](YT-089E-VERIFICATION.md) has final independent Reviewer `wc_job_ci3bjlyeh2KHRhta` PASS (zero issues/followups) after source-bound Linux/GNU/MSVC regressions. See [YT-089a evidence](YT-089A-VERIFICATION.md). The maintainer accepted **configuration generation rather than coupling Yetlet to deployment platforms** after the separately reviewed YT-087/YT-088 Linux/Windows CLI was committed at `d24a05033c57e5b98dd57bf83189bc1297f6f28c`. Cloud, Docker, Compose and Linux-targeted Kubernetes generator slices are independently accepted at their bounded scopes; 089e final integration is independently accepted, completing M15 at bounded configuration-only preview scope. The parent owns separately authorized isolated Linux K3s validation; this coding slice performs no cluster, image publication or release action.

**First independent design review:** Read-only Reviewer Job `wc_job_z7yv3PoGCoUC1PZJ` returned **FAIL** on two specific Docker source-context blockers: (P1) the original app-as-default-context/`.dockerignore` placement could leak source/secrets, and (P2) the original Yetlet framework export did not account for transitive `yetlet-html` and inherited Cargo workspace metadata. The three-context, deny-by-default/export-closure contracts below are corrective **design requirements, not observed Docker build successes**. The initial failure remains historical; a separate focused design re-review is required before implementation.

**Focused independent design re-review:** Read-only Reviewer Job `wc_job_9IplTI1HHkyZK9hv` returned **PASS**, `issues=[]`, `required_followups=[]`, after validating the three physically separate build contexts and deny-by-default source-export rule (P1) and the exact `yetlet-core`/`yetlet-html`/`yetlet-server-axum` transitive closure, minimal `[workspace]` manifest and inherited metadata (P2) against actual Cargo sources. The original FAIL stays documented. This PASS accepts **design only**: no source export helper, Dockerfile, image, Kubernetes YAML or cloud app runtime existed at that review, and no release, cluster action or commit was authorized. The separately authorized **YT-089a** implementation below does not inherit implementation acceptance from this design PASS.

## Decision and responsibility boundary

**Yetlet owns local project scaffolding and deterministic deployment-file generation.** The application owns routing, readiness business policy, secrets and hosting decisions. **Docker, Compose, Kubernetes, registries and operators own actual image building, publishing, deployment, runtime networking, scaling, backups and operational SLOs.**

- Generation is an **opt-in Rust CLI feature**, not a required framework crate, middleware, control plane, Docker/Kubernetes client SDK, long-running reconciler or hidden agent.
- No `docker build/run/push`, `kubectl apply`, Helm install, registry authentication, package installation, public-release activation or network access as a side effect of generation.
- Keep the accepted Linux and Windows x86-64 `doctor/new/build/dev` public contracts, strict source/Cargo.lock handling, CSP, escaping, no mandatory JavaScript/WASM and handler-first Axum/Tower interoperability unchanged.
- **Outputs target Linux containers** regardless of whether the generator runs on Linux or Windows; do not claim Windows container images, multi-architecture publication or a production-certified Kubernetes application.
- Existing YT-020 public-release readiness remains **Blocked**; generated manifests are not a production-release authorization.

## Proposed CLI user experience — subject to design review

```text
yetlet generate docker --manifest-path APP/Cargo.toml --output DIR
yetlet generate compose --manifest-path APP/Cargo.toml --output DIR --image IMAGE
yetlet generate k8s --manifest-path APP/Cargo.toml --output DIR --image IMAGE
```

These are **candidate command signatures**, not yet implemented or a frozen API contract. Avoid implicit cluster/environment discovery and credential reads. Choose explicit image names, container ports, deployment context and options for replicas/host only when a real implementation contract is established. Invalid options are usage errors.

The generator requires a standalone application Cargo manifest and its explicit `Cargo.lock`; path dependencies stay local. An external app without the required health/bind contract must receive a diagnostic or explicitly opt out of unsupported probes, rather than getting a seemingly runnable but broken Kubernetes manifest.

Generated files are immutable unless the user explicitly changes an output policy in a later reviewed task. Default to **new output directory only**, under the existing platform-safe Linux directory-FD/no-replace or Windows NTFS/ACL/reparse-safe adapter; reject traversal, symlink/junction substitution, existing destinations and unsafe parent ownership. Stage files in an owned temporary sibling and publish atomically/no-replace; unknown outcomes must not trigger blind overwrite or recursive deletion. Templates and filenames are deterministic and correctly escape TOML, Docker, YAML, shell/PowerShell, image references and Unicode names. Never serialize environment secrets, absolute personal home paths, .env contents or sensitive build-context files.

## Prerequisite: small application-owned cloud execution contract

Current `tools/yetlet-cli/templates/main.rs` binds **only** `127.0.0.1:YETLET_PORT`, has no dedicated probe routes, and calls `axum::serve` without an application SIGTERM graceful-shutdown future. Those are genuine runtime gaps; generating only YAML cannot resolve them.

**YT-089a implementation under verification:** `new NAME --framework-path PATH
[--dir DIRECTORY] --cloud` selects separate cloud main/README templates and adds
only the generated app's Tokio `signal` feature. The original default templates
remain unchanged. The cloud profile accepts only `127.0.0.1` (default) or explicit
`0.0.0.0`, and ASCII decimal ports 1 through 65535 (default 3000), rejecting invalid
or non-Unicode settings. Its Cargo metadata marker is
`[package.metadata.yetlet] cloud-profile = "native-ssr-v1"`; future generators may
recognize this declaration without guessing, but it does not attest edited source.
See [verification and pending gates](YT-089A-VERIFICATION.md). No 089b–e
generator or operational action is part of this slice.

The **opt-in cloud application template/profile** should supply:

- `YETLET_HOST` (default `127.0.0.1` locally; **explicit** container setting `0.0.0.0`) and `YETLET_PORT` (local existing default 3000, generated container setting 8080); invalid addresses/ports fail immediately. Do **not** silently expose applications on all interfaces outside the explicit cloud profile.
- Application-owned `GET /health/live` and `GET /health/ready` handlers, returning simple non-sensitive HTTP 200 without server state disclosure. Readiness must become unavailable during graceful termination and reflect only what the app can actually verify; do not equate a static 200 with readiness of Redis, SQL or other opt-in dependencies. Routes remain opt-in to avoid collisions in existing apps.
- `tokio::signal` support for SIGTERM on Linux and Ctrl-C where appropriate; use Axum's existing graceful-shutdown support rather than introducing a Yetlet TCP server or process manager. Test actual in-flight drain, termination timeout and no orphan listener. Kubernetes `terminationGracePeriodSeconds` must cover the selected drain budget.
- Publicly documented environment contract, stdout/stderr diagnostics with no secrets, non-root runtime compatibility, no writable application filesystem assumption, and explicit reverse-proxy TLS/ingress ownership.

For existing external apps, the generator must **not edit their handler/main source by default**. It may generate an optional small source snippet or require an explicit, verified health/bind contract. If the app lacks the expected readiness endpoint, fail closed or use an explicitly labeled TCP-only readiness mode; never emit HTTP probes guaranteed to return 404.

## Docker image generation

Generate `Dockerfile`, `.dockerignore`, a short build/run README and, where justified, a **generated container-only Cargo manifest overlay**. Its own source and main application's `Cargo.toml`/`Cargo.lock` must remain intact.

**Critical local-source dependency problem:** Existing Yetlet applications use `yetlet-core` and `yetlet-server-axum` with absolute local Cargo `path` dependencies, not published crates. A Dockerfile that simply copies `APP/` and runs `cargo build` will fail. The YT-089b design therefore requires **three physically separate, explicit BuildKit contexts**, never the live app or dirty framework checkout:

```text
DIR/                            # only generator-owned configuration; Docker's DEFAULT context
  Dockerfile
  .dockerignore                 # deny-by-default, allow only Dockerfile and cargo-overlay/*
  cargo-overlay/Cargo.toml      # generated APP overlay; original APP manifest unchanged
  build-context-inventory.json  # source/export contract; not automatically sent to Docker
APP_EXPORT/                     # operator-prepared, validated/sanitized named context app-src
  Cargo.lock                    # byte-exact original APP/Cargo.lock
  src/                          # explicit, audited application source files
  ...                           # only separately declared public assets, if supported
FRAMEWORK_EXPORT/               # operator-prepared, validated/sanitized named context yetlet-src
  Cargo.toml                    # GENERATED MINIMAL workspace root, not raw monorepo root
  crates/yetlet-core/           # declared package manifest and required source
  crates/yetlet-html/           # declared package manifest and required source
  crates/yetlet-server-axum/    # declared package manifest and required source
```

The generated README gives an **explicit, reviewed** user build command. Conceptually:

```sh
docker buildx build --build-context app-src=APP_EXPORT \
  --build-context yetlet-src=FRAMEWORK_EXPORT \
  --file DIR/Dockerfile --tag EXPLICIT_IMAGE DIR
```

The final positional `DIR` is the **only default Docker build context**. It contains deployment configuration only. `COPY --from=app-src` reads the sanitized application export (never `APP/`), `COPY --from=yetlet-src` reads the sanitized framework export, and `COPY cargo-overlay/Cargo.toml` places the generated container-only manifest in the image **after** copying app sources. The raw application's absolute-path `Cargo.toml` is **not in APP_EXPORT**, so neither its personal paths nor unrelated inputs are transmitted. `DIR/.dockerignore` applies to the default context; `APP_EXPORT` and `FRAMEWORK_EXPORT` also use physically allowlisted file inventories and verified root-level `.dockerignore` rules. Do not place `DIR` outside the context and assume its `.dockerignore` protects `APP/`: Docker normally selects the context-root ignore file or a Dockerfile-specific ignore file. This layout deliberately avoids that ambiguity. Document and test actual effective build-context transfer, including a planted `.env`, SSH key, dirty `target/`, `.git`, symlink and secret fixture: **none may enter any of the three contexts**. Reject unknown extra files, symlinks, changed hashes, path traversal, unsafe ownership or unrepresentable assets before a supported image build.

**Preparation remains an explicit, non-deployment action.** The generator writes configuration and an allowlist/hashes/recipe, **not** arbitrary source copies or a `docker build` invocation. The operator prepares both sanitized exports in protected fresh directories and verifies their inventories against the generated manifest. A later separately reviewed local source-export helper may automate this step; absent verified exports, the README must not claim one-command runnable Docker builds. The supported initial APP layout is exactly the existing small `yetlet new` standalone template (`Cargo.lock`, `src/**/*.rs`, explicitly reviewed non-secret assets only); apps with `build.rs`, arbitrary asset discovery, extra path dependencies or unknown Cargo features must fail closed or have a separately reviewed explicit export profile.

**Cargo workspace closure (initial supported framework graph):** `yetlet-server-axum` depends on both `yetlet-core` and `yetlet-html` (whose own dependency points back to `yetlet-core`). The three packages also inherit `version`, `edition`, `rust-version`, `license` and `repository` from the root `[workspace.package]`; `yetlet-server-axum` inherits `axum`/`tokio` from `[workspace.dependencies]`. The `FRAMEWORK_EXPORT/Cargo.toml` must therefore be a **deterministic minimal workspace root** with `members = ["crates/yetlet-core", "crates/yetlet-html", "crates/yetlet-server-axum"]`, `resolver = "3"`, and the **exact relevant source values** from `[workspace.package]` and `[workspace.dependencies]`. Copy all required source/manifests and supported package metadata for these three crate directories; recursively verify the exact **path-dependency closure** and fail closed on an unhandled path crate or workspace inheritance key. Do not silently copy the original root workspace manifest, which lists many unrelated examples/crates absent from the sanitized export.

The `cargo-overlay/Cargo.toml` is generated by parsing the **exact** application manifest, preserving all supported package/dependency/features/profile/workspace semantics and changing **only** the two source paths `yetlet-core` and `yetlet-server-axum` to container-internal `/work/yetlet/crates/...` locations. The internal `yetlet-html` relative paths remain valid inside `FRAMEWORK_EXPORT`. Do not substitute registry dependencies, rewrite versions/features, invent target dependencies or regenerate `Cargo.lock`; `APP_EXPORT/Cargo.lock` is the byte-exact source lockfile. The generator must compare normalized Cargo manifest semantics before/after only the authorized two path changes and reject unsupported manifest layouts. **YT-089b requires a real `cargo metadata --locked` plus `cargo build --locked --release` in the three-context container build** to prove workspace inheritance, dependency-closure and unchanged app lock, not just syntactically valid Dockerfile text. This initial graph is an allowlist, not a general Cargo dependency solver.

Container image build is a separate **explicit** user action. `cargo build --locked --release` within Docker may resolve already declared registry crates over the network on first build; this must be stated honestly and never be described as fully offline/frozen unless all dependencies are vendored/cached. Generate a reproducible lock-preserving build contract, recommended pinned builder/runtime base image policy, a minimal non-root runtime (e.g. glibc-compatible base for the selected Rust GNU target), exec-form process start, app-owned listening address/port and non-secret health verification. Do not emit a `HEALTHCHECK` that assumes `curl`, `wget` or `sh` exists in a distroless image. A small tested probe helper or an explicit no-HEALTHCHECK limitation is preferable to a fake green signal. Document runtime image/architecture, libc/musl and DLL assumptions separately.

## Docker Compose generation

Generate `compose.yaml` that references an **explicit image name** and documented user-built image. Use app `YETLET_HOST=0.0.0.0`, consistent container port (default 8080), and conservative localhost-only host port mapping unless the user explicitly chooses exposure. No implicit volume for framework source or secrets, privileged container or registry credential injection. If using Compose `build.additional_contexts`, only do so after verifying the required Compose/BuildKit version and matching Dockerfile source overlay, never assume it silently resolves local Rust path dependencies.

Compose healthcheck must be backed by a tool actually present in the image (or be intentionally omitted with an explicit limitation). Verify `docker compose config` offline where locally available, and test runtime HTTP/CSP/escaping and actual `docker stop` SIGTERM cleanup on a disposable image when the user separately authorizes an image build. Preserve exact image/port contract consistency across Dockerfile, Compose and Kubernetes manifests.

## Kubernetes generation (generic K8s/K3s compatible)

Generate at least a namespaced-by-operator-context `Deployment` and internal `ClusterIP Service` in standard Kubernetes YAML (`apps/v1`, `v1`); do **not** hardcode a namespace, kubeconfig, cluster credentials, load balancer IP, private registry, TLS secret or Ingress hostname. `--image` must be explicit. Do not automatically create or apply Namespace, Ingress, Helm chart, HPA, RBAC or Secret resources. Optional Ingress is a separately reviewed flag requiring an explicit host/TLS/ingress-class contract.

- Include meaningful `startupProbe`, `livenessProbe` and `readinessProbe` pointing to the tested application-owned paths and a named container port. Readiness removes unavailable Pods from Service traffic; it is not automatically proof of external dependencies.
- Set `terminationGracePeriodSeconds` aligned with the app's real SIGTERM drain behavior. Include conservative, documented example resource requests/limits rather than fabricated throughput/capacity assurances. Replicas are user-controlled; never claim HA for an untested deployment.
- Prefer `runAsNonRoot`, `allowPrivilegeEscalation: false`, `capabilities.drop: [ALL]`, `seccompProfile.type: RuntimeDefault`, `readOnlyRootFilesystem: true` where verified compatible, and `automountServiceAccountToken: false` by default. Add writable scratch `emptyDir` only when required by the actual app; no hostPath or privileged runtime by default.
- Handle non-sensitive environment settings with explicit `env` and optional references; never place raw credentials in generated ConfigMaps, labels, sample YAML or CLI diagnostics. Provide placeholders for Secrets refs, not fabricated secret values. Do not require the CLI to hold cluster credentials.
- Verify rendered YAML/schema and invariant names/ports/labels; a real `kubectl apply --dry-run=client` is extra evidence only when its schema and host execution can be shown to be offline and non-mutating. A successful client dry-run alone is **not** proof of a working Pod.

Target Kubernetes-compatible output, including K3s, without coupling to a distribution-specific CRD or an external ingress/controller installation.

## Ordered implementation and independent evidence gates

| Slice | Deliverable | Gate |
|---|---|---|
| YT-089a | Opt-in native SSR cloud profile: explicit bind/port, two health routes, graceful SIGTERM/Ctrl-C, documentation | Linux native HTTP health/SSR/CSP/escaping + real SIGTERM drain/cleanup; no extra framework dependency or default public listen; Windows existing CLI regressions; independent Reviewer PASS |
| YT-089b | `yetlet generate docker` and safe no-overwrite file generation; tested source/build-context/manifest-overlay contract | Rust unit and negative tests for output ownership/races, paths/images/manifest mapping, no secret leakage and no CLI network/build; actual disposable Linux image build + HTTP when Docker capability/authorization exists, else honestly Blocked; independent Reviewer PASS |
| YT-089c | Opt-in Compose YAML generator sharing port/image/build assumptions | Deterministic YAML, actual `docker compose config` and bounded disposable HTTP/stop where available/authorized; no secret/plaintext/unsafe exposure; independent Reviewer PASS |
| YT-089d | Opt-in generic Deployment/Service manifest generator | Validated YAML schema, probes/resources/security context, labels/service selection and external-image validation; no real cluster mutations; optional separately authorized ephemeral K3s/K8s dry-run; independent Reviewer PASS |
| YT-089e | Documentation, Linux/Windows generator parity and final source/evidence review | Source-bound Linux + Windows GNU/MSVC CLI fmt/check/tests/strict Clippy and deterministic output comparison, existing YT-087/088 regressions, docs/sample checks, true end-to-end Docker where available, final independent Reviewer PASS |

Every slice remains **Not accepted** until code, actual evidence and its separate Reviewer PASS are recorded. YT-089a/b/c/d each have distinct independent PASS results. The [YT-089e](YT-089E-VERIFICATION.md) final Linux/GNU/MSVC source/doc reconciliation also obtained independent Reviewer `wc_job_ci3bjlyeh2KHRhta` PASS, closing M15 at bounded unpublished Linux OCI/Kubernetes configuration-generator scope. Do not mark skipped runtime checks as success or let a generated YAML manifest stand in for a verified application. Owner authorization for planning is **not** blanket authorization for cluster deployment, image publication, registries or Git commit/push.

## External standards and design reference pointers

- Docker named build contexts and BuildKit overlays: https://docs.docker.com/build/concepts/context/
- Dockerfile `STOPSIGNAL` / `HEALTHCHECK` semantics: https://docs.docker.com/reference/dockerfile/
- Kubernetes startup, liveness and readiness probes: https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/

All operational assumptions are to be checked against the actual tested Docker/BuildKit/Compose/Kubernetes versions before claiming compatibility. No distribution-specific controller is mandatory.
