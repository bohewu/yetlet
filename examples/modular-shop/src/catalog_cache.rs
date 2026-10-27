//! Private cache for the existing public product-display projection only.
use crate::database::{Database, DatabaseError, Product};
use moka::future::Cache;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;
use tokio::{sync::Semaphore, time::Instant};

const MAX_AGE: Duration = Duration::from_secs(2);
const WAIT: Duration = Duration::from_millis(500);
const MAX_PRODUCTS: usize = 64;
const MAX_PAYLOAD: usize = 64 * 1024;

struct Entry {
    started: Instant,
    products: Vec<Product>,
}
impl Entry {
    fn fresh(&self) -> bool {
        self.started.elapsed() < MAX_AGE
    }
    fn cacheable(&self) -> bool {
        self.products.len() <= MAX_PRODUCTS
            && self
                .products
                .len()
                .checked_mul(std::mem::size_of::<Product>())
                .and_then(|size| {
                    self.products.iter().try_fold(size, |sum, p| {
                        sum.checked_add(p.id.len())?.checked_add(p.name.len())
                    })
                })
                .is_some_and(|size| size <= MAX_PAYLOAD)
    }
}
enum FillError {
    Unavailable,
    Uncached(Arc<Entry>),
}

pub(crate) struct CatalogCache {
    values: Cache<u64, Arc<Entry>>,
    generation: AtomicU64,
    fills: Arc<Semaphore>,
    #[cfg(test)]
    source_loads: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    source_barrier: std::sync::Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            std::sync::mpsc::Receiver<()>,
        )>,
    >,
}
impl CatalogCache {
    pub(crate) fn new() -> Self {
        Self {
            values: Cache::builder()
                .max_capacity(1)
                .time_to_live(MAX_AGE)
                .build(),
            generation: AtomicU64::new(0),
            fills: Arc::new(Semaphore::new(1)),
            #[cfg(test)]
            source_loads: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            source_barrier: std::sync::Mutex::new(None),
        }
    }
    #[cfg(test)]
    pub(crate) fn before_test_source(&self) -> Result<(), DatabaseError> {
        let barrier = self.source_barrier.lock().unwrap().take();
        if let Some((entered, resume)) = barrier {
            let _ = entered.send(());
            resume
                .recv_timeout(Duration::from_secs(3))
                .map_err(|_| DatabaseError::WorkerUnavailable)?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn test_source_loads(&self) -> usize {
        self.source_loads.load(Ordering::SeqCst)
    }
    pub(crate) fn committed(&self) {
        // Saturation permanently prevents reads; never wrap to an old key.
        let mut current = self.generation.load(Ordering::Acquire);
        while current < u64::MAX {
            match self.generation.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(latest) => current = latest,
            }
        }
    }
    pub(crate) async fn read(&self, db: &Database) -> Result<Vec<Product>, DatabaseError> {
        tokio::time::timeout(WAIT, self.lookup(db))
            .await
            .map_err(|_| DatabaseError::WorkerUnavailable)?
    }
    async fn lookup(&self, db: &Database) -> Result<Vec<Product>, DatabaseError> {
        let generation = self.generation.load(Ordering::Acquire);
        if generation == u64::MAX {
            return Err(DatabaseError::WorkerUnavailable);
        }
        if let Some(entry) = self.values.get(&generation).await {
            if entry.fresh() {
                return Ok(entry.products.clone());
            }
            self.values.invalidate(&generation).await;
        }
        let result = self
            .values
            .try_get_with(generation, async {
                let permit = self
                    .fills
                    .clone()
                    .try_acquire_owned()
                    .map_err(|_| FillError::Unavailable)?;
                #[cfg(test)]
                self.source_loads.fetch_add(1, Ordering::SeqCst);
                let started = Instant::now();
                let products = db
                    .catalog_source(Some(permit))
                    .await
                    .map_err(|_| FillError::Unavailable)?;
                let entry = Arc::new(Entry { started, products });
                if !entry.fresh() {
                    return Err(FillError::Unavailable);
                }
                if !entry.cacheable() {
                    return Err(FillError::Uncached(entry));
                }
                Ok(entry)
            })
            .await;
        let entry = match result {
            Ok(entry) => entry,
            Err(error) => match error.as_ref() {
                FillError::Uncached(entry) => entry.clone(),
                FillError::Unavailable => return Err(DatabaseError::WorkerUnavailable),
            },
        };
        if !entry.fresh() {
            self.values.invalidate(&generation).await;
            return Err(DatabaseError::WorkerUnavailable);
        }
        Ok(entry.products.clone())
    }
}

#[cfg(test)]
mod tests;
