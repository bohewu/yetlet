# YT-102 — Current CLI contract audit

Date: 2026-10-18. Checkout: `feat/yt090-configuration`.
Audited HEAD: `f4fa23b0b68e9a346347f0a7611b628e19ac73f8`.
Status: audit complete; one open P2 contract mismatch (F-01 / AC-14).
This is not an all-criteria PASS or a fresh platform/runtime qualification.

## Scope and method

The owner accepted a bounded audit of the current [CLI contract](CLI-CONTRACT.md):
commands, settings/environment/profile behavior, and acceptance evidence. Three
independent read-only reviewers covered grammar/scaffolds/doctor,
settings/execution, and generator/evidence claims. Native traces matched the
configured reviewer role, model and effective effort. The root agent synthesized
their results and inspected the reported source chain.

Only source, existing test definitions, retained receipts and component hashes
were inspected. No CLI operation under test, test suite, build, browser, Node,
Docker, WebCodex or service was started. Documentation validation is reported
separately. Product implementation, tests and the accepted contract were not
edited. Prior dirty YT-103/planning files were preserved; no commit or push.

Evidence root: `artifacts/cli-contract-audit-20261018/`. `before.json` records the
checkout boundary; `audit.json` contains the normalized matrix, exact source/test
references, finding and bounded reviewer traces. Acceptance criteria retain their
original authority; missing suggested checks do not become new requirements.

## Acceptance map

“Aligned” below means no concrete mismatch found by this scoped inspection,
supported by the cited implementation/test definitions and existing evidence.
It does not mean those tests were rerun or every platform was qualified.
Implementation/test paths in this table are relative to `tools/yetlet-cli/`.
Full references are retained in `audit.json`.

| Criterion | Assessment | Source and existing test seam |
| --- | --- | --- |
| AC-01 discovery without effects | Aligned | `src/lib.rs:119`, `src/main.rs:7`; `tests/commands.rs:95` |
| AC-02 grammar and literal arguments | Aligned | `src/lib.rs:162,266,278`; parser tests at `:318,435,488` |
| AC-03 project/settings/frontend roots | Aligned; dotenv exception under AC-14 | `src/settings.rs:75,90,109`, `src/platform/fs/mod.rs:141`; settings/root tests |
| AC-04 plain SSR and settings opt-in | Aligned | `src/scaffold.rs:617,648`, strict generated types; `tests/selected_scaffold.rs:25,219,237` |
| AC-05 complete Vite scaffold/publication | Aligned except F-01's dotenv guarantee | `src/scaffold.rs:684,727`; `tests/selected_scaffold.rs:25,180` |
| AC-06 generator source/lock/export boundaries | Aligned at bounded local-source scope | `src/docker.rs:146,525`, `src/k8s.rs:41`; `tests/generator_selected.rs:60,114,158` |
| AC-07 bounded environment-only doctor | Aligned at recorded component scope | `src/doctor.rs:17,36,105`, `src/platform/process/probe.rs:266,477`; doctor/schema/probe tests |
| AC-08 run/watch/build selection and ownership | Aligned | `src/app.rs:40,44,47,73,84`, `src/platform/process/foreground.rs:561,606`; frontend/watch tests |
| AC-09 native process lifecycle | Aligned at selected Linux/Windows GNU scope | Component-matched adapter/tests and retained lifecycle records; limitations below |
| AC-10 honest scope/evidence claims | Aligned | Current contract/CLI guide and separately dated implementation records |
| AC-11 Cargo-owned dependency sources | Aligned; registry delivery deferred | `src/lib.rs:207`, `src/scaffold.rs:619,631`; parser/scaffold tests |
| AC-12 narrow options and migration hints | Aligned | `src/lib.rs:31,131,184,196`; grammar/discovery tests |
| AC-13 shared Loader and inherited ENV isolation | Aligned | `src/settings.rs:74,125,133`, `src/platform/process/mod.rs:53`; loader/ENV product tests |
| AC-14 confined frontend dotenv | **Gap: F-01, P2** | `src/scaffold.rs:708`, `src/app.rs:53`; linked dotenv input is not checked |

## F-01 — Linked dotenv input escapes the frontend root

**Requirement:** [frontend confinement](CLI-CONTRACT.md#4-project-paths-and-frontend-layout)
and AC-14 require frontend-root-only dotenv loading and refusal of symlink escape.
This is an existing contract condition, not a new sandbox promise.

**Source-established trigger:** in an otherwise valid selected Vite application,
`frontend/.env` is a symlink to `../.env`. A selected frontend build or server
startup still passes the current directory/package checks. `run`/`watch` also
require an already prepared dist, as usual. No runtime reproduction was performed
in this audit.

The source chain is concrete:

1. [scaffold.rs](../tools/yetlet-cli/src/scaffold.rs), line 708, enables
   `envDir: root` in the selected production Vite configuration.
2. [vite.dev.config.mjs](../tools/yetlet-cli/templates/vite/vite.dev.config.mjs),
   line 11, inherits that configuration. Its HTTP `.env` deny rule at line 27
   controls requests, not startup filesystem reads.
3. [app.rs](../tools/yetlet-cli/src/app.rs), lines 53–64, validates the frontend
   directory/package and optional existing dist. It does not check dotenv inputs.
4. Both selected package templates pin Vite 8.3.4. The matching installed provider
   source in the retained CLI implementation app enumerates the root's `.env`,
   `.env.local`, `.env.MODE` and `.env.MODE.local`, then uses `fs.statSync` and
   `fs.readFileSync`. Those calls follow the link. The inspected provider file is
   `dist/node/chunks/node.js` beneath that app's installed Vite package, lines
   2257, 6053–6059 and 6074–6089. Its full checkout path is retained in the finding's
   `provider_path` field in `audit.json`.

**Impact:** outside-root dotenv content is consumed despite the documented
boundary. Imported `VITE_*` values can enter frontend output. Dotenv expansion
can also derive a public value from a backend-named key in that file. Removing
inherited APP variables from the child environment does not prevent this file
read. This finding concerns the generated, selected dotenv path; arbitrary
package-script effects remain outside a security-sandbox claim.

**Smallest followup:** reject linked/reparse dotenv inputs before frontend dotenv
loading while preserving ordinary frontend-local dotenv support. Add one focused
product fixture covering refusal of the outside-root link and acceptance of a
regular local file. Broader HMR tests, a new validator framework or platform
recertification are not required to establish this specific correction.

No repair was made during the audit. The existing root/ENV tests do not establish
refusal of a linked dotenv file, so their earlier PASS does not close F-01.

## Evidence reuse and qualification gaps

- **Doctor:** current `doctor.rs` and `platform/process/probe.rs` match retained
  doctor source manifests. Existing Linux bounds/cleanup tests and Windows
  normal/unavailable receipts support those unchanged components. They do not
  qualify application/frontend/generator readiness or every failure injection.
- **Process lifecycle:** current Windows adapter, Linux adapter, foreground
  supervisor and native test files match integrated-watch manifests. The watch
  filter matches its separately verified normalization correction. The older
  pair supervisor's unchanged `run_pair` body and later source-matched native
  watch evidence are complementary, not an exact whole-current-HEAD rerun.
  Windows integrated watch used generated console events. The additive `watch`
  command has no fresh native Windows invocation claim. Preserve the recorded
  Linux PTY exit 1 without inventing signal status.
- **Current generators:** [compatibility evidence](YT-102-GENERATOR-COMPATIBILITY.md)
  separates 110 Linux tests, MSVC compile-only, 14 generated-app tests, actual
  Vite/Docker runtime and exact cleanup. The structural dist fixture is distinct
  from actual producer output. **Native Windows execution of this extension
  remains missing and explicitly unclaimed.** It is a qualification gap, not a
  discovered generator implementation failure.
- **Historical records:** the initial CLI implementation report retains its
  then-current watcher/generator exclusions. Current contract/guide sections
  explicitly point to the later implementation records. Initial global HEAD or
  suite counts are not presented as current all-platform certification.

Sources inspected include the [doctor record](YT-102-DOCTOR-VERIFICATION.md),
[Windows pair record](YT-102-WINDOWS-VITE-VERIFICATION.md),
[integrated watch record](YT-102-RUST-WATCH-IMPLEMENTATION.md), and their retained
source manifests/native receipts. No historical failure or qualification limit
was removed.

## Deferred scope and next action

Registry-only source delivery/usable published templates and YT-096/YT-020 release
qualification remain separate. The broader YT-102 journey audit, cold/warm/change
measurements and DX roadmap remain Todo. Arbitrary generator sources, general
proxy/auth, Rust memory preservation, HA and production guarantees are not added
by this audit. The separate YT-103 local qualification is unchanged.

The next CLI closeout action is the narrow AC-14 correction above, followed by
native Windows qualification of the new generator representation. YT-092 remains
the independently dependency-ready application item; this report does not start
it or mark the broader YT-102 backlog item Done.

## Validation and preservation

Only this report and local audit evidence were added. `cargo xtask check docs`
passed (75 documents, 103 backlog items); `git diff --check` passed. The first
documentation check rejected a non-English directory name inside a provider path;
the full path was moved into `audit.json` and the focused check then passed.

`preservation.json` confirms unchanged HEAD, an empty index, unchanged hashes for
all six pre-existing dirty/planning files, and no product-source or CLI-contract
changes. No product runtime tests, new tool installation, remote environment
mutation, commit or push were performed.
