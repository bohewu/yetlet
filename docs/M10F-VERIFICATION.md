# M10f Verification — Platform, Firefox and Accessibility Source

**Date:** 2026-10-10
**Task:** YT-065 / M10f
**Status:** Automated local and hosted qualification PASS. YT-065 remains In progress for its declared manual IME/assistive-technology/device coverage; no production/accessibility/SLA guarantee is established.

## Authority and written scope

The bounded source assignment follows [M10 remaining plan](M10-REMAINING-PLAN.md),
planning reviewer `wc_job_n6i2DoBNKaltAWs2` PASS and prior YT-064 reviewer
`wc_job_CeUNYnHsO1yI_kBz` PASS supplied by the handoff. Neither prior review
qualifies this new implementation.

Only these source files are written:

- `.github/workflows/qualification.yml`: weekly/manual read-only native matrix
  and Linux Firefox release qualification; named seven-day artifacts.
- `scripts/smoke_forms_accessibility_firefox.py`: bounded stdlib keyboard/DOM
  forms checks with owned startup/exception cleanup and a sanitized fixed report.
- `docs/QUALIFICATION-MATRIX.md`: scope, historical/pending evidence,
  prerequisite/activation caveats and manual IME/AT/WebKit/mobile/zoom runbook.
- `docs/M10F-VERIFICATION.md`: this source-validation handoff record.

Existing M10a–e changes, Rust/Cargo/runtime/dependency inputs and
`.github/workflows/rust.yml` are preserved. The current original dirty `master`
checkout remains based on `7c5d1267fd2ea3490738da105b476e271278ae7a`.
No browser/server is launched by the executor. No staging, commit, push,
branch/worktree, visibility/settings or remote automation action is authorized.
Shared backlog/roadmap/catalog and documentation-checker changes belong to the
orchestrator. Temporary validation artifacts belong only under
`target/m10-remaining/` or `target/qualification/`.

## Validation evidence

Syntax and mocked startup/cleanup tests establish command/source behavior only;
they do not establish browser behavior, Rust platform portability or hosted
execution. The existing required baseline job names are preserved.

| Executor command/check | Actual result |
|---|---|
| Python `compile()` of new script, without bytecode output | PASS |
| YAML parse/policy and exact nine-library package list | PASS; only scheduled/manual events, read-only permissions, pinned approved actions, bounded matrix/timeouts and seven-day always-upload artifacts |
| `bash -n` for extracted workflow run blocks | PASS; nine blocks, all begin with `set -euo pipefail` |
| Seven existing/new build/smoke `--help` invocations | Exit 0; release/profile flags confirmed; no build, server or browser launched |
| `PYTHONDONTWRITEBYTECODE=1 python3 target/m10-remaining/YT-065-no-launch-tests.py` | Exit 0; eight focused mocked tests PASS |
| `python3 scripts/check_docs.py` | Exit 0; 68 documents / 67 backlog items; language, internal links, structure, whitespace and DAG PASS |
| `git diff --check` | Exit 0 |

The focused temporary test artifact mocks process creation, WebDriver,
readiness and ephemeral port allocation. It tests server readiness failure,
second-process spawn failure, driver readiness failure, session creation
failure, assertion failure, cleanup failure, success only after cleanup, and
standard Tab/Enter/Unicode key-action payloads. Both cleanup attempts continue
when one fails; synthetic exception text is absent from the report; temporary
trees are removed. No real process cleanup or keyboard/browser PASS is claimed.

The initial mocked test attempt omitted the port mock and hit a sandbox socket
`PermissionError` before startup. Classified as a harness failure: one smallest
fixture correction added the missing mock, followed by one focused successful
rerun. No source/product repair was needed; no browser/server was started.
One documentation patch context mismatch was corrected before application
(operational failure, no source mutation on the failed attempt).
Executor product repair attempts: 0; operational retries: 1.

The setup-python v6 SHA `ece7cb06caefa5fff74198d8649806c4678c61a1` is
handoff-provided as verified via `gh`; checkout/upload SHAs reuse the existing
baseline workflow. The executor checked the
[official Ubuntu image inventory](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md)
for installed Firefox/geckodriver and `GECKOWEBDRIVER`, and
[GitHub trigger documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)
for default-branch activation. Inventory is prerequisite evidence, not a
hosted-run result. The workflow observes actual tools and fails closed.

## Main-session local observations

All three asset builders and native Todo/dashboard/forms release binaries succeeded before browser execution. The existing 136 production dist files were byte-identical before/after builds. Three existing Firefox suites ran once, followed by one new forms suite with real WebDriver Tab/Enter/Unicode actions, error associations and first-error focus, root language, no-state 303 PRG and owned cleanup. The forms report explicitly lists OS IME, screen reader, WebKit/mobile, zoom and conformance as `not_verified`. Logs/results are retained under `target/m10-remaining/065-firefox-*` and `target/qualification/`; these are local Linux observations, not new hosted results.

## Pending runtime and hosted evidence

| Required observation | Current result |
|---|---|
| Local three existing Firefox release suites | Main-session Job `wc_job_sv9TmUH_rdBF2DdM`, terminal exit 0; Todo/Search, keyed and multi-island each PASS on Firefox 157.0 |
| Local new forms keyboard/semantic suite and JSON report | Main-session release execution PASS, five keyboard/semantic groups; `target/qualification/forms-accessibility-firefox.json` parsed |
| Current browser/CSP assertions and owned teardown | PASS locally; no surviving owned servers/profiles; forms geckodriver exited 0, server terminated -15, report cleanup PASS |
| Ubuntu/Windows/macOS native hosted check/test/doctest lanes | Run `38043565238` PASS: each nine-crate lane executed 80 tests and four doctests on Rust 1.99; Linux x86_64, Windows MSVC x86_64 and macOS ARM64 |
| Linux hosted Firefox and forms lane | Same run PASS: Firefox 157.0/geckodriver 0.37.1, all four release suites, parsed forms report cleanup PASS |
| Hosted run ID, source SHA, ImageOS/ImageVersion, Rust host, browser versions | Run `38043565238`, source `c38837835dac683b09194b475baab2d8eda5a875`; named artifacts downloaded and parsed, see delivery verification |
| Independent YT-065 implementation reviewer | `wc_job_FH0L7fH4UH6ICPoW` terminal exit 0, `gpt-6-astra`/high requested, read-only, PASS with no issues/followups; source/local gate only |
| Real OS IME, NVDA/VoiceOver, WebKit/mobile and zoom | Not verified; manual instructions only |

Main session should execute one bounded headless sequence from the
[matrix/runbook](QUALIFICATION-MATRIX.md), parse its report, retain exact logs
and exit statuses, and separately record compile/browser/CSP evidence. Failure
must be classified under the materiality gate before a focused repair. No
repeated visible probe or broad harness rewrite is authorized.

The maintainer subsequently authorized PR/merge and actual hosted execution. PR #1 merged as `c38837835dac683b09194b475baab2d8eda5a875`; the new qualification workflow was dispatched and reached terminal success in run `38043565238`. Its three native lanes and Linux Firefox lane are independently observed hosted evidence, not inferred from the earlier local tests. The weekly schedule is now present on the default branch; future scheduled outcomes are not preclaimed. Additional local Windows GNU and offline Docker runs passed on the identical candidate tree. See [delivery verification](M10-DELIVERY-VERIFICATION.md).

Open decisions: supported OS/architecture/browser version policy, real-device and assistive-technology test ownership, and manual evidence acceptance. Automated hosted execution is complete for the recorded platforms, while OS IME, screen reader, WebKit/mobile and zoom remain unverified. YT-065 retains those manual gates; independent provider-operations work remains separate. YT-020 remains Blocked.
