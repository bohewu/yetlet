# M11-P — Native Forms and Application-owned Assets

**Tasks:** YT-070, YT-071, YT-072. **Date:** 2026-10-12.
**Source boundary:** planning commit `d96f7af1073b12eab1a547d57f0d1b4d97589d7c` plus the uncommitted `examples/form-css-proof` implementation and its root workspace/lock entry. This is not evidence for the commit tree alone or for historical signed packages.

## Selected decisions

YT-070 delivers the isolated [native form/CSS proof](../examples/form-css-proof/README.md). Bootstrap 5.3.8 CSS has its original SHA-384 verified, MIT notice retained, and 23 embedded SVG occurrences deterministically externalized into a finite same-origin inventory. There are 22 hashed output files. Default form rendering has no script; an explicitly selected preview uses one local external module. No CSP relaxation, framework dependency upgrade, Bootstrap JS or remote asset is introduced.

YT-071 selects **native SSR controls plus application-owned read-only enhancement**, not a generalized shared-View form renderer. Boolean false is omission; textarea initialization preserves leading newline; validation uses bounded plain text and fixed enums; reset reads native restored properties; preview never rewrites a user's controls. The local owner protects queued work, abortable listeners and remounts. The existing generic DOM allowlist, HTML URL rules, core View and keyed reconciler are unchanged. User-supplied destination URLs, programmatic controlled-field rewrites, keyed control reorder, rich HTML and namespace support are not adopted. Existing small generic DOM restrictions remain documented, rather than being presented as solved by adding allowlist names.

YT-072 selects **retain application-owned wrappers**. Three source consumers were compared: Todo Search (one exact digest-bound CSS/loader/icon set), Multi-Island Dashboard (per-island file/existence fallback), and this proof (ordered/deduplicated stylesheet references matched to embedded bytes and MIME). They share safe document insertion, but not one asset availability policy or deployment lifetime. An additional public head or stylesheet API is not justified. The two independent existing consumers and the new proof have actual focused native regression below. No dependency, Page signature, document metadata, CSP default or existing browser asset file changes follow from this decision.

The owner expressly permits current-session self-review for the activated non-GPU work. These are bounded implementation/decision results, **not independent security review**, formal assurance or a claim that the earlier planning document's independent-assurance conditions were fulfilled. No widened core security boundary is accepted. Real IME, spoken assistive technology and broader product qualification remain separate unmet YT-074 coverage, not a synthetic PASS.

## Executed checks

| Check | Result |
|---|---|
| Asset build and `python3 examples/form-css-proof/build_assets.py --check` | PASS: pinned upstream CSS, deterministic SVG/CSS/JS output and exact manifest/inventory |
| `node --check examples/form-css-proof/assets/enhance.js` | PASS |
| Focused Cargo format, strict Clippy, tests and native build | PASS; 10 proof tests, zero ignored |
| `cargo test --locked --offline -p yetlet-form-css-proof -p yetlet-core -p yetlet-html -p yetlet-dom -p yetlet-multi-island-dashboard --all-targets` | PASS: 27 tests total, zero ignored/filtered |
| `cargo test --locked --offline -p yetlet-todo-server search_route_returns_full_ssr_markup_with_optional_island` | PASS: one selected test; nine unrelated tests filtered, not claimed executed |
| `python3 examples/form-css-proof/verify_browser.py` | PASS: one actual Chrome/155.0.8059.39 journey using the existing repository driver |

The native tests cover escaping, malformed/duplicate/unknown fields, unchecked submission, body bounds, exact asset bytes/MIME/cache policy, finite manifest path/type rejection, missing-asset fallback, and security headers on routing/body errors. One strict Clippy failure in a test initializer was corrected in place; the focused format/Clippy/tests/build rerun passed. No product failure was hidden or test skipped.

The actual browser journey checks initial native text/textarea/select/checkbox properties, associated labels, loaded Bootstrap and application styles, 390 CSS-pixel no-overflow, explicit preview activation, one input listener after remount, preserved input identity/selection, safe text-only preview, synthetic composition suppression/end, cancelled delayed work after disposal, reset defaults, live checkbox state, no local mutation POST/event circuit, and real native submit/PRG. It observed no CSP/JS/network errors or external asset requests.

Browser report and exact source/binary identities are retained in ignored `target/m11-form-proof/browser.json`. Measured debug binary SHA-256: `efe26a2863f1f68bf57ac29f33a12b16dd35db134e2d957e0b7a9438ade25681`. Screenshots were captured by the owned test, but the WebCodex artifact boundary refused image access; **visual screenshot review was not completed and no alternative-path retrieval was attempted**. Loaded/computed styles and layout assertions are not a visual-polish or assistive-technology verdict.

## Execution and review limits

The initial WebCodex browser target became stale before image capture. Its exact server Job `wc_job_qDyM4Fk_S3YA2lXC` was observed completed with exit 0, its listener closed, and current browser discovery returned no browser. Only after that terminal proof did the announced integrated headless test run. Its server/Chrome processes were reaped, both listeners closed and its temporary profile removed. No task-owned execution or uncertain process remains.

Current-session self-review inspected fixed markup/URL sinks, manifest validation, default versus live properties, owner lifetime, body/form rejection and test assertions. It found and corrected the README's mismatched default port to 3011. It did not find a remaining blocker to these selected native/application-owned decisions. Generic forms, controlled-field/IME guarantees, real OS composition, spoken AT, Firefox/mobile coverage, visual signoff and production behavior remain unverified. The subsequent Board must supply its own persistence, identity, object authorization, CSRF, history/conflict and status-stream evidence.

The original 12-sample catalog and nine library candidates are unchanged. This proof is an internal example until separate catalog acceptance. No implementation commit, push, release activation or GPU work is implied.
