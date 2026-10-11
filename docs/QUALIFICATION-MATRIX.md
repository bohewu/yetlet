# Platform, Browser and Accessibility Qualification Matrix

**Task:** YT-065 / M10f. **Current status:** **Done at automated desktop/native/browser qualification scope.** Run `38043565238` tested source `c38837835dac683b09194b475baab2d8eda5a875`. Real IME, assistive-technology speech, WebKit/mobile and zoom remain explicitly Not verified optional coverage. This matrix supplies no supported-device range, accessibility conformance, SLA or production guarantee.

## Coverage and evidence boundaries

| Lane | Defined scope | Current YT-065 evidence | Historical evidence |
|---|---|---|---|
| Ubuntu 24.04 native | Stable Rust check, library tests and doctests for nine reusable crates | Hosted Rust 1.99, x86_64-unknown-linux-gnu: 80 tests/four doctests PASS | [M9j](M9J-VERIFICATION.md) local native workspace freeze; not this hosted lane |
| Windows 2025 native | Same nine crates, native runner host target | Hosted Rust 1.99, x86_64-pc-windows-msvc: 80 tests/four doctests PASS | Separate authorized local Windows GNU Rust 1.88 run: 80/four PASS; not local MSVC |
| macOS 15 native | Same nine crates, native runner host target | Hosted Rust 1.99, aarch64-apple-darwin: 80 tests/four doctests PASS | No x86_64 macOS qualification claimed |
| Ubuntu 24.04 Firefox | Release Todo/Search, keyed bindings, multi-island suites; SSR forms keyboard/semantic checks | Local and hosted Firefox 157.0: all four suites PASS; hosted forms report cleanup PASS | M9j: three release suites PASS on local Firefox 157/geckodriver 0.37.1; forms keyboard suite did not exist |
| Hosted Chrome baseline | Existing required baseline workflow and existing three release suites | Candidate PR run `38042614403`: all three Chrome release suites PASS; not a new local Chrome claim | M9j: three local release suites PASS on Chrome 155 |
| WebKit/Safari desktop | Manual application journeys below | Not verified; no automated WebKit lane | No qualification claimed |
| iOS Safari / Android Chrome | Real device touch, soft keyboard, viewport and zoom | Not verified; no mobile lane | No qualification claimed |
| Real OS IME | Composition, candidate selection, caret and keyed input lifetime | Not verified; manual only | Existing synthetic keyboard/selection checks do not prove OS composition |
| NVDA / VoiceOver | Spoken names, language, errors, focus and navigation | Not verified; manual only | DOM assertions do not prove assistive technology output |
| Zoom/reflow | Desktop 200%/400%, narrow viewport and mobile text scaling | Not verified; manual only | No qualification claimed |

The native package list is exactly `yetlet-core`, `yetlet-reactive`,
`yetlet-html`, `yetlet-dom`, `yetlet-dom-keyed`, `yetlet-actions`,
`yetlet-server-axum`, `yetlet-request-id` and `yetlet-session`. It uses the
actual runner host target and records `rustc -vV`; runner labels do not imply
both x86_64 and ARM coverage. Sample binaries and Unix-specific sample smokes
are outside these native portability lanes. No failing tests are skipped to
manufacture a platform PASS. Browser packaging is qualified separately on
Linux. Stable Rust is explicitly installed and observed; the unchanged
baseline workflow remains responsible for the Rust 1.88 MSRV check.

## Workflow source and activation

[Qualification workflow](../.github/workflows/qualification.yml) has only
`schedule` (Wednesday 03:17 UTC) and `workflow_dispatch`. It grants only
`contents: read`, uses full-SHA GitHub-owned checkout/setup-python/upload
actions, disables persisted checkout credentials, cancels superseded runs,
limits native parallelism to two, and limits native/Firefox jobs to 35/60
minutes. It neither supplies secrets nor requests token write or id-token
permissions. The existing `Rust 1.88 MSRV` and
`Stable native, samples, and Chrome` required job names/workflow stay unchanged.

GitHub requires this workflow on the default branch for activation; schedules
run there and may be delayed or dropped. Manual dispatch also depends on the
default-branch workflow. Local uncommitted YAML cannot establish hosted PASS.
The maintainer authorized that delivery: PR #1 was merged without bypass and run `38043565238` completed successfully. The workflow is now on the default branch; future schedule execution is not preclaimed. No repository visibility or branch-protection setting was changed by this delivery. See
[GitHub trigger documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows).

The [official Ubuntu2404 runner image inventory](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md)
documents installed Firefox/geckodriver and the `GECKOWEBDRIVER` directory.
The Firefox job explicitly requires that directory's executable, puts it on
PATH, and checks Firefox plus both versions before builds. Missing tools fail
the job; no floating driver download or silent browser substitution occurs.
Image software can change: each run records `ImageOS`, `ImageVersion`, source
SHA, Rust host/tool versions, Python and browser/driver versions. Current
observations, rather than this inventory, establish the tested versions.

Artifacts retain only named qualification build/test/identity logs and the
sanitized forms JSON report for seven days, with unique lane/run/attempt names
and `always()` upload. No entire `target` tree, profiles, raw page state,
cookies or arbitrary environment dump is uploaded. Cargo and Python failures
propagate through Bash `set -euo pipefail` even when stderr/stdout feed `tee`.
Job timeout/cancellation may prevent normal harness cleanup; the hosted runner
is disposable, and an interrupted job is not a successful qualification.

## One bounded local sequence (main session only)

Use prebuilt release assets from `build_browser.py --release`,
`build_keyed_bindings.py --release` and
`build_multi_island_dashboard.py --release`, plus release native binaries
`yetlet-todo-server`, `yetlet-multi-island-dashboard` and
`yetlet-forms-validation`. Require Firefox and geckodriver on PATH and record
their versions together with Rust and source identity. Run each command once:

```sh
python3 scripts/smoke_firefox.py --profile release
python3 scripts/smoke_keyed_bindings_firefox.py --profile release
python3 scripts/smoke_multi_island_dashboard_firefox.py --profile release
python3 scripts/smoke_forms_accessibility_firefox.py --profile release
```

Stop at failure, classify it before repair, and retain exact exit statuses.
The executor does not run this sequence. Main-session logs belong under
`target/m10-remaining/` or `target/qualification/`. Existing build helpers and
suites retain their established output locations and behavior. Main session
must check teardown for those existing suites; this task does not rewrite them.

The new forms command defaults to `release` and also accepts `--profile debug`.
It owns one ephemeral loopback server, one headless WebDriver session and its
POSIX process groups; it tracks startup processes before waiting, closes the
session, stops/reaps both groups with bounded existing helpers, and removes its
temporary tree. Its fixed report is
`target/qualification/forms-accessibility-firefox.json`. It uses real W3C
Tab/Enter/text key actions, without assigning focus or input via JavaScript.
DOM reads assert name/email/submit traversal, both label/control associations,
first-invalid-field autofocus, error text/role/descriptions, root `lang=en`
on initial/error/success documents, Unicode input preservation, and no-state
success/refresh/empty-form behavior. A separate no-redirect HTTP observation
checks 303, Location, no-store and no cookie. Unicode key actions exercise text
entry, not a real operating-system IME.

Parse the JSON only after exit: require command exit 0, `status=pass`,
`cleanup=pass`, `stage=complete`, all five declared check groups, a browser
version, and terminated owned process exit codes. Failure reports retain
bounded fixed stage names, exception types, completed groups and process exit
codes; raw process output and exception messages are discarded. `not_verified`
must retain real IME, screen-reader and other untested scopes even after PASS.
Cleanup failure prevents PASS. No screenshots or visible browser are needed.

## Optional future manual qualification runbook (currently Not verified)

Record tester/date, source/build identity, OS/architecture, browser version,
assistive technology or IME version/settings, profile and observed result for
each attempt. Use synthetic text and loopback fixtures; retain only redacted
notes. Record PASS, FAIL or Not verified per journey, with reproduction steps
for failures. A blank record or successful automation is not manual evidence.

1. **Keyboard and forms:** Open `/form` in the forms sample. Traverse with Tab
   and Shift+Tab; confirm name, email, submit order and visible focus. Submit
   `x`/`invalid` with Enter; confirm first-field focus and understandable linked
   errors. Correct only name, submit and confirm email focus. Correct email,
   submit, refresh success, and return to a blank form. Check English language,
   labels and no saved data. Repeat relevant Todo/Search and dashboard controls
   without a pointer; inspect focus after keyed move, remove, disposal/remount.
2. **Real IME and caret:** Enable a real Chinese/Japanese/Korean OS IME on
   Linux, Windows and macOS where available. Compose multi-character input,
   choose candidates, cancel composition and insert in the middle of existing
   text. Observe caret/selection and committed text. In the keyed fixture,
   reorder while editing and verify ownership/focus/text/caret survive; test
   removal/disposal. Record engine and composition behavior explicitly; typed
   Unicode or dispatched synthetic composition events cannot satisfy this cell.
3. **NVDA and VoiceOver:** With Windows NVDA in Firefox/Chrome and macOS
   VoiceOver in Safari, read headings/landmarks and move through controls.
   Confirm spoken field names, English language, required/invalid state,
   linked error text and first-error focus after each server navigation. Confirm
   success content, keyboard operation of islands and absence of duplicate
   announcements after remount. Record actual speech/behavior and settings;
   automated label checks alone cannot satisfy this cell.
4. **WebKit and mobile:** Run SSR forms, Todo local/remote commands, Search
   activation and dashboard independent activation/disposal on desktop Safari,
   real iOS Safari and real Android Chrome. Check touch targets, soft keyboard,
   rotation, text entry, errors, focus and server fallback. Record device/OS
   versions and browser console/CSP outcomes separately. A resized desktop
   Firefox window does not prove a mobile browser.
5. **Zoom/reflow:** At 200% and 400% desktop zoom and a 320 CSS-pixel viewport,
   inspect clipped content, horizontal scrolling, control reachability and
   focus visibility. On mobile increase OS/browser text size and repeat forms
   errors and islands. Record observed defects and exact settings. No general
   WCAG certification follows from these bounded journeys.

## Bounded task acceptance and optional future coverage

The local/browser and hosted run IDs, terminal results, observed versions/architectures and artifact references are recorded in [M10f verification](M10F-VERIFICATION.md). The authorized workflow merge and actual hosted run reached terminal success. YT-065 is therefore Done at the bounded automated desktop/native/browser scope.

The manual cells above remain explicitly Not verified and optional. They do not block this bounded task, do not become supported-platform or accessibility-conformance claims, and must not be inferred from typed Unicode or DOM assertions. Supported platform/version policy, real-device/assistive-technology ownership and any application-specific manual evidence remain future decisions. YT-020 release qualification remains Blocked. YT-066 is a separate backlog item.
