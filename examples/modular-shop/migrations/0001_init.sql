CREATE TABLE products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    price_cents INTEGER NOT NULL CHECK(price_cents > 0),
    stock INTEGER NOT NULL CHECK(stock >= 0)
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner TEXT NOT NULL,
    checkout_key TEXT NOT NULL,
    total_cents INTEGER NOT NULL CHECK(total_cents >= 0),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(owner, checkout_key)
);

CREATE TABLE order_lines (
    order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL,
    product_name TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    price_cents INTEGER NOT NULL CHECK(price_cents > 0),
    PRIMARY KEY(order_id, product_id)
);

INSERT INTO products(id, name, price_cents, stock) VALUES
    ('rust-book', 'The Rust Programming Language', 3999, 8),
    ('travel-mug', 'Stainless Travel Mug', 1250, 5),
    ('keyboard', 'Compact Mechanical Keyboard', 8900, 2);
