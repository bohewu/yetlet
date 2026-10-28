# YT-096 — Existing remote history boundary for one source tag

Date: 2026-10-28. **Historical design proposal; later implementation and one-use native execution completed under separate authority.**
This packet preserves the original design and review scope.
D1–D5 remain Adopted (policy); D6 Pending; YT-096 In progress; YT-020 Blocked.
Design acceptance alone did not resume tag creation, Push or the hosted build.

The [actual hosted/custody checkpoint](YT-096-HOSTED-DRILL-PLAN.md#actual-hosted-and-custody-checkpoint--2026-10-28)
records the later installed `agents_pipeline` 0.44.0 managed hook upgrade,
strict/builtin doctor/preparation/actual native-hook PASS with zero outgoing
commits, consumed one-use binding, successful source-only tag Push, hosted build,
protected verifier and repository-external owner-host retention. The 13 historical
findings remain unresolved accepted-existing debt. The original design Reviewer
P2 FAIL and focused followup PASS (zero issues/required followups) are retained:
the correction distinguished direct planned-input scanning from native hook
execution. A later observation-wrapper failure concerned stdout versus stderr
after the successful Push; correction observed that same Push and remote refs,
without repush or rearm. This factual update grants no further operation.

All proposed/current-tool wording below describes the original inspected source
and design snapshot, not a current installation claim or reusable authorization.

## Problem and preserved evidence

The [hosted drill plan](YT-096-HOSTED-DRILL-PLAN.md) stopped at a direct
strict/builtin `check --pre-push` scanner assessment using planned tag-only
stdin, before any tag creation or Push. Git did not generate that input and
the installed native hook was not exercised. The selected
`C_release` is `3eacfe31f5e3281411d3bc4653a6baccab668b30`, with tree
`86385d60cb4c3e595c670dbacce117ebeeb1a993`; ordinary CI was accepted.
The recorded preflight observed that remote `refs/heads/master` equaled this
commit and that local and remote `refs/tags/v0.1.0` were absent.
These are dated observations, not current remote guarantees.

The strict/builtin first-screen checked 23 commits and 1,760 changed-file scan
operations, reporting 13 blocking findings in two historical commits across six
historical documents: six `PATH_PERSONAL` and seven `PATH_CURRENT_MACHINE`.
Coverage was `partial-first-screen`. No local or remote tag was created and
zero hosted build attempts were dispatched. The historical finding values are
not reproduced here; the original failure remains valid and retained.

The owner accepted the existing public `master` history for the earlier delivery
boundary. That acceptance did not make the history clean or supply a supported
tag-only Guard boundary. At that original stop, the installed Guard had no accepted-remote/base
override. A new tag can share a real branch-update row's base only when both
tips match; an isolated new tag falls back to its full ancestry.
Fabricating an unchanged branch row would misrepresent Git's actual push.

Local evidence labels are `authority.json`, `preflight.json`,
`guard-block-summary.json` and `tag-guard-inspection.json` in the
`yt096-hosted-build-20261028` run. They establish the stop, not future authority.

## Proposed scope and invariant

Adopt an explicit, one-use acceptance of existing remote history for one new
lightweight source tag. The accepted public repository is
[`bohewu/yetlet`](https://github.com/bohewu/yetlet). Acceptance must name its actual
single effective push endpoint, `refs/heads/master`, full commit OID `C`,
`refs/tags/v0.1.0`, and the required `strict` profile with `builtin` engine.
For this proposal, `C` is the selected full `C_release` above.

The optimized native push must contain exactly one four-field update row:

```text
refs/tags/v0.1.0 C refs/tags/v0.1.0 ZERO
```

`C` and `ZERO` denote the full commit OID and the object-format-appropriate zero
OID, not literal input. The actual local tag ref's raw object must be a commit
with OID `C`; the live approved remote branch tip must also equal `C`.
Only then is the outgoing commit set `C..C` empty. This deliberately excludes
operator-accepted existing public ancestry from this exact request's scan scope.
It does not suppress any detector finding within an outgoing commit.

The first slice excludes annotated/signed tags, arbitrary ancestors, new branches,
branch-plus-tag or other multi-ref pushes, deletions, reused/moved tags and tag
renaming. Lightweight tags add no tag-message content. Annotated support would
need its own object/content and identity checks and a separate review.
Existing default pre-push behavior, including full ancestry for new refs, stays
unchanged when this feature is not explicitly adopted.

## Trust and remote identity

The new scope trusts the owner's deliberate acceptance of the named public
history, the local operator/Git configuration and the authenticated remote peer.
It does not prove owner identity or defend against a local administrator who can
rewrite the tool, hook, configuration or record. Remote administrators remain
trusted. A public repository is not evidence that historical credentials are safe.

Resolve the configured destination to exactly one effective **push** endpoint,
including Git URL rewrites and `pushurl`, rather than using its fetch URL or
remote alias as identity. Bind a canonical credential-free transport/host/port/
repository-path identity plus the approved public repository identity. Preserve
meaningful path and transport differences; do not collapse distinct endpoints.
Ambiguous canonicalization, multiple push endpoints or an identity mismatch stop.
An endpoint embedded in the hook arguments must agree with this binding.

Use argument-vector Git calls with bounded time/output and exact full refs.
Permit only reviewed HTTPS or SSH endpoint forms with verified TLS/SSH peer
identity; credentials remain with normal Git authentication. Do not add tokens,
GitHub write scopes, authentication services, arbitrary shell or remote-helper
commands. Unsupported transports, ambiguous SSH routing/URL rewrites or unsafe
endpoint forms fail closed. Existing Git authentication facilities are trusted
operator configuration, not a reason to expose credentials in the new record.

Read the exact branch and tag using authenticated `git ls-remote` against that
effective endpoint. Suppress interactive prompting for this bounded assessment;
an authentication failure stops. A successful, complete response must contain
exactly the expected full branch ref with OID `C` and no exact tag ref.
Use fixed full names and compare every returned ref exactly: Git's patterns are
globs, and `--refs` suppresses peeled tag rows. Malformed, duplicate, unexpected
or truncated output is an error, not tag absence. A nonzero exit is never absence.
Local tracking refs and caller-supplied arbitrary `--base` values prove nothing
about the remote. There is no automatic baseline discovery or fallback.

## Proposed preparation flow

`prepare-remote-tag` is a proposed future command name, **not an existing CLI**.
Its proposed assessment scope is `new-tag-plan`; it must work before a tag exists.
Do not invoke hypothetical commands from this document.

1. Require explicit operator acceptance of the named endpoint/repository,
   accepted full branch ref and full source SHA, exact new tag ref, and known
   historical debt. Resolve no shortened SHA, floating branch or inferred source.
2. Validate a normal supported repository/Git-metadata layout and raw local
   commit `C`; reject missing/non-commit objects, shallow/incomplete history or
   local history substitutions that would invalidate the accepted identity.
   Check that the intended local tag does not already exist.
3. Resolve and validate the effective single push destination. Perform the exact
   live branch/tag reads above; require branch OID `C` and absent remote tag.
4. Resolve the effective Guard policy and initialize its engine, even though
   zero new commits will be scanned. Require `strict` and `builtin`; a conflicting
   override, invalid policy or engine initialization error stops.
5. Assess the plan without synthetic pre-push stdin or a real tag. Report
   `new-tag-plan`, the accepted boundary and zero outgoing commits. Assessments
   and direct checks are nonconsuming; they grant no Push or hosted authority.
6. Only after explicit acceptance and a complete passing assessment, persist the
   exact private binding for the future native hook. Report whether it was
   written. A read-only assessment alone must not arm or replace a binding.

Preparation never creates a tag, stages/commits, pushes, modifies policy or
installs hooks. Failure writes no usable acceptance record. No automatic retry,
replacement of a pending record or rearming after consumption is permitted.

## Minimal local binding

Use one small, versioned, private regular record under the repository's normal
`.git` metadata, analogous in purpose to the existing one-time policy
acknowledgment. It is nontracked; no registry, service or general allowlist is
needed. The exact filename and CLI spelling are future implementation details.

The strict schema binds repository identity, effective single credential-free
push endpoint, accepted full ref and full OID, exact tag ref and raw commit OID,
`strict`, `builtin`, and one-use state. Bind it to the local repository metadata
identity so copying it into a different clone does not authorize that clone.
Keep any local path identity inside private metadata, never in tracked reports.
Reject unknown fields, wrong schema/type, oversized/unreadable files, symlinks,
path redirection and unsupported shared/linked-worktree layouts.
Use private permissions, exclusive creation and atomic publication; do not
overwrite a pending record or follow an existing link.

Record selection must be explicit and fail closed: an armed record selects the
narrow path, so a mismatched or unsupported push while armed stops. A selected
path with missing, changed or already-used binding cannot silently become PASS
or rediscover a baseline. Without adoption, the ordinary full-history path still
applies. A consumed-state marker may preserve the one-use outcome; it is never
a reusable grant. The implementation must distinguish these states without
introducing a general bypass flag for direct checks.

## Actual native hook and consumption

The reviewed managed `pre-push` hook must forward Git's actual remote-name and
remote-location arguments as well as untouched four-field stdin to the Guard.
The installed hook currently forwards stdin only. This design therefore requires
a separately reviewed source implementation and explicit installer/hook upgrade;
manual hook edits, `--no-verify` and fabricated stdin are prohibited.

At hook time, perform all of the following before allowing the push:

1. Validate real hook arguments, exact one-row input, zero remote OID and equal
   exact local/remote tag refs. Reject every unsupported operation in this path.
2. Verify the binding and actual configured push endpoint against the hook's
   real destination. Re-read the actual local tag ref and raw object type/OID;
   peeling an annotated tag to `C` is insufficient.
3. Freshly repeat the exact authenticated remote branch/tag queries. Require
   branch tip `C`, absent tag and unambiguous complete reads. A preparation-time
   response is not a hook-time remote check.
4. Revalidate complete local identity/history and effective strict/builtin
   policy, initialize the engine, and complete the scoped Guard assessment.
   Recheck local ref/configuration/record stability before the final decision.
5. Only after the full hook assessment passes, atomically consume that exact
   binding immediately before permitting the push. Exclusive synchronization and
   compare-before-consume must permit at most one concurrent hook to succeed.
   Consumption failure stops; a racing process cannot receive a second grant.

Direct plan/check calls never consume. Failure before consumption grants no Push.
Failure or interruption after successful consumption, including a later transport
failure, leaves the binding used. No automatic rearming or transport retry occurs.
The operator must inspect actual remote state and explicitly prepare anew if the
tag remains absent and the same exact acceptance is still appropriate.
If remote completion is uncertain, stop rather than assume the push failed.

## Push semantics, races and stop rules

The authorized operation must be a normal nonforce Git push of exactly this tag:
no `+`, `--force`, force lease, seed ref through an API, fake branch-update row,
tag replacement, deletion or overwrite. Preserve native hooks and history.
Disable an inherited signed-push request for this unsigned operation through the
reviewed invocation; this does not authorize signing or change global Git config.

Hook stdin/arguments do not expose the caller's force options. Their absence
cannot be proved by this hook contract; the separately authorized invocation
must be inspected. Zero old OID and fresh tag absence are necessary checks,
not evidence that the caller omitted a force option.
Normal nonforce tag creation supplies server-side name-collision behavior.
It does not lock `master` at `C`: separate fresh reads have a time-of-check to
time-of-use window, and the remote branch may move before the tag is received.
This design retains that trusted-remote/administrator and race limit explicitly.

After a separately authorized push, read the exact remote tag and branch again;
require the tag's raw OID to equal commit `C` and record branch observation.
A tag mismatch, unexpected branch change or ambiguous response stops downstream
execution. Preserve the observed result; never repair it by moving/deleting a tag.
No check here automatically dispatches a build or consumes a hosted attempt.

Stop on wrong SHA/ref/type/endpoint, changed/missing/used record, remote tag
presence, moved/denied/unavailable remote, malformed/ambiguous reads, incomplete
history, changed policy, engine/runtime failure or failed atomic consumption.
No audit fallback, detector ignore, rule suppression or weakened enforcement is
introduced. The historical 13 findings remain unresolved accepted-existing debt.

## Honest reporting and privacy

Successful planned output must identify `new-tag-plan`, effective strict/builtin,
accepted full ref/OID and exact tag/commit, zero outgoing commits and whether the
binding is pending or consumed. Native success identifies the actual pre-push
scope. Label the result as a scoped first-screen with operator-accepted remote
history, never full-history clean, findings remediated or release qualified.
Report only safe ref/OID/profile/engine/state/reason counts. Do not persist raw
remote URLs, credentials, detected values, source lines or subprocess diagnostics
in tracked docs, chat or receipts. Redact errors before they leave the process.

## Proposed implementation and focused acceptance evidence

Implement only after a separate owner decision in the authoritative
`agents_pipeline` source repository, not the generated installed support tree.
The bounded seams are `tools/commit-guard.js`, `tools/commit-guard/check.js`,
`tools/commit-guard/common.js`, `tools/commit-guard/hooks.js`,
`protocols/COMMIT_GUARD.md`, `docs/commit-guard.md` and the existing
`tests/commit-guard.test.js`. Preserve current detectors and default scope tests.

| Future focused case | Required product evidence |
|---|---|
| Exact accepted branch/tag/commit; no tag yet | Plan passes with zero outgoing commits and no tag/ref mutation; only explicit acceptance plus a complete passing assessment arms the record. |
| Real exact lightweight tag push | Native hook receives actual remote arguments/stdin, freshly validates destination and consumes once. |
| No opt-in; historical blocking fixture | Default new-ref full-history behavior still blocks and retains its finding/count semantics. |
| Wrong SHA/ref/endpoint/tag, multiple refs, annotated/non-commit object | Opted-in path stops without broadening scope or granting another operation. |
| Stale/moved branch, present tag, denied/failed/malformed remote | Stops; local tracking state or a failed read cannot establish remote absence. |
| Record/ref/configuration tampering, missing/used record, concurrent hooks | Stops or permits only the single exact atomic consumption; no reuse. |
| Full hook PASS followed by transport failure | Binding stays consumed; explicit fresh preparation is required. |
| Invalid policy/engine with zero commits | Fails before PASS; empty scope cannot bypass initialization. |
| Credential-bearing endpoint or unsafe diagnostic fixture | Safe identities/reasons only; raw credentials and subprocess output never escape. |

These are proposed meaningful additions to existing focused tests, not executed
evidence or a request for a new validation framework. Independent implementation
review must precede explicit reviewed installer/hook upgrade authority; then
recheck source/installed Guard identities and the exact native path.

## Original owner decisions and next step (historical)

Decide whether to adopt this exact existing-remote-history boundary and authorize
its bounded source implementation after independent design review. Installation,
native qualification and renewed exact tag/Push/build authority remain separate.
Acceptance of this plan establishes no implemented, installed or executable API.
No product test, Guard execution, Git remote probe, key/Secret operation,
Git/ref mutation or hosted operation is part of this design task. Public Git
documentation is consulted; changed-document checks remain separate from product
qualification. No new candidate or task ID is created.
The next unblockable item at the design checkpoint was the owner implementation
decision. That later operation is now recorded above; current YT-096 work is D6
identity/signing approval and remaining trust, registry and final-release gates.
D6 stays Pending, YT-096 In progress and YT-020 Blocked. No new Commit/Push,
signing, publication or repeated hosted drill is authorized.

## Source references

Original Guard behavior was read from the then-installed copies of the source-relative
seams listed above and the existing source test cases; no Guard was executed.
The source context is `agents_pipeline` commit
`539dae1b7c2345eb99af85b95ad89ebe840a12bc`: the four installed CLI/check/common/
hooks files match source, and the protocol differs only in installer-materialized
command paths. This inspected source version does not supply the proposed feature.
Git references, verified separately for this packet, define the native boundaries:
[pre-push hook arguments/input](https://git-scm.com/docs/githooks#_pre_push),
[effective push URLs](https://git-scm.com/docs/git-remote),
[remote ref query output/patterns](https://git-scm.com/docs/git-ls-remote), and
[push/tag and signed-push semantics](https://git-scm.com/docs/git-push).
