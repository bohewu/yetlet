# YT-102 — Safe settings-error classification research

Date: 2026-10-20. Source HEAD: `e691dca`. Research only, following the
[settings decision table](../tools/yetlet-cli/README.md#settings-selection-decision-table).
The owner requested a practical minimum using the existing mechanism, with
no implementation. Independent Reviewer
`/root/yt102_settings_error_research_review_oct20` PASS has zero issues or
required followups.

This source-bound research retains the original method/variant names in its
analysis and evidence below. Current canonical names are `load_with_env` and
`Error::Key`; see the [pre-public naming migration](NAMING.md).

## Recommendation

Keep the shared Loader unchanged. If implementation is selected later, expose
three fixed CLI troubleshooting groups: `source`, `profile` and `values`.
Reuse existing Loader error variants and the CLI's existing validation sites;
keep the current `Result<Settings, String>` boundary and fixed human stderr.
Private constants and one mapping helper suffice; no new shared error API,
provider, registry, logging layer, report command or JSON schema is needed.

The current behavior already discards sensitive diagnostic inputs. This is a
DX improvement proposal, not remediation of a demonstrated disclosure.
Keeping today's generic message remains acceptable if the new decision table
is sufficient; this research does not make implementation a completion gate.

## Current behavior and concrete gap

The shared [configuration Error](../crates/yetlet/src/config/mod.rs) has eight
ordinary variants: Prefix, Environment, Input, File, Toml, EnvironmentKey,
Type and Settings. Its Display strings are fixed, the enum retains no input,
and its Error implementation exposes no parser/IO/Serde source chain. Optional
secret-provider variants exist separately; the CLI public loader never enables
Development secret-file or SecretProvider resolution.

In [settings.rs](../tools/yetlet-cli/src/settings.rs), `load_from_env` discards
that distinction and most local validation failures using the same `INVALID`
string. Invalid profile spelling, missing selected TOML, unsafe public root,
unknown override and wrong type therefore offer the same troubleshooting hint.
The [existing CLI cases](../tools/yetlet-cli/src/settings.rs) assert that shared
message for several distinct failures. The existing fixed browser-capability
message is already more specific and need not change.

The [application dispatcher](../tools/yetlet-cli/src/app.rs) maps settings
failure to exit 1; [main](../tools/yetlet-cli/src/main.rs) wraps it with the
existing action/prefix and writes stderr. This is separate from usage exit 2,
process failure 125 and native child status. The helper is also consumed by
Kubernetes generation in [lib.rs](../tools/yetlet-cli/src/lib.rs) and explicit
profile inspection in [docker.rs](../tools/yetlet-cli/src/docker.rs), reused by
the generator workflow. A later change must account for those human-message
wrappers as well; it is not exclusively a run/build change.

## Smallest proposed mapping

The names below are proposed human hints, not new machine protocol codes.
Only errors currently collapsed by `load_from_env` are candidates for change.

| Group | Existing evidence to use | Fixed troubleshooting hint |
|---|---|---|
| `source` | Public-root selection/protection/listing, required public-file checks, explicit-root or value overrides without settings files; Loader Error::File | Check the configured public directory and required base/profile files, including safe ownership/readability |
| `profile` | Existing finite APP_ENVIRONMENT selector; Loader Error::Environment | Select exactly Development, Staging or Production; absent selection means Production |
| `values` | Loader Error::Input, Toml, EnvironmentKey, Type or Settings; CLI Serde projection and public CLI/TLS value validation | Check public TOML syntax, base-declared keys/types, ranges and target/TLS requirements |

Unexpected/unmapped errors retain the current generic fixed message. Prefix
is not an operator choice here: this caller fixes it to APP. Do not turn an
unreachable prefix error or optional provider variant into a guessed user fix.
Keep the existing specific browser-capability message. Later `Tls::paths`
checks, asset errors, manifest preflight and child diagnostics are outside this
first proposal; do not reclassify them merely because they share a helper or
message constant.

These groups express where to investigate, not an exact cause. Error::File
does not distinguish a missing file from denied/unsafe access or a size check;
it also does not identify base versus profile. Error::Input covers more than
one source of invalid/oversized input. The CLI can recognize its own public
root/value guards without further probing, but must not claim a more precise
filesystem/parser cause from a discarded String.

Preserve current validation order: the CLI pins/inspects its public root before
its profile match, then checks required files and calls the shared Loader.
The Loader separately validates its selector before reading files. Do not
reorder either path, retry file reads, weaken protection or fall back to another
profile to improve an error message.

## Safety boundary and tradeoffs

All hints must be literal text selected by trusted branches/error variants.
Existing fixed names such as base.toml and the finite profile names are safe;
do not echo supplied values, unknown key names, configured paths, TOML excerpts,
Serde/parser diagnostics, Debug of loaded settings, ENV dumps or source chains.
Do not add a verbose mode that forwards them. Keep exit 1 and existing caller
channels/wrappers, redaction, source selection and secret isolation unchanged.
Classification does not start a tool, inspect a private secret or validate a
remote credential. Existing application/runtime errors remain application-owned.

| Option | Benefit | Cost / decision |
|---|---|---|
| Keep generic INVALID and use the decision table | No code or wording change; current safe behavior remains | Least work, but distinct causes still share one hint; acceptable to defer |
| Forward existing safe Error::Display only | Reuses already redacted Loader text | Safe for that enum today, but leaves CLI root/file prechecks and Serde/policy guards generic; does not cover the whole gap |
| Three private fixed groups | Consistent next action across existing CLI guards and Loader errors | Recommended optional next slice; deliberately coarse, with no new public type or caller signature |

Do not expose a raw upstream error as a shortcut. Unlike the shared Error,
parser, IO and Serde errors can contain supplied content. There is no need to
change the shared Loader or introduce a second settings mechanism to fix this
CLI presentation gap.

## Minimum verification if implementation is selected

Reuse existing fixtures in [CLI settings tests](../tools/yetlet-cli/src/settings.rs)
and [Loader tests](../crates/yetlet-config/tests/loading.rs); avoid a new harness.

| Case | Required observation |
|---|---|
| Invalid selector, including wrong case | Fixed profile hint, failure before child startup; no supplied selector echoed |
| Unsafe/invalid public root or missing selected file | Fixed source hint without guessing NotFound versus permissions; existing protection and ordering retained |
| Representative malformed TOML, unknown/type-invalid override and CLI projection/range guard | Values hint from the actual typed/branch evidence; no input or parser excerpt in output |
| Redaction and unchanged consumers | Distinct synthetic key/value/path sentinels absent from fixed output; representative actual CLI failure remains exit 1/stderr; existing generator rejection still produces no output tree |
| Valid and deliberately specific cases | Existing precedence, legacy defaults, watch/target behavior and fixed browser refusal remain intact; Development secret source is still not read |

These are a bounded future verification plan, not tests written or executed
by this research. Existing native/Windows/browser/performance evidence remains
accepted; no new runtime qualification is claimed. Source and existing test
inspection support this proposal. `cargo xtask check docs`, table-rendering and
`git diff --check` pass; independent review confirms research-only closure.
Whether to implement the three groups is the remaining owner decision.

## Selected CLI implementation (2026-10-20)

The research above describes its original `e691dca` snapshot. After independent
review, research commit `b07e630` records that decision input. The owner then
authorized the recommended minimum: three private literal hints and one mapping
helper in [settings.rs](../tools/yetlet-cli/src/settings.rs). Only the previously
generic `load_from_env` failures change. The shared Loader, String boundary,
filesystem protections, validation order and consumers remain unchanged.
The [CLI guide](../tools/yetlet-cli/README.md#diagnostics-and-platform-evidence)
describes the actual groups; they are human guidance, not machine codes.

Every changed error branch selects literal text, discarding its input/error.
Source hints do not identify a failed file or distinguish missing from unsafe
access. Generic fallback, the specific browser-capability refusal and later
TLS-file error text remain. Settings failures still use exit 1 and existing
stderr wrappers; usage, process-management and native child statuses are not
remapped. CLI tool policy still ignores backend overrides and Development
secret selectors; no new files, tools, retries or source probes are added.

Verification on Ubuntu uses the existing CLI settings, frontend-process,
Kubernetes and command fixtures. Cargo commands below use the CLI's existing
prepared static OpenSSL installation; no dependency or tool installation was
performed. All Cargo commands select `--offline --locked` and
`--manifest-path tools/yetlet-cli/Cargo.toml`. Tests use `--test-threads=1`
after the test-argument separator, alongside `--exact` where shown:

| Command selection | Result / observation |
|---|---|
| `cargo test --lib settings::tests` | 8 pass: precedence/defaults, public/secret isolation, typed/projection/range/TLS failures, source-before-profile order, non-Unicode selector and generic fallback; exact browser and later TLS-file messages retained |
| `cargo test --test frontend_process settings_hints_keep_exit_stderr_redaction_and_preflight_without_children -- --exact` | 1 pass, covering 18 actual CLI invocations: run/watch/build across invalid root, missing selected file, wrong-case profile, malformed TOML, unknown key and invalid type; exact fixed stderr, exit 1, empty stdout, unchanged base bytes and no child trace/target |
| `cargo test --test frontend_process watch_forces_rust_watch_after_settings_validation_without_changing_run -- --exact` | 1 pass: existing run/watch behavior and child cleanup, rejection before forced-watch startup |
| `cargo test --test k8s settings_rejection_keeps_generator_exit_wrapper_and_no_output_tree -- --exact` | 1 pass: out-of-range replicas retain exit 1 and generator stderr wrapper without creating the destination |
| `cargo test --test commands` | 4 pass: discovery/usage, doctor, real frozen Cargo build/run/native exit, lock/cache refusal and adopted-child cleanup |

The first settings test run had one fixture-only failure: its strengthened
browser-specific assertion encountered the existing earlier rejection of an
undeclared Rust watch key. Declaring watch=false in that fixture's base makes
the intended specific refusal reachable; the single focused rerun passed all
8 settings tests. No production change was needed for that failure.

Synthetic key/value/path/TOML markers and temporary absolute paths cannot enter
the exact-output assertions. These tests prove the bounded CLI presentation
and rejection behavior; they do not promise redaction of application/child
output or exact filesystem diagnosis. Existing Windows/browser/performance
evidence is retained, not rerun for this message-only slice.

`cargo clippy` for the CLI library/binary and changed integration targets with
`-D warnings`, `cargo fmt --check`, `cargo xtask check docs`, rendered-table
structure and `git diff --check` pass. Independent Reviewer
`/root/yt102_settings_error_impl_review_oct20` PASS has zero issues or required
followups after inspecting the diff, Loader and consumer boundaries. Native
test results are parent-provided session evidence, not independently rerun by
the reviewer. This bounded implementation is complete.
