# YT-214 — Source-installed CLI and registry-first application

Date: 2026-10-31. Status: Done at the bounded implementation/local GNU
package-and-install proof scope after independent Reviewer PASS0. Actual first
CLI publication remains Pending; the registry install interface is not live yet.

## Scope and contract

The owner requests the CLI installation/first-project DX followup and obsolete
branch cleanup. The already-published SDK remains exactly two packages,
`yetlet` and `yetlet-browser` at `0.9.0-alpha.5`. Preserve their source, version,
tag, archives, signatures, feature/security semantics and accepted qualification.
No third-party dependency upgrade, SDK change or full browser/platform replay.

The existing `yetlet-cli` package remains version `0.1.0`; its executable is
`yetlet`. The owner further requires installation without cloning the repo.
Prepare it as an independently publishable package (`publish = true`) for the
standard `cargo install yetlet-cli --locked` interface. Merely changing metadata
does not upload it. Deliver and locally verify a self-contained Cargo source
archive; the actual first CLI publication still requires separate authority.
It is a developer tool, not a third SDK package or part of the already-signed
alpha.5 two-library release. Prebuilt executables remain a later option. Native
source compilation requires Rust/Cargo 1.99+, C/link tools and OpenSSL development
files. Generated plain native applications do not inherit CLI signing/OpenSSL.

Package templates, the two pre-Wasm startup assets, dual licenses, the workspace
manifest/lock baseline and the existing fixed public SDK source closure under the CLI root.
Use the exact published registry SDK dependency. No build-time network fetch,
code generator or new dependency is introduced; normal Cargo dependency
preparation remains explicit. The small maintainer refresh command copies only
Git-tracked public SDK files, rejects symlink traversal, checks the matching
SDK version and normalizes source line endings. A repository test detects
resource drift. This bundled compatibility data is not another public SDK.

Default native/browser creation already emits exact registry dependencies;
retain that behavior and correct its outdated setup text. Docker/Compose/K8s
generation still requires the explicitly qualified local SDK source closure,
checks exact source identity and generates configuration only. Registry-only
container delivery and arbitrary source export remain unselected. Process,
filesystem, environment, frozen Cargo and safe error behavior stay unchanged.

## Minimal user path

After the first CLI publication, install with `cargo install yetlet-cli --locked`
without cloning the repo. For this pre-publication proof, install from the
unpacked verified CLI source archive using `cargo install --path ... --locked`;
maintainers can also select `tools/yetlet-cli` from a trusted checkout. Use the
installed `yetlet` binary, or choose an owned private prefix with `--root`.
Then, outside the repo:

1. `yetlet new firstapp --parent /absolute/owned/parent`.
2. In that new directory, `cargo generate-lockfile` and `cargo fetch --locked`.
3. `yetlet build`, then `yetlet run`; inspect the printed loopback address.
4. Ctrl+C stops the owned foreground process tree.

The app needs no SDK checkout, path override or `[patch]`. Creation does not
install tools, prepare a Cargo lock/cache, build or start a server. The detailed
[first-app guide](FIRST-APP.md), [CLI reference](../tools/yetlet-cli/README.md)
and generated native/browser READMEs describe the same separation. npm is needed
only when selecting a frontend workflow, not to install this native CLI.

## Bounded verification and limitations

Use existing focused resource/scaffold/browser/generator tests and normal Cargo
package verification, without `--no-verify`. Install from the unpacked normalized
archive into a task-private prefix outside the repo, not directly from the
checkout. Check version/help/doctor, create a native app, prepare its lock/cache,
build/run and verify escaped HTTP output and owned listener/process cleanup.
Inspect the manifest/lock to prove registry-only SDK consumption and unchanged
third-party CLI lock identities. Browser generation/asset correspondence is
covered without repeating accepted browser/WASM/GPU execution.

This is local GNU source-install evidence. It is not new Windows install,
prebuilt binary, browser or production qualification. No source Commit/Push,
CLI/SDK publication, hosted release or signing is authorized by this item.
The first offline test command could not resolve the newly published SDK in
the original cache and started no tests; retain that operational failure.
Prepare the task-private public registry cache before the affected checks.
Two doctor setup failures are also retained: a short Rustup selector did not
name an installed physical directory, followed by refusal of group-writable
host toolchain ancestors. The latter is confirmed by ownership/mode inspection;
use a protected task-private copy without changing the original installation.
These are environment preparation limits, not SDK/package failures or reasons
to weaken executable policy.

## Actual verification

- CLI check, formatting and lock correspondence PASS. The only lock change is
  the published SDK registry source/checksum; all third-party package identities
  are unchanged.
- Twelve distinct affected tests PASS: two resource tests, one selected-scaffold,
  one browser creation, one startup-asset, one selected generator, one Docker,
  one Compose and four Kubernetes tests. The shared fixture's external lock
  include is also relocated into package resources; its affected tests pass.
  Two existing selected-scaffold external-tool cases remain ignored, not counted.
- Normal Cargo packaging/verification PASS, followed by installation from the
  unpacked normalized archive under a private repo-external prefix. All literal
  compile-time includes, including test fixtures, resolve inside the CLI package.
- Installed version/help/doctor, `new`, explicit app lock/cache preparation and
  frozen build PASS. The app's SDK lock entry is registry-backed with checksum
  `662d67180cec2fd6e50c3757bdc910d057cd37428785ab695b0d275a9e4e5091`;
  no path, patch or replacement is selected.
- Two HTTP200 routes pass escaped-content and strict-CSP assertions. Afterwards,
  an incorrect harness expectation of two PIDs encounters three owned processes
  (including supervision); that observation is retained without replaying HTTP.
  Ctrl+C reports owned children stopped, all three recorded PIDs disappear and
  the loopback listener closes. Response bytes were not retained; the result
  records completed assertions and that evidence limit explicitly.
- Current documentation check PASS:75 required documents,214 items, internal
  links, Markdown structure, whitespace and DAG. No accepted SDK/browser suite
  is replayed. No runtime/SDK source or third-party dependency is changed.

Evidence is retained under `target/yt214-cli-install-20261031/`: focused and
fixture-resource test logs, startup/package/install logs, lock and package
receipts, installed-app build and HTTP/cleanup receipts, original environment
failures and branch-cleanup records. Subsequent package metadata/README changes
for the no-clone distribution interface do not change the installed CLI runtime
inputs; final archive correspondence and token-free publish dry-run are checked
separately. The current normally verified135-file source archive has SHA-256
`113de409e62a40c2da6e6499e8f49a70ea4aef2f7d95766b4321324cd8f122fc`.
Normal token-free `cargo publish --dry-run --locked --allow-dirty` PASS;
no upload occurs. Final independent Reviewer PASS0 has no issues or required
followups; all33 reviewed hashes (32 Git paths plus the release-body proposal)
match. Only completion metadata changes after review. Actual first CLI
publication, any source delivery and wider platform qualification remain separate.

## Branch cleanup and historical publication status

All fourteen remote work branches have exact closed/merged PR correspondence
and are deleted with direct strict/builtin deletion-range Guard PASS and normal
native hooks. Nineteen retired local refs are deleted after verifying a repo-
external all-heads/tags recovery bundle. Remote `master` remains; local `master`
and `codex/yt020-local-master-24e0772` remain. The latter has four non-ancestor
commits whose inclusion is unproven, so it is deliberately preserved. Tags,
source histories and the working changes remain intact. No source Push occurs.
Run-local inventory, bundle verification and deletion receipts are retained in
`target/yt214-cli-install-20261031/`.

The [alpha.5 publication record](ALPHA5-HOSTED-RECORD.md#completed-publication-and-subsequent-cleanup)
and current reader entry points distinguish completed two-package publication
from older sixteen-package evidence. The owner separately deleted the old
`yetlet-actions` registry package and PR29 is closed unmerged. The release-note correction is applied and its API readback matches the reviewed
text. Release ID, original C/tag, publication state and all six asset IDs,
digests/sizes remain unchanged. The local migration contract now annotates its
historical preservation statement; the fixed-C snapshot remains immutable.
The owner subsequently reports token scope `yetlet-*` and `yetlet`, which covers
the planned CLI name. This is owner-reported configuration, not a token readback
or authenticated upload check. Actual first CLI publication still requires later
authorization and its preflight. No Secret is changed or CLI upload attempted.
