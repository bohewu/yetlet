# YT-066b1 — Real Redis Semantics Prototype

**Date:** 2026-10-11
**Task:** YT-066 / M10g, bounded YT-066b1 private spike
**Status:** Bounded private prototype completed and independently reviewed PASS. The full-provider cookie-ordering gate remains unsatisfied. YT-066 remains In progress and YT-020 remains Blocked.
**Related:** [Design](YT-066B-REDIS-SESSION-DESIGN.md), [design review](YT-066B-PLANNING-VERIFICATION.md), [accepted plan](YT-065-066-PLAN.md), [fixture/runbook](../qualification/yt066b/README.md), [M10g](M10G-VERIFICATION.md).

## Authority and scope

The maintainer explicitly requested the real Redis semantics prototype and an independent reviewer, without WebCodex, in the original checkout while preserving existing changes. YT-066b0 design review thread `01a12873-7480-7ce3-99d4-0dc9afcb0e48` had terminal exit 0 and canonical PASS. The current slice starts at `d93e1da645dff1a9bedafb0fe940ce3ac9cb82fb` on `master`, with the two design documents untracked. The earlier local commits remain intact; there is no new commit, push, worktree, publication or settings change.

The authorized implementation is the unpublished `qualification/yt066b` Cargo workspace, its pinned single-service Compose fixture and focused product tests. No validator framework, public Redis provider, HTTP middleware integration or generalized store/pool/driver was added. Existing Memory Session source/API, root Cargo files, framework/browser crates, workflows, samples and YT-066a fixture remain unchanged. The approved plan authorizes these focused semantics tests; it does not authorize a broad repeat of completed YT-065/YT-066a qualification.

## Client and dependency decision

The direct redis-rs experiment uses **1.7.1**, with default features disabled and `tokio-comp`, `connection-manager`, `script` selected. Resolved Redis features are `aio`, `bytes`, `connection-manager`, `script`, `tokio-comp`. Exact registry checksums and versions are in the separate lockfile. Rust 1.88 resolved 131 packages including the experiment and dev/platform dependencies; the normal dependency graph has 80 packages including the experiment, with no Axum, reqwest, Hyper or cookie-store in that normal graph. Axum 0.8.9 and reqwest 0.12.28/cookies are test-only HTTP consumers.

The inspected redis-rs source confirms that a disconnected command returns an error and triggers reconnect, rather than replaying application work. `Script::invoke_async` performs one load/retry only on explicit `NOSCRIPT`. Its multiplexed connection documents that dropping a request future does not cancel the server command. This experiment therefore retains admission permits in owned workers after caller cancellation/timeout, and bounds drain/reaping explicitly. The actual pressure/cancellation/drain tests passed, so the design's conditional deadpool comparison was not triggered. This selects direct redis-rs only for the private spike; it does not accept a shipped provider's final API or automatic recovery policy.

Primary sources: [ConnectionManager](https://docs.rs/redis/latest/redis/aio/struct.ConnectionManager.html), [MultiplexedConnection](https://docs.rs/redis/latest/redis/aio/struct.MultiplexedConnection.html), [Script](https://docs.rs/redis/latest/redis/struct.Script.html), [Redis Lua semantics](https://redis.io/docs/latest/develop/programmability/eval-intro/), [official Redis image](https://hub.docker.com/_/redis). Version-specific API behavior was also read directly from the downloaded `redis-1.7.1` source; the design's comparison with deadpool/fred/tower-sessions remains as recorded, without implementing multiple backends.

The root workspace does not include this nested Cargo workspace. Its original manifest/lock and all original source hashes are unchanged; the root lock contains no Redis package. This establishes dependency isolation without rerunning unchanged Memory, browser or whole-workspace suites.

## Exact execution identity

| Item | Observed value |
|---|---|
| Committed baseline | `d93e1da645dff1a9bedafb0fe940ce3ac9cb82fb`; prototype source is additional uncommitted content |
| Lockfile SHA-256 | `b0c21fe49964ea2df6acfb824c6c2d25bf952a79d4d99168682e626ed5e80150` |
| Initial six-test runtime manifest digest | `c98e7bd8104dea2d489ce208f946c2a51c97e7c860f15c97df8c1508506d3fdc` |
| Final focused-test runtime manifest digest | `0694bdf4daf55087eca8a0c84427b09e2051199aa03358db2050a99e520d2524` |
| Redis | `8.2.10`, 64-bit, jemalloc 5.3.0 |
| Image | `redis@sha256:47670742d7924adbcdb404d1288b6327815b23141969c0939b8e5d61f78c2634` |
| Compose projects | Initial `yt066b1-72bf7fad`; focused counter-order check `yt066b1-b71cd68e` |
| Listeners | Initial `127.0.0.1:32768`; focused `127.0.0.1:32769`; no non-loopback published port |
| Persistence | `save ""`, `appendonly no`, tmpfs `/data`; no persistent volume |
| Execution toolchain | Rust 1.88.0 (`6b00bc3880198600130e1cf62b8f8a93494488cc`) |
| Stable lint toolchain | Rust 1.99.0 (`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`) |
| Evidence | Ignored `target/yt066b1-20261011/attempt1.json`, `tests-attempt1.txt`, `focused-counter-order.json`, `focused-bounds.txt`, `focused-semantics.txt`, `counter-order.diff`, dependency metadata/summary/tree, toolchain records and `validation.json` |

The runtime manifest hashes the lockfile, manifest, Compose file, Rust/Lua implementation and three test files. Its digest is SHA-256 of the sorted compact JSON path-to-SHA-256 mapping. After the initial six tests passed, source inspection found that releasing permits before decrementing the active-worker counter could transiently overcount the next worker. The sole runtime delta moves the permit guard before the counter guard so reverse drop order decrements first. The original file reconstructed from this exact delta matches its recorded initial hash. Lua, dependencies, fixture and all tests are unchanged.

The final source was rechecked against real Redis using only the two affected cases: `cancelled_waiter_keeps_capacity_until_reply_and_shutdown_reaps` and `lost_reply_after_commit_and_callback_values`, each with `--ignored --exact --test-threads=1` on Rust 1.88. Both passed with exit 0. The final eight files match that focused snapshot. The initial full result remains attached to its own source identity; it is not relabeled as a second full execution on the final source.

## Actual observations

`cargo +1.88.0 test --manifest-path qualification/yt066b/Cargo.toml --locked -- --ignored --test-threads=1` completed with exit 0: **six real-Redis tests passed**, no failures or ignored integration cases in that invocation. The initial full suite passed on its first execution. The later counter-order correction and its two focused passing checks are recorded separately above; both Redis fixtures were cleaned up.

| Area | Retained observation |
|---|---|
| Shared state/CAS | Three independent manager connections read the same typed state. Same-revision updates produce exactly one success and one conflict; the winner's state survives and ordinary update preserves expiry. These are connections in one test process, not three deployed apps. |
| Update/rotate/revoke | Racing update and rotation preserve the successful ordered update or reject the stale operation. Rotation changes CSRF/generation, two rotations have one winner, and every manager rejects retired identities. Tombstones contain no state, CSRF or replacement identity. |
| Independent outcomes | Dedicated result keys survive later update, rotation and revocation. Recorded conflict remains conflict. Repeating an acknowledged create returns historical status without recreating its retired identity. |
| Actual reply loss | Redis `CLIENT REPLY OFF` suppresses the mutation reply. The caller returns `OutcomeUnknown`, another manager observes the actual commit, mutates/rotates/revokes it, and still resolves the original operation. No callback or mutation replay is used. |
| Callback result | A non-Clone/non-Serialize `R` returns only on acknowledged success. Application rejection writes nothing. `CLIENT PAUSE ... WRITE` induces unknown outcome after one callback; `R` is dropped and later resolution returns commit status only, with no second callback invocation. |
| Expiry/corruption | Session expiry rejects access; expired operation evidence remains unresolved and its expired immutable create cannot execute again. Unsupported schema, invalid JSON, oversized stored payload and oversized outbound state are rejected. Unknown browser IDs cannot be adopted by create. |
| Admission | Exactly 64 retained outcomes are admitted per namespace; the next operation reports capacity without creating a Session. Tested manager limits include one active request/one waiter; excess requests return Busy/QueueTimeout. Observed peaks are one active worker in the pressure test and two outstanding reply-less workers in the lost-reply test. |
| Cancellation/drain | Cancelled callers leave admission occupied until the server reply. A 400 ms call timeout does not free the reply-less worker. Two-second shutdown aborts/joins the remaining worker(s); Redis subsequently reports zero blocked clients for the blocked-command case. |
| Connectivity | Killing the owned Redis client yields an unavailable read, followed by bounded reconnect without state loss. Recreating the manager retains state. A refused loopback connection fails closed. Whole-process application restart and whole-Redis outage/restart are later qualification cases. |
| Cookie ordering | The actual Axum/reqwest cookie jar preserves a replacement when the final check already sees rotation. Rotation after that check, an older prepared Set response, and a logout Clear delivered after a new login reproduce replacement-cookie loss while Redis still rejects retired identities. |

State serialization is deliberately a tiny typed JSON counter/note, with provider metadata in separate Hash fields and schema 1. No schema migration is guessed. Fixed bounds and error limitations are in the fixture runbook. Redis script isolation does not provide rollback after a script runtime failure; crash/restore/OOM recovery is not inferred from these controlled tests.

## Cookie decision and next boundary

**The complete replacement-cookie guarantee is not established.** Three HTTP cases intentionally assert the known counterexample. Their passing test result is evidence of the unresolved issue, not provider qualification. The fixture forces request finalization and prepared-response delivery order over a real HTTP socket; it does not claim browser/device testing or network packet-reordering coverage.

YT-066b2 is gated on a maintainer-selected issuance/fencing strategy compatible with SSR-first operation, or an explicit decision to narrow the guarantee while remaining experimental/private. Neither decision was made here. No mandatory client JavaScript, old-ID forwarding, or silent fallback was added. Stable provider integration, three application processes, production TLS/ACL/secrets, Redis persistence/restart, HA/Cluster, production SLO/RPO/RTO and YT-020 release remain unqualified.

## Local verification and cleanup

- Rust 1.88 compile and six actual Redis tests: PASS.
- Final counter-order correction: two affected real-Redis tests PASS; unchanged semantics retain the initial evidence.
- Stable `cargo clippy --manifest-path qualification/yt066b/Cargo.toml --locked --all-targets -- -D warnings`: PASS.
- `cargo fmt --manifest-path qualification/yt066b/Cargo.toml --check`: PASS.
- `docker compose -f qualification/yt066b/compose.yaml config --quiet`: PASS.
- `python3 scripts/check_docs.py`: PASS, 75 configured documents and 67 backlog items. Calling the same existing checker with the six additional planning/prototype documents included: PASS, 81 documents and 67 items; no validator changes.
- `git diff --check` and per-untracked-file `git diff --no-index --check /dev/null <file>`: PASS. No staged files; all repository Markdown passes the existing English-language check.
- Both Compose down operations exited 0. Each exact-project container/network/volume query returned **0 / 0 / 0**. No application image was built; the pulled immutable base image and Cargo build cache are ordinary local caches. HTTP servers shut down inside their tests; all cargo/test processes reached terminal exit 0.

No previously sufficient Docker/browser/native suite was rerun. The two earlier commits, source history and YT-065/YT-066a retained reports remain intact. YT-065 and YT-067 keep their bounded Done labels; YT-066 stays In progress, and YT-020 stays Blocked.

## Changed files

The worktree contains these fourteen unstaged paths. The two pre-existing untracked YT-066b design documents are retained with a brief pointer to this later execution record. The other 250 files from the 254-file starting snapshot remain byte-identical; only the four listed existing documents changed, plus ten new files.

- `docs/BACKLOG.md`
- `docs/M10G-VERIFICATION.md`
- `docs/YT-066B-PLANNING-VERIFICATION.md`
- `docs/YT-066B-REDIS-SESSION-DESIGN.md`
- `docs/YT-066B1-VERIFICATION.md`
- `qualification/yt066b/Cargo.toml`
- `qualification/yt066b/Cargo.lock`
- `qualification/yt066b/README.md`
- `qualification/yt066b/compose.yaml`
- `qualification/yt066b/src/lib.rs`
- `qualification/yt066b/src/session.lua`
- `qualification/yt066b/tests/bounds.rs`
- `qualification/yt066b/tests/cookie_order.rs`
- `qualification/yt066b/tests/semantics.rs`

## Independent reviewer

The independent reviewer accepted this bounded prototype and its evidence. This does not satisfy the unresolved full-provider cookie guarantee or authorize YT-066b2 expansion.

| Item | Retained evidence |
|---|---|
| Dispatch | Local ordinary `codex exec`, role-directed process; no WebCodex |
| Role contract | `/home/bohe/.codex/agents-pipeline/agents/reviewer.md`; saved workspace reviewer configuration |
| Workspace preflight | `health=ok`, `catalog_state=current`, `profile_eligibility=eligible`, `project_trust=trusted`, `configuration_compatibility=current` |
| Resolver | Adaptive, task intent `review`, context `ad-hoc-review`, proven `strong` tier; `multi_file`, `data_integrity`, `concurrency_or_ordering`, `bounded_tradeoff` signals; no conflict or degradation |
| Thread | `01a12884-69a7-7e20-beb5-9710e561c4d9` |
| Model | Saved/expected and observed `gpt-6-astra` |
| Effort | Shared-resolver requested and session-observed `high` |
| Sandbox | Requested and session-observed `read-only` |
| Terminal state | Completed, exit 0; same process observed to completion and no longer active |
| Result | Canonical JSON PASS; `issues=[]`, `required_followups=[]`, `optional_notes=[]` |
| Preservation | All 264 tracked/untracked source and document files, HEAD and Git status unchanged across review; zero staged files |
| Reviewed file-map digest | `1c743ef18f1e29336b1a5e07c2fc2aa31942dcad3e63556433ab21ffb40610ca` |
| Local evidence | Ignored `target/yt066b1-20261011/review/review-evidence.json`, `before.json`, `process.json` and `result.json` |

This is an independent role-directed review, not native managed-role identity or formal assurance. After terminal PASS, only this record and the M10g/backlog summaries were updated to record the result; runtime bytes and all reviewed implementation/test files remain unchanged. Final document/link/whitespace checks passed. No commit or push occurred.
