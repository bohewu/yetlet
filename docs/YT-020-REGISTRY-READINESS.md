# YT-020 — Sixteen-package crates.io readiness inventory

Date: 2026-10-30 (Asia/Taipei). Status: **Inventory complete; registry activation
remains Blocked.** This is read-only registry research and documentation, not
publication approval or new package qualification.

## Scope and source

The owner authorized committing the seven reviewed upload-tool/SOP paths with
strict/builtin Guard, then inventorying sixteen crates.io names, owner rights,
publication order and missing verification. Commit
`6371ba822a766c9682524cbbd3e71d7aac16d669` passed the direct staged Guard and
native hook; no Push occurred. Its tree is
`fdfeaf9642b6a05096c4992b009377a9f6ac6fd6`. This report and its navigation/status
updates are subsequent, uncommitted documents.

The reference product source remains alpha.3 C
`557ab76e03b619e959eae7529dbe03317fe1f952`, immutable tag `v0.9.0-alpha.3`;
actual signer/verifier S/V remains
`30baf2504a14859223416bccdc2d05755ae000f6`. The inventory commit changes no
library payload, manifest or lockfile relative to C. All sixteen source versions
are `0.9.0-alpha.3`; their publish settings permit crates.io. The retained
normalized archives and current source agree on all eighteen internal dependency
declarations, including optional, target and versioned development dependencies.
Each retained archive digest matches its original provenance subject.

This does not select a new registry candidate or authorize reusing a GitHub
publication grant for crates.io. CLI, samples, benchmarks, common loader assets
and the physics Lab remain outside the sixteen-library cohort.

## Live names and public account observations

Unauthenticated GET requests to `https://crates.io/api/v1/crates/<name>` ran
between **10:02:33 and 10:02:57 UTC+08**. All sixteen returned HTTP 404 with an
explicit crate-does-not-exist JSON error; no lookup was unknown. No public owner
record exists to inspect for those requested names at this observation point.

| Requested name | Registry observation | Owner/publish right |
|---|---|---|
| `yetlet-actions` | Not found (404) | Unverified |
| `yetlet-auth` | Not found (404) | Unverified |
| `yetlet-config` | Not found (404) | Unverified |
| `yetlet-core` | Not found (404) | Unverified |
| `yetlet-dom` | Not found (404) | Unverified |
| `yetlet-dom-keyed` | Not found (404) | Unverified |
| `yetlet-html` | Not found (404) | Unverified |
| `yetlet-rate-limit` | Not found (404) | Unverified |
| `yetlet-reactive` | Not found (404) | Unverified |
| `yetlet-request-id` | Not found (404) | Unverified |
| `yetlet-secrets` | Not found (404) | Unverified |
| `yetlet-server-axum` | Not found (404) | Unverified |
| `yetlet-session` | Not found (404) | Unverified |
| `yetlet-session-redis` | Not found (404) | Unverified |
| `yetlet-signing` | Not found (404) | Unverified |
| `yetlet-wgpu` | Not found (404) | Unverified |

The public [bohewu account API](https://crates.io/api/v1/users/bohewu) returned
HTTP 200 at 10:10:28 UTC+08: login `bohewu`, user ID `446398`, with a matching
GitHub username. This confirms a public account, not possession of its session,
verified email, token scopes, new-crate permission or Trusted Publisher setup.
No credential values or authenticated registry settings were inspected.

**Name absence is not a reservation or proof of claimability.** Registry naming
rules, allocation changes and server-side acceptance still apply. GitHub repo
ownership, Release upload rights and Sigstore identity do not establish registry
publisher rights. The [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
describes account/email setup, first-come name allocation and publication
permanence. Recheck live names before an authorized first upload; do not create
placeholder packages merely to reserve them.

## Dependency order and first-publication caveat

The normal/build dependency graph, including optional and target declarations,
is acyclic. These are dependency layers, **not an executable sixteen-command
first-publication plan**:

| Layer | Packages, after their earlier-layer dependencies |
|---|---|
| 1 | `yetlet-actions`, `yetlet-core`, `yetlet-rate-limit`, `yetlet-reactive`, `yetlet-secrets`, `yetlet-signing`, `yetlet-wgpu` |
| 2 | `yetlet-config`, `yetlet-dom`, `yetlet-dom-keyed`, `yetlet-html` |
| 3 | `yetlet-server-axum` |
| 4 | `yetlet-request-id`, `yetlet-session`, `yetlet-session-redis` |
| 5 | `yetlet-auth` |

Versioned development dependencies require separate first-publication planning:

- `yetlet-config` optionally depends on `yetlet-secrets` at runtime.
- `yetlet-secrets` has versioned development dependencies on `yetlet-config`,
  `yetlet-core` and `yetlet-server-axum`.
- `yetlet-config` and `yetlet-auth` also have versioned development dependencies
  on `yetlet-core` and `yetlet-server-axum`; `yetlet-rate-limit` has one on
  `yetlet-config`.

Consequently a conservative ordering that requires every versioned development
dependency to exist first has a Config/Secrets cycle and leaves Rate Limit
waiting on Config. This is a packaging/bootstrap question, **not a runtime
dependency cycle or a demonstrated registry rejection**. Cargo documents that
[versioned development dependencies remain in published packages](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#development-dependencies).
The exact first-upload behavior needs focused evidence; do not silently remove
these dependencies or disable package verification.

Cargo has supported [native multi-package publication and dry-run verification
since 1.90](https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/). It can verify
selected packages as though the group were published; uploads remain non-atomic.
The root workspace contains eleven selected libraries. Config, Secrets,
Redis Session, Signing and WGPU each have their own workspace boundary. A root
`cargo publish --workspace` therefore does not select all sixteen, and the root
also contains nonpublishable development packages. Start with Cargo's existing
group selection/packaging behavior when planning the bootstrap; do not introduce
a new publisher framework or change workspace membership during this inventory.

## Reuse accepted evidence; close only registry-specific gaps

The [alpha.3 record](ALPHA3-HOSTED-RECORD.md#signing-independent-verification-and-publication--2026-10-30)
already records the selected source, sixteen package subjects, build/readback,
two signatures, protected independent verification and exact public attachments.
The [version preflight](ALPHA3-VERSION-PREFLIGHT.md) retains normal package checks,
external packaged-consumer proof and affected lock/source reconciliation.
Prior platform, browser and security results retain their exact scopes; this
inventory does not replay them or claim new test passes.

| Remaining condition | Current state and smallest next evidence |
|---|---|
| Registry source/cohort/version and authority | Alpha.3 is the reference, not a newly approved registry batch. Fix the actual immutable source, sixteen-name allowlist, failure limits and upload authority before execution. |
| Account and name rights | Public account exists; email/session/token or Trusted Publisher rights and name claimability remain unverified. Confirm through the intended account without putting credentials in Git, arguments or reports. |
| Bootstrap and registry-facing preflight | Resolve the Config/Secrets development edge across independent workspaces; use a bounded normal Cargo publish dry-run for the selected group/order. No such registry-specific dry-run ran here. |
| Archive identity | Compare actual publish-package bytes to the accepted sixteen archives before upload. All accepted archives omit `.cargo_vcs_info.json`; a Git checkout or regenerated lock metadata can change repackaged bytes. Changed bytes require their own evidence binding, not a claim that old signatures cover them. |
| Actual registry receipt | After separately authorized upload, record each accepted version/index entry, download/checksum all sixteen actual registry archives and verify a clean consumer without local path or directory-source overrides. This cannot be completed by local package evidence alone. |
| Partial publication/failure handling | Record per-package outcomes. A client index-wait timeout can follow a successful upload; inspect registry state before any separately approved retry. Preserve partial success and never try to overwrite a published version. |

The [Cargo publish reference](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
distinguishes packaging, verification, upload and index polling; its `--dry-run`
does not upload. [Cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html)
describes archive normalization. A dry-run proves neither authenticated write
permission nor successful server acceptance.

**Recommended next bounded step:** settle the first-publication bootstrap and
archive-identity contract, then validate only that missing registry-facing path
with normal Cargo verification. Confirm the intended publisher's email and
credential method alongside that plan. Select and authorize actual publication
only after these results; no new framework feature, platform matrix or repeated
benchmark is needed for this gap. If metadata changes become necessary, review
their source/version/evidence impact before changing the accepted candidate.

## Retained observations and limits

Raw public JSON, HTTP status/date receipts, package metadata/dependency graphs
and the seven-path commit/Guard receipt are retained under
`target/yt020-registry-inventory-20261030/`. These local files are observations,
not public signed release attachments. The first local inventory serializer
failed on a Python set; the retained wrapper note records the single sorted-list
correction before a successful report. It did not change product/package input
or invalidate any accepted release result.

No registry login, token creation, owner mutation, namespace claim, publish
dry-run, build, upload, new tag, hosted run, signing or Git Push was performed.
YT-020 remains **Blocked** pending the registry-specific conditions above;
historical nine-package Private preparation/signatures and all later failures
remain unchanged. No independent-review acceptance is claimed for this inventory.
