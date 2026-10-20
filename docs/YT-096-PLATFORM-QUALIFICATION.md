# YT-096 selected-platform qualification followup

Date: 2026-10-20. Candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, experimental unpublished 0.1.0,
original nine libraries, Rust MSRV 1.99 and qualification pin 1.99.0.

**State: partial.** The [Azure Run Command continuation](YT-096-AZURE-RUN-COMMAND.md)
now passes native tests/doctests and five wasm builds on an actual Ubuntu
24.04.5 guest; the earlier container evidence remains preserved. A fresh Firefox workflow passes on the Ubuntu
26.04.1 desktop host. The resumed Windows MSVC standalone-package lane now
passes all nine package commands: 83 native tests and four doctests. The earlier
workspace-resolution failures remain historical evidence. YT-096 remains
In progress; YT-020 remains Blocked. The earlier independent ordinary slice
review is recorded below; the Windows continuation has a separate review record.
Neither is final release review.

## Source identity and preserved evidence

The [candidate decision](YT-096-CANDIDATE-QUALIFICATION.md),
[supply-chain delta](YT-096-SUPPLY-CHAIN-DELTA.md) and their two accepted
closeouts remain controlling. No package preflight, license/source scan,
RustSec audit or SBOM generation was repeated. No product source, dependency,
manifest, lockfile, tool pin or support policy changed.

Evidence is under `artifacts/yt096-platform-qualification-20261020/`.
[Baseline](../artifacts/yt096-platform-qualification-20261020/BASELINE.json)
records HEAD/dirty state, all 63 selected input digests, nine unchanged archive
digests and 365 files from the two previous evidence directories. Their bytes
were checked before reuse and again after execution. Existing unrelated
untracked artifacts and accepted owner waivers remain intact.

The previous unsigned source archive SHA-256 is
`9096cabe9a7b6d8b809c5bd337c2a8ab88f33ff19b6b4c8b59f31267c7335546`.
WebCodex transferred those exact bytes to the Windows test Project, without
changing that host's existing checkout or creating a detached worktree.
Only the established 18 license aliases were materialized in the owned export;
all 63 selected input digests matched before and after the failed preflight.

## Ubuntu 24.04 userspace and packaged wasm

An owned Docker container used the official Ubuntu image pinned to
`sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac`.
The actual userspace reports **Ubuntu 24.04.5 LTS**, x86_64, glibc
2.39-0ubuntu8.9, GCC 13.3.0 and build-essential 12.10ubuntu1.
Build tools were installed only in this disposable container. It used the
existing Rust 1.99.0 toolchain through a read-only mount; cargo/rustc/rustdoc
digests match the accepted first-slice tools.

This is a native GNU userspace execution, not an emulated target, but it shares
the Ubuntu 26.04.1 host's 7.0.0-38-generic kernel. It does not attest an
independently booted Ubuntu 24.04 system, kernel, desktop or browser. Retain
that distinction for the final supported-platform acceptance decision.

Each root is the unchanged standalone package already extracted from its
accepted archive. Packages and the existing standard Cargo directory source
were mounted read-only. A new private Cargo home selected that source and a
fresh owned target directory received builds. No excluded research workspace
or consumer-only dependency was a root. Defaults were used without extra
feature flags; ordinary Cargo feature resolution still applies.

For each of the nine packages:

```sh
cargo test --locked --offline --target x86_64-unknown-linux-gnu \
  --manifest-path <unchanged-package>/Cargo.toml -- --test-threads=1
```

Cargo's normal test invocation includes the doctest phase. Raw per-package logs,
exact argv/environment and native exits are in
[Ubuntu results](../artifacts/yt096-platform-qualification-20261020/UBUNTU-RESULTS.json).

| Package | Native unit/integration tests | Doctests |
|---|---:|---:|
| yetlet-core | 5 | 0 |
| yetlet-actions | 3 | 0 |
| yetlet-reactive | 16 | 0 |
| yetlet-html | 4 | 0 |
| yetlet-dom | 2 | 0 |
| yetlet-dom-keyed | 1 | 0 |
| yetlet-server-axum | 35 | 3 |
| yetlet-request-id | 7 | 1 |
| yetlet-session | 10 | 0 |
| Total | 83 | 4 |

All nine commands exit 0. Separately, core/actions/reactive/dom/dom-keyed each
pass `cargo build --lib --locked --offline --target wasm32-unknown-unknown`
against those same standalone package roots. These five commands compile wasm
libraries; they do not execute a wasm test suite or qualify browser behavior.

## Linux browser reconciliation

The existing CLI commands `cargo xtask build browser` and
`cargo build --locked --offline -p yetlet-todo-server` passed with offline
resolution. They retained the existing debug asset digest `2b8ea18c1298900c`.
[Browser reconciliation](../artifacts/yt096-platform-qualification-20261020/BROWSER-RECONCILIATION.json)
binds 133 relevant tracked source/build inputs to the selected Git blobs and
records the generated asset hashes and installed browser/tool versions.

`cargo xtask smoke browser-firefox --profile debug` then passed, exit 0, on
the actual Ubuntu 26.04.1 host. Observed installation versions are Firefox
158.0 and geckodriver 0.37.1; the driver log identifies the Firefox Snap binary.
This existing Todo/Search suite covers script-free SSR/CSP, local rendering and
interaction, explicit HTTP actions/errors, lazy search and disposal/remount.
The new receipt is `target/m12/yt085/browser-firefox-o6NBFZ/`; a byte copy is
retained in the new artifact directory.

The driver log contains browser-internal startup/shutdown diagnostics, including
Snap configuration and resource:// Firefox messages. The functional suite and
cleanup pass; no claim of a silent browser log or general accessibility/IME
qualification is made.

Retain the accepted [YT-105 Chrome consumer journey](YT-105-GRAPH-SNAPSHOT.md#consumer-verification)
and `target/m12/yt085/browser-chrome-dSWXtV/`. Its original result/cleanup are
PASS and its accepted application/library source contract remains applicable.
That raw old receipt has no independent candidate input manifest: it is
historical consumer evidence, not a fresh candidate Chrome run. The new source
reconciliation does not retroactively rewrite its identity. No broad historical
browser, Windows OIDC, IME, assistive-technology or performance matrix was rerun.

## Windows MSVC: historical workspace-resolution failures

The first discovered GNU Yetlet Project had an older checkout. Read-only
discovery also inspected the unrelated mint-game Project; no tests or edits
were made in either Project. The existing MSVC environment is instead in
WebCodex Project `agent:ncut-bohe-244143825ae3488e:localtasks`, Session
`wc_sess_y8CJQjwRbo4kknP1`. Its owned qualification root is
`C:\repos\localtasks\yt096-platform-20261020`.

The actual OS reports Windows x64 build 26300. Rust/Cargo 1.99.0 MSVC and
Visual Studio 2022 Professional were reused. Source was the exact candidate
archive above; no checkout switch, installation or global environment change
was made. Runtime native commands returned synchronously and terminally; no
agent Job model/effort applies to those commands.

The initial Python/cmd vcvars invocation failed before any Cargo test. A single
wrapper quoting correction obtained the compiler environment. Then the exact
root-workspace test selection was:

```sh
cargo test --locked --offline --target x86_64-pc-windows-msvc \
  -p yetlet-core -p yetlet-actions -p yetlet-reactive -p yetlet-html \
  -p yetlet-dom -p yetlet-dom-keyed -p yetlet-server-axum \
  -p yetlet-request-id -p yetlet-session -- --test-threads=1
```

| Attempt | Native result | Meaning |
|---|---|---|
| Default existing Cargo cache | Exit 101: missing wasm-bindgen index entry | Offline resolution failed before compilation |
| Existing prepared MSVC test cache | Exit 101: missing openidconnect for the unselected Board workspace member | The root workspace still resolves excluded research/example dependencies despite the package selection |

The second attempt corrected only the cache selection; it kept --locked and
--offline. No native test or doctest ran. These are qualification-environment
failures, not demonstrated defects in the selected libraries. The initial
wrapper failure and both cache failures are retained in
[returned Windows evidence](../artifacts/yt096-platform-qualification-20261020/windows-returned/WINDOWS-RESULTS.json)
and the companion logs, including the separately named initial-cache record.

After two consecutive missing-cache resolution failures, the existing
Materiality Gate's stop rule ended this retry path. No third cache attempt,
automatic network fetch, source workaround or product repair was made.
The selected continuation strategy was to qualify the nine already accepted standalone
archive roots with the existing standard directory source, as on Ubuntu, rather
than resolve the whole research workspace. Reuse the matched export, MSVC
identity and failure records. This historical attempt has no test PASS; the
separate continuation below supplies the passing results.

## Windows MSVC: standalone-package continuation PASS

The owner resumed Windows qualification through the same WebCodex Project and
Session. The owned subdirectory is
`C:\repos\localtasks\yt096-platform-20261020\standalone`.
The nine original `.crate` files were transferred unchanged and extracted as
standalone roots. All 76 normalized archive input files, including nine
lockfiles, matched before execution and remained unchanged afterward. Pinned
Cargo/rustc/rustdoc 1.99.0 MSVC binary digests matched the earlier tool identity.
No product code, package metadata, lockfile or installed toolchain changed.

A fresh private Cargo home uses standard Cargo directory-source replacement
and a fresh owned target directory. All commands retain `--locked --offline`;
the root research workspace is not resolved. The first staging transfer
contained 75 directories selected from target-filtered metadata. Core passed
five tests, then actions exited 101 before compilation because Cargo also
needed the inactive-target locked `wasi` source. This is a cache-input failure,
not a demonstrated library defect.

One bounded correction supplied the remaining 256 directories from the same
existing 331-directory vendor staging used on Ubuntu. No network fetch or
dependency change was needed. Only the failed/unrun eight packages, starting with actions,
were resumed; the original passing core result was retained. These cached
directories include inactive-target resolution inputs and are not the selected
release dependency inventory. The existing 23 SBOMs and their limits remain
unchanged.

Each package used the ordinary command, with no added feature flags:

```sh
cargo.exe test --locked --offline --target x86_64-pc-windows-msvc \
  --manifest-path <standalone-package>/Cargo.toml -- --test-threads=1
```

The per-package native/doctest counts equal the table above: **83 native tests
and four doctests, nine package commands exit 0**. Cargo's normal test command
includes its doctest phase. The actual host is Windows x64 build 26300 with
Visual Studio 2022 Professional and the pinned MSVC toolchain. This is native
MSVC execution, not Windows GNU, cross-compilation or Windows browser coverage.

Initial standalone native Job `wc_job_FHmpDSwndsvkZcE0` terminated at exit 101;
the focused resume `wc_job_5qRKVydnFAJfXF18` completed at exit 0. These are native
executable Jobs, with no agent model/effort. Counts come from raw Cargo phase
summaries and native exits, not the gateway's aggregate test-count field.

The [Windows closeout](../artifacts/yt096-platform-qualification-20261020/windows-standalone/CLOSEOUT.md)
links terminal observations, returned raw logs and input/tool identities. The
returned evidence ZIP SHA-256 is
`061f249271894ed1d5f6ea2daeb58f9343ddea2658dcf963ee76dc7439513d94`.
It includes the initial failure as well as the successful resume. All 419
preexisting files in the three earlier qualification evidence roots were
preserved; earlier closeouts and review records were not overwritten.
No persistent application or listener was started. Owned staging/build/log
files remain for later qualification. This consumes the original archives;
it does not rebuild them on Windows or prove independently controlled building.

## Cleanup, current boundary and remaining gates

[Cleanup](../artifacts/yt096-platform-qualification-20261020/CLEANUP.json)
records the removed owned Ubuntu container with no published ports. Firefox
exited 0; its owned server retained native SIGTERM 15. Their receipts verify
closed listeners/streams, reaped trees and browser-profile removal. Windows
started no application or listener; its source/logs remain for continuation.
The candidate, previous 365 evidence files and all pre-existing artifacts were
preserved. No staging, commit, push, signing, tag, publication or deployment
occurred.

The selected-platform gate is still partial: Windows native qualification now
passes. Actual Ubuntu 24.04 guest execution now supplies the native baseline
beyond the earlier shared-kernel container. Reconcile the retained Chrome
evidence against the final selected browser/support boundary. The subsequent
[two-host archive rebuild](YT-096-TWO-HOST-REBUILD.md) now matches all nine
Windows archives exactly to Linux; that is separate from this platform slice.
Independently controlled builder evidence, build-bound inventory, formal
provenance/trust/signature/registry checks, final independent release/security
review and maintainer approval remain open. No subsequent release gate is
marked passed by this platform slice.

## Independent ordinary slice review

Reviewer `/root/yt096_platform_partial_review_oct20` PASS has zero issues,
required followups and optional notes. Native trace matched the registered
reviewer role, gpt-6-astra model and effective high effort; the shared resolver
observation is enforced. This accepts the accuracy and bounded evidence of the
partial platform report, not Windows completion or final release/security.

The reviewer independently inspected all 63 source identities, 365 prior
evidence files, nine archive hashes and extracted inputs, 133 browser source
inputs, eight assets and the original 56 review snapshot entries. Raw native
results, Firefox implementation/receipts, both Windows cache failures and the
removed container were checked; no runtime suite was repeated. The
[acceptance record](../artifacts/yt096-platform-qualification-20261020/REVIEW-ACCEPTANCE.json)
retains the bounded identity and preservation result. Documentation/internal
links/Markdown/whitespace/backlog DAG and `git diff --check` pass.

## Independent Windows continuation review

Reviewer `/root/yt096_windows_qualification_review_oct20` independently returned
PASS with zero issues, required followups or optional notes. Native trace
verified the registered reviewer role, gpt-6-astra model and effective high
effort; the shared resolver observation is enforced. The reviewer reconciled
all nine commands and raw Cargo phase counts, archive/normalized inputs and
locks, MSVC tool digests, returned ZIP members, terminal Jobs, 419 preserved
historical evidence files, 42 review snapshot entries and 63 source inputs.
No tests were repeated. The [Windows acceptance record](../artifacts/yt096-platform-qualification-20261020/windows-standalone/REVIEW-ACCEPTANCE.json)
accepts this Windows slice only, with all later release gates still open.

## Historical Azure SSH clean-builder attempts

The [Azure attempt](YT-096-AZURE-CLEAN-BUILDER.md) created two temporary VMs
but stopped before source transfer/build after consecutive SSH timeouts.
No Azure toolchain installation, Cargo fetch, package verification, native
test, wasm build or build-bound inventory result is claimed. Separate cleanup
records preserve both attempts. The accepted platform/archive/SBOM/checksum
evidence and all remaining trust/approval boundaries stay unchanged.

## Azure Run Command/private Blob continuation

The [owner-resumed clean builder](YT-096-AZURE-RUN-COMMAND.md) now passes on
an actual Ubuntu 24.04.5 guest, kernel 7.0.0-1017-azure, with upstream tool and
dependency downloads into an empty cache. Nine archives are byte-identical
to the original Linux/Windows outputs; 83 native tests/four doctests and five
wasm libraries pass. All temporary resources and private Blob storage are
deleted. The earlier failed SSH attempts remain unchanged.

There are 140 actual Cargo compiler-artifact events. Native 70 observed package
versions comprise 68 in retained target SBOMs and 2 known dev-only dependencies,
tower-http 0.6.11 / bitflags 2.13.2, already covered by retained dev metadata.
All 32 wasm package versions occur in retained wasm SBOMs. This aggregate
observation does not prove per-library feature equivalence, output-file
digests, Rust sysroot/native-linker inventory or formal builder trust.
Existing source inventory bytes, coverage differences, checksums and owner
waivers remain intact; final inventory/trust/release approvals remain open.
