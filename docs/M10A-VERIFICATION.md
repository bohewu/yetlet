# M10a Verification — License and Security Policy

**Date:** 2026-10-10
**Task:** YT-060 / M10a
**Status:** Done. Focused validation and independent read-only reviewer `wc_job_MPF4l2t8JaJXBguv` PASS.

## Decision

Yetlet uses the conventional Rust ecosystem dual-license expression:

```text
MIT OR Apache-2.0
```

Users may choose either license. The Apache option supplies an explicit patent grant; the MIT option preserves compatibility with consumers that standardize on the shorter permissive license. The license does not require the upstream project to accept external pull requests.

## Repository policy files

- `LICENSE-MIT` and `LICENSE-APACHE` contain the complete license texts.
- Workspace package metadata declares `license = "MIT OR Apache-2.0"` and the canonical repository URL.
- Every Yetlet-owned package inherits the workspace license/repository metadata.
- Example/benchmark/application packages additionally use `publish = false`; only the reusable library crates remain technically publishable candidates. This does not authorize publishing any crate.
- `SECURITY.md` prohibits public vulnerability Issues, documents the current private-repository contact boundary and the future GitHub private-reporting path, and makes no response-time or production-support claim.
- `CONTRIBUTING.md` states that external pull requests are not accepted. Issues and forks remain available under the license; no CLA, DCO or sign-off process is introduced.

## Required validation

```bash
cargo metadata --locked --no-deps --format-version 1
cargo package --list -p yetlet-core --allow-dirty
python3 scripts/check_docs.py
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
git diff --check
```

Validation must prove all 28 workspace packages carry the exact SPDX expression and repository URL, all 19 example/application packages are `publish = false`, all nine reusable crates remain publishable candidates, license texts are present in package file lists where applicable, YT-020 remains Blocked, and no crate is published.

## Observed validation

- Cargo metadata: **28/28** packages carry `MIT OR Apache-2.0` and the canonical repository URL.
- Reusable crates: **9** publishable candidates; every package file list contains both canonical license texts through exact package-local symlinks.
- Example/application/benchmark packages: **19/19** use `publish = false`.
- Documentation/catalog: **65 documents / 67 tasks**, **12 samples**, and **4/4** focused catalog tests PASS.
- License/policy wording, symlink targets, tracked/untracked whitespace, empty index, unchanged Private visibility and unchanged `master` HEAD PASS.
- No crate, release, tag or repository visibility change occurred.

## Reviewer gate

Independent role-directed reviewer Job `wc_job_MPF4l2t8JaJXBguv` completed at terminal exit 0 with `gpt-6-astra`, high reasoning effort and a read-only sandbox. It independently checked all 28 manifests, all nine library package archives, the 18 exact license symlinks, both complete license texts, `SECURITY.md`, `CONTRIBUTING.md`, documentation and Git hygiene. It returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": [
    "Verified 28/28 package metadata, 19 nonpublishable examples, nine library candidates, 18 exact license symlinks, and both licenses in all nine package lists."
  ]
}
```

This was ordinary local `codex exec <reviewer prompt>`, not `codex exec review`, native ACP dispatch, legal advice or publication approval. The reviewer made no edits, stages, commits, pushes, releases or settings changes. YT-060 is **Done**; public publication task YT-020 remains Blocked.
