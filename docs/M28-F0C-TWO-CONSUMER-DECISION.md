# F0-C — Two-consumer composition abstraction decision

- **Date:** 2026-11-18.
- **Status:** Private research recommendation; independent review is recorded separately. No architectural acceptance or public API promotion.
- **Inspected HEAD:** `b93cf249f3bf8a1a4d1cd6dbd589ec92eea33550` on `docs/m28-composable-architecture`.
- **Scope:** Compare the committed F0-B1 Headless and F0-B2 Pure Web consumers. Produce this document and ignored evidence/handoff only.
- **Parent contract:** [F0-A](M28-F0A-PRIVATE-COMPOSITION-CONTRACT.md), [M28-E](M28-E-PROMOTION-DECISION.md), and [ADR 0013](adr/0013-composable-application-and-hosts.md). ADR 0013 stays **Proposed**; physical D4 stays **Not verified**.
- **Authority:** The owner requested this comparison and independent actual review. F0-C is a candidate label, not a new YT backlog identifier. No helper, Host trait, F0-B3, SDK/CLI/feature change, Stage, Commit or Push is authorized.

## 1. Decision

**Keep the two concrete application-owned roots. Share the written ownership principles; extract no common Rust lifecycle helper or Host trait for these two current consumers.**

Inspection found **zero ownership-sensitive operations meeting the extraction threshold**: the same operation must already be implemented in both consumers, have compatible failure/completion semantics, and become simpler at both call sites through a type-safe common implementation. This is a scoped design judgment, not a syntactic clone metric or proof that future abstraction can never help.

Headless has an actual persistent local owner with two capabilities and synchronous stop. Pure Web has a concrete Router configuration function; the selected HTTP test caller separately owns binding, native serving and asynchronous drain. Generalizing these different operations would introduce adapters and policy without removing a demonstrated repeated algorithm.

The successful outcome of F0-C is an evidence-based decision. It does not require a new type to exist and does not start the optional F0-B3 implementation.

## 2. Source map and accepted evidence

Line references in this document refer to the inspected HEAD above. They are navigation aids; the ignored source manifest contains SHA256 identities.

| Key | Inspected source | Relevant locations |
|---|---|---|
| H | [Headless composition](../examples/composable-host-research/src/headless_composition_f0b1.rs) | InputSource 60–113; observer 118–157; root construction 160–190; readiness/start 197–218; submission/step 221–247; stop 249–267; five tests 285–428 |
| S | [C1 simulation](../examples/composable-host-research/src/simulation.rs) | Simulation owns Rc at 132–169; HeadlessRunner 243–259 |
| D | [Headless demo](../examples/composable-host-research/src/bin/f0b1-headless-demo.rs) | Concrete synchronous call sequence 7–28 |
| W | [Pure Web composition](../examples/composition-web-research/src/lib.rs) | SiteState/handlers 14–53; configure_router 55–70; test-only module 72–278; bind 160–172; serving/drain 175–275 |
| A | [Existing Axum App](../crates/yetlet/src/server_axum/app.rs) | Router ownership 46–48; native state bounds 154–157; into_router 170–173; try_configure 191–198; with_state 277–284 |
| HM | [Headless manifest](../examples/composable-host-research/Cargo.toml) | Independent unpublished std-only research workspace |
| WM | [Web manifest](../examples/composition-web-research/Cargo.toml) and [private lock](../examples/composition-web-research/Cargo.lock) | Independent unpublished workspace; existing server-axum selection, Axum 0.8.9 and Tokio 1.53.3 |

The exact H SHA256 is `4fa03e63e135eaedf7af1b4793549941a64c0d410dc4554b15cd4f8033f73a45`; W is `889fae1ba1875488e3936900186a94b0816bfddbced8daf52d25f50525365dab`. The F0-A contract is `9986fcef8c746dcafba470699e75d727ae517172995e35a39d27620b6dfb73d4`.

| Reused evidence | Accepted scope | What it does not establish |
|---|---|---|
| F0-A contract and its actual R1 PASS | Explicit ownership, distinct completion, native Web preservation and extraction threshold | Runtime qualification of an unimplemented abstraction |
| F0-B1 committed at 348b8a2; five focused tests PASS; Reviewer wc_job_07W0L0gfugIEGixv terminal exit0, actual PASS | One C1 authority, local immutable observation, startup unwind, bounded input, stop/failure semantics and join not applicable | A worker, async scheduler, socket, HTTP or universal Host contract |
| F0-B2 committed at b93cf24; three focused tests PASS; Reviewer wc_job_VDCnbnLibhdtRcBe terminal exit0, actual PASS | Router SSR/API responses, native bind conflict and one real Ubuntu loopback HTTP/graceful in-flight drain scenario | Hard termination deadline, detached handler-task supervision, TLS, production throughput, Windows or external LAN |
| B0 seven selected source blobs still match the 30b0625 tested baseline | Existing Web/App/Actions/feature sources remain preserved | A newly executed B0 regression or zero-overhead benchmark |

Original raw logs, failures and Reviewer outputs remain in their original F0-A/B1/B2 directories. F0-C reads them and compares source identities; it does not rerun the tests, HTTP E2E or historical Reviewers. F0-B1's already reviewed module declaration was added after the F0-A baseline; this historical change is explicitly reconciled rather than mislabeled current drift. F0-C's own document checks and independent review are new evidence for this document only.

## 3. Application-owned resources

### Headless

H 160–190 owns one `HeadlessRunner`, one finite `InputSource` and one `SnapshotObserver`. The runner owns the C1 `Simulation` (S 243–259); neither attachment receives unrestricted mutable access to it. The source stores complete input frames and the observer holds only the latest immutable `Rc<Snapshot>`.

The caller invokes `submit_frame` and `drive_one(&mut self)`. An admitted frame has not yet been applied. Only the explicit drive calls the unchanged reducer. Empty input is an explicit tick; an empty queue returns `NoFrame` without advancing time (H 221–247, 324–325). This root allocates no task, thread, runtime, listener, renderer or transport.

Stopping changes capability state and clears pending frames. It does not destroy the root's runner or require all snapshot clones to disappear. The retained publication stays readable after stop (H 151–157, 249–267, 375–389). `Cleanup::Complete` is complete local capability shutdown for this finite root; `Join::NotApplicable` truthfully reports that no worker was created.

### Pure Web

W 58–69 consumes ordinary `SiteState`, builds the existing `App<SiteState>`, resolves state with `with_state::<()>`, and returns native `Router<()>`. The normal root owns configuration until return; the caller then owns the returned Router. Its two routes are two handlers, not two Hosts.

Listener binding, a one-shot shutdown signal, native serving and the original serving `JoinHandle` occur in the **test caller** (W 175–198). The test demonstrates how an application can own that optional serving lifetime; it does not implement a reusable production WebRoot or a general failure supervisor. Tokio is provided by the caller's test runtime, not created or owned by `configure_router`. Native Axum owns its internal connection handling.

The in-flight request gate is `cfg(test)` only (W 18–19, 44–51, 72–101). Ordinary Web handlers do not acquire the Headless input queue, C1 owner, tick, snapshot policy or a synthetic lifecycle attachment.

## 4. Readiness, failure, stop and completion

| Aspect | Headless operation and evidence | Web operation and evidence | Decision consequence |
|---|---|---|---|
| Prepared | Validate finite capacity and domain generation; own three concrete fields (H 68–77, 171–190) | Reject empty title/status during Router setup (W 58–67) | Both are fallible construction, already expressed by Rust Result; error/resource types differ |
| Ready | Root, source and observer all Running; initial snapshot attached before readiness (H 197–218) | Configured Router, bound listener and successful HTTP response are separate observations (W 55–69, 160–172, 181–225) | A shared boolean would retain consumer-specific meaning or require new stages |
| Required startup failure | Observer attachment can fail after source.start; close source/observer and retain the original cause (H 209–215; test 330–350) | Router validation fails before any listener; a native second bind fails with AddrInUse (W 103–112, 160–172) | Web does not implement the same two-attachment rollback; no shared rollback operation is available |
| Running failure | Rejected C1 tick retains prior publication, latches cause and stops input (H 240–244; test 392–417) | HTTP status and connection outcomes stay native; there is no root primary_failure field or common escalation policy | An ordinary HTTP error is not automatically application failure |
| Incoming work | Root and source check Running; finite frame queue enforces capacity and input count (H 85–104, 223–227) | Native Router/HTTP request handling; graceful shutdown is selected by the test caller | Closing a local frame queue and native HTTP shutdown are different policies and mechanisms |
| Request stop | Synchronous, idempotent local stop; discard buffered unapplied frames, close observer, preserve Failed if latched (H 106–113, 255–267) | Consume the one-shot sender and let native Axum observe shutdown (W 237–239) | One-shot sender consumption is not the same repeatable stop method |
| Completion | Stop returns after local capability closure; join not applicable (H 255–267) | Await the same serving handle after releasing the in-flight request; check native result, rebind original port (W 240–272) | A status report cannot replace owned asynchronous completion |
| Timeout | No asynchronous wait or deadline in H | First wait times out, handle stays owned and unfinished, later wait observes completion (W 240–266) | Headless has no duplicated pending-owner timeout operation |
| Executor lifetime | None is required | Caller runtime remains present after drain (W 273–275) | Ownership principle is common; there is no common runtime-management code |

The Web test's successful HTTP responses establish that this selected serving path was driven. Router construction, binding and task creation individually do not establish that observation. One response is not a perpetual health guarantee.

Headless discards **unapplied queued frames**; Web allows the selected **already in-flight request** to finish. A universal drain policy would have to preserve this distinction explicitly. It could not replace both behaviors with one common meaning of empty, closed or done.

The Web result nesting also matters: an observation timeout, a task `JoinError`, and the serve future's `io::Result<()>` are distinct types (W 260–266). For the pinned native Axum implementation, graceful serving resolves `Ok(())` after shutdown/drain; this test is not evidence of arbitrary fatal-serve-error recovery. No detached handler-owned jobs are qualified.

## 5. Caller-local Rc and native async type boundaries

| Boundary | Actual requirement | Consequence for a proposed shared abstraction |
|---|---|---|
| H/S authority and publication | Simulation and observer contain `Rc<Snapshot>`; operations borrow `&mut self` or clone a local Rc | Keep the root caller-local. Making the whole root Send would require a material ownership redesign |
| Existing App / Router state | A 154–157 requires `S: Clone + Send + Sync + 'static`; native Router uses S as missing state | Preserve native Web bounds at this boundary. Router<()> means state has been supplied, not that SiteState disappeared |
| Axum handlers | Pinned Handler requires a Send future and native handler bounds | A common local trait does not relax the selected HTTP implementation's requirements |
| Selected Web task | W 192 uses tokio::spawn; pinned spawn requires `F: Future + Send + 'static` and `F::Output: Send + 'static` | The test's current_thread runtime does not remove this API's bounds |
| Native graceful shutdown | The selected Axum shutdown signal future is Send + 'static | Do not capture the Headless Rc root in it merely to make lifecycle signatures match |
| Completion observation | W 241 and 262 borrow `&mut serving`, preserving the exact handle across timeout | A future helper must preserve this owner, not return only a timeout label |

Rust's [Rc documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html), the pinned [Tokio spawn source](https://docs.rs/tokio/1.53.3/src/tokio/task/spawn.rs.html), [Tokio JoinHandle](https://docs.rs/tokio/1.53.3/tokio/task/struct.JoinHandle.html), [Axum Handler source](https://docs.rs/axum/0.8.9/src/axum/handler/mod.rs.html), and [Axum serve source](https://docs.rs/axum/0.8.9/src/axum/serve/mod.rs.html) support these boundaries. F0-C also inspected the matching installed Cargo source: Tokio spawn 174–177; Axum Handler 148–150; graceful-shutdown signal 151–153 and IntoFuture 332–348. Exact file identities are recorded in ignored evidence.

A native task API can impose Send even when the selected executor is currently one-threaded. Conversely, being async does not itself imply that every value must be Send: local-only execution patterns exist, but F0-B1 needs none and F0-B2's selected native bounds remain. Do not impose Tokio, Arc conversion, unsafe Send or a local-executor adapter to make these roots appear uniform.

## 6. Duplication audit

The comparison includes both the normal Web configuration function and its exercised test caller. Counting the caller's operations gives the strongest available case for extraction; it still does not establish a shared ownership algorithm.

| Candidate shared operation | Exact compared locations | What actually repeats | Extraction result |
|---|---|---|---|
| Validate and construct | H 171–190; W 58–69 | Result propagation and ordinary local ownership | Language-level composition already handles this; validation and returned resource differ |
| Start dependencies and unwind | H 205–218; W 181–198 | A general ordering obligation | Only H implements attachment rollback; Web config/bind/spawn has different ownership stages |
| Publish or expose state | H 230–252 and S 156–169; W 39–69 | Access to application-selected data | Local immutable tick snapshots and native handler state are different domain policies |
| Close admission and dispose pending work | H 85–113, 223–227, 255–267; W 237–258 | Intention to stop future work safely | Queue discard and native graceful request completion are not equivalent operations |
| Preserve cause and cleanup state | H 210–215, 240–267; W 58–67, 167–169, 260–266 | Need to report truthful outcomes | H latches a root cause; W retains concrete setup/bind/task/native results, not an identical report algorithm |
| Wait for owned completion without losing owner | H 255–267; W 240–266 | Only the written completion invariant | H has no handle/future to wait for; the pending-owner operation exists only in W |
| Preserve caller runtime | H 160–190; W 273–275 | Do not destroy resources owned elsewhere | H has no runtime; neither root contains a common runtime shutdown implementation |

There is small **within-Headless** repetition: start-failure handling and stop both close the concrete source and observer (H 212–213, 256–257). It is not duplication across the two consumers. F0-C does not refactor it or use it to justify a common Web/Headless abstraction.

Similar names, Result assertions, setup functions, or test timing utilities are insufficient. No new benchmark, validator or synthetic consumer is needed to decide that this threshold is unmet.

## 7. Alternatives and actual costs

| Alternative | Possible benefit | Necessary design/maintenance cost for these consumers | Decision |
|---|---|---|---|
| Concrete roots plus written invariants | Native types, ownership and failure paths remain visible; existing App.try_configure already composes setup | Maintain two small, intentionally different roots and their own evidence | **Retain** |
| Narrow private typed helper | Useful if a specific operation is truly duplicated; can remain static and avoid global Send bounds | Must identify a real operation, preserve consumer-specific inputs/errors, handle local versus asynchronous completion, expose pending ownership, and choose a shared-source/dependency location across separate workspaces | **Do not extract now** |
| Generic Host trait with static dispatch | Can describe capabilities using associated types without boxing, a registry or universal Send | Requires a meaningful common caller/algorithm, stage/readiness semantics, error/completion types, and an adapter around the Web serving lifetime; H has no native asynchronous join operation | **No demonstrated value for F0-C** |
| Dynamic Host registry / uniform spawned Hosts | Could support separately required runtime membership | Adds erasure/object-safety/future choices, registry/failure policy and scheduling assumptions absent from either requirement | **Outside scope and unjustified** |

### Why a private helper does not yet simplify the call sites

A synchronous cleanup helper accepting an operation callback could close H's source/observer, but cannot claim Web drain completion before awaiting its native task. A generic asynchronous completion helper could represent H's result as an immediately ready value, but that wrapping adds no useful ownership operation to H. A report-only shared enum would describe both outcomes without owning, stopping or joining anything.

A callback-based start/rollback helper would still need H's concrete phase and failure logic inside one set of closures, while Web must supply different config/bind/serve ownership decisions. There is no repeated two-consumer rollback body to remove. This is an alternative analysis, not a helper specification or implementation.

Cross-workspace reuse would also require an explicit placement choice: shared source, a new private package, or a dependency between consumers. None is automatically required by Rust, but each is new coupling to justify. Keep the Headless normal dependency graph std-only and the Web workspace independent of game/GPU/UDP code.

### Why rejecting a Host trait is not a claim about traits in general

A static trait can use associated error/output types, allow local values and avoid heap allocation or runtime dispatch. It does **not inherently** require Send, Arc, boxing, an OS thread, a process or an executor. Those costs belong to particular trait/scheduler/registry designs.

For this pair, a uniform contract would need either capability-specific methods/associated types or explicit not-applicable paths for readiness and completion. H's synchronous StopReport and W's optional owned serving handle cannot be treated as the same resource. No common algorithm currently consumes both roots. Defining that contract now would add concepts and adapters while retaining the consumer-specific work.

No alternative has been implemented or benchmarked. These are concrete source/type/maintenance costs and design inferences, not measured latency, allocation counts, throughput, compile-time savings or a zero-cost claim.

## 8. Reopening condition and next owner decision

Reconsider one private helper only when an authorized real consumer exposes an operation already duplicated in another actual consumer. The proposal must identify both source bodies, compatible failure and completion semantics, and the exact code removed at both call sites. It must preserve:

1. Caller-local Rc where applicable, with native Web type bounds left at their real boundaries.
2. Ownership of pending work across timeout or partial failure.
3. Explicit caller runtime ownership and only actual owned work being joined.
4. The ability to use either consumer without selecting the helper.
5. Simpler call sites and failure paths after accounting for new types, callbacks and dependency placement.

Two consumers alone are not a sufficient count-based trigger. A third synthetic consumer created only to satisfy the threshold would not supply a product need.

**Recommended next owner decision:** retain concrete roots and record the reviewed F0-C result. If the owner wants Git delivery, authorize only this document for strict/builtin guarded local Commit in a separate turn. Do not proceed automatically to F0-B3. A future implementation request should first name a real workload or duplicated operation that the existing roots cannot express simply.

F0-C does not accept ADR 0013, change public Yetlet APIs, SDK/CLI, Cargo features, the original M27-C work, or any release/merge status. Physical D4 remains **Not verified**; successful test-only-relayed D4VM evidence remains a separate historical supplement.

## 9. Review and verification boundary

The changed deliverable is this English decision document. Task-fit checks are repository documentation validation and new-file whitespace/link inspection. Runtime tests, Clippy, formatting, native HTTP E2E and historical Reviewer runs remain reused because their sources did not change.

The actual F0-C `cargo xtask check docs` run returned exit 1: all 18 reported failures are CJK lines in the unchanged, ignored `.pipeline-output/m28-f0b2-20261118/COMMITTED-TWO-CONSUMER-NEXT-HANDOFF.md` supplied as the entry handoff. The checker recursively includes that directory. Its required-document/backlog traversal completed, and it reported no F0-C-document error. This is an inherited documentation-scope failure, not an overall docs PASS. The original handoff, checker and failure log are preserved. The new file separately passes English-content, relative-link, final-newline and no-index whitespace checks; no global validation gate is silently waived.

The independent reviewer must inspect this document against H/W/S/A and the original acceptance records, checking the six requested comparison topics, actual duplication versus vocabulary, honest static-trait costs, pending ownership and the restricted evidence claims. Ordinary ad hoc actual PASS is required; neither a conversational summary nor self-review satisfies that gate.

Exact new document/evidence hashes, current profile/resolver observations, the new Reviewer Job/output, terminal state, and closeout facts belong under `.pipeline-output/m28-f0c-20261118/`. The final ignored handoff records the actual outcome without retroactively upgrading any earlier test or architectural gate.
