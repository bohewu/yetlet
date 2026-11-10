# Yetlet CLI 0.3.1 — Release preparation (YT-250)

**State (2026-11-10): Local candidate and original hosted build/custody PASS.**
Resolver, release-consumer integration and documentation passed independent
semantic/source-freeze review. Clean-source packaging, archive installation and
local native/browser consumers passed. The separate S review/delivery/CI gate,
prepare/signing and independent V remain pending. Formal GitHub/crates.io publication is excluded
from this preparation. Published SDK alpha.7 / CLI 0.3.0 and the same Reader
Edition remain authoritative; `site/pages.toml` is unchanged.

## Scope and version authorities

- Exactly one independent `yetlet-cli` package, source version `0.3.1`, using
  existing schema3 / `cli` release machinery and the corresponding
  `yetlet-cli-v0.3.1` tag identity. The T4 source is recorded below; no tag was
  created during local qualification.
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
runtime or hosted signatures. T4 candidate/local consumers subsequently passed
as recorded below after the independent parent-supplied semantic/source-freeze
gate; T5 hosted preparation remains pending.
YT-250 stays In progress until its requested preparation endpoint is evidenced.

Any later authorized CLI publication must also complete the
[required reader update](RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication),
including consumer/native/Wasm proof and live Pages readback. This preparation
does not advance that edition or claim publication.


## T4 local candidate qualification — 2026-11-10

This checkpoint completes T4 for YT-250; it is candidate evidence, not publication.
The accepted source is `f489a089ec2b814e830942318f84b0e45b0a67bd`, tree
`413cfdb7ddac0c8f1cf32e92a54f38b83fc5ad35`, after PR #77. Parent-supplied
independent semantic/source-freeze re-review passed with zero required followups.
All 28 frozen file SHA256 bindings matched before packaging and after consumers;
this evidence addition follows clean-source archive creation. Original reviewed
masthead files, build SDK and immutable snapshot remain unchanged.

Immediately before normal package creation, `python3 scripts/release_docs.py check`
and `cargo xtask check docs` both passed. `cargo package --locked --manifest-path
tools/yetlet-cli/Cargo.toml` passed with neither dirty-source nor verification
bypass flags. The retained 163-file archive has SHA256
`15803c419cc5c9d6ea7be3239cf09a47adb336ee8100d39da17fedb199099493`.
Its `.cargo_vcs_info.json` binds the clean source above; normalized metadata,
package README, exact alpha.7 build dependency, policy digest and all embedded
resource identity hashes passed inspection. The actual installed executable
reports `yetlet 0.3.1`, matching its package README, with SHA256
`cd05bd4aeea2a16c457fa558e179704958e4bb60254e00f6b2aed83103fc50bf`.

Normal locked archive-path installation and the existing
`release_cli_consumer.qualify()` ran outside the checkout with a fresh,
credential-free Cargo home and no source replacements or SDK path patches.
All six creations passed: refreshed default native selected actual Registry SDK
`0.9.0-alpha.7`; explicit native, browser, native Style and browser Style selected
exact alpha.7; prepared offline browser Style selected alpha.7 from offline cache.
Captured metadata independently establishes the default selection. Exact manifest
pins, package checksums, app/Style locks and receipt hashes/provenance agree;
repeated creation refused without lock changes, and an empty offline cache failed
atomically. These observations establish current metadata behavior, not future
SDK compiler or API compatibility.

Native Style installed-CLI build and no-JS SSR passed, including escaped content,
fetchable generated CSS and strict CSP. Browser Style production Vite/Wasm build,
HTML/CSS/Wasm requests and Chrome mount passed. Computed toolbar style was flex,
12px padding and weight 700, retained after a keyed update; inline style count,
console/CSP errors and failed responses were zero. App and Style locks remained
unchanged after build/run. Native/browser servers exited with -15, Chrome exited
0, all three server/debugger ports closed, the Chrome profile was removed and no
owned process remained at terminal verification.

Evidence is retained under
`artifacts/verification/cli031-latest-sdk-20261110/T4/`: original gate/package and
installation logs, archive and executable, metadata/resources, exact consumer
commands/results, generated sources/production assets, locks, captured metadata,
receipts, screenshot and teardown. The original direct-inspection assertion
failure (clean Cargo VCS records omit the dirty field) and workspace-inheritance
setup failure remain preserved. One assertion correction and the parent-approved
canonical private root outside the workspace resolved those setup issues; no
product repair was needed. Rust 1.99 and wasm-bindgen were copied privately;
shared tool permissions were unchanged. The shared private target was created
with mode 0700 before installation. Rebuildable compiler output remains in the
round cache; the separately recorded owned private consumer/tool root is retained
for final authorized cleanup. No deletion or Git operation occurred in T4.

T5 hosted preparation is the next unblockable YT-250 slice after the parent's
required delivery/CI inputs. Windows Style stays paused/unverified; Cloud,
Config and standalone Tailwind opt-ins were not rerun. Published distribution and
Reader remain SDK alpha.7 / CLI 0.3.0. No registry installation of unpublished
CLI 0.3.1, hosted release execution, tag, signature or publication is claimed.


## T5 phase 1 — original hosted build and custody

The [CLI 0.3.1 hosted preparation record](CLI-031-HOSTED-RECORD.md) records
unsigned build `42830865959/1` and original custody verifier `42831991715/1`,
both SUCCESS, exact artifact/provenance identities and retained cleanup receipts.
The selected hosted archive SHA256 is
`305f67eac6c302ff9efeddc6d0bb20dce2508c4ddf899fe7727545f5e6168f9d`,
545352 bytes. It is distinct from the T4 archive above; all 162 common files are
byte-equal, with only the local Git VCS file omitted by hosted package staging.
Future frozen/signing/public plans must use the actual hosted subject.

This phase prepares a concrete docs-only S dossier. Separate independent review,
guarded S delivery and its PR/master CI are the next external gate. No S is
selected here; no prepare/sign/independent V or formal publication ran. T5 stays
In progress. Published SDK alpha.7 / CLI 0.3.0 and Reader remain unchanged.
