# YT-066b3 — Three-application Redis qualification

**Date:** 2026-10-11
**Status:** Done at bounded single-host, single-primary, no-persistence qualification scope: actual execution, owned cleanup and final independent reviewer PASS. YT-066 remains In progress; YT-020 remains Blocked.
> **Later parent closeout:** [M10g](M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11) marks YT-066 Done only at single-host Docker multi-instance and selected Redis-provider scope, with a separate final documentation review. This record retains b3's execution/review/delivery identities and its parent-task state at that stage; no new b3 run or durability result is claimed.

**Related:** [fixture](../qualification/yt066b3/README.md), [design](YT-066B-REDIS-SESSION-DESIGN.md), [b2 evidence](YT-066B2-VERIFICATION.md), [Cookie decision](YT-066B-COOKIE-ORDERING.md), [M10g](M10G-VERIFICATION.md).

## Authority and prior delivery

The maintainer explicitly authorized peon to commit reviewed YT-066b2 first, then
continue three-application Redis qualification and independent review, without
WebCodex or push, using the same checkout. Peon committed the exact 24 reviewed
provider/Cookie paths as `1f38d20b8ebaf9b679aa69ce461b495db1856db2`, parent
`6fd6876124ba4bc3120544498de14917f6c8fedd`, tree
`2a02346267fe5ee73055ca22e2e52e4f4e0c851c`. Input hashes were preserved and the
index/worktree were clean immediately afterward.

Peon runtime trace matched role `peon`, saved model `gpt-6-luna` and shared-resolver
requested/effective effort `high`. The deterministic staged commit guard exited
0 under the existing default audit/builtin profile, with first-screen coverage
and four document-path findings reported as warnings. Audit did not block them;
no setting or hook was bypassed or weakened. Guard tree matched the committed
tree. Local evidence is in ignored `target/yt066b3-20261011/commit-guard.json`.
No push occurred and `origin/master` remains `8e157b43244be38c9afc22ed7bcd7bf06351416e`.

## Exact actual execution identity

| Item | Observed value |
|---|---|
| Checkout | Original sole checkout, `master`, committed baseline `1f38d20b8ebaf9b679aa69ce461b495db1856db2`; b3 source additionally uncommitted |
| Execution source | 285 tracked/untracked nonignored paths; working-source SHA-256 `0378994d46c5eb52a98a3de3cd5ce47c464207098bae8efbf8ac0291182e9ece` |
| Runtime input map | SHA-256 `47f8376be53e4404319e6b5ae2090622ad805a8fcbf2bddeeffa0d17b8decd77`; exact entries in `execution-inputs.json` |
| Project | `yt066b3-1009004-5a0d472a` |
| Build | Pinned Rust image `af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0`, Rust/Cargo 1.88, release, locked/offline vendored inputs, network disabled, source read-only |
| Application binary | SHA-256 `e8d8ea3cc42cc57b5f1e99e953a3667255677c668cc303192247f9da5f861b92` |
| Generated application image | `sha256:380bdab91b17602e564d80ae62d324ee82c6b3518c51e98543536ebc44e36004`; removed after execution |
| Nginx | 1.30.5; pinned image digest `0985e772fb9f729e6fa0980da05fca5d9c468e870eed43071545afa9d2e27d94` |
| Redis | 8.2.10; pinned image digest `47670742d7924adbcdb404d1288b6327815b23141969c0939b8e5d61f78c2634` |
| Topology | Three separate app processes and one Redis primary on an internal backend; Nginx adds a front bridge; only random loopback proxy port published; apps/Redis have zero host bindings |
| Persistence | `appendonly no`, `save ""`, tmpfs; no persistent volume |
| Browser | Actual Chrome 155.0.8059.39, script execution disabled, isolated temporary profile |
| Execution | `python3 qualification/yt066b3/run.py`, terminal exit 0, report status `pass-at-single-host-no-persistence-scope` |

The exact source identity was independently recomputed immediately after execution
and matched the pre-execution identity. Later documentation updates are separate;
they do not relabel the executed source. The separate fixture workspace/lockfile
selects the provider explicitly. Root Cargo files, Memory API/source, existing
provider source, workflows, samples and browser assets remain unchanged by b3.

## Actual observations

| Boundary | Actual passing observation |
|---|---|
| Create/read/distribution | Session created on A read the exact synthetic typed counter on A/B/C; pooled stateless requests reached all three without creating Cookies |
| Cross-process CAS | A and B reached synchronous callbacks from the same revision before either mutation; one 200 and one explicit 409, one callback per call, authoritative count 2 on A/B/C |
| Staleness seen by finalizer | A created state and held its handler before finalization; B rotated the privately observed synthetic identity; A's actual provider finalizer suppressed Set; old identity rejected everywhere and replacement stayed usable |
| Prepared response reorder | A held a real HTTP response after provider finalization; B rotated and delivered its replacement first; releasing A overwrote the shared standard HTTP Cookie jar; the next C read was 401, replacement Redis state still valid, one fixed Cookie, explicit creation recovered |
| Rotation/revocation/logout | Delayed logout emitted no Set/Clear and could not erase a later login; competing B/C rotations produced one winner and no losing Cookie; C revocation was rejected on all workers |
| Unknown result | After A's snapshot/callback, a 700 ms WRITE pause caused 202/unknown with one callback and no Cookie; A's occupied slot rejected a waiting read by deadline; B then updated and C rotated; A resolved its historical commit independently of current identity; replacement count 3, old identity rejected |
| Result expiry | After 5.1 seconds, the same server-owned ticket resolved Unresolved without replay; retired credential remained rejected |
| Cancellation/bounds/drain | A dispatched a write during a 5 s WRITE pause, then cancelled the caller task; underlying work retained the one active slot, one waiter timed out, excess admission was 429; explicit shutdown finished within 3 s, closed admission returned 503, stateless route stayed available |
| Application restart | A's restart retained unexpired Redis state; while A stopped, pooled health requests reached B/C; A rejoined and the same Session read on all workers |
| Absolute TTL | A short-lived Session updated on B after 300 ms; another 400 ms later all workers rejected it, showing ordinary update did not refresh absolute expiry |
| Outage/restart | Stopping Redis made Session reads fail closed on A/B/C while stateless health stayed 200; restarting no-persistence Redis had no Session records; explicit application restart recovery rejected old Cookies and allowed a new Session |
| No-JS browser | Actual HTTP requests across A/B/C shared state and rotation; A replayed its original prepared Cookie header after B replacement; C rejected it; explicit creation recovered and B logout was rejected on C; one HttpOnly Cookie |

The browser replay is a separate real-browser observation of a retained prepared
header. It is not described as the runner's genuinely delayed in-flight HTTP
response. The selected pure-SSR contract permits login loss; neither observation
establishes unconditional replacement-cookie retention, real authentication,
screen-reader/IME evidence or broad browser qualification.

## Checks, cleanup and evidence

- Rust 1.88 `cargo check --manifest-path qualification/yt066b3/Cargo.toml --locked --offline`: PASS after correcting the fixture shutdown future to own its manager values.
- Stable all-target `cargo clippy --manifest-path qualification/yt066b3/Cargo.toml --locked --offline --all-targets -- -D warnings`: PASS.
- Stable `cargo fmt --manifest-path qualification/yt066b3/Cargo.toml -- --check` and in-memory Python compilation of the two fixture scripts: PASS.
- Actual offline-container build and qualification: PASS, exit 0. Existing YT-066a broad proof and unchanged b1/b2 suites were retained without rerun.
- Cleanup reused the existing reviewed exact-owner Docker removal/recheck method: five service containers, two networks, zero volumes, build container and generated application image all absent; vendor/build work removed. Shared pinned images retained.
- Chrome process gone, debug port closed and temporary browser profile removed. No owned server/browser/container or uncertain execution remains.

The sanitized report `target/yt066b3-20261011/yt066b3-1009004-5a0d472a.json`,
`execution-inputs.json`, `preflight.json` and `logs-5a0d472a/` retain source/build,
actual observations and cleanup. No raw Cookie, Session ID, CSRF, credentials,
ticket, synthetic payload or Redis error text is printed in qualification reports.
This is one focused actual execution, not a newly generalized validation framework.

## Review gate and remaining scope

Before review, source inspection identified two cleanup paths needing a bounded
runner-only correction: a first cancellation at the transition into teardown
could escape before Docker cleanup, and forcibly timing out a separate browser
driver could bypass that driver's Chrome cleanup. The runner now catches the
transition and invokes the existing browser probe in the same process, so its
owned Chrome cleanup unwinds before Docker cleanup. No provider, application,
Lua, dependency, Compose or qualification assertion changed.

Focused in-memory checks passed for normal completion, cancellation at the
transition, first cancellation during cleanup and report-write failure. A focused
actual Chrome cancellation before application navigation confirmed process
removal, closed debug port and removed profile. Evidence:
`target/yt066b3-20261011/focused-cleanup.json`. Python compilation passed after
the correction. The successful Docker behavior execution above was retained,
not rerun or relabeled: its source map keeps the original runner/driver hashes;
the focused cleanup record binds the later two script hashes. This is a cleanup
delta, not a provider requalification.

`python3 scripts/check_docs.py` passed its 75 required documents; the same existing
checker also passed all 99 repository Markdown documents with its input list
supplied in memory. Both runs checked 67 backlog items. Internal links, English
documentation, Markdown/backlog structure/DAG and tracked/untracked whitespace
checks passed. No validator framework was added.

### First independent review and material repair

Read-only reviewer thread `01a129cb-b8f6-74a0-9f99-6c838549fed4` reached terminal
exit 0 with observed `gpt-6-astra` / `high`. All 286 inputs, HEAD/status/index were
unchanged. Its canonical JSON returned **FAIL** with one P2: a first cancellation
during browser `client.close()` could still interrupt the browser teardown before
Chrome termination, even though the runner then cleaned Docker. This directly
violates owned cleanup and was admitted through the Materiality Gate.

The bounded repair shares the runner's cleanup-phase transition with the browser
probe. The browser's body-to-cleanup transition has an outer recovery boundary;
the runner enters non-raising cleanup mode before client/process teardown, while
recording any first cancellation as failure. A nested finalizer still terminates
Chrome if closing the DevTools client itself fails. No Redis/app behavior or
qualification assertion changed.

Four focused checks passed: normal completion, cancellation at client close,
cancellation at browser teardown transition, and actual Chrome cancellation at
client close using the actual runner signal handler. All cancelled results were
nonpassing; browser stop occurred before Docker cleanup, actual Chrome PID and
debug port were gone and the profile removed. Evidence is
`target/yt066b3-20261011/focused-browser-teardown.json`; the earlier focused record
retains its own prior script hashes. No Docker or Redis behavior was rerun. The
first reviewer explicitly required only this cleanup repair and affected checks,
not broader qualification. Its terminal evidence is retained under
`target/yt066b3-20261011/review-1/`.

### Final independent reviewer PASS

The second reviewer inspected the actual final b3 fixture/diff, adjacent
authoritative documents, retained execution evidence and focused cleanup repair.
It reached terminal exit 0 as local ordinary `codex exec`, role-directed by
`/home/bohe/.codex/agents-pipeline/agents/reviewer.md`, with read-only sandbox.
This was not WebCodex, `codex exec review`, native managed-role dispatch or
current-session self-review.

| Review evidence | Observed value |
|---|---|
| Thread | `01a129d2-3cce-72d3-a0de-bd286565dade` |
| Process | PID `1021477`, terminal exit 0, process absent |
| Model | Saved configuration expected `gpt-6-astra`; runtime observed `gpt-6-astra` |
| Effort | Shared resolver requested `high`; runtime observed `high`; observed-effort resolution has no conflict or degradation |
| Workspace profile | `health=ok`, `catalog_state=current`, `profile_eligibility=eligible` before dispatch |
| Review input | 286 tracked/untracked nonignored paths; source-map SHA-256 `9ad12f8487623a5d62f19616d6d51f1bfcd2985a63af846887a920d91e3a2612` |
| Review isolation | All input hashes, HEAD, worktree status and empty index unchanged by reviewer |

Canonical result:

```json
{"overall_status":"pass","issues":[],"required_followups":[],"optional_notes":[]}
```

Evidence is retained under `target/yt066b3-20261011/review-2/`. The review source
identity is separate from the original Docker execution identity. After PASS,
only Markdown completion status and this review receipt were reconciled;
reviewed code, test, dependency and topology bytes remained unchanged. The existing
documentation and whitespace checks passed again. No additional Docker execution
was needed. No b3 commit or push is authorized by the earlier b2 commit
instruction; these changes remain unstaged.

## Final changed files and task states

The b3 changes consist of these 18 files:

- `README.md`
- `docs/BACKLOG.md`
- `docs/M10-REMAINING-PLAN.md`
- `docs/M10G-VERIFICATION.md`
- `docs/PROVIDER-OPERATIONS.md`
- `docs/RELEASE-READINESS.md`
- `docs/YT-065-066-PLAN.md`
- `docs/YT-066B-REDIS-SESSION-DESIGN.md`
- `docs/YT-066B3-VERIFICATION.md`
- `qualification/yt066b3/Cargo.toml`
- `qualification/yt066b3/Cargo.lock`
- `qualification/yt066b3/Dockerfile`
- `qualification/yt066b3/README.md`
- `qualification/yt066b3/compose.yaml`
- `qualification/yt066b3/nginx.conf`
- `qualification/yt066b3/src/main.rs`
- `qualification/yt066b3/run.py`
- `qualification/yt066b3/browser.py`

YT-066b2 is locally committed; YT-066b3 is Done only at the bounded scope above.
The later [responsibility clarification](YT-066-PERSISTENCE-RECOVERY-PLAN.md)
keeps production topology/recovery policy and operational ownership with the
application/operator. The next framework step is bounded documentation closeout
and review; those deployment decisions are not mandatory Yetlet implementation.
Cancelled YT-066c is not reopened.

This qualifies only the selected single-host, single-primary, no-persistence
configuration. Automatic client reconnect recovery, durable AOF/RDB/crash/restore,
TLS/ACL/secrets, HA/Cluster, partitions/cross-host operations, load/SLO/RPO/RTO,
production ownership, SQL Session and public release remain unqualified. YT-066
remains In progress, YT-020 Blocked, YT-065/YT-067 Done at their prior bounded scopes.

## Later authorized local delivery — 2026-10-11

After the closeout above, the maintainer explicitly authorized peon to commit b3
before planning production persistence/recovery. Commit
`4da0f6c1879c9a56598caec7cad449cd9127c7de`, parent
`1f38d20b8ebaf9b679aa69ce461b495db1856db2`, tree
`e500a7baeed459e5a32ca7dedfe1da104f3fafd6`, contains exactly the 18 paths above.
All 286 input hashes were preserved and index/worktree were clean immediately
afterward. Saved/native peon model and effort matched `gpt-6-luna` / `high`.
The direct staged guard exited 0 under existing audit/builtin settings with
first-screen coverage, `warn` and two findings. No bypass, policy change, push
or new qualification execution occurred. The earlier unstaged/uncommitted
statements record the review closeout before this later authorization.

The [next planning document](YT-066-PERSISTENCE-RECOVERY-PLAN.md) is a separate,
unstaged documentation-only proposal, not production implementation or review PASS.
