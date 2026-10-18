# YT-102 — Native Windows current-generator verification

Date: 2026-10-18. Status: native verification PASS; independent Reviewer PASS.

## Scope and source

This followup addresses the native Windows generator evidence gap identified by
the [CLI audit](YT-102-CLI-CONTRACT-AUDIT-20261018.md) and
[local-source generator compatibility record](YT-102-GENERATOR-COMPATIBILITY.md).
It exercises the existing current-service-profile integration and focused
Docker/Compose/Kubernetes unit tests. It does not repeat browser/HMR testing or
build/deploy an image or cluster.

The source baseline is local HEAD
`f4fa23b0b68e9a346347f0a7611b628e19ac73f8` plus the reviewed
[dotenv correction](YT-102-DOTENV-BOUNDARY-FIX.md). A source-only snapshot was
transferred to the Windows Runner through WebCodex. Its original checkout stays
at `3128d23591feee15ce976d7b83a56dcd587048d7`; that older checkout is not the tested
CLI source. The owned verification directory, relative to that Windows checkout,
is `artifacts/cli-generator-windows-20261018/`.

The initial archive contains 287 files with SHA-256
`41008a4492d7abc88d4c9ac3b429aefac461eec9b5a0aa099707ff0ddc378696`.
Windows verified extracted bytes before execution. Eighteen tracked public
license links were deliberately materialized as ordinary files for the snapshot;
the local manifest records their original targets. Product source bytes were
unchanged by packaging. The subsequent fixture/product deltas are recorded
separately and do not overwrite the initial manifest or failures.

Native tools are existing Rust/Cargo 1.99.0 for `x86_64-pc-windows-gnu`, the
toolchain's bundled GCC linker, and installed GNU binutils 2.47. No tool
installation or dependency download is part of this verification. Commands use
explicit tool paths, offline Cargo resolution and a task-owned target directory.

## Retained failures and correction

1. The first native compilation failed before tests because the bundled
   `dlltool` could not launch its helper (`CreateProcess`). After checking the
   installed binutils, execution selected its exact `dlltool` through Rust's
   existing `-C dlltool` setting and placed that binutils directory first in the
   child PATH. This was operational handling, with no product change.
2. Native compilation then succeeded, but the fixture's `canonicalize()` produced
   a Windows verbatim path. The CLI intentionally rejects that path class. The
   single fixture correction obtains the existing absolute framework root through
   the manifest directory's parents, preserving the product restriction.
3. That corrected fixture exposed an actual compatibility defect: selected
   generator inspection constructed an expected dependency string with mixed
   separators and rejected a valid native Windows Cargo path. Inspection now
   compares paths through the existing native `Path`/`framework::absolute`
   semantics against the same qualified workspace. Only then does it preserve
   the original path string for exact comparison of every remaining manifest
   field. The configuration dependency must also match that workspace.
4. The first run after that product correction reached protected publication and
   rejected the fixture's inherited writable output-parent ACL. The fixture now
   creates its output parent through the existing CLI writer, following the
   established Docker unit-fixture pattern. The product ACL requirement remains
   intact; this is a distinct fixture setup correction.

Product changes are limited to
[docker.rs](../tools/yetlet-cli/src/docker.rs). The existing
[generator integration fixture](../tools/yetlet-cli/tests/generator_selected.rs)
keeps its four profiles and alternates native/Cargo-style separator spellings;
it also contains the two fixture setup corrections above. No new option,
dependency, source resolver or validation framework is introduced.

The initial logs and command exit codes remain in
`artifacts/cli-generator-windows-20261018/windows-results/`. The first attempt
never reached a test; subsequent failures retain native Cargo exit 101. Compiler
temporary files were removed only from this task's dedicated fixture directory.

## Results and review

| Native Windows GNU check | Result |
| --- | --- |
| `cargo test --offline --locked --manifest-path <snapshot>/tools/yetlet-cli/Cargo.toml --test generator_selected -- --exact current_service_profiles_generate_all_outputs_with_explicit_offline_locks --nocapture` | 1 passed; all four profiles and 12 output generations completed |
| Same manifest, `cargo test --offline --locked --lib docker::tests -- --nocapture` | 12 passed, 0 failed |
| Same manifest, `cargo test --offline --locked --lib compose::tests -- --nocapture` | 6 passed, 0 failed |
| Same manifest, `cargo test --offline --locked --lib k8s::tests -- --nocapture` | 5 passed, 0 failed |

All four native Cargo commands exited 0. WebCodex Job
`wc_job_2XIepFGKUJwwubwZ` completed with exit 0 in session
`wc_sess_a1BYZrwd8xKDUXCL`. The exact literal executable/argv, elapsed times and
complete logs are retained in `windows-results/`; execution environment settings
are in `native-tools.json` and the retained `native-final.py` command script.
Native counts are distinct from Linux's 14 Docker and 7 Compose tests.

The integration verifies explicit separately prepared offline locks, unavailable
tools/cache during each generator call, Production custom frontend root and
replica settings, public inventory source mapping, token/private-file exclusion,
unchanged application inputs, and edited-source refusal. It alternates native
and Cargo-style path separators without expanding the original profile matrix.

Linux's focused checks passed (integration 1, Docker 14, Compose 7, K8s 5).
The full CLI regression after the product correction passed 111 tests with five
explicit ignored tests. Clippy and formatting passed. After the final
output-parent fixture correction, the focused integration, focused Clippy and
formatting passed again; the broader suite was not repeated for fixture setup.

Independent read-only review returned PASS with zero issues and required
followups. Its native trace matched the registered reviewer, configured model
and effective effort. `review.json` retains the bounded result. No product source
or fixture changed after review. No all-platform or production claim follows
from these bounded native results.

## Limits and preservation

The existing four profiles are service, service/settings, service/Vite and
service/Vite/Tailwind, each with all three generator outputs. Their asset inputs
are a declared structural dist fixture, separate from the previous actual
Vite/OCI producer proof. Registry-only sources, arbitrary edited applications,
native MSVC execution and production deployment remain unqualified here.

Both checkout identities, unchanged tested source hashes, exact command logs and
owned-fixture cleanup are retained with the evidence. Prior YT-103/planning and
dotenv changes remain outside this correction. No commit or push is included.

`source-tested-final.json` records the original archive plus the two changed
paths: the product comparison and its existing fixture. All 287 final Windows
file hashes match the current local bytes, including materialized public license
contents. The source stayed unchanged during the successful commands. The
Windows checkout HEAD/index/status stayed unchanged, and the task's fixture
directory was empty afterward. Only task-owned source/build/log evidence remains.

Registry delivery and broader YT-102 DX research remain open. YT-092 is still the
independently dependency-ready application item; no backlog status is changed.
`cargo xtask check docs` and whitespace checks passed; final receipts and the
14-file pre-existing-change preservation check are retained in the evidence
directory.
