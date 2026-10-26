# YT-201 — Publication metadata and packaged-consumer preflight

Date: 2026-10-26 (Asia/Taipei). Status: **Done at metadata/local packaged-consumer
preflight scope after independent Reviewer PASS, zero issues/required followups.**

## Authority and contract before changes

After the guarded YT-199/200 planning commit
`f955251b75adc7bcd5750e9df2626653058e7f06`, the owner authorized continuation
with the next item. Apply the reviewed [YT-201 contract](YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md#verification-work-registered-for-later-execution):
prepare all sixteen at experimental 0.1.0, seven publication-eligible manifests,
compatible normal/optional and retained dev dependency versions, package-local
license texts/public guides and isolated archive/consumer preflight. Preserve
independent workspaces, default features, runtime/security/API behavior and
prior evidence. Independent Reviewer PASS gates Done. No new Commit/Push,
publication, service/cloud/browser execution or formal trust operation.

Metadata eligibility follows the original nine's omitted `publish` field.
This is not registry availability or permission to upload. Change three local
0.0.0 package versions and only the corresponding consumer lock identities;
do not upgrade third-party dependencies. Existing CLI generator fixtures with
synthetic 0.0.0 input remain historical/fixture vocabulary unless real consumers
require a change. Do not redesign template delivery or merge workspaces.

Use existing Cargo packaging/standard directory-source patterns for unpublished
siblings, not a new registry, validator framework or validation orchestrator.
Archive collection without verification, if needed to populate that ordinary
source, is preparation only; actual archives still require normal Cargo package
verification and external consumer checks. Dirty source/hash-bound preflight
is not final immutable candidate qualification: an authorized guarded commit
and explicit later source C selection remain required before that claim.

Scope is native package/default and selected native feature consumers only.
Wasm/browser/GPU, native platform/MSRV matrices, inventory/advisory/security
scans and final rebuild/reader publication qualification remain YT-202–205.
Reuse all previously accepted runtime/provider/benchmark evidence. No old
product test suite is replayed for metadata-only changes.

Evidence is retained under
`target/yetlet-yt201-packaging-20261026-olp8afef/evidence/`.

## Implemented preparation

Seven manifests now follow the original nine's omitted-publish-field convention,
with explicit README metadata. Config/Secrets/Redis Session change from 0.0.0
to 0.1.0; isolated packages receive repository metadata and Signing receives a
description. The three normal/optional local dependency edges and retained
local dev dependencies now combine version 0.1.0 with their existing paths.
Feature tables, target conditions, independent workspace boundaries and all
33 library source files remain unchanged.

Each of the seven receives byte-identical root MIT and Apache-2.0 texts.
Config and Signing gain archive-local introductions with actual prerequisites,
minimal usage, failure/trust boundaries and local license links. Existing Auth,
Redis Session and wgpu guide text is updated only where nonpublishable metadata
claims became stale. wgpu's selected historical GPU support is unchanged.
Current API/capability/compatibility/publication-source guides distinguish this
sixteen-package preparation from the unchanged nine-package ce56e08 candidate.
Historical task/scan/review reports are not rewritten.

`cargo update --offline --workspace` updated ten affected locks: root;
Config, Secrets and Redis Session; Admin, Axum workload comparison and Starter;
YT-066b3 and YT-125 qualification fixtures; CLI. Only local Config/Secrets/Redis
version records changed. Every third-party package/source/version/checksum and
dependency record matches its pre-change value. Synthetic CLI fixture version
0.0.0 remains unchanged; no generator runtime repair is needed for this metadata
change. No feature, dependency algorithm, public Rust signature or runtime code
was altered.

## New native package and consumer evidence

Host is Ubuntu 26.04.1 LTS GNU x86_64, Rust/Cargo 1.99.0. Existing development
headers expose OpenSSL 3.5.5 through pkg-config. This local package/link preflight
does not replace YT-093's isolated 3.5.9 recipe or qualify the proposed Ubuntu
24.04/MSVC matrix. No system tool/library installation or configuration changed.

1. Existing Cargo created sixteen bootstrap archives with
   `--no-verify --exclude-lockfile` only to obtain its own normalized manifests
   for unpublished siblings in an ordinary directory source. These flags are
   archive collection, **not verification** and not Git/hook/Commit Guard
   bypasses. No bootstrap result is counted as a passing package check.
2. Ordinary Cargo vendor staged locked third-party sources. Each unpublished
   sibling uses standard per-file directory-source checksums and a null registry
   package checksum: no actual registry identity exists here. The staged pool
   includes broader development/target inputs and is not the release inventory.
   No custom registry, source normalizer or validation framework was introduced.
3. All **sixteen normal `cargo package` checks PASS** with `--offline --locked
   --allow-dirty`, explicit manifest/target/config and four jobs, **without**
   `--no-verify` or lock exclusion. Cargo builds each extracted package normally.
   Every archive includes a lock, README, matching dual-license texts, normalized
   version 0.1.0 and versioned local dependency declarations with paths removed.
   Original manifests and all included src/tests/examples match checkout bytes.
4. The ordinary directory source is then populated from those sixteen verified
   preflight archives. One new external Cargo workspace declares only registry
   version requirements (`=0.1.0`) for the sixteen libraries, with no repository
   path dependencies or patches. Its Cargo home/build directory are separate.
   Two new native runs PASS: default, then locked `integrations` selecting
   Auth `session`, Config `secret-provider` and Redis provider `tls`.
5. Both runs print only `packaged consumer OK`. They exercise published API
   reachability for identity/config/rate-limit/Secrets/native digest/CPU GPU
   lifetime; the selected run also links the Memory bridge and shared SecretString
   binding types, with missing Session still refused. TLS compiles but performs
   no network operation. A locked metadata read confirms sixteen 0.1.0 roots
   resolve solely to the extracted directory source, not checkout packages;
   all final archive file bindings remain unchanged through execution.

Exact commands, native exits and timings are in `package-results.json` and
`consumer-results.json`; source/archive/normalized metadata bindings are in
`package-inventory.json` and `consumer-binding.json`. Local-only lock/source
preservation is in `source-lock-preservation.json`. The two external programs
are focused new packaging consumers, not a replay of existing functional suites.
No actual registry fetch/upload/name-rights proof or signed release is claimed.

## Retained failures and limits

Initial offline vendoring could not obtain locked core-foundation 0.10.1 for
the broader target pool. One bounded operational recovery permitted ordinary
locked source downloads, then vendoring passed; versions were not upgraded.
`vendor-result.json`/`vendor.log` retain exit 101, and separate locked-fetch
artifacts retain the subsequent success. This is a cache prerequisite failure,
not a native product defect or reason for another platform qualification.

Initial archive inspection assumed Cargo always emitted `git.dirty`; unchanged
original-package entries omit false. It stopped with KeyError before consumers
were created. The single inspection correction reads an omitted value as false
and preserves Cargo's real VCS object. No product or archive was changed and no
passing build was repeated. This failure/correction is retained separately.

Package-level VCS flags do not identify this whole dirty preparation. Even an
unchanged package's omitted dirty flag cannot authenticate newly generated lock
inputs or a future release. Exact source/archive records carry this preflight's
bindings; the final selected immutable source C is **not selected** here.
Directory-source null package checksums and registry-style IDs do not prove
crates.io ownership, canonical registry checksums or real downloaded consumption.

No browser/Wasm/GPU, live Session/Valkey/IdP, Windows, benchmark, advisory/security
scan, final source/rebuild or hosted CI was run. No old provider/load/capacity
experiment was replayed. Those source/support gaps remain YT-202–205; formal
trust/source-workflow/public metadata and actual registry activation remain
YT-096/D1–D6 and YT-020. D1–D6 stay Pending. No listener, external resource,
private key or credential was created; inert sources/build logs are retained
under the excluded task directory. New changes remain unstaged/uncommitted.

## Documentation and independent review

Repository documentation checks, direct new-guide links/fences/EOF/whitespace,
exact source/evidence/status checks and independent read-only Reviewer are
required before Done. Results are recorded here at closeout. This item can
complete at metadata/local-preflight scope without selecting final C; a later
authorized guarded commit and explicit source selection precede its final
candidate qualification. The next registered item is YT-202 after that boundary.

`cargo xtask check docs` passed with 75 fixed required documents and 205 backlog
items. Direct checks passed for fourteen changed Markdown files (local links,
fences and EOF), tracked whitespace and seventeen new-file whitespace checks.
No-index exit 1 means the expected new content difference only when diagnostics
are empty. Feature/workspace tables remain unchanged and staged files are zero.

Independent Reviewer `/root/yt201_package_preflight_independent_review_20261026`
returned PASS, zero issues/required followups. It verified all 45 source and
55 evidence/consumer bindings, the 33 unchanged library sources, ten local-only
lock changes, sixteen actual archives/final directory-source contents, package
logs and both isolated consumer runs. HEAD/status/staged-zero matched throughout
review. Native trace verified registered role/model and matching effective
resolver effort, without conflict/degradation. Review was read-only and replayed
no product checks. Only this closeout and three task/navigation summaries change
after PASS; package/readme/source/archive bytes stay as reviewed. YT-202–205
remain Todo; this completion selects no final C or publication and creates no
new Commit/Push authority.
