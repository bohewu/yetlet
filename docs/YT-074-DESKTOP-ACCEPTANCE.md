# YT-074 — Desktop Acceptance Record

**Date:** 2026-10-13. **Candidate:** `f00ec597e43f129a649fd1b8805fdd89edb93fbb`, existing Ubuntu `master` checkout, plus the uncommitted live-status compatibility repair and qualification/evidence changes. **Real OS IME: PASS. Spoken AT: PASS. Browser selection: Chrome/Firefox PASS under the owner's amendment.** Windows Edge remains unverified and is no longer a required gate for this continuation.

The owner first accepted manual blockers after Windows SSH discovery, then selected Chrome and Firefox instead of mandatory Windows Edge, identified the installed Ubuntu Chewing input method, and authorized repair of Firefox/Orca. Those instructions supersede the attachment's browser selection while preserving real composition and actual speech. The IME evidence below remains unchanged. After the live-status repair, the canonical Chrome 155.0.8059.39 and Firefox 157.0.1 journeys passed on the final binary; current report hashes are in [qualification](YT-074-QUALIFICATION.md).

## Gate A — Real OS Chewing composition: PASS

Environment: Ubuntu 26.04.1 LTS, kernel 7.0.0-38-generic, Xfce X11 on display `:0`; Fcitx5 5.1.19-1, Fcitx5 Chewing 5.1.10-1; headful Firefox 157.0.1. The owned fixture used `http://127.0.0.1:3012`, an isolated SQLite database and the existing release binary SHA-256 `085011ee5c237aa141283a08fab15975e45385c7feb35dca16e9ad4a8ef00346`.

Native Linux AT-SPI keyboard synthesis sent keysyms through the real X11/Fcitx input path. Before each key, the foreground window's PID was checked against the owned Firefox PID. WebDriver selected controls, submitted native forms and read results; it did not supply the Chinese strings. Observational DOM listeners recorded 54 trusted `compositionstart`, `compositionupdate`, `compositionend` and `input` events across Title and Description. No composition event was dispatched from JavaScript. The actual OS candidate window is visible in the [seven-image gallery](screenshots/yt074-ime/README.md).

| Step | Expected and observed |
|---|---|
| Title composition | `s u 3 Down` displayed candidates; `2 Return` selected U+59B3; `c l 3 Return` committed U+597D. Final title: U+59B3 U+597D. |
| Description | Native Tab focused the textarea. `s u 3 c l 3 Return Return h k 4 g 4 Return` produced U+4F60 U+597D, newline, U+6E2C U+8A66. Both controls recorded trusted composition start/update/end. |
| Create/read/history | Created once with Closed status. The list, detail, history title and read-only SQLite assertions matched the exact composed text. History records title/status, not a description snapshot. |
| Transition validation | Editing Closed to Active returned HTTP 422. The global alert explained reopening; both Chinese fields were retained. Changing status to Open and saving created version 2. |
| Conflicting edits | Two tabs loaded version 2. Native composition appended U+597D in the first and saved version 3. Native composition appended U+6E2C U+8A66 in the stale tab; its single submission returned HTTP 409 with the exact Chinese draft and current-record link. Storage remained version 3 with the first title. Following the link confirmed that authoritative detail/history. |
| Field relationship | A separate whitespace-title submission returned HTTP 422, retained the Chinese description, set `aria-invalid=true` and `aria-describedby=title-error`, and rendered the matching title error and validation summary. This additional rejected draft did not mutate storage. |
| Focus | Error/conflict navigation began at BODY rather than automatically focusing the alert. Native Tab revealed the visibly outlined Skip to content link; Enter moved to main, then Tab reached Back to work items with a visible outline. This verifies the observed keyboard path, not spoken announcements or general accessibility conformance. |

The textarea uses LF in its DOM value; native form submission stored CRLF and the edit textarea read back LF. Read-only storage assertions checked the exact CRLF form, preserving every Chinese code point and the line break. This is normal [HTML form newline normalization](https://blog.whatwg.org/newline-normalizations-in-form-submission), not lost text.

The sanitized [desktop evidence JSON](YT-074-DESKTOP-EVIDENCE.json) retains the event observations, exact Unicode strings, 422/409 results, read-back and cleanup. SHA-256: `be57f2b4d5949c923112aaefe15d68849284cca0d2ecd6b1cdedb6dee0a5f566`. Screenshot hashes are in the gallery.

### Preserved observation and operator failures

- An initial read-only assertion expected LF in storage; fresh observation showed native form CRLF normalization. Only the expected newline representation was corrected; no source/data change or mutation replay occurred.
- An observation object used a key named `error`, which the existing WebDriver wrapper mistakes for a protocol error. The fresh observation used a different key; no shared helper edit was made.
- Two broad `button[type=submit]` clicks selected the header Sign out button. The record stayed at version 1. Fresh sign-in and explicit `form[action=...]` selectors allowed the existing journey to continue. These were operator selection errors, not Session expiry or failed Board saves.
- Capture 05 preceded the native window's completed paint and was excluded from acceptance. Capture 06 shows the settled conflict page. The excluded original remains in the local desktop artifact directory.

## Gate B — Actual Firefox/Orca journey: PASS

Headful Firefox 157.0.1 registered on the existing desktop AT-SPI bus when launched with `GNOME_ACCESSIBILITY=1` and the isolated profile preference `accessibility.force_disabled=-1`. Orca 50.2 and Speech Dispatcher 0.12.1 were already installed. Private configuration/data directories and `GSETTINGS_BACKEND=memory` kept the task's Orca settings separate. This repaired browser startup without changing the system screen-reader flag or installing software.

The [spoken evidence JSON](YT-074-SPOKEN-AT-EVIDENCE.json) contains 546 actual Speech Dispatcher utterances with local timestamps, split into three attributed transcripts. SHA-256: `5b9cd8b3c9ba2f68c8154e75020f2a9ba05567689645631f3350b6a724375fda`. These are Orca's actual generated backend speech records, including Say All output, not an audio recording or text inferred from the accessibility tree. The baseline non-live transcript uses binary `085011ee...`; the final live and linear-reading transcripts use `dc2d6d16e310ed5df2b118251546fd3d2e5a7b0656862e58032c7813406f4228`. Non-live pages did not change in the repair.

| Required observation | Actual speech and interaction (Asia/Taipei) |
|---|---|
| Title, heading and navigation | `Work items | Lab Operations`, `main content`, `Work items`, `heading 1`; `navigation`, `Project navigation`, `alice / Operator / Project 1`. Initial and final linear transcripts both identify the page. |
| Form controls | Native Tab reached Title, Description (plain text), Status, Priority, Assignee and Create work item in order. Orca announced entry/required and combo-box roles; visible labels and names matched. |
| Validation | A real whitespace-title rejection retained the multiline description. Orca read `invalid entry`, the field-specific message and, with native Down navigation at 11:17:01, `Correct the highlighted fields. Your draft has not been saved.` Automatic summary announcement was not established. |
| Detail/history | At 11:18:44–11:19:13, Orca read the item heading, title, status/priority/assignee/version, multiline description, timestamps, History heading, six column headers and the persisted create row in order. |
| Conflict | A stale native submission returned HTTP 409 and retained `AT retained stale draft`. At 11:20:28 Orca read the conflict notice; at 11:21:12 it read `Open the latest stored version before reapplying this draft`; Tab then read the retained Title value. |
| Caption, headers and row link | Final native linear Down navigation at 11:58:23–24 read `table with 2 rows 5 columns`, `Project 1 — 1 matching items; 50 per page.`, `caption`, then Work item, Status, Priority, Assignee, Version and the `AT verified live result` link with active/normal/Unassigned/version 9. |
| Live update | A separate authorized client persisted version 9. At 11:48:43 Orca actually spoke `Status updated. Refresh authoritative records to see the latest work-item changes.` The counts and revision changed while the stored row stayed at version 8 until explicit refresh. |
| Pause, reconnect and refresh | At 11:49:15 Orca spoke the paused notice. Native Tab/Return reached and activated Reconnect live status; the updated notice was spoken again, then Tab announced Refresh authoritative records. Return fetched the authoritative version 9 row. A later five-minute SSE expiry/reconnect notice is retained in the transcript. |
| Visible focus | Native focus observations cover navigation, form controls, conflict recovery and live controls. The recovery link and final authoritative-refresh link have visible outlines in the captures below. No automatic alert focus is claimed. |

### Live-status compatibility repair and its limits

The original status paragraph mapped to AT-SPI STATUS_BAR, which installed Orca classified as a UI update before its live-region path. A status wrapper alone and a JavaScript ordering-only prototype still lost rapid text events to Orca's event filter. The final markup retains the semantic `role=status` container and gives its message child `role=generic`, explicit polite live behavior and atomic speech. Firefox maps that child to SECTION, which Orca admits through its live-event path. The enhancement emits the message before adjacent counts and skips unchanged text writes. A focused Firefox observation verifies both the changed message/count mutation order and no mutations for unchanged snapshots.

The final utterance above was collected after rebuilding the actual application; a successful runtime prototype alone was not counted. No authentication, transaction, retry, CSP or database-durability behavior changed. This is a local Firefox/Orca compatibility result, not broad accessibility certification.

An initial Say All/table-jump path did not read the caption. Flat review first lacked context, then spoke only `table`. A restart without document activation produced only `window`. Those observations are retained as operator/setup failures, not passing evidence. Normal Down-key document reading subsequently spoke the caption and its following table contents without another source change. A diagnostic that overlapped headless Chrome is excluded from Firefox acceptance. Repeated footer utterances after reaching the document end remain visible in the accepted transcript.

### Actual captures and current cleanup

- [Conflict recovery focus](screenshots/yt074-at/conflict-focus.png): unchanged form markup and retained stale draft.
- [Final live notice and refresh focus](screenshots/yt074-at/accepted-live-focus.png): counts reflect version 9; stored row still version 8.
- [Authoritative refreshed row](screenshots/yt074-at/accepted-authoritative.png): persisted version 9 after explicit refresh.

All three images were visually inspected; their SHA-256 values are in the spoken evidence JSON. No login code, Cookie or CSRF value is visible.

The native sign-out completed. Server PID 226538, geckodriver 175310 and Firefox 175387 were reaped; listeners 3012 and 51769 closed. All owned Orca processes exited, and the identity-checked task-created Speech Dispatcher 181770 and its modules were stopped. The control session exited 0. Private database, logs, Firefox profile and Orca configuration/debug logs were removed. `ScreenReaderEnabled` read back false. This AT continuation did not change the IME; after all editable contexts closed, Fcitx reported no current input context. The prior IME lane's explicit keyboard-us restoration remains its separate evidence. No task-owned desktop process remained when the new full load run started.

### Historical initial Orca blocker — superseded by the journey above

Orca 50.2 and Speech Dispatcher 0.12.1 were already installed; nothing was installed. Two owned Orca processes ran sequentially and each exited 0. The first generated actual Speech Dispatcher output for `Screen reader on.`, `tab`, `+` and shutdown. The debug log includes backend rate/pitch/volume/language responses. It contains no Board title, label, error, table or live-status announcement.

The owned Firefox did not appear as an application on the desktop AT-SPI bus. `ScreenReaderEnabled` was initially false. One bounded setup correction temporarily set it true and started Orca again against the same owned browser. Native Tab, Enter and Say All still produced only key echo. The corrected attempt also lacked Board content and was stopped. The cause of Firefox's missing desktop accessibility registration was not established; installed tools, startup speech and key echo do not close the requested journey.

Only sanitized speech-output lines are retained in the evidence JSON. Raw debug logs and private Orca configuration were removed. There is no audio recording or claim that application labels were spoken. The manual checklist below remains open; no further capability recovery or software installation was attempted.

## Earlier Windows discovery and scope amendment

Read-only WebCodex discovery returned two Windows desktop targets and no Ubuntu Computer target. Runner `ncut-bohe-c420c5de1cac4bd4` reported Windows 11 Pro 10.0.26300, Edge 155.0.4283.45, Narrator 10.0.26100.8972 and `en-US`/`zh-Hant-TW`; NVDA was absent from standard installation locations. Runtime discovery found zero active/queued/recovering/uncertain Jobs. Existing unrelated Edge windows were preserved.

Direct SSH to the Ubuntu LAN address timed out. The existing ZeroTier route reached SSH. A task-owned `known_hosts` file containing the exact locally read public ED25519 host key preserved strict host verification; authentication then failed with `Permission denied (publickey)`. No password, private key, trust bypass or new authorization was introduced. The owned Windows directory `target/yt074-local-20261013-0208`, containing only the public host-key file, was removed. Fresh checks found no SSH process or port-3012 listener on that runner. No Edge or tunnel was launched.

A read-only snapshot of an existing RustDesk window on the other runner showed the Ubuntu desktop. A bounded activation attempt returned unknown focus outcome; fresh observation showed RustDesk was not foreground, so no keyboard input was sent through that route. The completed IME journey instead used the local OS input path described above. No unrelated Windows window or RustDesk process was closed or restarted.

## Historical IME and initial AT resource cleanup

The browser signed out before closing. Owned server PID 120935, geckodriver PID 120962 and Firefox PID 121042 terminated; both Board port 3012 and the ephemeral driver listener were closed. Both Orca processes (135022, 138615) exited 0. Speech Dispatcher PID 135052, absent before this task, was identity-checked and stopped; a follow-up confirmed all these PIDs absent. Its cache directory was absent. The Python control session exited 0.

The isolated SQLite database, credential-bearing server log, browser profile, private Orca configuration and raw debug logs were removed through owned cleanup. The original `keyboard-us` input method and false `ScreenReaderEnabled` state were restored and read back. Normal IME candidate learning from test typing was not rewritten. The load fixture used a separate port/database and also reached terminal cleanup; no application or uncertain task-owned Job is deliberately left running.

## Reusable manual fixture rules

Use a fresh owned database and the unchanged [startup procedure](BOARD-START.md), keep loopback binding and use synthetic identities. Keep login codes, Cookie/CSRF values and credential-bearing requests out of screenshots and transcripts. Record actual OS/browser/AT versions, actions, output, failures and cleanup. Do not infer speech from DOM/ARIA or install software solely to bypass a blocked cell.

## Historical Gate B checklist — now covered above

Use an already available screen reader, such as Ubuntu Firefox with Orca or Windows Edge with Narrator. Use NVDA Speech Viewer only when NVDA is already available or separately authorized. An actual speech log/viewer or reviewable recording is required; reading the accessibility tree aloud in a report is not evidence.

| Step | Required spoken/focus observation |
|---|---|
| Initial page | Document title and main heading; record the actual utterances and order. |
| Navigation | Navigation region, current identity and role; actionable links have intelligible names. |
| List | Work-item table caption, column headers and row links in usable reading order. |
| New work item | Each visible label matches the announced control name; required fields and control types are understandable. |
| Validation | Submit a whitespace-only title with a retained synthetic description; record announcement/discovery of the summary and the field-specific error relationship. |
| Detail/history | Title, description, metadata and history are read in a coherent order; history headers remain meaningful. |
| Conflict | Submit an intentionally stale edit; record the conflict notice, retained draft and current-record link. |
| Live status | Enable the optional status stream, save a change from another authorized context and capture the actual `role=status` announcement and authoritative-refresh link. Stop and reconnect the stream explicitly. |
| Keyboard/focus | Record the actual Tab order and visible focus through navigation, form controls, errors, conflict recovery and live-status controls. |

Attach a sanitized transcript summary identifying the source of each utterance. Mark missed, repeated, ambiguous or absent speech explicitly. Restore only the screen-reader state changed by the task and perform the same owned-resource cleanup as Gate A.

**Initial result:** Blocked. The two initial Orca observations produced startup/shutdown and key echo only. **Current result:** PASS for the attributed Firefox/Orca journey above, with its explicit navigation and announcement limitations.

## Optional future Windows Edge journey

After the SSH prerequisite is resolved, recheck the exact runner and active/uncertain Jobs. Start one owned tunnel and one clearly owned Edge tab/profile; preserve existing windows. State the next bounded visible action before input and observe fresh state after every state change.

Execute native sign-in, create a Unicode/plain-text record, inspect list/detail/history, edit, submit one stale edit and verify HTTP 409 with retained draft and no overwrite. Enable live status, cause a real persisted update, stop/reconnect and follow the authoritative-refresh link. Capture sanitized console/network deltas and classify only the asserted intentional 409 as expected. Exercise a narrow viewport if the admitted surface supports it. Record exact browser version, screenshot references, outcomes and cleanup.

Observe any promoted process through the same Job to a terminal result. A timeout is not termination and does not authorize a replacement. Close only this task's tab/process/tunnel, verify closed listeners and remove its synthetic data. Stop after one completed journey.

**Result:** No Edge journey was started. The owner-selected Chrome/Firefox coverage is satisfied by passing canonical journeys on the final binary; Windows Edge is unverified and outside this continuation's required browser scope.
