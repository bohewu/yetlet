# YT-080 — Experimental Candidate Qualification

**Date:** 2026-10-14. **Status: Done at the selected Experimental opt-in scope.**
Current-source CPU/API/WASM/dependency checks, exact runtime-byte correspondence
and the distinct default-profile browser fallback check **PASS**. YT-079's
actual opt-in candidate browser and independent API/dependency/security review
remain accepted. This item's [separate independent Reviewer](YT-080-INDEPENDENT-REVIEW.md)
also returned **PASS**, zero issues and zero required followups. The completed
Job and all 37 reviewed source/document hashes were recovered and verified
before these final disposition updates; no browser or reviewer was rerun.

## Scope and authority

The owner explicitly authorized completion through YT-080, with each item
independently reviewed, and approved YT-079 as Experimental opt-in without a
performance guarantee. [The decision](M11-G-EXPERIMENTAL-PROMOTION-DECISION.md)
supersedes the earlier speedup prerequisite only for this Experimental candidate.
Future 3D remains motivation, not implementation scope. No publication, push,
additional commit, host change or detached worktree is authorized.

The first local commit already completed as
`8c524520256e5fd9884d4d023dbdfbd76181c561`, after explicit strict/builtin
Commit Guard exit 0 over 27 files. The scanned index and resulting commit tree
were both `24a43ed715030c25c9244aeaee3fc96e6157797c`; no push occurred.
Subsequent YT-078/079/080 work remains in this original Ubuntu checkout.

## Existing real candidate acceptance

[YT-079 actual candidate qualification](YT-079-VERIFICATION.md) and its
[independent review](YT-079-INDEPENDENT-REVIEW.md) are complete. The selected
Firefox 158 X11/Intel Vulkan opt-in receipt at
`target/m11-gpu/yt079-20261014/browser/page-module-hook/result.json` has hash
`b0af8d7c0526b2c35b53801aadd89cd77d9f8b2aa3323d1627ce1eaa262ee253`.
It contains 65 bracketed actions and five 120/120 pixel proofs for the actual
extracted crate, native SSR and Wasm consumer, plus data updates, picking,
controlled loss/recovery, late completion cleanup and semantic fallbacks.
The independent reviewer verified all 34 inputs and all five PNGs.

This item did not rerun that positive browser journey solely for duplicate
proof. After the crate-level rustdoc update, the consumer was rebuilt. All
**seven generated JS/WASM/loader assets are byte-identical** to the accepted
YT-079 inputs, as is the native server binary. The scene digest remains
`68a16c64a18ef9d1` (287,094 Wasm bytes); counter remains `81c388455a88d98a`
(151,613 Wasm bytes). The non-crate-doc Rust source body is unchanged. The only
changed prior inputs are the crate README, two crate-level rustdoc lines and
manifest source-hash metadata; no runtime logic, ABI or asset bytes changed.

The exact comparison is in
`target/m11-gpu/yt080-20261014/runtime-correspondence.json`, alongside
`pre-qualification.json`. The current manifest hashes current source while the
positive YT-079 receipt retains its historical source identity. This explicit
byte correspondence supports reuse of the actual positive runtime result; it
does not rewrite the prior receipt or claim the source files are identical.

## Final changes and checks

The crate README, crate-level rustdoc, consumer README and documentation entry
points identify the Experimental opt-in status and link the selected
[support boundary](../crates/yetlet-browser/guides/wgpu-support.md). No runtime logic, WGSL,
API signature, limit, dependency pin, scheduler or default feature changes were
made in this item.

## Actual default-profile browser observation

One distinct Firefox 158 headful X11 test used a fresh WebDriver profile with
an empty requested preference map: **no WebGPU preference override**. Automation
may apply its own WebDriver defaults; this is not an assertion about every
ordinary user profile. The current native consumer and source-hashed assets
returned a secure context with **`navigator.gpu` absent**. Explicit activation
entered `unavailable`, with zero owned GPU surfaces/buffer bytes, named HTML
fallback, usable records, independent counter, keyboard selection of record 120
and native navigation to its server-rendered detail. All eight action records
completed. No positive GPU pixels are claimed for this default configuration.

Command, executed once:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_support.py
```

Receipt: `target/m11-gpu/yt080-20261014/default-profile/result.json`, SHA-256
`c2a7c1c436ef92ace8de4c8f4a200d6a16bd2051f9eea12f1cd75d7bcceb23a5`.
Status `passed_yt080_default_profile`; native exit 0. All 23 captured inputs
were unchanged. WebDriver session deletion, Firefox exit, driver/server reaping,
profile removal and both port closures passed; fresh PID/port checks confirmed
cleanup. The default lane is **fallback PASS**, not default WebGPU support.

## Focused source and package checks

All 20 named commands in `target/m11-gpu/yt080-20261014/checks.json` exited 0.
Their logs are under the adjacent `checks/` directory. Actual results include:

| Check | Result |
| --- | --- |
| Crate CPU contracts and compile-checked rustdoc example | 4 tests + 1 doctest PASS |
| Actual consumer native contracts | 12 tests PASS |
| Native and explicit wasm32 browser strict Clippy | Crate and consumer PASS with `-D warnings` |
| Native and wasm32 default CPU-only graphs | Crate only, no dependencies or device initialization |
| Explicit browser feature graph | Pinned wgpu std/webgpu/wgsl; no native backend features |
| Core/HTML/DOM/default HTTP-server normal/build graphs | No renderer/wgpu dependency edge; older native research remains separate |
| wasm32 default compile; native consumer and release Wasm builds | PASS |
| Native/browser rustdoc with warnings denied, formatting and Python syntax | PASS |
| Local package archive and Cargo compile verification | PASS, 10 package files; `publish = false`, no publication |
| Documentation links/DAG and Git whitespace checks | PASS |

The support table was finalized after the default-profile observation, then
the local package was verified once against the final documentation. The archive
contains 10 files; README, SUPPORT and all four Rust/WGSL source/test entries
match the checkout. Final archive SHA-256 is
`cdb139f4eca53c193da2b1bd5c10b1f3398909cd3837bef63ea89e541b237b3f`.
The root lockfile and all eight unrelated Todo/Search assets remain unchanged;
all 23 default-profile test inputs still match. The extended checker passed
83 documents and 81 backlog items, with tracked/untracked whitespace clean.
`target/m11-gpu/yt080-20261014/final-checks.json` records these facts. No browser
was relaunched for package/documentation metadata. The [independent final
qualification review](YT-080-INDEPENDENT-REVIEW.md) is now **PASS**, zero issues
and zero required followups, completing YT-080 at this selected scope.

## Independent review and interruption recovery

The separate read-only role-directed Job `wc_job_MkLEsrhXO-l7QrYx`
completed with native exit 0. Its exact Codex thread
`01a13ab2-6d98-7740-8f4f-f3c3ff7ef20b` reported saved model
`gpt-6-astra` and actual resolver-selected effort `high`. The result is
`overall_status: pass`, with no issues or required followups. Its SHA-256 is
`8417e4f4bbf4f33cad183d9b5d84176f24cc6b337400d20143c8f5e2718d0221`;
[the review record](YT-080-INDEPENDENT-REVIEW.md) details scope and provenance.

After the interrupted ChatGPT response, the same completed Job was observed,
not dispatched again. All 37 reviewed files, six retained acceptance/reviewer
receipts, 23 default-profile inputs, 20 check logs and final ten-file archive
were verified before recording Done. No task-owned process remained. Only
review/status documentation changed in this recovery; no new browser test,
package rebuild, source modification, commit or push was performed.

## Limits

Only the declared Firefox/X11/Intel experimental configuration can become
qualified here. Prior failed Linux Chrome combinations and Windows standalone
JavaScript results remain separate, not candidate support. No new Windows/macOS,
Safari, physical mobile, native backend, assistive-speech, real IME, production,
FPS or 3D claim follows. YT-077's no-demonstrated-speedup result is preserved.
YT-020 remains Blocked; YT-081 is a separate closeout task, not executed here.
