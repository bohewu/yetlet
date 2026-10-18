# YT-102 — Current CLI scaffold generator compatibility

Date: 2026-10-18. Status: Done at bounded local-source compatibility scope;
independent Reviewer PASS, zero issues and required followups.
Source baseline: `9850820f4ea5e35f5b09e521d8c24e00aa937d89` plus this working patch.
Evidence: `artifacts/cli-generator-compat-20261018/`.

## Authority and bounded change

The owner explicitly authorized continuing compatibility after identifying that
the revised CLI's settings/frontend scaffolds were refused by deployment
generators. This is an implementation extension of the accepted
[CLI contract](CLI-CONTRACT.md), separate from YT-102's broader research-only DX
audit. YT-103 deployment qualification and all earlier unrelated planning changes
are preserved. No commit, push or deployment is part of this slice.

Docker, Compose and Kubernetes now accept exact current service scaffolds:

| Creation | Required preparation |
| --- | --- |
| `new NAME --service` | Compatible explicit local Cargo paths and separately prepared lock; no runtime settings dependency |
| `new NAME --service --settings` | Same, plus exact public runtime TOMLs and a separately supplied runtime token |
| `new NAME --service --vite` | Settings implied; explicit complete production frontend build |
| `new NAME --service --vite --tailwind` | Same with the pinned official plugin |

The manifest keeps its generated pinned Yetlet versions while the operator adds
absolute local path dependencies to one qualified framework checkout. Versions,
features, dependency sets and Rust/frontend source bytes remain exact. Registry
declarations alone do not provide the source closure needed by this local
generator; usable registry-only delivery remains a release/source-qualification
gate. No framework-source option, custom resolver or source environment variable
was introduced. Plain non-service apps and standalone Tailwind remain outside
the deployment profile.

## Settings, paths and runtime assets

Inspection reuses the current scaffold assembly and existing strict settings
loader. Every public CLI profile is validated; declared CLI values may vary.
Runtime server/service public tables still match the established generated
contract, including empty token declarations. No backend Development secret or
ambient backend APP values enter inspection. Kubernetes replica selection keeps
the existing public settings/ENV mechanism.

The Production file settings select the frontend root for the image's asset
closure. Development/Staging may use different valid roots. Existing protected
frontend directory handles read the finite inputs and built graph. Only three
exact public names were added to the native input allowlist:
`vite.dev.config.mjs`, `vite.dev.bridge.mjs` and the generated `.env.example`.
Arbitrary dotenv and credential files stay outside it.

Docker inventory schema 1 adds `app-src-source-paths` only for the current
profile. It maps every non-synthesized exported app file to its project-relative
input, including canonical `dist/` destinations from `frontend/dist/` or a custom
root. Operators explicitly prepare and hash-check three physically separate
contexts. No automatic export is introduced; legacy inventories and fixed output
hashes remain unchanged.

The current app-owned Vite loader now honors the existing explicit absolute
YETLET_ASSET_ROOT. Images already select `/opt/yetlet/assets`; normal local
execution retains its configured project-relative frontend root. Public CLI
TOMLs are delivered independently of the optional runtime settings dependency.
Generation starts neither watch nor Vite and performs no frontend build,
regardless of the execution settings.

## Native and generated-app verification

| Command/check | Observed result |
| --- | --- |
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets` | 110 passed, 0 failed, four existing explicit ignored tests |
| Focused Docker/Compose unit checks | 14/14 and 7/7; legacy fixed-output hash test passed |
| New `generator_selected` integration | Four current profiles, real separately prepared offline locks, all 12 output generations passed with PATH/Cargo cache unavailable to generators |
| `cargo clippy --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets -- -D warnings` | PASS |
| `cargo check --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --target x86_64-pc-windows-msvc --tests` | PASS, cross-target compilation only |
| Real selected service/Vite/plugin app | Explicit local overrides, offline npm install, actual typecheck/build, offline Cargo lock and 14 generated app tests passed |

The product integration test uses a declared finite structural dist fixture;
the separate real app uses actual pinned Vite output. The integration verifies
changed CLI watch/auto switches, replicas 3, custom Production `web-ui/` with a
different Development root, no secret export, untouched app inputs, and refusal
of edited Rust. Existing tests retain stale/extra asset, lock/source substitution,
path/link and no-overwrite checks. Production runtime proof is recorded separately
below; no browser/HMR rerun is needed for this generator/path change.

## Retained failures

During implementation, a TOML missing-key insertion panicked and the legacy native
read seam refused frontend-relative files. Both product defects were corrected:
table insertion preserves the exact manifest, and the existing protected frontend
handle keeps reads finite. A test-only missing borrow was corrected separately.

The new structural dist fixture initially inherited a group-writable host umask.
One correction set only its owned fixture directories to 0700; production
permissions checks were preserved. The real smoke command also initially supplied
an unsupported unqualified image name; exit 2 was retained, then only that argument
was corrected to an explicit registry/repository/tag. The same prepared app and
Docker output continued without recreation. `initial-integration-failure.txt`,
`initial-generate-compose-invalid-image.log` and `live-commands.json` preserve
the relevant observations.

The first image runtime probe incorrectly expected the unselected companion
entry to be served. It returned the intended 404. One correction limited success
assertions to the selected app entry's reachable graph and recorded the companion
404 separately. No app source, generated output or export was changed; only the
focused runtime check repeated from the same frozen contexts.

Removing the temporary `rust:1.99.0-bookworm` tag also removed the pre-existing
Rust digest reference. The exact original immutable digest was restored with
`docker pull rust@sha256:b36c246742b4d323472588f789601be34eb4af053255e2af53969078ee8dcec7`.
The continuation created no base alias. Final inspection matched all five original
image references, with none missing or extra. Original cleanup output and the
restoration receipt are retained; this incident is not hidden as a clean first
attempt (`original-rust-reference-restoration.json`,
`images-after-preservation.json`). No global prune occurred.

## Actual image, cleanup and review

The real app uses a custom Production frontend root and disables automatic
frontend execution. Its Docker inventory was explicitly exported to contexts
with 3 default files, 18 application files and 16 framework files. Every physical
file matched the declared byte count/SHA-256, with exact file allowlists and no
links (`physical-exports.json`). Actual Buildx build/load passed from those three
contexts. The final OCI index was
`sha256:83bdc10a446596d421c99e55053e151cb5131b784bccde669e324f9c6c768e73`,
with runtime config digest
`sha256:281c8bffdb950f18e10b1d5123113e9024ab450111383d9d9a55d3d4bd8aabe8`.
The initial and focused repeated build shared that config digest; an OCI index
also includes build attestation metadata and is a separate identity layer.

The non-root container served SSR with CSP, live/readiness and all seven reachable
JS/CSS/SVG files, including lazy assets. Each asset matched its frozen SHA-256 and
retained nosniff/no-store. The unselected companion returned its expected 404.
`/work/app` was absent, proving startup used the explicit runtime asset root rather
than a surviving build-time project path. No node_modules or dotenv file was
present in the inspected runtime roots. The runtime token remained absent from
responses (`runtime-result.json`).

SIGTERM stop completed with application exit 0. The owned container and image
tag were removed, and the original Rust digest remained available. Only source,
export and log evidence remains intentionally (`live-cleanup.json` and
`live-commands.json`). Source/runtime cleanup evidence is distinct from a
production deployment or browser qualification. Independent Reviewer
`/root/generator_compat_review` returned PASS with zero issues and required
followups (`review.json`). Native role, model and effective effort matched the
dispatch (`review-trace.json`). Review covered the scoped source patch, public
settings/source/path contracts, lock/export closure, product tests, actual runtime
receipts, retained failures and cleanup. Only review-status bookkeeping followed.

## Changed files and remaining boundaries

Runtime implementation paths are `tools/yetlet-cli/src/docker.rs`,
`src/docker/delivery.rs`, `src/docker/tests.rs`, `src/platform/fs/mod.rs`,
`src/scaffold.rs` and `templates/vite/selected-loader.rs`, all relative to the CLI
package. The new product test is `tests/generator_selected.rs`; selected-only
operator guidance is `templates/docker/SELECTED-DELIVERY.md`. CLI and selected
app READMEs, this report, the CLI contract and documentation index describe the
new bounded profile. Core/framework crates, process adapters and legacy generator
templates are unchanged.

Native Windows execution of this extension is not claimed by MSVC compilation.
Registry-only source delivery, arbitrary edited apps, further generator formats,
HA/asset retention and production qualification remain separate. YT-102's full
journey audit remains Todo; this compatibility extension does not change the
YT-096/YT-020 release gates. The next independent application item remains YT-092.

## Subsequent native Windows qualification

The [native Windows GNU followup](YT-102-WINDOWS-GENERATOR-VERIFICATION.md)
verifies the four current service profiles and all 12 generator outputs against
the current source snapshot. It also records the narrow Windows dependency-path
comparison correction, native test counts, preserved earlier failures and exact
source/checkout preservation. This is separate runtime evidence; MSVC compilation
alone remains insufficient. Registry and production boundaries above remain.
