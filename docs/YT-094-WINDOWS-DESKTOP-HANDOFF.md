# YT-094 — native Windows IME and Narrator handoff

Date: 2026-10-19. Status: **manual verification pending**.

Historical preparation note: the [returned Windows verification](YT-094-WINDOWS-WASM-VERIFICATION.md#returned-native-windows-evidence--2026-10-19)
now records operator-passed native journeys, drain/exit-0 images and final cleanup.
The pending/running descriptions below preserve the state before that handoff;
they are not instructions to repeat completed tests or a current desktop status.

## Target and current desktop state

Use the operator-selected Windows host associated with the mint-game project.
The prepared directory is:

```text
C:\repos\yetlet\artifacts\yt094-mint-desktop-20261019
```

Narrator was enabled for this attempt and was still running at the last cleanup
observation. Its blue rectangles are the Narrator cursor. Press **Windows+Ctrl+Enter**
once to turn it off before starting the manual journey, then enable it again for
the spoken-AT check. The attempted termination of the exact owned Narrator PID
was denied; do not assume that automated cleanup disabled it. The on-screen
keyboard was switched off and its process was absent. No IME language settings
or clipboard contents were changed.

The test server is stopped, its CLI exited 0 with the drain marker, and port
3019 is free. The disposable runtime database and private startup log were
removed. An owned Edge InPrivate application window remains available for this
handoff. Preserve the unrelated normal Edge/HybridAuth window.

## Source and tool identity

The Ubuntu source checkout is the source checkout, branch
`feat/yt090-configuration`, HEAD
`a402b6f33525568ee330e18e0b6a04652c3734d5`.
The existing Windows Git checkout remains on
`maint/m10-qualification-20261010` at
`3128d23591feee15ce976d7b83a56dcd587048d7`; it was not reset or updated.
Run the application from the prepared `source` directory above.

The prepared copy reuses a prior source fixture and current committed text
inputs. All 310 recorded file hashes matched the selected candidate inputs
before building; `source-hashes.json` records that bounded comparison. This is
not complete Git-tree equivalence. Five frontend artifacts from the already
qualified Windows build were transferred by the artifact API with byte/hash
verification. The WASM SHA-256 is
`cd8f240239cb1a233e6ef932c3fdd470ff3a7f47161fc70521220a71cf4e757b`.

This host uses its retained, previously qualified GNU CLI:

```text
C:\repos\yetlet\artifacts\cli-generator-windows-20261018\target\x86_64-pc-windows-gnu\debug\yetlet.exe
SHA-256: 57390e617acfb5600eb1f43c37873bb97efba77b58e1528048256885a2e8434f
```

The current CLI source build on this host stopped at a missing OpenSSL
development dependency. It was not repaired for this input/AT handoff. This
handoff does not qualify the current CLI binary on mint-game; its separate
Windows qualification remains in the earlier verification record.

Rust 1.99 GNU was already installed. SQLite additionally required a C compiler.
Portable LLVM-MinGW 20261006 MSVCRT was unpacked under the task's `tools`
directory. Its official archive SHA-256 was
`24847b37a951c8583c3f7ec061de2eb87f6394215c8ef3fdc4565ed0835396e1`.
The existing Rust GNU GCC remains the linker; LLVM Clang compiles SQLite.
All PATH/compiler overrides are process-local.

## Start from an interactive PowerShell terminal

```powershell
. 'C:\repos\yetlet\artifacts\yt094-mint-desktop-20261019\environment.ps1'
& $YetletCli run --manifest-path $YetletManifest
```

The environment file was parsed with zero syntax errors; CLI identity, compiler,
manifest and built executable paths were checked. The application was already
built successfully with this environment and the retained CLI. Frontend
`auto_build` is false because this handoff consumes the verified built assets.
No frontend target, bindgen or npm installation is needed merely to run it.

Keep the terminal open. Open `http://127.0.0.1:3019/board/login` in a native
Windows browser. Enter identity `alice` and its fresh terminal-only
`LOCAL_LOGIN` code. Do not include that code in screenshots, transcripts or
handoff replies. On a fresh database, first open
`http://127.0.0.1:3019/board/projects/1/items/new`. Create two work items with
distinct titles, such as `IME sample` and `Switch sample`, using the native
**Create work item** button; keep the default Status/Priority. Repeat the New
work item form for the second item. This supplies the two records required by
the draft-switching journey. If two suitable records already exist, reuse them.
Then open:

```text
http://127.0.0.1:3019/board/projects/1/triage
```

The manual database is `manual.sqlite3` in the task directory. It is separate
from the disposable automated runtime database and persists across manual
restarts. If a rebuild is actually needed, use the same prepared environment:

```powershell
& $YetletCli build --manifest-path $YetletManifest
```

## Remaining journeys

| Journey | Native action and required observation |
| --- | --- |
| Real OS IME | Select a loaded item. Use the installed Traditional Chinese Microsoft Bopomofo IME and a real keyboard to compose a short phrase such as U+4F60 followed by U+597D in Title, then Description. Observe the OS composition/candidate UI before committing, commit the text, and confirm exact characters without duplication or loss. Direct Unicode filling or dispatched composition events do not qualify this cell. |
| Draft retention | After committing the IME text, reverse the queue, select another item and return. The first item's unsaved Title/Description must remain exact. |
| Explicit save | Save selected once, observe success/version advance, then reload and verify the committed Chinese text remains. Reload intentionally discards unsaved drafts. |
| Native keyboard | Use Tab/Shift+Tab through editor controls and Save selected. Confirm visible focus and that controls remain usable. |
| Spoken AT | Enable Narrator with Windows+Ctrl+Enter. Navigate the triage heading, item selection, Title/Description labels, status/priority and Save selected. Listen to actual names, roles and the save result. Preserve Speech recap if available, or a short application-only recording and a precise account of what was heard. An accessibility-tree dump or cursor rectangle alone is not speech evidence. |
| Physical Ctrl-C | Stop the foreground CLI with the actual keyboard. Record `$LASTEXITCODE`, the drain marker and absence of a listener on 3019. |
| Restore | Disable Narrator if it was initially off, close only test windows and confirm no owned server remains. Preserve manual records until the result has been reviewed. |

Useful native checks after stopping:

```powershell
$LASTEXITCODE
Get-NetTCPConnection -State Listen -LocalPort 3019 -ErrorAction SilentlyContinue
Get-Process Narrator,osk -ErrorAction SilentlyContinue
```

Return the Windows/browser versions, which journey actually ran, exact observed
text/focus/speech behavior, application-only evidence paths, actual exit code and
cleanup state. Then request independent reviewer review. Keep YT-094 In progress
until remaining qualification is assessed; these two input/AT cells do not
automatically close every YT-094 acceptance criterion.

## What is already proved and what blocked automation

Earlier Windows evidence separately passed 32 native tests, build/run, browser
local drafts, explicit saves, 409/422 handling, restart and two console stops.
Do not repeat that matrix merely to complete this handoff.

On mint-game, the candidate application built successfully through the retained
CLI; CLI run served the native sign-in screen; a native console control event
then produced exit 0, drain and port closure. Physical keyboard Ctrl-C, OS
composition, enhanced-page interaction and actual speech were not proved here.

The desktop gateway clamped accessibility traversal to depth 8; native Edge form
fields were outside that inspected subtree. Several requested keys were outside
its closed vocabulary. Text insertion was admitted for an empty focused address
field, while replacement of a non-empty field was rejected. Clipboard and Plugin
inspection required unavailable connector scopes. The on-screen keyboard exposed
only its root window, and two positioning controls did not produce the expected
change despite completed pointer reports. These are observed automation limits,
not an established Yetlet defect or a proven OS-level root cause.

The initial SSH probe had no authorized login key. An unused Chisel download was
blocked by Defender; it was not executed, and its archive was removed. No SSH
authorization or antivirus configuration was changed. Protected executable
artifact export was refused and was not bypassed. Current source preparation used
normal text-file operations; public frontend artifacts used the admitted transfer
API. Earlier failed build/preparation logs remain in the task directory.

## Prompt for a local Windows Codex continuation

Read this handoff and the applicable repository instructions. Continue only the
remaining native Windows IME, keyboard and actual Narrator journeys against the
prepared a402b6f application. Use the supplied environment and retained CLI; do
not reset the older C:\repos\yetlet checkout, rebuild unrelated components,
weaken Windows/connector protections, or count synthetic DOM input as real IME.
Preserve prior passing evidence and historical failures. Keep startup codes out
of reports. Record actual observations and cleanup, then request independent
reviewer review. Do not commit, push or deploy without a separate instruction.
