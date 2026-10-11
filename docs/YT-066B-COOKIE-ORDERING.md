# YT-066b — Cookie response ordering gate

**Date:** 2026-10-11
**Status:** The maintainer explicitly selected the pure-SSR contract below on 2026-10-11. Its corrected fixed-name implementation has focused passing tests and independent reviewer **PASS**. The earlier unique-name candidate remains historical **FAIL**. The [subsequent YT-066b2 implementation](YT-066B2-VERIFICATION.md) has separate passing focused runtime evidence and independent reviewer PASS at bounded experimental-provider scope.
**Related:** [Accepted plan](YT-065-066-PLAN.md), [Redis design](YT-066B-REDIS-SESSION-DESIGN.md), [original spike evidence](YT-066B1-VERIFICATION.md), [fixture](../qualification/yt066b/README.md).

## Authority and committed baseline

The maintainer initially explicitly authorized peon to commit the reviewed YT-066b1 result, then address cookie response ordering without weakening the original guarantee, and proceed to YT-066b2 only after independent reviewer PASS. No WebCodex or push is authorized.

Peon committed exactly the fourteen previously reviewed files as `6fd6876124ba4bc3120544498de14917f6c8fedd`, parent `d93e1da645dff1a9bedafb0fe940ce3ac9cb82fb`, tree `da72caa6e78231afaaffea9ae0ac3fd83ea29bcb`. All file bytes were preserved and the index/worktree became clean. Native trace verified role `peon`, model `gpt-6-luna`, effective effort `high`, thread `01a1288b-01a3-7692-8950-54d15d4c0600`. The deterministic staged Commit Guard exited 0 under **default audit / builtin**, with two non-blocking documentation path warnings; this is not a strict guard PASS. No push occurred.

The following cookie work is a separate uncommitted change in the same checkout. The original six-test execution and reviewer PASS remain attached to their original bytes. Their recorded unresolved single-name cookie result is historical evidence, not rewritten as a success for this later candidate.

## Current selected contract and explicit authority

After the failures recorded below, the maintainer questioned whether ordinary Session use needed JavaScript. The proposed choice was: no JS, preserve authorization safety, and permit login loss under extreme response reordering. The maintainer explicitly accepted that choice on 2026-10-11 (`explicit_user`), then asked how Redis differs from Memory Session. This supersedes the earlier unconditional replacement-cookie retention requirement. It is a deliberate narrowing of availability, not a claim that the former guarantee was solved. The JavaScript coordination proposal was not selected or implemented.

Both Session types use an opaque browser cookie and can support SSR without JS. Memory is synchronous and process-local, loses its records when that process restarts and cannot directly share them across workers. Redis uses async remote operations and shared records, with explicit contention, outages and ambiguous writes. Redis surviving an application restart is separate from Redis persistence. The Cookie delivery race exists independently of either backing store; this correction neither changes nor broadens the Memory API's retained guarantees.

The selected protocol is:

- One exact application-scoped cookie name, normally `__Host-` with Secure, HttpOnly, SameSite=Lax, Path=/ and no Domain. The local HTTP fixture explicitly uses a different insecure name.
- Only a positively resolved create/rotate/replace can issue a Set directive, followed by an authoritative load of the issued identity. An unresolved operation returns an unknown outcome; observed retirement suppresses emission. Reads, ordinary updates, errors and successful logout issue no Set/Clear. Logout revokes in Redis.
- Every presented credential is checked against Redis. Duplicate or malformed configured-name cookies fail closed; legacy issuance names are ignored. No alternate-cookie selection, old-ID forwarding, automatic creation or Memory fallback occurs.
- Retirement/expiry/corruption/unavailability cannot authenticate the old bearer. If retirement happens after finalization, a prepared Set may overwrite the replacement and make a subsequent request unauthorized. Explicit login may recover. A dead cookie can remain in storage until overwrite/expiry; it conveys no authorization.
- One name avoids provider-induced per-issuance accumulation, without promising immunity to general browser deletion or eviction. Independent still-live Sessions are separate credentials; this layer does not implement browser-wide latest-account intent, global logout or implicit revocation of all independent Sessions.

The implementation remains in the unpublished spike. The existing schema-2 issuance field and strict integer validation are retained as experiment metadata; the current cookie name/value selection does not use that counter. No stable middleware, mandatory client JS, authentication framework or persistent event circuit is introduced. Root/default/WASM graphs and the Memory API remain unchanged.

## Current focused validation

The first command was `cargo +1.88.0 test --manifest-path qualification/yt066b/Cargo.toml --locked --test cookie_fencing -- --include-ignored --test-threads=1 --nocapture`. Four tests passed: exact-name parsing, emission eligibility, forced real HTTP races, and authoritative loading without creation/fallback. They cover competing rotations, unresolved results, expiry/corruption/shutdown, ignored independent legacy cookies, before-check suppression, after-check/header login loss and delayed logout without Clear.

The fifth, real-Chrome test first hit a harness-only navigation race: the document changed between `DOM.getDocument` and `DOM.getOuterHTML`. The test now retries only that exact stale-node observation within its existing bounded wait. The sole affected rerun was `cargo +1.88.0 test --manifest-path qualification/yt066b/Cargo.toml --locked --test cookie_fencing real_chrome_without_js -- --ignored --test-threads=1 --nocapture`; it passed, exit 0. This is four passing cases on the first source plus one affected passing rerun after a Python-only synchronization correction, not a claimed fresh five-test pass.

Actual Chrome **155.0.8059.39** ran with page script execution disabled, static SSR responses and CSP `default-src 'none'`. Over real HTTP against the Redis fixture, it received a replacement cookie, then the same prepared retired cookie 180 times. Exactly one cookie remained; the retired credential was unauthorized although the replacement record stayed live. Explicit synthetic login restored access. Logout and a later retired cookie remained unauthorized. These are focused protocol observations, not full authentication, browser/device qualification or the earlier 180-independent-cookie eviction experiment.

Project `yt066b-ssr-39ed5a19` had test exit 101 for the harness failure and cleanup exit 0. The affected browser rerun project `yt066b-ssr-browser-e0ab5632` had test/cleanup exit 0. Both exact projects had zero containers/networks/volumes afterward. HTTP tasks and Store workers were shut down; the owned Chrome PID, debug port and temporary profile were removed. Existing broad Docker/Memory/browser suites were not repeated.

Rust 1.88 compilation and stable Clippy (`--all-targets -- -D warnings`) passed. The browser rerun's twelve-file fixture map has SHA-256 `1b62d3c8df753cd72c40184982543604c6cb8ee75af67f0d9d5fdc791943b02a` (sorted compact JSON path-to-SHA-256 map), retained with exact commands and cleanup in ignored `target/yt066b-cookie-20261011/ssr-evidence.json` and `ssr-browser-evidence.json`. The unchanged lock SHA-256 is `b0c21fe49964ea2df6acfb824c6c2d25bf952a79d4d99168682e626ed5e80150`; the same pinned Redis image is used. The four passing cases' Rust inputs match that map; only the affected Python file changed for the rerun. Its eleven non-Markdown inputs still match the checkout; the fixture README was subsequently reconciled to this decision and evidence.

## Current review and delivery boundary

Independent read-only review of this selected contract, actual diff and evidence returned canonical **PASS** with empty issues, required followups and optional notes. Thread `01a128b4-dbc6-7922-87e7-ab67c171665a` completed with exit 0; expected and observed model/effort were `gpt-6-astra` / `high`, sandbox read-only. The healthy/current/eligible saved reviewer configuration and shared adaptive resolver were checked before ordinary local `codex exec` dispatch. This is a role-directed process, not native managed-role identity or formal assurance. All 268 input file hashes, HEAD, index and status were unchanged across review; the exact PID is gone. Details are retained in ignored `target/yt066b-cookie-20261011/review-ssr/review-evidence.json`. Only factual review status is added after PASS. Earlier FAIL results remain failures of the former candidate and requirement. The selected Cookie gate now permits the separately authorized YT-066b2 stage; no provider implementation is part of this Cookie review. This change remains unstaged/uncommitted on `master` at `6fd6876124ba4bc3120544498de14917f6c8fedd`; no push or new worktree is authorized. YT-066 remains In progress and YT-020 remains Blocked.

## Historical candidate and failed requirement

Everything in the following historical sections describes the superseded unique-name candidate at its recorded checkpoint. Its source maps and evidence are retained, including a copy of the nine prior changed files under ignored `target/yt066b-cookie-20261011/pre-ssr-contract/`. Statements about an unchanged original guarantee or a blocked next stage below refer to that earlier checkpoint, before the explicit maintainer decision above.

### Historical required invariant

After a replacement credential reaches the client, an older request or prepared/in-flight Set/Clear response must not remove or replace that credential or cause authentication to fall back to a retired identity. This includes the gap after a final authoritative check and before header construction. Merely rejecting the old bearer while losing the usable replacement is insufficient.

No mandatory JavaScript, persistent remote event circuit, old-identity forwarding, automatic Memory fallback or weaker guarantee is introduced. The existing Memory implementation/API and the root/default/browser dependency graphs remain unchanged.

### Historical: Candidate protocol

The cookie storage key includes an issuance number and a SHA-256 digest of that issuance's opaque bearer, under an application-specific prefix. A cookie for identity X and one for its replacement Y therefore use different names at the same host and root path. Neither identity is ever installed under the other's name. The normal policy uses `__Host-`, Secure, HttpOnly, SameSite=Lax and Path=/ with no Domain; the loopback test explicitly selects a distinct insecure name.

Redis assigns a strictly increasing namespace-wide issuance number during each successful create/rotate/replace, in the same script as activation. Ordinary CAS preserves it. The counter never expires and fails closed on invalid type/encoding or exhaustion; the experiment bounds it to the exact Lua integer range. Schema 2 adds this field and explicitly rejects schema 1 rather than guessing a migration. This disposable namespace is not an accepted reset/restore procedure; production recovery must invalidate the namespace and cookie scope before counter reuse.

Request selection uses the highest presented issuance and verifies its bearer digest and authoritative record issuance. Cookie header order does not decide authorization. A missing, revoked, corrupt, expired or unavailable record for that presented identity fails closed. However, if the browser has already evicted its cookie, the parser cannot detect the omitted higher issuance and can select a lower live identity. That is the unresolved failure below. Duplicate issuance claims and malformed names are rejected. Fabricating ordering metadata cannot authenticate a request.

Create/rotate may issue a pending cookie after a final authoritative check. If that check sees retirement, it emits nothing. Ordinary read/update must not slide or refresh the cookie. The final check still cannot retract an already prepared response: different names prevent direct replacement of another issuance's cookie, but do not prevent storage eviction.

A successful logout writes a non-bearer `!` marker under that identity's own name. It does not remove the highest issuance and expose a lower independent Session left from concurrent initial creation. The marker's sixty-second retention covers the maximum remaining server-side Session lifetime. A late old Set for the same revoked name is still rejected by Redis, with no fallback. A later login uses a higher, different name, so an old logout cannot delete it.

Cleanup clears only names present in the originating request and strictly below the selected/new issuance. It never clears the selected name or guesses a future name. Sequential rotation with cleanup retains one cookie; an old prepared Set can temporarily reintroduce a lower slot, removed on the next successful load.

The parser scans the headers admitted by the host HTTP stack while retaining only the highest 32 slots for selection and cleanup. It does not impose a new total-byte or slot-count rejection on legitimate delayed lower cookies. Each cleanup response removes at most 31 lower names; remaining lower names can be cleared by later requests that still authenticate successfully. Memory and response work remain bounded without rejecting the replacement merely because an earlier cleanup could not observe delayed names.

Browser storage is also finite. Passing the ordinary reorder tests does not close the accumulation/eviction failure caused by delayed old responses. This candidate failed independent review on that boundary. A new accumulation or concurrency precondition has not been approved and must not replace the original guarantee.

The key-selection rationale follows the standard cookie replacement tuple and the fact that cookie values have no built-in version comparison. See [RFC 6265 storage semantics](https://www.rfc-editor.org/rfc/rfc6265.html#section-5.3) and [cookie limits](https://www.rfc-editor.org/rfc/rfc6265.html#section-6.1). Independent user deletion, browser privacy policy and unsupported devices are not provider qualification claims; provider-induced accumulation must still be evaluated.

### Historical: Actual execution

`cargo +1.88.0 test --manifest-path qualification/yt066b/Cargo.toml --locked -- --include-ignored --test-threads=1` exited 0: **nine tests passed**, comprising eight actual-Redis integration cases and one local cookie-policy/bounds test. The three new cases establish:

- Real HTTP delivery before final check, after check, after header preparation, and delayed successful logout: the replacement remains in the actual reqwest jar and its authenticated route returns 200; retired IDs remain rejected.
- Authoritative ordering across independent initial issuances, no fallback after logout or expired tombstone, exactly one competing-rotation winner, delayed old cleanup, and 36 further rotations with one-cookie cleanup.
- Secure defaults, name/value binding, duplicate rejection and explicit input bounds.

The existing single-name counterexample remains a negative control. Its test still passes by demonstrating failure of that old policy; it does not describe the new protocol. The shared Lua envelope changed, so the existing bounded spike semantics/cancellation cases were also rerun. No YT-066a Docker, framework/browser regression suite or broader qualification was repeated.

The same pinned Redis 8.2.10 image and separate Cargo lockfile were used. Project `yt066b-cookie-f82fe224` reached terminal test exit 0 and cleanup exit 0; exact-project container/network/volume counts were 0/0/0. HTTP fixture servers shut down in the test. Stable Clippy with `--all-targets -- -D warnings` passed. Execution snapshots and logs are retained under ignored `target/yt066b-cookie-20261011/`; no new validator or test framework was added.

The initial ten-file runtime map in `attempt1.json` has SHA-256 `12d251d916cc247f2ec7b06fddea783aff559bbaee3824d7e44489c95fd7cba2` (sorted compact JSON path-to-SHA-256 map). The committed baseline is `6fd6876124ba4bc3120544498de14917f6c8fedd`; these candidate changes are additional uncommitted content. The lock hash remains `b0c21fe49964ea2df6acfb824c6c2d25bf952a79d4d99168682e626ed5e80150`; image remains `redis@sha256:47670742d7924adbcdb404d1288b6327815b23141969c0939b8e5d61f78c2634`.

### Historical: First review and material repairs

Local read-only reviewer thread `01a12896-2e58-7f20-930d-f7fea74de622` returned canonical FAIL with two findings. Session telemetry observed the expected `gpt-6-astra` model and `high` effort; all 267 source/document files remained unchanged across review. The outer execution wrapper ended with exit 143 after the completed-turn event and result file were written; the exact reviewer PID was confirmed gone, but its individual exit code was not recovered. This is not a normally accepted completed review. Its concrete findings were independently checked and admitted under the materiality gate; no model/effort recovery was used.

- **P1, delayed-cookie admission:** 32 previously unobserved older headers plus the replacement caused the old parser to reject the entire request. The parser now scans all transport-admitted headers and retains the highest slots for bounded cleanup, without raising the old limits or accepting a concurrency precondition. A regression holds 56 actual HTTP responses with prepared lower-issuance cookies, delivers a later replacement first, then releases the old responses. The resulting 57-cookie header exceeds 8192 bytes; both subsequent authenticated requests return 200, retain the replacement and reduce the jar to one cookie in bounded batches.
- **P2, corrupt integer metadata:** Lua accepted fractional issuance metadata that the Rust reader rejected, allowing a previously prepared rotation to commit. Lua now requires bounded non-negative decimal integers for numeric envelope fields before mutation. The focused regression corrupts issuance, revision, generation and expiry to `1.5`; each prepared rotation returns invalid, leaves the source active and never creates its replacement.

Only `cookies.rs`, `session.lua`, `cookie_fencing.rs` and `semantics.rs` changed in the runtime between the initial execution and these repairs. The affected command was `cargo +1.88.0 test --manifest-path qualification/yt066b/Cargo.toml --locked --test cookie_fencing --test semantics -- --include-ignored --test-threads=1`: **eight tests PASS**, seven with real Redis plus the local policy case. The original bounds and single-name negative-control evidence were retained rather than rerun. Project `yt066b-cookie-fix-d068ee61` had test/cleanup exit 0 and zero remaining containers/networks/volumes. Final stable Clippy and format checks passed. Exact final hashes and logs are in ignored `focused-repairs.json` and `tests-focused.txt`.

### Historical: Final independent review and confirmed storage failure

The second local reviewer completed normally, exit 0, thread `01a1289c-e4e8-77b2-b374-c6f4153e3a4f`. Expected and observed model/effort were `gpt-6-astra` / `high`; sandbox was read-only. Dispatch used ordinary local `codex exec` with the saved repository reviewer configuration and canonical reviewer contract, after healthy/current/eligible profile verification and shared reasoning resolution. It was not native managed-role dispatch. All 267 tracked/untracked input files were unchanged across review. The canonical result was **FAIL**, with one P1 issue, one material required follow-up and no optional notes. Exact metadata and JSON are retained in ignored `review-followup/review-evidence.json` under the evidence directory above.

The reviewed ten-file runtime map is SHA-256 `5bb0299da7fde6d846f9ce63dd4d214f6c355163514b2f8b172b1bc87ff652eb`. It matches the affected passing execution and remains unchanged after review. The later edits only record review outcome, the focused storage observation and task status; they are not a new reviewer PASS.

The counterexample prepares 180 independent lower-issuance responses over three operation-result admission windows. A later replacement reaches the browser first; then the older headers arrive before request-driven cleanup. Sixty-second Session lifetimes and five-second result expiry do not prevent accumulating that many pending headers. Equal-priority cookies are subject to the browser's finite storage quota. If the replacement or logout marker is evicted, `load` sees only lower cookies and can select a still-live independent Session. The 57-cookie reqwest regression does not model this eviction and therefore cannot establish the full guarantee. See [Chromium cookie storage](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/net/cookies/cookie_monster.cc) for its per-domain quota and eviction implementation.

A bounded follow-up used actual **Chrome 155.0.8059.39**, revision `3ff7ac5a9224be9156d7f8703a06e22890aafd34`, in a new temporary profile. The existing repository DevTools helper inserted a synthetic issuance-181 cookie, followed by 180 lower issuance cookies with the candidate's Secure/HttpOnly/host-only/root-path/Lax policy and sixty-second expiry. All insertions succeeded; **150 cookies remained, the issuance-181 cookie was absent and the highest remaining issuance was 180**. Repeating with the issuance-181 value set to the candidate's logout marker produced the same loss. Both cases had Medium storage priority and completed before expiry.

This is actual browser-storage evidence using CDP, **not** an end-to-end HTTP/Redis authentication execution or broad browser qualification. Lower-identity selection is the accompanying source-level consequence. The first probe misformatted the reported remaining issuance by reading the digest field; one local reporting correction and focused rerun fixed that output. The original observation is retained and is not product failure evidence by itself. `chrome-storage-evidence.json` contains the final sanitized result. The owned Chrome exited 0; its PID, temporary profile and debug listener were all gone afterward. No Redis or application server was started for this probe.

The finding is material to the maintainer's unchanged replacement-retention requirement. Raising parser/browser limits, assuming fewer delayed responses, adding mandatory client JavaScript, or accepting only retired-ID rejection would not close it. No speculative protocol expansion or replacement review was started without a substantive correction. The next required work is a cookie issuance/delivery design that preserves replacement retention and logout rejection under this failure, followed by focused verification and independent PASS. YT-066b2 remains blocked at that gate.

### Historical: Final workspace and validation

The nine current changed files are:

- `docs/BACKLOG.md`
- `docs/M10G-VERIFICATION.md`
- `docs/YT-066B-COOKIE-ORDERING.md` (new)
- `qualification/yt066b/README.md`
- `qualification/yt066b/src/cookies.rs` (new)
- `qualification/yt066b/src/lib.rs`
- `qualification/yt066b/src/session.lua`
- `qualification/yt066b/tests/cookie_fencing.rs` (new)
- `qualification/yt066b/tests/semantics.rs`

The existing documentation checker passes its 75 required documents and 67 backlog items. Applying that same checker to all repository Markdown files, including the additional planning/spike and sample documents, passes **95 documents and 67 items**. Internal links, English-only documentation, Markdown structure, whitespace and backlog dependencies pass. `git diff --check` passes. The index is empty and all cookie-candidate changes remain uncommitted. Branch is `master`, HEAD is the peon commit `6fd6876124ba4bc3120544498de14917f6c8fedd`, with no push. No root Cargo/dependency files, framework implementation, Memory API, workflows or sample sources changed. Both Redis fixture projects and the owned browser/reviewer processes are terminal and cleaned up.

YT-065 remains Done at automated native/desktop browser qualification scope, YT-066 remains In progress, YT-067 remains Done at its recorded internal scan/fix scope, and YT-020 remains Blocked. The cookie guarantee has not passed; public release, provider implementation and production qualification are not established by these partial results.
