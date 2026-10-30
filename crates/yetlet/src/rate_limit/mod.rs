//! Opt-in, per-process Tower admission budgets. See the crate README for scope,
//! startup composition, trusted key selection and capacity limitations.

#![doc = include_str!("../../guides/rate-limit.md")]

#[cfg(target_arch = "wasm32")]
compile_error!("yetlet-rate-limit is native-only; keep it out of browser dependencies");

use axum::{
    http::{Request, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use governor::{
    Quota, RateLimiter,
    clock::FakeRelativeClock,
    state::{InMemoryState, NotKeyed},
};
use std::{
    collections::HashMap,
    fmt,
    future::Future,
    num::NonZeroU32,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tower::{Layer, Service};

const MAX_REFILL: Duration = Duration::from_secs(86_400);
// Governor uses unchecked u64 nanosecond arithmetic. Keep every clock sample
// below 100 years; even start + relative debt + one day remains far below u64
// (~584 years). Snapshot the validated time so a suspended thread cannot cross
// this bound between validation and Governor's clock read.
const MAX_AGE: Duration = Duration::from_secs(100 * 365 * 86_400);

/// Invalid policy or key construction. Contains no caller-provided values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidRateLimitInput;
impl fmt::Display for InvalidRateLimitInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid rate-limit configuration or key")
    }
}
impl std::error::Error for InvalidRateLimitInput {}

/// Immutable configuration. Reusing a Policy does not share accounting.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    refill_interval: Duration,
    burst: NonZeroU32,
    max_keys: usize,
    full_refill: Duration,
}
impl Policy {
    /// Validate a nonzero one-cell refill interval and burst, 1–4096 retained keys,
    /// and a checked full-refill duration of at most one day.
    pub fn new(
        refill_interval: Duration,
        burst: u32,
        max_keys: usize,
    ) -> Result<Self, InvalidRateLimitInput> {
        let burst = NonZeroU32::new(burst).ok_or(InvalidRateLimitInput)?;
        let full_refill = refill_interval
            .checked_mul(burst.get())
            .ok_or(InvalidRateLimitInput)?;
        if refill_interval.is_zero() || full_refill > MAX_REFILL || !(1..=4096).contains(&max_keys)
        {
            return Err(InvalidRateLimitInput);
        }
        Ok(Self {
            refill_interval,
            burst,
            max_keys,
            full_refill,
        })
    }
}

/// Opaque 1–512 byte accounting identity. Debug output never reveals the key.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Key(Box<[u8]>);
impl Key {
    /// Checks length before copying; encoding and trust provenance are application-owned.
    pub fn new(bytes: impl AsRef<[u8]>) -> Result<Self, InvalidRateLimitInput> {
        let bytes = bytes.as_ref();
        if !(1..=512).contains(&bytes.len()) {
            return Err(InvalidRateLimitInput);
        }
        Ok(Self(bytes.into()))
    }
}
impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key([REDACTED])")
    }
}

type Direct = RateLimiter<
    NotKeyed,
    InMemoryState,
    FakeRelativeClock,
    governor::middleware::NoOpMiddleware<governor::nanos::Nanos>,
>;
struct Entry {
    limiter: Direct,
    last_success: Duration,
}
struct State {
    policy: Policy,
    entries: HashMap<Key, Entry>,
    snapshot: FakeRelativeClock,
    snapshot_time: Duration,
    last_observed: Duration,
    unavailable: bool,
}
#[derive(Debug, Eq, PartialEq)]
enum Denial {
    Limited(Duration),
    Unavailable,
}
impl State {
    fn new(policy: Policy) -> Self {
        Self {
            policy,
            entries: HashMap::new(),
            snapshot: FakeRelativeClock::default(),
            snapshot_time: Duration::ZERO,
            last_observed: Duration::ZERO,
            unavailable: false,
        }
    }
    fn observe(&mut self, now: Option<Duration>) -> Result<Duration, Denial> {
        match now {
            Some(now) if !self.unavailable && now >= self.last_observed && now <= MAX_AGE => {
                self.last_observed = now;
                Ok(now)
            }
            _ => {
                self.unavailable = true;
                Err(Denial::Unavailable)
            }
        }
    }
    fn check(&mut self, key: Key, mut now: impl FnMut() -> Option<Duration>) -> Result<(), Denial> {
        let before = self.observe(now())?;
        self.snapshot.advance(
            before
                .checked_sub(self.snapshot_time)
                .ok_or(Denial::Unavailable)?,
        );
        self.snapshot_time = before;
        if !self.entries.contains_key(&key) {
            // A post-success sample plus a complete refill is conservative even
            // when time passes during admission. Rejections never move that sample.
            let refill = self.policy.full_refill;
            self.entries.retain(|_, entry| {
                before
                    .checked_sub(entry.last_success)
                    .is_none_or(|age| age < refill)
            });
            if self.entries.len() >= self.policy.max_keys {
                return Err(Denial::Unavailable);
            }
            let quota = Quota::with_period(self.policy.refill_interval)
                .expect("validated nonzero interval")
                .allow_burst(self.policy.burst);
            self.entries.insert(
                key.clone(),
                Entry {
                    limiter: RateLimiter::direct_with_clock(quota, self.snapshot.clone()),
                    last_success: before,
                },
            );
        }
        let entry = self
            .entries
            .get_mut(&key)
            .expect("entry inserted under the same lock");
        match entry.limiter.check() {
            Ok(()) => {
                let after = self.observe(now())?;
                self.entries
                    .get_mut(&key)
                    .expect("entry retained under the same lock")
                    .last_success = after;
                Ok(())
            }
            Err(denial) => Err(Denial::Limited(denial.wait_time_from(before.into()))),
        }
    }
}
struct Inner {
    origin: Instant,
    state: Mutex<State>,
}
/// Independent accounting at construction; all clones share this exact budget.
/// State is released when its final Budget/layer/service handle is dropped.
#[derive(Clone)]
pub struct Budget(Arc<Inner>);
impl Budget {
    pub fn new(policy: Policy) -> Self {
        Self(Arc::new(Inner {
            origin: Instant::now(),
            state: Mutex::new(State::new(policy)),
        }))
    }
    fn check(&self, key: Key) -> Result<(), Denial> {
        self.0
            .state
            .lock()
            .map_err(|_| Denial::Unavailable)?
            .check(key, || Instant::now().checked_duration_since(self.0.origin))
    }
}

/// Binds an explicit budget to synchronous, immutable request-parts extraction.
/// Install with native `MethodRouter::route_layer` after registering methods to
/// preserve 404/405. Extractors return a trusted Key or an explicit Response.
#[derive(Clone)]
pub struct RateLimitLayer<F> {
    budget: Budget,
    extract: F,
}
impl<F> RateLimitLayer<F> {
    pub fn new(budget: Budget, extract: F) -> Self {
        Self { budget, extract }
    }
}
impl<S, F: Clone> Layer<S> for RateLimitLayer<F> {
    type Service = RateLimitService<S, F>;
    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            budget: self.budget.clone(),
            extract: self.extract.clone(),
        }
    }
}
/// Tower service produced by [`RateLimitLayer`]. Clones retain shared accounting.
#[derive(Clone)]
pub struct RateLimitService<S, F> {
    inner: S,
    budget: Budget,
    extract: F,
}
impl<S, F, B> Service<Request<B>> for RateLimitService<S, F>
where
    S: Service<Request<B>, Response = Response>,
    S::Future: Send + 'static,
    S::Error: 'static,
    F: Fn(&Parts) -> Result<Key, Response>,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    #[allow(
        clippy::result_large_err,
        reason = "preserve native Axum rejection responses"
    )]
    fn call(&mut self, request: Request<B>) -> Self::Future {
        let (parts, body) = request.into_parts();
        let result = (self.extract)(&parts)
            .map_err(no_store)
            .and_then(|key| self.budget.check(key).map_err(rejection));
        if let Err(response) = result {
            return Box::pin(async move { Ok(response) });
        }
        // Call the actual ready instance. Do not clone/swap it or hold the lock
        // while downstream executes. Charging is synchronous and never refunded.
        Box::pin(self.inner.call(Request::from_parts(parts, body)))
    }
}
fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}
fn rejection(denial: Denial) -> Response {
    match denial {
        Denial::Unavailable => {
            no_store((StatusCode::SERVICE_UNAVAILABLE, "Service unavailable").into_response())
        }
        Denial::Limited(wait) => {
            let seconds = (wait.as_secs() + u64::from(wait.subsec_nanos() != 0)).max(1);
            let mut response =
                no_store((StatusCode::TOO_MANY_REQUESTS, "Too many requests").into_response());
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, seconds.into());
            response
        }
    }
}

#[cfg(test)]
mod tests;
