# Yetlet CLI 0.3.0 — Release preparation (YT-248)

**State (2026-11-10): Owner authorized YT-248 candidate preparation and Git delivery. Local CLI 0.3.0 archive installation, resolver/Style consumers and Chrome verification PASS; corrective independent implementation Reviewer PASS has zero issues/followups; Git/CI delivery follows the [candidate record](YT-248-CLI030-CANDIDATE.md). Formal release execution and publication remain separate.**

## YT-247 / YT-248 sequencing amendment

The Owner redefines [YT-247](YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md) as
Dynamic SDK Version Resolution. Phase A produces the proposed CLI contract and
[ADR 0010](adr/0010-dynamic-sdk-version-resolution.md), independent review and
Git delivery, then stops for Owner acceptance. Phase B implements the accepted
resolver. The Owner has accepted that design and authorized the
[Phase B implementation](YT-247-SDK-VERSION-RESOLUTION-IMPLEMENTATION.md).
This release plan moves to **YT-248** and depends on that implementation.
The later Owner instruction authorizes this candidate-preparation slice and its
reviewed Git delivery, with no publication or Reader Edition upgrade.
The [later Windows supplement](YT-247-WINDOWS-VERIFICATION.md) qualifies the
Resolver/new/lock path only; the separate Windows Style runtime pause below
retains its original scope.

The original Style acceptance below remains required. Its corrective independent
Reviewer result reports JSON PASS, zero issues/followups in
`correction-reviewer-result.json` under the retained external preparation
`yt246-alpha7-qualification-20261109/yt247-cli030-plan-review/` evidence directory.
That prior role-directed Job review used saved Reviewer configuration; it is not
native managed-role trace evidence and does not review the new resolver design.
Original failures and both review results remain preserved. The directory's old
YT-247 name stays historical; do not rename or regenerate it.

The alpha.7 pin below now identifies the CLI **build SDK**, and the closed alpha.7
snapshot identifies the source-export baseline. Generated app versions instead
come from the accepted resolver policy, explicitly passed to scaffold and locked
per new project. Resource equality still applies to the selected build/snapshot
inputs; it must not constrain every generated app to that one SDK version.
This amendment supersedes any original single-generated-pin implication below.


## Purpose and immutable prerequisites

Deliver one independent source-installable CLI package that makes the existing application-owned Style templates and build/watch integration available as a supported CLI release. This is a separate **schema3 / `cli`** cohort with exactly one subject, `yetlet-cli`; it is not a third package in the two-package SDK cohort.

The SDK prerequisite is satisfied: [Yetlet SDK 0.9.0-alpha.7](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.7) is already owner-approved and published as the two crates `yetlet` and `yetlet-browser`. Its immutable source C/tag and package bytes are not to be modified or republished.

The currently published CLI remains **0.2.0**, deliberately pinned to SDK `=0.9.0-alpha.6`. Historical CLI 0.1.0/0.1.1/0.2.0 bindings and their records must remain unchanged. The public Reader Edition remains SDK alpha.6 / CLI 0.2.0; CLI 0.3.0 does not promote the site, first-app guide, docs-site registry manifest or existing tutorial pins.

## Selected CLI 0.3.0 contract

- Package: exactly `yetlet-cli`, package version `0.3.0`, independent build-time registry SDK dependency exactly `yetlet = "=0.9.0-alpha.7"`; no `path`, `patch`, `replace`, git dependency or unrelated third-party dependency refresh. Generated app selection follows the accepted YT-247 compatibility policy; it is not an arbitrary compatibility override.
- Release identity: schema `3`, cohort `cli`, exactly one package subject, immutable tag `yetlet-cli-v0.3.0`. The SDK is a consumed dependency, never a third CLI-release subject.
- Include the existing `new --template native-style` and `new --template browser-style` options in the actual packaged CLI and its published command inventory. Preserve `native` as the default and ordinary CSS as the default behavior.
- Include the already implemented finite app-owned Style generator, explicit style registration, external build-time CSS output and native/browser build/watch sequencing. Tailwind/Vite composition and existing ownership/recovery behavior remain intact.
- Do not add a public Style crate, implicit runtime CSS, inline styles, renderer/CSP API, automatic package installation, new runtime API or Python dependency.
- Refresh the CLI's closed package-owned SDK snapshot only after verifying the SDK alpha.7 registry package is available. Run the canonical `tools/yetlet-cli/scripts/sync_resources.py` from the trusted reviewed checkout to regenerate `resources/framework.json`, `resources/workspace.toml`, `resources/framework.lock` and the existing browser startup assets. Its exact source correspondence and package inclusion must be tested.
- Update the CLI package README, release-neutral installation/version instructions, command inventory and style-template section for CLI 0.3.0. Do not describe Style templates as absent from the 0.3.0 package. Current-publication documentation remains CLI 0.2.0 until actual owner-approved publication receipts exist.

## Trusted version and resource bindings

Extend the closed `scripts/release_record.py` CLI-to-SDK binding with exactly `0.3.0 -> 0.9.0-alpha.7`; preserve `0.1.0/0.1.1 -> alpha.5` and `0.2.0 -> alpha.6`, and reject unknown CLI versions or wrong build SDK pins before installation/publication. Preserve the historical generated-pin checks for old CLI releases; update only the new CLI consumer path to validate resolver results separately.

Update release documentation checks and their tests so:
- CLI 0.2.0's published command inventory continues to exclude `native-style` / `browser-style`.
- The selected CLI 0.3.0 candidate and its eventual published inventory include both style selectors and accurately describe the supported output/build/watch behavior.
- The package README, exact CLI build dependency, release binding, source snapshot and package archive agree on build/source provenance. Generated-app exact/default choices, manifests, own Cargo.lock and resolution receipts agree with the shipped compatibility policy rather than being forced to the CLI build dependency.
- The source-only checkout map and current-publication status distinguish the candidate source from the still-published CLI 0.2.0. After actual publication, reconcile current documentation from the accepted CLI 0.3.0 record and keep the separate Reader Edition unchanged.

The existing `release.yml`, schema3/cli signing, independent verification and publication automation is the intended pipeline. Correct any stale workflow input descriptions that omit schema3/cli, and add focused regression coverage for the existing schema3 path; do not create a second publisher or weaken its closed cohort, source, tag, Environment, checksum, signing or owner-approval gates.

## Required local and hosted consumer evidence

- Exact Rust 1.99 locked CLI tests, YT-247 fixture/cache/locking tests and release-interface/resource tests; real binary `--version`, help and all four template selectors.
- Resource snapshot equality against tracked SDK alpha.7 source, root workspace and lock, plus browser startup asset equality. Confirm the archive includes all `templates/styles/**`, style generator source, README, resource snapshot and tests intended by the package manifest.
- Build the CLI package/archive and inspect its Cargo metadata and exact version/SDK pin. Verify its checksum and retained package manifest against the immutable candidate source.
- Exercise the installed registry CLI with a fresh isolated Cargo home and **without** a crates.io publish token, source path override, patch or replacement. Verify exact `yetlet 0.3.0` version output, help and successful `new` for default/native/browser/native-style/browser-style.
- Add a fixed explicit alpha.7 installed-consumer case and a separate default case validated against captured refreshed metadata. Check exact SDK pins, generated app lock/checksums, receipt source and offline behavior; never assert a live default is permanently alpha.7. No live registry dependency in normal tests.
- For the two Style templates, inspect generated manifests and the complete stylegen dependency/resource closure; build and **run each generated app through the installed CLI**. Native: verify a real loopback SSR response, its stylesheet link, fetchable generated CSS and expected class/content. Browser: verify the Vite/WASM production build, start the CLI's static preview, fetch the real index and emitted CSS/WASM assets, and check the expected Style output. Use bounded ephemeral ports/readiness deadlines and verify the actual process exit/termination; clean only owned children and temporary roots in `finally` paths. A successful scaffold or build without runtime/CSS evidence is not a completed consumer test.
- At local Linux qualification, run a bounded Chrome smoke against the generated browser Style app to verify computed styles and CSP/console behavior. The installed Registry Consumer receipt must retain both native and browser runtime evidence without forwarding credentials. Keep existing strict CSP, external CSS, output ownership, hashes and failure/cleanup evidence.
- Use the normal PR exact-head Rust/MSRV, Documentation and CodeQL checks, then all required post-merge master CI. Run the actual hosted build, protected artifact/evidence readback, signer-source delivery, signing, independent V, exact five-file public allowlist, one-package registry plan and registry-only installed consumer.
- Independent Reviewer PASS and Strict/Builtin Commit Guard are required for implementation and release automation changes. Preserve original failures and do not retry a failed signing/upload/publication job automatically.

## Platform and release boundaries

The existing CLI 0.2.0, YT-239/243 Style, Linux browser, Windows MSVC and SDK alpha.7 evidence may be reused only where the source and tested behavior are identical. **New Windows Style-template revalidation remains paused/unverified**; do not convert older Windows evidence into a claim that the new Style paths were tested on Windows. State this limitation in the final release record and notes unless a separately authorized Windows validation is completed.

The release process must stop at the protected `release-publish` Environment until the owner explicitly approves. Do not publish, approve or substitute a registry token on the owner's behalf. After approval, verify the actual GitHub pre-release, exact single CLI crate/index/archive digest and a fresh credential-free registry installation/consumer. Only then update the accepted CLI publication authority and current-publication documentation.

**Explicitly excluded:** SDK republishing or tag changes; a third SDK package; reader-edition/Pages promotion; CLI features beyond the accepted YT-247 resolver and already implemented Style templates and their existing integration; Windows revalidation while paused; production/HA certification; release token/OIDC policy changes; automatic release retries.
