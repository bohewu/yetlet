# YT-096 Azure Run Command clean-builder qualification

Date: 2026-10-20. Candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, original nine experimental,
unpublished 0.1.0 libraries, Rust MSRV 1.99 and qualification pin 1.99.0.

**PASS at the native Ubuntu 24.04 and uncached archive-build scope:** nine
archives are byte-identical to their accepted Linux/Windows counterparts,
83 native tests and four doctests pass, and five wasm libraries build. The
dedicated resource group and all eight resources are deleted. Independent
ordinary slice review passed with no issues or required followups. YT-096 remains In progress; YT-020 remains
Blocked. Shared controller ownership and final release/trust gates remain.

## Resumed strategy and isolated inputs

The owner explicitly resumed the proposed Azure Run Command/private Blob
strategy after the [two SSH failures](YT-096-AZURE-CLEAN-BUILDER.md). Those
failures, their cleanup and accepted historical results remain intact. This
continuation is additive under
`artifacts/yt096-azure-clean-builder-20261020/run-command-blob/`.

The same immutable source archive has SHA-256
`9096cabe9a7b6d8b809c5bd337c2a8ab88f33ff19b6b4c8b59f31267c7335546`.
The 3,526,433-byte transport bundle contains only that archive, expected input
hashes and the two existing guest command preparations. Its SHA-256 is
`bc55ada698b08f63f058b9154a018c846b438bc1b6ccea477481c304b1cfa360`.
No Cargo dependency cache, vendor staging, Cargo build target or installed local
toolchain was transferred. The guest verifies the bundle, archive, all 63 selected source
inputs and the established 18 materialized license aliases.

The actual guest reports **Ubuntu 24.04.5 LTS**, x86_64, kernel
**7.0.0-1017-azure**. Azure metadata identifies the pinned
`Canonical:ubuntu-24_04-lts:server:24.04.202610160` image, East Asia,
Standard_D2s_v5 and VM ID `dcc32898-bc5d-4810-acb7-047f8292a5a9`.
This is an actual booted Azure guest, separate from the earlier Ubuntu 24.04
container sharing the Ubuntu 26.04 desktop kernel. It does not qualify desktop
browsers or every Ubuntu hardware/kernel configuration.

Official Rust 1.99.0 and the wasm target were downloaded in the guest. The
installer/channel digest checks and tool installation logs are retained.
Cargo/rustc/rustdoc binary SHA-256 values exactly match the accepted GNU tools.
Qualification starts with an empty private Cargo home and absent build target;
`cargo fetch --locked` obtains the root workspace's registry inputs upstream.
Subsequent commands retain --locked/--offline and no extra feature/profile
flags. No source, manifest, lock, dependency version or tool pin changed.

## Execution and raw results

[Commands](../artifacts/yt096-azure-clean-builder-20261020/run-command-blob/returned/COMMANDS.json)
record exact argv, cwd, UTC times, native exits and raw logs. The command groups
are ordinary Cargo commands against the same source workspace:

1. `cargo fetch --locked --manifest-path <source>/Cargo.toml`.
2. `cargo package --locked --offline`, the owned target and all nine selected packages; normal package verification is enabled.
3. `cargo test --locked --offline --target x86_64-unknown-linux-gnu --message-format=json`, the nine packages and `-- --test-threads=1`.
4. `cargo build --lib --locked --offline --target wasm32-unknown-unknown --message-format=json`, selecting core/actions/reactive/dom/dom-keyed.

All four native commands exit 0. Raw native log summaries independently total
**83 native tests and four doctests**. The wasm command builds all five
selected libraries; it does not execute a wasm/browser test suite. Source
inputs, including the root workspace lock, remain unchanged afterward.

Managed Run Command `qualify-candidate` records executionState Succeeded,
exitCode 0, start 10:47:16 UTC and end 10:50:06 UTC. The guest process also
returns 0. ProvisioningState alone was not used as the success condition.
This is a native Azure execution, not an agent Job with model/effort.

The returned ZIP SHA-256 is
`c19e5237694118ac0c7217dc08487dd70b64d6b1af78f5439852e5bf2726fe57`.
Its 30 members include nine newly generated `.crate` files, raw logs, tool/host
identity, installer/channel hashes, installed dpkg versions, command results
and compiler-artifact events. Each actual returned archive matches the
original accepted archive in complete bytes, size and SHA-256. The
[existing nine-archive table](YT-096-TWO-HOST-REBUILD.md#archive-comparison)
still applies; no archive or checksum bundle was replaced.

## Actual Cargo observations and inventory limits

Standard Cargo JSON output retained **140 compiler-artifact events**: 97 from
the native test group and 43 from the wasm group. These represent 70 and 32
distinct package/version pairs, respectively; all occur in the candidate root
lock. The [observation reconciliation](../artifacts/yt096-azure-clean-builder-20261020/run-command-blob/CARGO-BUILD-OBSERVATIONS.json)
binds this data to the returned ZIP and unchanged target SBOM digests.

The native observations contain 68 pairs in the union of nine retained GNU
library SBOMs. The other two, `tower-http 0.6.11` and `bitflags 2.13.2`, occur
in the existing server-axum development-only metadata. Its HTTP-pipeline test
uses tower-http; bitflags is transitive. The generator's documented normal/build
scope excludes dev-only edges. Original license/source/audit scope includes
package development dependencies. All 32 wasm pairs occur in the union of five
retained wasm SBOMs. There is no unexplained package/version difference.

These are aggregate observed compilation groups, not per-library feature
equivalence or a complete compiled-binary/system inventory. Cargo records
features, profiles and output paths; output-file digests were not captured.
Rust sysroot and implicit/native linker inputs remain outside Cargo metadata.
Tool digests and dpkg versions are separate observations. Existing conservative
Serde derive over-inclusion, non-Cargo omissions and the absence of new Windows
compiler-artifact observations remain explicit. No SBOM generation or passing
license/audit suite was repeated. A local report adapter initially treated the
retained development-pair arrays as objects; one pair-read correction completed
the reconciliation without repeating runtime tests or changing prior evidence.

## Transport, cleanup and remaining release boundary

[Managed Run Command](https://learn.microsoft.com/en-us/azure/virtual-machines/linux/run-command-managed)
carried the script and protected parameters. The account disables anonymous
Blob access; the private container uses authenticated HTTPS on the public Blob
endpoint, not a Private Endpoint. Per-blob SAS grants input read and separate
evidence/status create/write access for two hours. The account key stays in a
local subprocess environment; SAS values are protected parameters. Neither is
retained in evidence. The guest removes these parameters from the child build
environment. No direct SSH/SCP command was used and the NSG has zero custom
inbound rules. The public IP supplies outbound connectivity.

After evidence download/hash verification, resource group
`rg-yetlet-yt096-blob-7e45ec` was deleted. The eight recorded resources were VM,
OS disk, NIC, public IP, VNet, NSG, Storage Account and managed Run Command;
the private source/status/evidence blobs were inside the deleted account.
[Cleanup verification](../artifacts/yt096-azure-clean-builder-20261020/run-command-blob/CLEANUP-VERIFICATION.json)
checks all three exact task-owned groups, including the two earlier failures:
all absent, with no owned resources after case-insensitive inventory filtering.
Ephemeral local administrative SSH material was removed. No VM or storage is
retained for continuation.

All 570 preexisting evidence files and 63 selected source inputs are preserved.
No product code, API, trust policy, owner waiver or supported browser scope
changed. No staging, commit, push, signing, tag, publication, hosted CI activation
or production deployment occurred.

This supplies actual Ubuntu 24.04 native execution and an upstream-fetched,
empty-cache archive rebuild. It still uses the same maintainer/agent controller:
independently controlled builder trust and formal provenance/signature/registry
checks are not established. Historical Chrome scope acceptance, final build-bound
inventory review, independent release/security review and final maintainer
approval remain open. No final release or production qualification is claimed.

## Independent review

The registered reviewer completed a read-only review with PASS, zero issues and
zero required followups. Native execution evidence verifies role `reviewer`,
model `gpt-6-astra` and effective effort `high`; the shared reasoning resolver
reports enforced selection. The reviewer independently checked raw returned
archives/logs, Cargo/SBOM differences, credential handling, live resource
absence, all 570 prior evidence hashes, 87 review snapshot entries and 63 source
hashes. No test, build or provisioning was repeated. The exact verdict and
bounded identity acceptance are retained in
[REVIEW-RESULT.json](../artifacts/yt096-azure-clean-builder-20261020/run-command-blob/REVIEW-RESULT.json)
and [REVIEW-ACCEPTANCE.json](../artifacts/yt096-azure-clean-builder-20261020/run-command-blob/REVIEW-ACCEPTANCE.json).
This accepts this ordinary qualification slice, not final release/security
approval or independently controlled builder trust.
