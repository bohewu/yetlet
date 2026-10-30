//! Explicit fixture-only, fixed-cardinality observations. Never records identity/input.
use serde_json::{Value, json};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use yetlet::session_redis::RedisSessionError;
const BOUNDS_US: [u64; 13] = [
    100,
    250,
    500,
    1000,
    2000,
    5000,
    10000,
    25000,
    50000,
    100000,
    250000,
    750000,
    u64::MAX,
];
#[derive(Default)]
pub(super) struct Histogram {
    count: AtomicU64,
    sum_us: AtomicU64,
    max_us: AtomicU64,
    buckets: [AtomicU64; 13],
}
impl Histogram {
    pub(super) fn record(&self, elapsed: Duration) {
        let us = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        let bucket = BOUNDS_US
            .iter()
            .position(|bound| us <= *bound)
            .expect("overflow bucket");
        self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
        self.sum_us.fetch_add(us, Ordering::Relaxed);
        self.max_us.fetch_max(us, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }
    fn snapshot(&self) -> Value {
        json!({"count":self.count.load(Ordering::Relaxed),"sum_us":self.sum_us.load(Ordering::Relaxed),"cumulative_max_us":self.max_us.load(Ordering::Relaxed),"bounds_us":BOUNDS_US.iter().map(|b|if *b==u64::MAX {None}else{Some(*b)}).collect::<Vec<_>>(),"buckets":self.buckets.iter().map(|b|b.load(Ordering::Relaxed)).collect::<Vec<_>>()})
    }
}
const SQL_NAMES: [&str; 5] = [
    "pool_timeout",
    "pool_other",
    "query_deadline",
    "query_statement_timeout",
    "query_other",
];
const SESSION_NAMES: [&str; 8] = [
    "busy",
    "queue_timeout",
    "dispatch_deadline",
    "unavailable",
    "lifecycle",
    "invalid_cookie",
    "record",
    "other",
];
#[derive(Default)]
pub(super) struct Observations {
    pub(super) pool_acquire: Histogram,
    pub(super) query: Histogram,
    pub(super) session_read: Histogram,
    sql_errors: [AtomicU64; 5],
    session_errors: [AtomicU64; 8],
}
impl Observations {
    pub(super) fn acquire_error(&self, error: &sqlx::Error) {
        let index = if matches!(error, sqlx::Error::PoolTimedOut) {
            0
        } else {
            1
        };
        self.sql_errors[index].fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn query_deadline(&self) {
        self.sql_errors[2].fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn query_error(&self, error: &sqlx::Error) {
        let index = if error.as_database_error().and_then(|e| e.code()).as_deref() == Some("57014")
        {
            3
        } else {
            4
        };
        self.sql_errors[index].fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn session_error(&self, error: &RedisSessionError) {
        use RedisSessionError::*;
        let index = match error {
            Busy => 0,
            QueueTimeout => 1,
            DispatchDeadline => 2,
            Closed | Unavailable => 3,
            Missing | Expired | Rotated | Revoked => 4,
            InvalidCookie => 5,
            InvalidRecord | Codec | Protocol | PayloadTooLarge => 6,
            _ => 7,
        };
        self.session_errors[index].fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn snapshot(&self) -> Value {
        let sql = SQL_NAMES
            .iter()
            .zip(&self.sql_errors)
            .map(|(n, v)| (n.to_string(), json!(v.load(Ordering::Relaxed))))
            .collect::<serde_json::Map<_, _>>();
        let session = SESSION_NAMES
            .iter()
            .zip(&self.session_errors)
            .map(|(n, v)| (n.to_string(), json!(v.load(Ordering::Relaxed))))
            .collect::<serde_json::Map<_, _>>();
        json!({"pool_acquire":self.pool_acquire.snapshot(),"query":self.query.snapshot(),"session_read":self.session_read.snapshot(),"sql_errors":sql,"session_errors":session})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_bucket_count_sum_and_closed_errors() {
        let observations = std::sync::Arc::new(Observations::default());
        let threads = (0..4)
            .map(|_| {
                let o = observations.clone();
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        o.pool_acquire.record(Duration::from_micros(100));
                        o.pool_acquire.record(Duration::from_micros(101));
                    }
                })
            })
            .collect::<Vec<_>>();
        for t in threads {
            t.join().unwrap();
        }
        observations.acquire_error(&sqlx::Error::PoolTimedOut);
        observations.query_deadline();
        observations.query_error(&sqlx::Error::RowNotFound);
        observations.session_error(&RedisSessionError::QueueTimeout);
        observations.session_error(&RedisSessionError::Revoked);
        let s = observations.snapshot();
        assert_eq!(s["pool_acquire"]["count"], 800);
        assert_eq!(s["pool_acquire"]["sum_us"], 80400);
        assert_eq!(s["pool_acquire"]["cumulative_max_us"], 101);
        assert_eq!(s["pool_acquire"]["buckets"][0], 400);
        assert_eq!(s["pool_acquire"]["buckets"][1], 400);
        assert!(s["pool_acquire"]["bounds_us"][12].is_null());
        assert_eq!(s["sql_errors"]["pool_timeout"], 1);
        assert_eq!(s["sql_errors"]["query_deadline"], 1);
        assert_eq!(s["sql_errors"]["query_other"], 1);
        assert_eq!(s["session_errors"]["queue_timeout"], 1);
        assert_eq!(s["session_errors"]["lifecycle"], 1);
    }
}
