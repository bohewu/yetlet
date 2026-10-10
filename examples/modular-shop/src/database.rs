use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, TransactionBehavior, params};
use tokio::sync::Semaphore;

use crate::UserId;

const MIGRATION_1: &str = include_str!("../migrations/0001_init.sql");
const SCHEMA_VERSION: i64 = 1;
const MAX_DB_OPERATIONS: usize = 4;
const BUSY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Product {
    pub id: String,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrderLine {
    pub product_id: String,
    pub product_name: String,
    pub quantity: u8,
    pub price_cents: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Order {
    pub id: i64,
    pub owner: UserId,
    pub total_cents: u32,
    pub created_at: i64,
    pub lines: Vec<OrderLine>,
}

#[derive(Debug)]
pub enum DatabaseError {
    Sqlite(rusqlite::Error),
    Io(io::Error),
    UnsupportedSchema(i64),
    SchemaMismatch,
    NotFound,
    Forbidden,
    EmptyCart,
    InvalidQuantity,
    UnknownProduct,
    OutOfStock,
    Overflow,
    WorkerUnavailable,
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::UnsupportedSchema(version) => {
                write!(
                    formatter,
                    "database schema version {version} is newer than supported"
                )
            }
            Self::SchemaMismatch => formatter.write_str("database schema does not match version 1"),
            Self::NotFound => formatter.write_str("order was not found"),
            Self::Forbidden => formatter.write_str("order belongs to another user"),
            Self::EmptyCart => formatter.write_str("cart is empty"),
            Self::InvalidQuantity => formatter.write_str("cart quantity is invalid"),
            Self::UnknownProduct => formatter.write_str("cart references an unknown product"),
            Self::OutOfStock => formatter.write_str("one or more products are out of stock"),
            Self::Overflow => formatter.write_str("order total overflowed"),
            Self::WorkerUnavailable => formatter.write_str("blocking database worker unavailable"),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<io::Error> for DatabaseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone)]
pub struct Database {
    inner: Arc<Inner>,
}

struct Inner {
    path: PathBuf,
    gate: Arc<Semaphore>,
}

impl Database {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, DatabaseError> {
        let path = path.into();
        if path.as_os_str().is_empty() || path.is_dir() {
            return Err(DatabaseError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "database path must be a file",
            )));
        }
        if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        prepare_file(&path)?;
        let mut connection = open_connection(&path)?;
        migrate(&mut connection)?;
        verify_schema(&connection)?;
        Ok(Self {
            inner: Arc::new(Inner {
                path,
                gate: Arc::new(Semaphore::new(MAX_DB_OPERATIONS)),
            }),
        })
    }

    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, DatabaseError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, DatabaseError> + Send + 'static,
    {
        let permit = self
            .inner
            .gate
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        let path = self.inner.path.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut connection = open_connection(&path)?;
            operation(&mut connection)
        })
        .await
        .map_err(|_| DatabaseError::WorkerUnavailable)?
    }

    pub(crate) async fn catalog(&self) -> Result<Vec<Product>, DatabaseError> {
        self.run(|connection| {
            let mut statement = connection
                .prepare("SELECT id, name, price_cents, stock FROM products ORDER BY id")?;
            statement
                .query_map([], product_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(Into::into)
        })
        .await
    }

    pub(crate) async fn product(&self, id: String) -> Result<Product, DatabaseError> {
        self.run(move |connection| {
            connection
                .query_row(
                    "SELECT id, name, price_cents, stock FROM products WHERE id=?1",
                    params![id],
                    product_from_row,
                )
                .optional()?
                .ok_or(DatabaseError::UnknownProduct)
        })
        .await
    }

    pub(crate) async fn place_order(
        &self,
        owner: UserId,
        lines: BTreeMap<String, u8>,
        checkout_key: String,
    ) -> Result<Order, DatabaseError> {
        self.run(move |connection| {
            if lines.is_empty() {
                return Err(DatabaseError::EmptyCart);
            }
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            if let Some(existing) = transaction
                .query_row(
                    "SELECT id FROM orders WHERE owner=?1 AND checkout_key=?2",
                    params![owner.as_str(), checkout_key],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
            {
                let order = load_order(&transaction, existing, owner)?;
                transaction.commit()?;
                return Ok(order);
            }

            let mut resolved = Vec::with_capacity(lines.len());
            let mut total = 0_u32;
            for (product_id, quantity) in lines {
                if quantity == 0 || quantity > 20 {
                    return Err(DatabaseError::InvalidQuantity);
                }
                let product = transaction
                    .query_row(
                        "SELECT id, name, price_cents, stock FROM products WHERE id=?1",
                        params![product_id],
                        product_from_row,
                    )
                    .optional()?
                    .ok_or(DatabaseError::UnknownProduct)?;
                if product.stock < u32::from(quantity) {
                    return Err(DatabaseError::OutOfStock);
                }
                total = total
                    .checked_add(
                        product
                            .price_cents
                            .checked_mul(u32::from(quantity))
                            .ok_or(DatabaseError::Overflow)?,
                    )
                    .ok_or(DatabaseError::Overflow)?;
                resolved.push((product, quantity));
            }

            transaction.execute(
                "INSERT INTO orders(owner, checkout_key, total_cents) VALUES (?1, ?2, ?3)",
                params![owner.as_str(), checkout_key, i64::from(total)],
            )?;
            let order_id = transaction.last_insert_rowid();
            for (product, quantity) in resolved {
                let changed = transaction.execute(
                    "UPDATE products SET stock=stock-?1 WHERE id=?2 AND stock>=?1",
                    params![i64::from(quantity), product.id],
                )?;
                if changed != 1 {
                    return Err(DatabaseError::OutOfStock);
                }
                transaction.execute(
                    "INSERT INTO order_lines(order_id, product_id, product_name, quantity, price_cents) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        order_id,
                        product.id,
                        product.name,
                        i64::from(quantity),
                        i64::from(product.price_cents)
                    ],
                )?;
            }
            let order = load_order(&transaction, order_id, owner)?;
            transaction.commit()?;
            Ok(order)
        })
        .await
    }

    pub(crate) async fn orders_for(&self, owner: UserId) -> Result<Vec<Order>, DatabaseError> {
        self.run(move |connection| {
            let mut statement =
                connection.prepare("SELECT id FROM orders WHERE owner=?1 ORDER BY id DESC")?;
            let ids = statement
                .query_map(params![owner.as_str()], |row| row.get::<_, i64>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            ids.into_iter()
                .map(|id| load_order(connection, id, owner))
                .collect()
        })
        .await
    }

    pub(crate) async fn order_for(&self, owner: UserId, id: i64) -> Result<Order, DatabaseError> {
        self.run(move |connection| load_order(connection, id, owner))
            .await
    }

    #[cfg(test)]
    pub(crate) async fn stock(&self, id: &str) -> u32 {
        self.product(id.to_owned()).await.unwrap().stock
    }

    #[cfg(test)]
    pub(crate) async fn order_count(&self) -> i64 {
        self.run(|connection| {
            connection
                .query_row("SELECT COUNT(*) FROM orders", [], |row| row.get(0))
                .map_err(Into::into)
        })
        .await
        .unwrap()
    }

    #[cfg(test)]
    pub(crate) fn test_connection(&self) -> Connection {
        open_connection(&self.inner.path).unwrap()
    }
}

fn prepare_file(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    match options.open(path) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

fn open_connection(path: &Path) -> Result<Connection, DatabaseError> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(connection)
}

fn migrate(connection: &mut Connection) -> Result<(), DatabaseError> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(DatabaseError::UnsupportedSchema(version));
    }
    let _: String = connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    if version == 0 {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute_batch(MIGRATION_1)?;
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        transaction.commit()?;
    }
    verify_schema(connection)
}

fn table_columns(connection: &Connection, table: &str) -> Result<Vec<String>, DatabaseError> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn verify_schema(connection: &Connection) -> Result<(), DatabaseError> {
    let products = table_columns(connection, "products")?;
    let orders = table_columns(connection, "orders")?;
    let lines = table_columns(connection, "order_lines")?;
    if products != ["id", "name", "price_cents", "stock"]
        || orders != ["id", "owner", "checkout_key", "total_cents", "created_at"]
        || lines
            != [
                "order_id",
                "product_id",
                "product_name",
                "quantity",
                "price_cents",
            ]
    {
        return Err(DatabaseError::SchemaMismatch);
    }
    Ok(())
}

fn product_from_row(row: &Row<'_>) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get(0)?,
        name: row.get(1)?,
        price_cents: u32::try_from(row.get::<_, i64>(2)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        stock: u32::try_from(row.get::<_, i64>(3)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
    })
}

fn load_order(connection: &Connection, id: i64, owner: UserId) -> Result<Order, DatabaseError> {
    let header = connection
        .query_row(
            "SELECT owner, total_cents, created_at FROM orders WHERE id=?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or(DatabaseError::NotFound)?;
    if header.0 != owner.as_str() {
        return Err(DatabaseError::Forbidden);
    }
    let total_cents = u32::try_from(header.1).map_err(|_| DatabaseError::SchemaMismatch)?;
    let mut statement = connection.prepare(
        "SELECT product_id, product_name, quantity, price_cents \
         FROM order_lines WHERE order_id=?1 ORDER BY product_id",
    )?;
    let lines = statement
        .query_map(params![id], |row| {
            Ok(OrderLine {
                product_id: row.get(0)?,
                product_name: row.get(1)?,
                quantity: u8::try_from(row.get::<_, i64>(2)?).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Integer,
                        Box::new(error),
                    )
                })?,
                price_cents: u32::try_from(row.get::<_, i64>(3)?).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Integer,
                        Box::new(error),
                    )
                })?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Order {
        id,
        owner,
        total_cents,
        created_at: header.2,
        lines,
    })
}
