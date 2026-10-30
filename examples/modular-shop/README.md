# Modular Shop — Explicit Rust Modules, Session Cart and SQL Orders

This larger reference application composes four ordinary Rust modules: `identity`, `catalog`, `cart` and `orders`. Each module owns its routes and dependencies; the application uses no plugin registry, service locator, DI container, Controller hierarchy, ORM, repository framework or browser runtime.

## Run

From the repository root:

```bash
cargo run -p yetlet-modular-shop
```

The anonymous synthetic demo listens on `127.0.0.1:3008` and rejects non-loopback listeners. Its default SQLite file is `target/yetlet-modular-shop.sqlite3`. Override local paths with `YETLET_SHOP_LISTEN` and `YETLET_SHOP_DB`.

At startup the app generates terminal-only synthetic Alice/Bob login codes. Tests may supply deterministic codes using both `YETLET_SHOP_ALICE_CODE` and `YETLET_SHOP_BOB_CODE`; setting only one fails closed.

Routes:

- `/healthz` — process health outside the Session modules.
- `/identity/login` and `/identity/logout` — synthetic login/logout with Session fixation rotation and CSRF.
- `/shop/catalog` — public SQL-backed catalog and inventory view.
- `/shop/cart`, `/shop/cart/add`, `/shop/cart/decrement`, `/shop/cart/clear` — authenticated Session cart.
- `/shop/orders`, `/shop/orders/{id}`, `/shop/orders/checkout` — authenticated transactional orders.

Set `YETLET_SHOP_DISABLE_ORDERS=1` to demonstrate a disabled feature module. The orders routes are absent, the catalog omits the orders link and the cart replaces checkout with an explicit disabled-module message. Invalid combinations such as Orders without Identity/Cart/Catalog are rejected during setup.

## Transaction and authorization boundary

The cart lives in the opt-in `yetlet::session` provider. Products, inventory, orders and lines live in application-local bundled SQLite through direct parameterized `rusqlite`; synchronous work is limited by a four-operation semaphore and executes through `spawn_blocking`.

Checkout uses `BEGIN IMMEDIATE`. Inventory decrements and order/line inserts commit together; insufficient stock or a line-insert failure rolls back the whole order. An idempotency key derived from the Session CSRF identity plus cart revision returns the same committed order if a database commit succeeded before Session-cart cleanup completed. A concurrent cart change is never cleared by an older checkout response.

Every order read/list derives the user from server-side Session state. Order IDs and form fields never grant ownership; cross-user reads return HTTP 403. Mutation forms require the current CSRF token and cart revision.

This is **not** a payment gateway, financial ledger, production identity provider, tax/shipping system, encrypted-at-rest store, backup design or multi-process inventory service. “Place order” is a local mock transaction only.

## Verify

```bash
cargo test --locked -p yetlet-modular-shop --all-targets
cargo build --release --locked -p yetlet-modular-shop
cargo xtask smoke modular-shop
cargo tree --locked -p yetlet-modular-shop --edges normal
cargo xtask check samples
cargo xtask check docs
```

See [M9i verification](../../docs/M9I-VERIFICATION.md) for retained integration, rollback, reviewer and dependency-isolation evidence.


## Optional catalog display cache — YT-183

[The selected contract](../../docs/YT-183-CATALOG-CACHE-CONTRACT.md) applies only
to the existing public product projection used by catalog and cart displays.
Default `Database::open` and normal startup remain uncached. Enable both the
application feature and its single native startup environment selection:

```sh
YETLET_SHOP_CATALOG_CACHE_ENABLED=true cargo run --locked \
  -p yetlet-modular-shop --features catalog-cache
```

Embedded callers use `Database::open_with_options(path,
DatabaseOptions { catalog_cache: true })`, then pass the Database to the existing
`app` constructor. `DatabaseOptions::default()` leaves caching off even when
compiled. Unsupported true selections fail before opening/creating storage.
[YT-185](../../docs/YT-185-OPT-IN-OPTIONS.md) records the intentional pre-public
constructor migration. Options do not read ambient settings; the binary maps
its existing native environment value into them. There
is no new Settings-provider service, framework middleware/trait or Core/WASM
cache feature. Missing/`false` selection stays off; only exact `true`/`false`
is accepted, and true without compiled feature fails redacted startup validation.
Selection is startup-only. See [new evidence](../../docs/YT-183-CATALOG-CACHE-IMPLEMENTATION.md).

Native Moka coalesces fallible fills. The application admits at most 64 products
and an estimated 64 KiB payload per retained projection, with native best-effort
capacity one; these are not hard process-memory bounds. Oversized successful
results are returned uncached, not truncated. One actual cache-fill SQL worker
is admitted at a time; an enabled lookup waits up to a cooperative 500 ms and
returns existing storage-unavailable behavior on timeout/load failure, without
an application retry or stale-on-error fallback. A started blocking worker keeps
its fill/SQL permits after caller timeout/drop. Existing four SQL workers and
two-second SQLite busy timeout are unchanged.

Displayed data has a two-second maximum age since source-load start at selection,
including source wait, plus native insertion TTL. Hits do not extend the age.
Enabled pages label price/stock advisory and cart total estimated. Add-to-cart
still reads current product SQL; checkout still rereads current price/stock and
commits inventory/order/lines in its original transaction. Session, CSRF, order
ownership, durable idempotency and private `no-store` response policy are uncached.
This is data reuse; fresh pages/forms are still rendered for every request.

Checkout publishes a new local generation inside the successful blocking worker,
including if its caller disappears. Reads begun before publication may finish
with their prior bounded-age projection; later reads use a new key, so a late
old fill cannot repopulate the active generation. Rollbacks do not advance it.
Generation saturation refuses cached reads until cold reopen without changing
checkout authority. Separate Database opens/processes and out-of-band SQL edits
have independent caches and only the two-second freshness bound, not immediate
cross-instance invalidation. New tenant/private catalog rules need a separate
scope/visibility contract; this public projection is not a reusable authorization
cache. No speedup, throughput, production or distributed coherence claim.
