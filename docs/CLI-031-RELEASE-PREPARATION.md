# Yetlet CLI 0.3.1 — Release preparation (YT-250)

**State (2026-11-10): Source preparation in progress.** Resolver and release
consumer integration are implemented in the current checkout. Documentation
alignment and local gates are recorded below. Independent semantic/source-freeze
review, candidate selection, packaging, hosted build/custody, signing and
independent V remain pending. Formal GitHub/crates.io publication is excluded
from this preparation. Published SDK alpha.7 / CLI 0.3.0 and the same Reader
Edition remain authoritative; `site/pages.toml` is unchanged.

## Scope and version authorities

- Exactly one independent `yetlet-cli` package, source version `0.3.1`, using
  existing schema3 / `cli` release machinery and the corresponding
  `yetlet-cli-v0.3.1` tag identity. No candidate C or tag is selected by this doc.
- Build SDK remains exactly `=0.9.0-alpha.7`. Embedded SDK snapshot remains
  alpha.7 at source `5dc8a9a990ac083e07081ea3be008748239c8a58`; its original
  resource/source digests are immutable. No SDK republishing or source relabeling.
- Generated app SDK is independent of those two inputs. The packaged policy is
  exactly `{"revision":"sdk-latest-v2","selection":"latest-semver"}`;
  its SHA256 is `de1d1a1f32c3b5013ca48e9f5931e437802fe40474dbf14208826c5805d2a70b`.
- Without `--sdk-version`, choose maximum available non-yanked SemVer,
  including future releases/prereleases, with required same-version Browser
  package availability. Check required features and Browser dependency after
  selection; fail selected latest without falling back to an older version.
  There is no approval list, stable-first preference, build-SDK admission bound
  or frozen MSRV ceiling. Cargo owns actual toolchain/dependency feasibility;
  future compiler/build/API success is not established by metadata or a lock.
- Explicit overrides stay exact. Generated registry pins, app Cargo.lock,
  receipt and actual online/offline metadata source remain preserved. Later
  builds do not re-resolve. Existing snapshot-source exporters stay bounded.
- Preserve CLI 0.1.0/0.1.1/0.2.0/0.3.0 bindings, published archives, tags and
  historical `sdk-compat-v1` verification. The
  [CLI 0.3.0 plan](CLI-030-RELEASE-PREPARATION.md),
  [candidate](YT-248-CLI030-CANDIDATE.md) and
  [hosted record](CLI-030-HOSTED-RECORD.md) retain their original scope/evidence.

The [current contract](YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md) and
[ADR 0010 amendment](adr/0010-dynamic-sdk-version-resolution.md) supersede the
finite v1 selection rules for this source. The reviewed docs-site masthead
change in `examples/docs-site/README.md` and `examples/docs-site/src/main.rs`
is preserved exactly; it changes no Reader version.

## Source documentation and independent review gate

Before selecting C or packaging, follow the existing
[release documentation gate](RELEASING.md#release-documentation-gate):

```sh
python3 scripts/release_docs.py sync
python3 scripts/release_docs.py check
cargo xtask check docs
PYTHONPATH=scripts python3 -m unittest scripts.test_release_automation scripts.test_release_docs scripts.test_release_cli_consumer
```

Retain original failures and report actual results separately from product
qualification. The parent must supply independent semantic review of the
package README, current contract/ADR and release claims before source freeze
and T4 packaging. Source gates cannot be skipped or weakened. Guarded Git
delivery and required exact-head PR/post-merge master CI remain required under
the existing release SOP; these local checks grant no Git or hosted authority.

## Required local and hosted consumer evidence

Use the existing [CLI 0.3.0 consumer requirements](CLI-030-RELEASE-PREPARATION.md#required-local-and-hosted-consumer-evidence)
for the changed CLI 0.3.1 candidate, preserving the original evidence and
reusing only source-identical results. Apply the current latest policy rather
than the superseded v1 list. Pre-publication installation consumes the selected
candidate archive; do not attempt to install unpublished CLI 0.3.1 from Registry.

Required evidence includes exact version/help, archive metadata and resource
closure, explicit alpha.7 and refreshed-metadata default selection, exact app
pins/lock/checksums/receipt source, native Style SSR/fetchable CSS, browser Style
production assets and runtime, and bounded Chrome computed-style/CSP smoke.
Retain process ownership, bounded readiness/termination and original failures.
Compilation, browser/CSP and cleanup results are separate claims. New Windows
Style revalidation remains paused/unverified; historical Windows proof does
not qualify these changed paths.

After independent source/release-automation review and required Git/CI gates,
use existing hosted build, protected evidence readback/custody, signing and
independent result verification with exact C/S/V and artifact hashes. Preserve
closed one-package cohort, source/tag/checksum/Environment and owner gates.
Prepare the existing five-file public allowlist and single-CLI registry plan;
stop before `release-publish`, any formal publication or crates.io upload.
No new dependency, publisher, Runner permission or validation infrastructure.

## Evidence and remaining order

The run retains commands, logs, original failures and hashes under
`artifacts/verification/cli031-latest-sdk-20261110/`, with T1 resolver, T2 release
integration and T3 documentation evidence in separate subdirectories. Rebuildable
Cargo output remains under `.cache/yetlet/builds/cli031-latest-sdk-20261110`.

T1 focused resolver/scaffold tests pass, with opt-in live/compilation cases
reported as ignored. T2 resource tests pass; its original full Python result is
66/68, with two current-source README version failures now resolved by T3 alignment.
These original logs remain retained. T3 release-doc sync reports zero further
version-derived changes; release-doc check and Cargo docs/link/structure/DAG
check pass (75 required documents, 247 backlog items). All 68 tests across the
three Python modules pass. The initial T3 sync rejection of combined source
version/pending-status wording is retained; the document statements were
separated without changing the gate. These results do not qualify candidate
runtime or hosted signatures. T4 candidate/local consumer work and T5 hosted preparation remain
pending, following the independent parent-supplied semantic/source-freeze gate.
YT-250 stays In progress until its requested preparation endpoint is evidenced.

Any later authorized CLI publication must also complete the
[required reader update](RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication),
including consumer/native/Wasm proof and live Pages readback. This preparation
does not advance that edition or claim publication.
