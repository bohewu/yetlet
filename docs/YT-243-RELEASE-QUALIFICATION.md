# YT-243 — Source release qualification and existing CI coverage

Date: 2026-11-08. Status: **Done at source/Linux/local Windows MSVC qualification scope after independent PASS**.
Baseline: `b1b8fa08c70269cd6083b933c6a200895829ea8d`, current checkout
`feature/style-css-roadmap`. YT-242 is independently accepted and committed.
This task selects no candidate, version, release, hosted dispatch or production
assurance. Parent owns independent review and final acceptance; Windows execution
below supplements, rather than rewrites, the original Linux implementation record.

## Requirement and evidence matrix

Evidence root: `artifacts/verification/yt243-release-qualification-20261108/`.
`YT-243-IMPLEMENTATION-HANDOFF.md` binds commands/exits, source/package hashes,
consumer selection, retained failures and cleanup. Existing
[YT-239](YT-239-STYLE-INTEGRATION.md), [YT-240](YT-240-TODO-STYLE-QUALIFICATION.md)
and [YT-241](../examples/style-showcase/README.md) proof retains its original scope.

| Requirement | Actual implementation / evidence | Result and limit |
|---|---|---|
| Existing CI integration | Both MSRV/stable wasm32 lists include the actual Showcase browser companion. Existing Chrome/Firefox release lanes build browser assets before the native release binary, run Showcase smoke and retain diagnostics. Smoke evidence uses the ordinary verification root rather than a dated YT-241 folder | Installed actionlint1.7.12 PASS; existing permissions, pins, concurrency, timeouts and cache policy unchanged. Hosted CI not run |
| Independent CLI workspace | Stable lane explicitly runs CLI library and selected scaffold/process/release-interface tests. Scheduled native lane adds Showcase tests and supported Linux/Windows Style units/scaffolds | Source CLI library 107 PASS, one existing ignored, no filters; integrations 18/3/2 PASS with two existing scaffold cases ignored. No full legacy CLI-suite claim |
| Native/browser source checks | Showcase native tests4, native and wasm32 strict Clippy, actual browser companion MSRV check, existing builders7 and smoke-protocol10, xtask strict Clippy | PASS on Linux Rust1.99.0; stable alias is the same compiler. Earlier unchanged debug proof reused |
| New release-profile browser scope | Chrome157.0.8103.38 and Firefox159.0, actual release WASM/native binaries; no-JS HTML, MIME/nosniff/exact CSP, one stylesheet, computed style/selection, disclosure/focus/source containment | Both smoke commands PASS. Observed Chrome390/768/1280; Firefox500/768/1280, requested390 clamped. No Firefox390 PASS |
| Firefox disposal gap | Existing BiDi transport imports into the actual page realm initialized by loader.js; enhancement removal and unclaimed native-link default with unchanged selection asserted | PASS in both engines; browser product behavior unchanged. Full Firefox console/CSP logging remains unverified |
| Retained cloud-lock failures | Initial unfiltered library run reproduces two failures (105 PASS, one ignored). Config fixtures now derive the SDK's activated dependency closure from the canonical framework lock | Focused Docker16 and subsequent unfiltered library107 PASS; production lock guard unchanged; original failures preserved |
| Normal SDK/CLI packaging | Release-doc gate first; existing normal two-SDK-package recipe stages the verified base archive/checksum before Browser verification; separate CLI recipe normally verifies its archive | PASS. Initial CLI archive verification fails because nested templates/styles/Cargo.toml hides that directory from Cargo. Stored template renamed manifest.toml, emitted stylegen/Cargo.toml unchanged; corrected155-file archive verifies |
| Exact external consumers | SDK native typed-Tag execution and wasm32 check; archive-built CLI emits both Style templates. Generated native app builds and serves no-JS HTML/checksum-matched CSS; browser app builds release WASM and one CSS bundle with conditional registered styles | PASS with directory sources and lock checksums selecting the exact local SDK archives, no accidental registry alpha.6. Generated apps remain standalone, outside SDK workspace membership |
| Existing repository gates | Release regression suite93 (one existing skip), docs/DAG, thirteen-sample catalog, root/CLI/xtask formatting and whitespace | PASS; installed tools only. No broad legacy/platform replay |
| Owned resources | Browser/server exit receipts, removed browser profiles, closed listener ports and PID absence; generated-native HTTP server reaped | PASS. No active or uncertain task resource remains |
| Windows MSVC source | Exact transferred execution inputs, Rust1.99.0: View14, typed-tag doctests5, Showcase4; accepted Style units6, cloud fixtures13, Style scaffold1, Windows creation3/build6 and release-interface2 | PASS at local MSVC scope; targeted groups are not a disjoint total or a complete Windows suite. Existing ignored cases retained |
| Windows exact packages | Native/WASM typed consumer selects the same new SDK archives; Showcase wasm32 check and archive-built CLI compile pass; both Style templates created with unchanged emitted manifest; native Style build and actual HTTP/CSS pass | Same CSS SHA256 as Linux; exact DEFAULT_CSP, zero scripts/inline styles, one CSS link and CSS MIME/nosniff. Owned server/console processes absent and port closed |
| Reviewed CI prerequisite correction | First independent review found missing Windows OpenSSL configuration. Existing workflow now validates image MSVC headers/libraries and handles DLL lookup for restricted-PATH children; explicit prepared static prefix is available for local proof | Exact final setup body and its two Style commands run on Windows: 6+1 PASS. Hosted dynamic-image branch remains unexecuted; no installation or global ACL/environment change |
| Independent acceptance | Original source/Linux review and one-P2 finding retained; final corrective Reviewer `wc_job_ukE1zeoMQlt3wLvu` inspected current source, CI and reconciled Windows receipts | PASS, zero issues/followups; all11 paths, 397 execution inputs, three package hashes and 58 Windows receipts match. Parent acceptance is bounded to this report, not release/hosted qualification |

## Package identity and retained failures

These local source-preflight bytes keep the already published SDKalpha.6 and
CLI0.2.0 versions. They are **not releasable replacement archives**, registry
proof or selected C. SHA256:

| Local archive | SHA256 |
|---|---|
| yetlet-0.9.0-alpha.6.crate | fb9fc6b727279ae18a5b1ae2f7d9f2131c71050a2f8c7c655fa5c8c5fc905e58 |
| yetlet-browser-0.9.0-alpha.6.crate | 885669fed852b77b9860e807bab6d7a7e18b8a466789cea73bf3689593eab388 |
| yetlet-cli-0.2.0.crate | 029419a39c47af4b7141cd6cc384c600224f5acd315b533566146e7cbc6a6491 |

Normal CLI package verification uses its declared published SDK dependency;
generated app/typed consumers use the exact newly packaged SDK directory sources.
The consumer renderer-Result omission and task-cache permission setup failures
received one bounded correction each. An extra direct unit-test attempt against
the staged CLI vendor entry encounters parent-workspace discovery; it is retained
as a harness limitation, not an archive-unit-suite PASS or product defect.
Source unit/integration tests, normal archive verification and archive-built CLI
consumer proof pass separately. Existing dead-code/generated unused-item warnings
remain; no warning suppression or test-filter waiver is introduced.

## Windows execution and retained setup corrections

The installed stable MSVC alias reports the same complete compiler identity as
installed Rust1.99.0; its already-installed wasm32 target supplies the missing
named-toolchain target without an installation. Normal archive-derived directory
sources and lock checksums bind the three package hashes above. All transferred
execution-source hashes still match the checkout; the later delta is CI/docs only.

The source CLI/default Cargo tests and packaged CLI use an existing prepared
OpenSSL3.5.9 static prefix. The exact final CI setup explicitly selects that
prefix and verifies x64 archives with no DLL imports before static linking.
Its hosted default still selects the image's dynamic import libraries and puts
matching DLLs beside the CLI/test binaries so restricted child PATH does not
remove runtime availability. Local static proof does not attest the hosted image.

Generated native Style execution additionally requires a real processed console
and trusted compiler/Cargo files. The normal installed compiler had an untrusted
writable ACL and was correctly refused. A task-owned owner-only copy of its
existing binary/library sysroot was byte-verified; no original ACL was changed.
An unnecessary absolute RUSTC task override was then refused by the existing
Windows executable allowlist. Using the normal vetted PATH selection succeeds;
no product allowlist was broadened. Original failed scripts/logs remain, as do
the earlier missing registry-source checksum and nested-workspace setup failures.
The working path uses the archive-built CLI, exact local SDK and unchanged app
manifest. No Windows browser window was opened; browser-style creation and
wasm32 compilation do not claim a Windows browser-bundle/runtime test.

## Remaining gates

Final independent Reviewer `wc_job_ukE1zeoMQlt3wLvu` returned PASS with zero issues
or required followups; the parent accepts the stated local scope. Next unblockable
item is the separately authorized local strict/builtin commit with normal Git hooks.
Hosted CI/image execution, Windows browser-style bundle/browser
runtime, macOS runtime, Firefox390/full console-CSP logging, Safari/mobile devices,
production, release version/C selection, signing and publication remain separate.
This qualification does not supply release authority or select source C. No Push,
tag, installation, global configuration/ACL change or cache deletion occurred.
