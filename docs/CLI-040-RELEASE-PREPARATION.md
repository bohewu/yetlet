# Yetlet CLI 0.4.0 release preparation

Date: 2026-11-12. Scope: one Experimental `yetlet-cli` package for YT-259.
CLI 0.4.0 is the local release candidate. The public CLI and Reader Edition
continue to select CLI 0.3.1 and SDK 0.9.0-alpha.8.

## Selected behavior

- The exact build SDK and embedded SDK snapshot remain at alpha.7. Generated
  apps choose their registry SDK independently.
- The M26 capability row matches published SDK `0.9.0-alpha.8` and the
  `yetlet` archive SHA-256
  `3e850fe4320f3511121b37cfa25bb2ce3ef422d5fe11ccbea59908687912fd59`.
  [Accepted SDK publication](ALPHA8-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-12)
- Native and Native Style select `App::serve()` with that capability and the
  compatible flag set. Alpha.7 keeps the legacy listener. Settings, Vite,
  Service and Browser retain their documented startup profiles.
  [Scaffold contract](YT-259-CLI-NEW-STARTUP-CONTRACT.md)

## Activation proof

Before activating the source row, an isolated CLI build with the injected
published policy generated two fresh registry-only Native consumers. The
exact alpha.8 lock selected the verified public crate and `m26-app-serve`.
The alpha.7 lock selected its own published checksum and `legacy-axum`.
Both generated projects passed locked compilation, preserving their lock and
resolution receipt. There were no Cargo path overrides or source replacements.

## Packaged candidate

The reviewed source snapshot passed normal locked Cargo packaging. The
`yetlet-cli-0.4.0.crate` archive has 535492 bytes and SHA-256
`5639c8b34f6aab810269031978d74550a8be2516109dedc84f3ff8336099c36e`.
The archive includes the capability policy and the unchanged alpha.7 SDK
snapshot. The exact archive was installed on Linux and Windows MSVC.

- Linux qualification covered Default, Native, Browser, Native Style, Browser
  Style and Offline. Runtime checks covered Native Style HTTP, CSS and CSP
  and Browser Style Vite, Wasm and Chrome. Additional locked checks covered
  alpha.7 Native and Native Style and alpha.8 Settings, Service, Tailwind,
  Vite and Browser.
- Windows MSVC locked checks covered alpha.8 Native, Native Style and Service,
  and alpha.7 Native. The cases verified the registry SDK and startup profile
  and preserved the generated lockfiles.

The next checkpoint is source PR and CI. Immutable source selection,
hosted build, signing, publication and the later Reader Edition update
follow the [release SOP](RELEASING.md).
