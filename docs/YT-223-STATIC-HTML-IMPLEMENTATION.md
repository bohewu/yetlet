# YT-223 — Minimal static export and CLI integration

Date: 2026-11-01 (run started2026-10-31). Status: In progress; implementation and
local affected evidence complete, independent local implementation review PASS0;
required MSVC execution is blocked by WebCodex artifact policy.
Baseline:9b46c306193e29bb4692df7db9f0c588d0f75824, the owner-authorized strict/
builtin commit of15 accepted YT-221/222 paths, no Push. Guard checked15 files,
tree56320e6df8c4bc5131ca3bf9271dc6998e5aa211,0 blocking findings; two PATH_UNC
warnings are negative URL test literals. Native hook passed; worktree was clean.

## Selected implementation and compatibility

Implement the independently accepted [YT-222 contract](YT-222-STATIC-HTML-CONTRACT.md)
without changing Core/SSR/browser APIs, existing settings wire or package versions.
The [CLI parser/main](../tools/yetlet-cli/src/lib.rs) adds explicit build --static,
--export-bin, --output, --base-path, --release and --wasm-islands selection.
Cli::BuildStatic and its execution method are Experimental CLI support additions;
exhaustive downstream enum matches may need another arm. Existing Build fields,
ordinary build/run/watch/new/generate and inspect schema1 retain their roles.
These flags are unreleased checkout behavior, absent from published CLI0.1.1.

[Static orchestration](../tools/yetlet-cli/src/static_export/mod.rs) validates
existing settings/native target, captures bounded source inputs, executes literal
frozen Cargo under an owned deadline and cleans up before returning completion.
The [protocol/document writer](../tools/yetlet-cli/src/static_export/protocol.rs)
validates finite explicit fragments/assets, schema/keys/routes/paths/metadata,
independent island selection and resource inventory before public construction.
It uses the existing HTML escape helper through CLI's newly selected existing
html feature; no new dependency version, SDK module/type or runtime provider.

[Scoped static filesystem operations](../tools/yetlet-cli/src/platform/fs/static_output.rs)
use existing protected directory creation and native regular-file/handle checks,
bounded inventory, source/output hashes, no-replace publication and exact owned
cleanup. Old generator allowlists are not widened. Linux reuses descriptor-
relative opens/renameat2; Windows source reuses native NTFS handles/security/
non-replacing rename. App-local output is confined below artifacts/; a separate
protected output parent is also accepted. No arbitrary force/overwrite/clean.

The existing owned foreground group/Windows Job gains a separately selected
deadline for static execution only (600 seconds plus bounded native cleanup).
Normal commands use the prior unbounded-invocation policy. Source receipts are
bounded observations and do not claim sandboxing/hermetic compilation. Publication
or final receipt/cleanup uncertainty cannot report success or delete an output.

Static preflight captures CARGO_TARGET_DIR, falling back to CARGO_BUILD_TARGET_DIR
then app/target; intermediate output uses a plain CARGO_BUILD_BUILD_DIR or that
target. Child --target-dir and the explicit build-directory environment bind
execution to those checked paths; file-config directory selection is overridden
for static mode only. Existing aliases are compared physically, and unresolved
Cargo build-path templates are refused. This follows Cargo's documented
[directory selectors and precedence](https://doc.rust-lang.org/cargo/reference/config.html#buildtarget-dir)
without introducing a second Cargo configuration parser or a preflight child.

The [standalone consumer](../examples/static-site/README.md) uses registry SDK
0.9.0-alpha.5 without path overrides, native HTML-only home/nested-guide/404 and
explicit stylesheet/PNG. Separately prepared local Wasm assets use the existing
browser build algorithm and copied startup helper. Activation mounts one local
counter, with readable static content, bounded failure/reload guidance and
explicit listener disposal. No mandatory Node/Wasm/server or Settings loader in
the HTML-only application; no SDK reissue or public-site replacement.

## New local evidence

Evidence is retained under target/yt223-20261031/. Earlier accepted suites are
reused, not replayed. This is scoped implementation evidence, not release,
production, TLS, cloud or all-sample qualification.

| Evidence | Actual result |
|---|---|
| New native unit cases |7 unique cases PASS across retained executions; fixture continuations skipped passed cases, and the affected argv case was refreshed for the material Cargo-directory fix. Parser/legacy Build, descriptor/metadata/CSP, route/resource bounds, binary/no-replace/cleanup, link refusal and literal protocol argv |
| New owned deadline case |1 PASS; separate ignored sleep helper is invoked only as its child and not counted as a passed test. Two-second deadline reports timeout and confirms child PID absent |
| CLI builds | Native offline/locked check and real binary build PASS |
| Normal CLI packaging | Normal offline/locked cargo package including verification PASS; no --no-verify. Separate source snapshots retain the initial build, supervision-error correction and reviewed Cargo-directory correction. The latest archive contains current CLI code/guide; no replacement for published CLI0.1.1 |
| Actual external consumer | Source copied outside repo, registry SDK/lock and no path overrides; HTML-only / and /yetlet/ exports PASS. The latter two distinct base cases are new behavior, not old qualification replay |
| Optional island build/export | Matching wasm-bindgen0.2.129 and prepared wasm32 release build PASS; project-prefix export with explicit --wasm-islands PASS |
| Dependency observations | Native app normal graph excludes browser/server/provider toolkits; Wasm graph excludes native server/config/signing/providers. No existing dependency lock upgraded |
| Actual failure/control cases | Existing-output refusal preserves last-good hashes; child23 remains23; added source aborts publication; invalid settings are not bypassed; actual child sees no APP_SECRET/development selector; SIGTERM returns signal identity and leaves no child/output |
| Cargo-directory regression | Previously unchecked CARGO_BUILD_TARGET_DIR and intermediate-directory overlaps refuse before private allocation/child execution; cache-alias overlap refuses. An actual config-file case proves child uses the captured target/intermediate tree instead of two different configured directories; output succeeds only outside it |
| Chrome156 | JS-disabled root/project HTML reading, escaping, image/CSS MIME, nested route/anchor/refresh and real404 PASS. Explicit activation only, actual Wasm counter2, JS/Wasm MIME/CSP, surrounding content and disposal PASS. Aborted Wasm shows failure guidance, no automatic retry |
| Firefox159 | Project-prefix direct page/anchor/text/image, actual activated Wasm counter1 and app pagehide listener disposal PASS. Passed static/island cases are reused from retained executions; no old DOM suite replay |
| Cleanup/receipt | Seven known-outcome CLI child runs report temporary cleanup complete and no public stages. One new missing-Cargo case correctly reports125, unknown process cleanup and retained private empty export; it is not counted as successful cleanup. Chrome and all four Firefox processes/profiles closed, three servers stopped/ports closed |

Packaging is local affected-source verification, not a selected immutable release
candidate or authority to publish this version.
Existing published SDK/CLI/tag/signature identities remain unchanged.

### Retained failures and limits

- The first commit-receipt inspection expected singular profile/engine fields;
  Guard actually emits arrays. Its exit0/strict/builtin/tree were verified using
  the actual schema before committing; no scan/hook was bypassed.
- Initial unit fixture creation used a permissive temporary parent. Correcting
  that parent exposed a second fixture seam: permissive child directories.
  Protected native creation corrected only those directories; first5/then2
  failures remain, and already passed cases were skipped. Product protection
  was not weakened for the fixture.
- Offline consumer lock preparation initially lacked the registry browser
  package; explicit online lock/cache preparation completed before frozen
  executions. This is not an implicit CLI install or replay of a passed build.
- Initial strict Clippy failed on three unchanged CLI legacy locations and one
  new collapsible-if. The new location was fixed; subsequent lint observation
  has only single_element_loop in docker.rs/framework.rs and iter_kv_map in
  framework_source.rs. No ignore or unrelated refactor was added; whole-CLI
  strict Clippy is not claimed PASS.
- Firefox's first harness used dictionary property access in Python before any
  case; one key-access correction advanced to static PASS. Its next diagnostic
  passed statements to an expression-only WebDriver helper; one expression
  correction advanced to island PASS. Finally, direct dynamic import through
  the WebDriver execution context did not verify page-owned module disposal.
  That check was corrected to dispatch the application's actual pagehide event;
  listener disposal passes. Each distinct seam was corrected once, earlier
  product passes were reused, and all scripts/failures/cleanup/logs remain.
- Implementation inspection found that an unknown supervision outcome must not
  be followed by deleting private export files or a false cleanup claim. The
  error branch now retains them and records its bounded error kind. A new actual
  missing-Cargo case passes with125/no public output and process_cleanup_verified
  false. Earlier positive cases use the unchanged verified-completion branch.
- Initial independent implementation review returned P2: only CARGO_TARGET_DIR
  was checked while the child could use another directory selector/configuration.
  The narrow binding fix above and new actual directory regressions close the
  demonstrated path; the affected literal-command test was updated. Its initial
  call-site argument-order compile failure and correction are retained. Earlier
  runtime/browser cases were not replayed; independent followup PASS0 closes it.

## Required Windows gate and authority

WebCodex sees the connected Windows Yetlet Project, but its listed default was
GNU1.88 and installed GNU1.99. A cargo +1.99.0-x86_64-pc-windows-msvc version
query unexpectedly caused rustup to install that missing toolchain. No claim
is made that VC++/SDK/runtime validation passed; later commands must set
RUSTUP_AUTO_INSTALL=0. This side effect is retained as an operational observation,
not silently represented as a previously configured environment.

A bounded fleet lookup then found the previously prepared Windows localtasks
Runner used by [the original native tooling record](WINDOWS-CLI-125-VERIFICATION.md#reuse-the-prepared-runner-tools).
It already has Rust1.99 MSVC, VS2022 VC tools and SDK26100 (plus older SDKs).
A tiny new temporary compile/run readiness probe prints MSVC_READY and its
temporary directory is removed. The retained OpenSSL header/library prefix is
present and the protected Cargo hash still matches that record. No installation
or global configuration change occurred on this prepared Runner. This removes
the environment blocker without replaying old SDK/product qualification, but
does not test the changed YT-223 source.

The sole official transfer attempt for a source snapshot ZIP was refused
as source_read_failed / policy_rejected (sensitive artifact path). No destination
source was transferred. Do not move/rename/re-encode it or switch transports to
evade that refusal. The current no-Push instruction remains binding. Required
MSVC new process/filesystem cases and current-source compilation therefore remain
Not verified. Completing them needs a formal artifact-policy correction or a
separately authorized, reviewed Git delivery; neither is selected by this item.

Independent review must distinguish local implementation/evidence from this
unmet platform gate. **YT-223 remains In progress**; no Done, new Commit/Push,
hosted job, tag, signing, publication or Pages change is authorized. YT-224
remains Todo.

## Independent local acceptance and next delivery boundary

The first review FAIL/P2 and its46 matching inputs remain preserved. Scoped
followup is **PASS0**, zero issues/required followups: all46 current/followup
input hashes match, the original46-source snapshot is unchanged, and19 current
package source/guide bindings match. Only five inputs changed for the material
fix and documentation. Registered reviewer model gpt-6-astra and effective high
effort are verified. The reviewer reran no validation and made no source edits.

This accepts local implementation readiness for separately authorized Git
delivery; it explicitly does not close the MSVC gate or permit Done. All36 new
paths remain unstaged/uncommitted. Read-only remote observation confirms master
64d10742d62059c91dd2e46931aab1cb3b65d897. The current local master additionally
retains five accepted unpushed commits (YT-218 closeout and YT-219–222). A future
delivery including this item would therefore expose53 combined changed paths,
not merely this item's36 paths. The exact path/commit inventory and proposed
Windows handoff are retained in target/yt223-20261031/.

## Authorized delivery and first platform attempt (2026-11-01)

The owner authorized the36-path implementation commit and delivery together with
the five earlier unpushed commits. Commit6b42803 and its six-commit outgoing range
passed strict/builtin Guard and normal hooks, with zero blocking findings.
[PR44](https://github.com/bohewu/yetlet/pull/44) preserves the existing history;
its accepted remote-master boundary is64d10742d62059c91dd2e46931aab1cb3b65d897.
No Merge or publication is authorized.

The prepared Windows Runner cloned that exact Git source successfully. Its first
MSVC test compilation failed before executing tests: four old Windows console
fixtures omitted the new optional budget argument to run_owned. The smallest
compatibility correction explicitly passes None, preserving their prior unbounded
behavior. The CLI binary build/runtime steps had not started. Fetch succeeded;
it is not repeated by the followup.

The first PR documentation build failed because one new multiline Markdown link
escaped the existing line-based rewrite. Keeping that link on one line fixes the
actual local eight-chapter build without changing the publisher. Stable CI stopped
on rustfmt differences in the earlier Todo controls example; formatting that file
now satisfies the workspace format check. The first MSRV job passed. These results
are retained independently; no failed job or unchanged source was automatically
retried. New-source MSVC/runtime verification, CI and independent acceptance remain
pending. Receipts are under target/yt223-20261101-delivery/.

The source repair23fb5bc passes the documentation CI and MSRV job. On that exact
source, Windows compiles and passes all seven static-export parser/filesystem/
protocol tests; the ignored helper is not counted. The sole new deadline fixture
fails before starting its child because it calls the Cargo-only cwd admission
with an absolute libtest helper. Its one scoped fixture correction uses the
existing private Windows native test seam and inherited-DACL byte-copy pattern.
The product admission policy and GNU execution stay unchanged. Source review
passes with zero findings; only that unpassed Windows case remains to execute,
followed by the previously unstarted binary/static CLI runtime checks.
