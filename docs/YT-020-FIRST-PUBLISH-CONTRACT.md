# YT-020 — Config/Secrets first-publication dependency contract

Date: 2026-10-30 (Asia/Taipei). Contract fixed before the bounded dry-run.
Secrets and Config/secret-provider normal publish dry-runs PASS; archive-byte
correspondence does not pass. Independent Reviewer PASS, zero issues and required
followups, accepts this bounded local-model study. Registry activation
remains **Blocked**.

## Authority and unchanged product inputs

The owner authorized committing the five registry-inventory documents with
strict/builtin Guard, then establishing this dependency contract and performing
minimal dry-run verification with independent review. The five-path commit is
`d71d976bde7b1f18aa0a5f9e7a516d068cd7bf4c`, tree
`c8812143e5a3e19262d4bf51cb21b3f7eb5edb74`; direct Guard and native hook PASS.
No Push. Subsequent changes remain uncommitted. Actual crates.io upload is not
authorized.

Use reference C `557ab76e03b619e959eae7529dbe03317fe1f952` and version
`0.9.0-alpha.3`. The sixteen-library cohort, tag, original archives, signatures,
source/API/features and lockfiles stay unchanged. This work does not select a
new registry candidate, rewrite an accepted artifact, remove development
dependencies, merge workspaces or add a publisher framework.

## Chosen dependency treatment

`yetlet-config` has an optional normal dependency on `yetlet-secrets`.
`yetlet-secrets` has versioned development dependencies on Config, Core and
Server Axum; Config has development dependencies on Core and Server Axum. The
[inventory](YT-020-REGISTRY-READINESS.md) distinguishes their normal dependency
DAG from this mixed development edge.

Keep both manifests intact. For bounded local verification, reuse the accepted
alpha.3 directory-source inputs, including fully verified Yetlet package
contents, so Cargo can resolve unpublished sibling development versions. This
uses the existing release preflight's source arrangement, not new registry
infrastructure, a path patch, a dependency deletion or a verification bypass.
Source replacement must be disclosed in commands/results; it models the intended
registry contents and cannot prove live registry availability or acceptance.

For a future separately authorized first publication, order Secrets before
Config because Secrets has no normal Yetlet dependency and Config's optional
Secrets declaration still belongs to the published manifest. Core, HTML and
Server Axum can also precede the pair under the inventory's normal dependency
order. Do not treat the development cycle as a runtime cycle or require Config
to be uploaded before Secrets. Wait for Secrets' actual index entry before a
Config upload.

**The local model is not an approved actual-publish configuration.** Cargo's
source-replacement contract requires replacements to be subsets of the original
source, with identical contents; these unpublished Yetlet versions are absent
from crates.io. The dry-run shows Cargo's packaging/build behavior under an
explicit prospective-source model, not a supported live mirror. Do not reuse
this replacement as authority for a real first upload. Actual first-publication
preflight must use a supported Cargo-native selected-package bootstrap, or an
explicitly reviewed metadata treatment for the development edge, with its source
and artifact consequences. The current independent workspaces and resulting
checksum observations below leave that registry-facing condition open.

If this unchanged-input approach fails, preserve the failure and report the
specific unresolved condition. Do not silently edit Cargo metadata/version,
disable verification, repeatedly rerun the same failure or expand platform tests.
Any necessary source/metadata change must disclose its candidate/archive impact.

## Minimal verification fixed before execution

1. Retain an isolated source archive from C, outside Git discovery. Match the
   relevant manifests/locks and accepted directory-source Yetlet contents to
   their retained package inputs before use. Give Cargo a temporary home with
   no credentials; keep lockfiles fixed. The baseline is offline; any required
   public registry-configuration read for the selected dry-run must be disclosed.
2. Make one Secrets `cargo publish --dry-run --registry crates-io --offline
   --locked` observation without dependency-source replacement, using the
   existing public registry cache. This diagnoses the unfilled first-publication
   path; preserve a missing-sibling failure without labeling it package/runtime
   regression or retrying it unchanged.
3. Using the accepted directory-source configuration, run one normal Secrets
   publish dry-run and one Config publish dry-run with `--features secret-provider`.
   Both keep `--registry crates-io --locked`, use fresh output paths,
   retain complete command/exit logs and keep normal extracted-package build
   verification enabled. `--dry-run` prohibits upload. A
   source-replacement-specific refusal remains a failure to report.
4. Compare the two produced archives with the accepted signed package digests
   and inspect any member difference. Matching hashes establish correspondence
   only for these two exact dry-run outputs. Different bytes must not inherit
   old signatures; do not regenerate or sign the accepted release to hide drift.
5. Check the new English documents/links and whitespace, verify source/lock
   preservation, then obtain independent Reviewer acceptance. Reuse previous
   package, external consumer, security, platform and signature evidence without
   rerunning those suites. YT-020 remains Blocked for actual registry rights,
   approved upload authority and downloaded registry resolution/bytes.

Each Cargo observation has a ten-minute upper bound; stop on a substantive
failure rather than launching a replacement verification system. No network
service, token, registry owner change, hosted run, tag, signing or upload is part
of this contract. The temporary source, Cargo home and build directories are removed after
evidence preservation and review; retained evidence stays outside tracked source.

The [Cargo publish reference](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
defines dry-run and offline/locked options. [Source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html)
models identical source contents; [packaging](https://doc.rust-lang.org/cargo/commands/cargo-package.html)
still normalizes and builds the archive. These controls do not authenticate a
publisher or simulate the server's final authorization/acceptance checks.

## Execution and review

Before either selected positive dry-run, the baseline exits 101 with
`attempting to make an HTTP request, but --offline was specified`. It does not
reach the sibling-resolution diagnosis; do not claim that it proved a Config
dependency failure. This operational observation is retained, not retried.
The two selected dry-runs may read public registry configuration normally;
their credential-free home, fixed locks, accepted directory sources and explicit
`--dry-run` remain. This amends the offline assumption, not the owner's prohibition
on upload or a product input. No replacement test framework is introduced.

The first input-collection check also found that the old local preflight vendor
contains VCS metadata absent from the signed hosted archive. Before any Cargo
observation used that source, collection was corrected once in the same scratch:
Yetlet dependency trees now come directly from the sixteen accepted signed
archives; only third-party directories are reused. Archive/member/checksum
identity is recorded. No accepted evidence or original vendor input was changed.

| Observation | Actual result |
|---|---|
| Baseline Secrets, no replacement, offline/locked | Exit 101 before dependency diagnosis; HTTP-required observation retained, not replayed |
| Secrets, accepted archive directory sources, locked | Exit 0; normal extracted-package build PASS; Cargo explicitly aborts upload because of dry-run |
| First Config/secret-provider | Exit 101 during dependency build scripts: rustup proxies attempt concurrent stable updates outside source toolchain discovery; not a Config source failure |
| Config/secret-provider with pinned compiler | Exit 0 after pinning existing Rust 1.99.0 `RUSTC`, `RUSTDOC` and `RUSTUP_TOOLCHAIN`; normal extracted-package build PASS; upload aborted because of dry-run |
| Source preservation | All six relevant source/root manifests and lockfiles unchanged; no tracked product edits |

Only the failed Config observation was repeated after the environment correction;
Secrets' successful dry-run and old qualification suites were not replayed.
Cargo itself is 1.99.0 in both selected commands. Secrets' rustc proxy was not
explicitly pinned, so this is not new proof of its exact compiler/MSRV; reuse
the accepted qualification instead. The failed Config log retains the attempted
stable update/rollback errors. Direct existing stable `cargo`, `rustc`,
`rustfmt` and `clippy-driver` version probes subsequently exit 0 at 1.99.0;
these probes are not full toolchain/component-health qualification. No deliberate
installation or global toolchain repair was performed.

### Byte comparison and its consequence

The actual publish dry-run archives are retained from Cargo's
`package/tmp-crate/` output and match its `tmp-registry/` copies. They have the
same member sets and all member bytes match the accepted hosted archives except
**Cargo.lock**:

| Package | Accepted archive SHA-256 | Dry-run archive SHA-256 |
|---|---|---|
| `yetlet-secrets` | `c4a3f8162f9451049b6528be15a6272640264d068b15ecb2de8764b976953a2b` | `a9e07c75bd1bc77336b9a6597f747cd0f5bb89f85a6a2c0f82c5c89388f38da3` |
| `yetlet-config` | `0ad1d3fe2aee9fc02b7286ed923c7d5655c4973504f22767865ce3a9bb090d69` | `85dcaaaae592c91c25362b139c64022856d55ad4bf05dd1d7042e34fd9728044` |

The directory-source descriptors use each accepted archive's real package
checksum. Cargo adds four Yetlet checksum fields to each generated packaged
lock. There are no added/removed package identities, dependency-version changes,
third-party checksum changes or other changed fields in either packaged lock.
Source lockfiles remain unchanged despite this packaging normalization.

Thus the development dependency edge does not prevent local model verification
with unchanged runtime/API/manifests. It does not yet establish an approved live
first-publication procedure. **The new archives are not the
signed archives and are not approved registry payloads.** In particular, each
new packaged lock refers to accepted sibling hashes, not a demonstrated final
registry set. Do not assume these two outputs form an upload-ready closed set,
silently drop checksums, or transfer the old signatures to them.

The next bounded contract is the supported first-publication bootstrap and
sixteen-package registry artifact/checksum policy: determine how actual publish
bytes will be frozen and bound to evidence, including versioned development
references. The earlier accepted recipe's
bootstrap directory descriptors use a null package checksum; that observation
is not authority to weaken registry integrity or rerun the accepted build.
Review that policy before any further packaging, source change, new candidate,
signature or upload. Authentication, actual name rights, registry acceptance and
downloaded consumer proof also remain separate open conditions. These local
dry-runs do not close them.

The original failed Cargo commands/logs and the one input-staging correction
remain retained. The first failed Config archive was not separately captured:
initial collection expected the normal package output path, then located
publish's temporary archive paths. Only final archive digests are claimed; no
Cargo execution was repeated merely to capture an archive. This does not relabel
the failed compile or its command/status as passing.

Evidence, exact argv/exit records, both final archives, packaged-lock/member
diffs and input correspondence are retained under
`target/yt020-first-publish-20261030/`. Independent Reviewer
`/root/yt020_first_publish_review_20261030` PASS, zero issues/required followups;
all five reviewed source hashes match before/after review, and native reviewer,
configured model and effective effort are verified. Only completion metadata
follows acceptance. Owned temporary source, credential-free Cargo home, vendor
staging and build directories are removed; original input directories are
untouched. The source archive, final dry-run archives and failure logs remain.
Docs/internal links/Markdown/whitespace/DAG and focused diff checks PASS; no old
qualification suite was replayed. No new Commit/Push, actual upload, signature
or release. YT-020 remains Blocked at the registry conditions described above.
