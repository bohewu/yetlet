# YT-210 — Public package consolidation study

Date: 2026-10-30. Status: contract and final independent reviews PASS;
YT-210 Done at bounded four-module feasibility scope. The owner paused alpha.4 registry continuation, asked to
consider one package and accepted continuing with contract review followed by
the Core/HTML/Axum/DOM proof. One package/default HTML is selected for this
publish=false experiment only. Full migration, production API/feature adoption
and a new release candidate are not selected by this acceptance.

## Goal and recommendation

Recommend evaluating **one implementation package, provisionally `yetlet`,
with internal modules and additive Cargo features**. Reduce dependency/import
choices for users and the number of Yetlet-owned release artifacts. Preserve
the small HTML-first model, separate renderer/HTTP responsibilities, native
Axum/Tower composition and opt-in providers.

One package must not install every capability. Compiled availability, explicit
application startup registration and optional Settings loading remain separate.
No mandatory DI, subscriber, storage connection, auth middleware or GPU startup
is introduced. Authentication remains separate from business authorization.

The recommendation is a feasibility conclusion, not an accepted production API
or a performance claim. Independent contract review passes with zero issues;
the four-module experiment follows the owner's continuation authority. Final
prototype review passes with zero issues; YT-210 is Done at this bounded scope
and YT-020 remains Blocked.

## Current evidence and alternatives

At source `678adfec81f1878b0f3d0a886d8a12ef2e83eccc`, static inspection finds
sixteen library manifests and 31 Rust files under their `src` directories,
333,365 source bytes in total. This is not an archive-size/build-time estimate;
READMEs, licenses, fixtures and generated/documentation inputs are outside that
count. There are no manifest build-dependencies in these sixteen libraries.
The inventory is retained at `target/yt210-packaging-study-20261030/`.

| Shape | Yetlet library packages to publish | Main tradeoff |
|---|---|---|
| Keep current split | 16 | Existing crate identities and isolation, many imports/version/artifact bindings |
| Add an entry facade | 17 if all current libraries remain | Easier imports, no reduction in required registry artifacts |
| Consolidate into a few packages | A selected small set | Retain physical browser/native/provider boundaries, still coordinate cross-package versions |
| One implementation package | 1 | Simplest public installation/release unit; feature/target rules replace crate boundaries |

A facade with registry dependencies on unpublished implementation crates is not
a one-package publication solution. Cargo removes local `path` locations from
published dependency declarations; consumers need their registry versions.
See [Cargo dependency locations](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-locations).
A genuine single package moves implementation source into its own archive;
modules/subdirectories can retain separate ownership without being separate
Cargo packages. Examples, the CLI and application-owned integrations may remain
unpublished workspace packages. Third-party crates remain normal dependencies.

A read-only `GET https://crates.io/api/v1/crates/yetlet` returned404 during this
study. The proposed name is not reserved and must be checked again before
adoption/publication. No PUT, token access, registration or quota probe occurred.

New-name throttling is a one-time delivery concern, not the sole consolidation
criterion. The official [crates.io rate-limit documentation source](https://github.com/rust-lang/crates.io/blob/579521fbfd23607271215aa6390e4cf189f7c4de/svelte/src/routes/docs/rate-limits/%2Bpage.svelte#L21-L40)
distinguishes a five-name burst/ten-minute replenishment for new names from
thirty versions/one-minute replenishment for existing names, per account.
Unknown account/deployment overrides remain unknown. This study does not
authorize a different continuation cadence or another publication attempt.

## Candidate module and feature map

These are proposed production names, not currently available framework APIs.
Only `view`/`html`/`server_axum`/`dom` are exercised in the local experiment.
Use snake_case Rust modules and hyphenated Cargo feature names, following the
[naming guide](NAMING.md). Re-export only common View/Element primitives at the
root; retain qualified capability modules so names such as `Policy` have context.

| Current package | Candidate module | Candidate feature and dependency boundary |
|---|---|---|
| `yetlet-core` | `view`, selected root re-exports | Always present; preserve no external normal dependencies |
| `yetlet-html` | `html` | `html`; pure serializer, proposed default |
| `yetlet-actions` | `actions` | `actions`; serde/JSON, browser transport dependencies only on wasm32 |
| `yetlet-reactive` | `reactive` | `reactive`; explicit local graph, no runtime startup |
| `yetlet-dom` | `dom` | `dom`; wasm32 browser dependencies remain target-gated |
| `yetlet-dom-keyed` | `dom_keyed` | `dom-keyed`; keep separately selectable from ordinary DOM |
| `yetlet-server-axum` | `server_axum` | `server-axum` implies `html`; native Axum/Tower |
| `yetlet-request-id` | `request_id` | `request-id` implies `server-axum`; explicit registration |
| `yetlet-session` | `session` | `session` implies `server-axum`; bounded synchronous Memory Session |
| `yetlet-auth` | `auth` | `auth`; native Axum identity extraction; `auth-session` adds `auth` and `session` bridge |
| `yetlet-session-redis` | `session_redis` | `session-redis` implies `server-axum`; async Redis/Valkey protocol; `session-redis-tls` additionally selects existing TLS |
| `yetlet-config` | `config` | `config`; native startup loading; `secret-provider` additionally enables `config` and `secrets` adapter |
| `yetlet-secrets` | `secrets` | `secrets`; native explicit provider; no automatic settings installation |
| `yetlet-signing` | `signing` | `signing`; native OpenSSL dependency only when selected |
| `yetlet-rate-limit` | `rate_limit` | `rate-limit`; native Axum/Tower/governor; no required Settings loader |
| `yetlet-wgpu` | `wgpu` | `wgpu` CPU contracts; `wgpu-browser` adds wasm32 WebGPU dependencies; no Lab/Rapier merge |

The candidate minimal default is `default = ["html"]`: View and pure HTML
serialization, with no HTTP stack, Auth/Session, settings, secrets, OpenSSL,
Redis, browser or GPU dependency compiled by default. A view-only consumer can
select `default-features = false`. This default is a proposed DX choice, not a
production manifest change. It is used in the local experiment below. Native
SSR+Axum explicitly selects `server-axum`; a browser
consumer explicitly selects its DOM/action/reactive needs.

Illustrative future dependency declarations, **not runnable current guidance**:

```toml
# Native SSR application; version/name are not selected for publication.
[dependencies]
yetlet = { version = "<future-version>", features = ["server-axum"] }
```

```toml
# Separate browser consumer.
[dependencies]
yetlet = { version = "<future-version>", default-features = false, features = ["dom", "actions"] }
```

No `full` preset or new generic provider is proposed. Audit, data/output cache,
tracing composition, OIDC and the 3D lab keep their existing application/research
boundaries. No new feature means automatic provider initialization.

## Target isolation and feature composition contract

Cargo features are additive and unify across consumers of a package; disabling
defaults in one consumer does not undo another consumer's feature selection.
See [Cargo feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification).
Do not rely on mutually exclusive `server`/`browser` features for isolation.

Use optional dependencies under real target tables and matching source `cfg`s:
native capabilities under `cfg(not(target_arch = "wasm32"))`, browser bindings
under `cfg(target_arch = "wasm32")`. Native module visibility requires both its
feature and a native target. A native-only feature unified into a wasm32 build
must not compile/expose its native module or dependencies; importing that module
on wasm32 must fail rather than installing a substitute. The feature name alone
does not promise the module on every target. This permits shared-workspace
feature unions without making server dependencies browser dependencies.

Keep the resolver3 baseline. Its target-feature rules do not replace explicit
source/dependency target gates. Do not put `cfg(feature = "...")` into Cargo
target dependency table keys: Cargo requires ordinary optional dependencies and
the `[features]` table for that choice. See
[target-specific dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
and [resolver target-feature rules](https://doc.rust-lang.org/cargo/reference/resolver.html#feature-resolver-version-2).

Optional features avoid compiling unselected integrations; they do not promise
an empty lockfile/resolution graph, identical compiler output, no downloaded
optional source, a smaller published archive or improved compile/request times.
One archive includes the selected package's source even when features are off.
Assess those costs from a future real package, not from source-byte counts.

Keep one View type across all selected modules. Preserve Memory sync versus
Redis async methods, Session authority/CSRF/revocation, principal trust,
application authorization, escaping/CSP and GPU ownership/failure contracts.
Cargo features establish availability; explicit typed options/registration
establish startup behavior. Settings loading stays optional.

## Migration and release impact

Consolidation is a source-breaking migration. Existing `yetlet_*` imports and
Cargo dependency names change; modules do not preserve the identity of old
external crate types. Migrate each consumer together and verify relevant trait
implementations, re-exports, `crate::` paths, rustdoc/examples and dependencies.
Do not promise old/new Action or View types interoperate, or ship sixteen
compatibility wrappers that recreate the public-package count by accident.

Preserve already published `yetlet-actions0.9.0-alpha.4`, GitHub alpha.4 and all
old archives/tags/signatures/evidence. Do not yank or overwrite them. Existing
proof remains evidence for its original inputs; it cannot sign or certify the
new module layout/package. Select a new version/C and new package subjects only
after implementation acceptance; do not repurpose the frozen alpha.4 record.

Build/rebuild, checksum/provenance, signer/verifier/publication scripts currently
bind sixteen names and filename sets. A one-package change must update their
closed schemas/subject counts and affected consumers/tests coherently. Do not
weaken identity checks, include arbitrary files, reuse an old signature for new
bytes or change the final owner publication gate. Package inclusion must cover
required source/docs/licenses/fixtures and exclude private evidence/keys/logs.

[PR29](https://github.com/bohewu/yetlet/pull/29) remains open and unmerged, with
no auto-merge; publication remains paused. Its accepted response-parser fix is
useful maintenance, but the fifteen-name continuation must not automatically
run after a packaging redesign. Separate any future delivery decision from
the old one-shot continuation authority. No PR closure/merge is authorized here.

## Bounded next steps and acceptance

1. Independently review this proposal; owner chooses one package versus a small
   set, default features, names/modules and treatment of old package imports.
2. After explicit implementation authority, first exercise real Core/HTML,
   native Axum and browser DOM modules plus a mixed-consumer feature union in
   one package. Establish target isolation before moving the other capabilities.
   Consolidate without behavior changes or third-party upgrades, then migrate
   current consumers/generators. This is not a facade over the old sixteen.
3. Verify only affected package/target boundaries: core/default; native SSR;
   selected Auth/Memory, Redis/TLS, Config/Secrets and signing combinations;
   browser DOM/keyed/actions/reactive and optional WebGPU; a mixed consumer
   feature union. Check native-only dependencies/API are absent on wasm32.
   Reuse unchanged safety evidence and run targeted output/security/lifecycle
   checks where moving modules actually affects those paths. Do not enumerate
   every possible feature subset or replay unrelated deployment/benchmark proof.
4. Check a normally verified standalone package and native/browser consumers,
   rustdoc and required feature combinations, with the new exact source/subject
   binding. Only then plan a new release and changed signer/publisher scope.

The initial static study used manifest/source inspection and documentary checks
only. Its inventory and no-runtime-proof boundary remain historical. Subsequent
contract review and bounded execution follow; neither establishes a production
package, full target/browser qualification or release proof. No production
manifest/library/example/workflow change, Commit/Push, hosted execution, signing
or publication occurs in this slice.

## Bounded Core/HTML/Axum/DOM experiment

Independent contract Reviewer `/root/alpha4_hosted_checkpoint_review_20261030`
returns PASS, zero issues/required followups; all four documentary source hashes
match. Verified native role/model/effort are reviewer / gpt-6-astra / high.
The later owner continuation selects the recommended direction only for this
local experiment, not a full sixteen-module migration.

The retained `target/yt210-packaging-study-20261030/prototype/yetlet/` is a
standalone `yetlet0.0.0`, `publish = false` package. It contains ten copied
source files from the four current crates, plus one new feature-gated root.
Changes to copied files are imports/module paths, doctest paths and rustfmt;
renderer/HTTP logic is unchanged. There are no dependencies on old Yetlet
packages and no symlinks to their source. Third-party locked identities and
checksums remain a subset of the existing root lock; no upgrades occur.

The normal `cargo +1.99.0 package --offline --locked --allow-dirty` operation
performs Cargo verification without bypasses. Its archive has18 files,
26,640 compressed bytes, SHA-256
`38c3ee3cae8011de40c6d3406421bf776d7ab11bb15acdb612f5afcd1fbf6517`.
This measurement covers four modules, not the future sixteen-module package.
All eleven archived source files equal the tested prototype source. Required
licenses/readme/test source are present; no private evidence is packaged.

| New or affected check | Result |
|---|---|
| Core-only library check; core/default normal dependency trees | PASS; no external normal dependencies |
| Cross-module View identity/exact escaping, unsafe URL rejection, in-process Axum route/CSP/SSR response | 3 new tests PASS; no TCP server or browser runtime needed |
| Migrated native API documentation paths | 3 doctests PASS |
| Native workspace consumers of extracted archive only | PASS after one fixture correction |
| wasm32-unknown-unknown workspace build of extracted archive consumers | PASS; browser cdylib produced |
| Actual consumer feature union and target dependency tree | `default,html,dom,server-axum` together; no Axum/Tower/Tokio/serde/OpenSSL/Redis in wasm32 normal graph |
| Native API import from wasm32 with native feature explicitly selected | Expected E0432/exit101; module configured out, no fallback |
| Archive/source/lock reconciliation | PASS; no other Yetlet package or changed third-party identity |

Commands use the isolated manifests, Rust1.99.0 and `--offline --locked` after
initial offline lock reconciliation. Logs, exact archive/member list, source
transformation map, dependency profiles and consumer source are retained beside
the prototype. No prior benchmark/platform/browser/deployment qualification is
replayed. This proves the bounded package/import/feature mechanism, not a new
browser runtime qualification, Windows build, full provider migration or speedup.

Two original local failures remain: the virtual-workspace formatter invocation
reported no targets and was corrected by formatting only the two owned consumer
files; the native consumer dropped an unconstrained Router and required
`App::<()>::new()`. The latter was a fixture-only E0283/exit101; only the failed
native check was repeated. Neither correction changes package bytes/logic or
replays the passed wasm32 check. All Cargo processes exited; no network server,
browser, container or cloud resource was started.

Final independent Reviewer `/root/alpha4_hosted_checkpoint_review_20261030`
returns PASS, zero issues/required followups, and `item_can_be_done=true` for
this bounded scope. All four reviewed document hashes and96 evidence hashes
match at acceptance; reviewer/model/high effort are verified. Only this
acceptance/state metadata follows the reviewed source boundary. No product
verification is replayed. YT-210 is Done; final production API adoption and
full migration remain open. The next unstarted slice is YT-211: adopt the final
package/module/feature contract, consolidate remaining
providers and migrate actual consumers/generators/release subjects. Keep alpha.4
publication paused and PR29 unmerged until a separately authorized disposition.
