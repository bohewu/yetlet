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

The cart lives in the opt-in `yetlet-session` provider. Products, inventory, orders and lines live in application-local bundled SQLite through direct parameterized `rusqlite`; synchronous work is limited by a four-operation semaphore and executes through `spawn_blocking`.

Checkout uses `BEGIN IMMEDIATE`. Inventory decrements and order/line inserts commit together; insufficient stock or a line-insert failure rolls back the whole order. An idempotency key derived from the Session CSRF identity plus cart revision returns the same committed order if a database commit succeeded before Session-cart cleanup completed. A concurrent cart change is never cleared by an older checkout response.

Every order read/list derives the user from server-side Session state. Order IDs and form fields never grant ownership; cross-user reads return HTTP 403. Mutation forms require the current CSRF token and cart revision.

This is **not** a payment gateway, financial ledger, production identity provider, tax/shipping system, encrypted-at-rest store, backup design or multi-process inventory service. “Place order” is a local mock transaction only.

## Verify

```bash
cargo test --locked -p yetlet-modular-shop --all-targets
cargo build --release --locked -p yetlet-modular-shop
python3 scripts/smoke_modular_shop.py
cargo tree --locked -p yetlet-modular-shop --edges normal
python3 scripts/check_samples.py
python3 scripts/check_docs.py
```

See [M9i verification](../../docs/M9I-VERIFICATION.md) for retained integration, rollback, reviewer and dependency-isolation evidence.
