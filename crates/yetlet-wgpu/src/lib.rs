//! Experimental, explicitly opt-in flat rectangle renderer. No performance,
//! stable API, production readiness or publication guarantee. No 3D API.
//!
//! Default features provide CPU-only validation, batches, picking and lifetimes,
//! with no dependencies or GPU initialization. The `browser` feature on wasm32
//! exposes `browser::SceneOwner`: an explicitly initialized, owned canvas/device.
//! See the package README for lifecycle, error and selected support limitations.
//! Selected Firefox 158 X11/Intel opt-in candidate evidence is in the README.
//! Other configurations and stable/production support are not implied.
//!
//! Packed records contain eight little-endian f32 values: center x/y, half width/
//! height and RGBA. Geometry fits normalized `[-1,1]`, extents are positive and
//! colors are `[0,1]`. Maximum 100,000 records, 32 bytes each. No user shader text.
//!
//! ```
//! use yetlet_wgpu::{Batch, PickViewport, pick_records};
//! let bytes = [0f32, 0., 0.5, 0.5, 1., 0., 0., 1.]
//!     .map(f32::to_le_bytes).concat();
//! let batch = Batch::new(bytes)?;
//! let viewport = PickViewport { width: 600., height: 400., dpr: 1. };
//! assert_eq!(pick_records(batch.bytes(), viewport, 300., 200.)?, Some(1));
//! # Ok::<(), &'static str>(())
//! ```
#![deny(missing_docs)]

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

/// Experimental maximum accepted rectangle count (100,000).
pub const MAX_RECORDS: u32 = 100_000;
/// Experimental packed rectangle stride: eight little-endian f32 values.
pub const RECORD_BYTES: usize = 32;

/// An initialization can commit only while its own lifetime is initializing.
#[derive(Clone)]
/// Experimental CPU contract; no device or application domain state.
pub struct Lifetime(Arc<AtomicU8>);
const INERT: u8 = 0;
const INITIALIZING: u8 = 1;
const READY: u8 = 2;
const FAILED: u8 = 3;
const DISPOSED: u8 = 4;
impl Lifetime {
    /// Experimental constructor; no GPU initialization occurs.
    pub fn new() -> Self {
        Self(Arc::new(AtomicU8::new(INERT)))
    }
    /// Experimental guard: true only before failure or disposal.
    pub fn active(&self) -> bool {
        self.0.load(Ordering::SeqCst) < FAILED
    }
    /// Experimental one-shot transition from inert to initializing.
    pub fn begin(&self) -> bool {
        self.0
            .compare_exchange(INERT, INITIALIZING, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
    /// Experimental one-shot transition from initializing to ready.
    pub fn commit(&self) -> bool {
        self.0
            .compare_exchange(INITIALIZING, READY, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
    /// Experimental terminal failure; recovery requires a fresh owner.
    pub fn fail(&self) -> bool {
        self.0
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |state| {
                (state < FAILED).then_some(FAILED)
            })
            .is_ok()
    }
    /// Experimental idempotent terminal disposal, invalidating pending results.
    pub fn retire(&self) {
        self.0.store(DISPOSED, Ordering::SeqCst);
    }
}
impl Default for Lifetime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Experimental bounded physical canvas dimensions and clamped DPR.
pub struct BackingSize {
    /// Experimental width in this type's documented coordinate units.
    pub width: u32,
    /// Experimental height in this type's documented coordinate units.
    pub height: u32,
    /// Experimental DPR, clamped to `[1,2]` for backing allocation.
    pub dpr: f64,
}

/// CSS pixels and scene coordinates stay separate from bounded physical pixels.
/// Zero/detached targets keep their previous backing allocation and suspend.
/// Experimental CPU contract; no device or application domain state.
pub fn backing_size(
    width: f64,
    height: f64,
    dpr: f64,
    connected: bool,
    device_limit: u32,
) -> Result<Option<BackingSize>, &'static str> {
    if !width.is_finite()
        || !height.is_finite()
        || !dpr.is_finite()
        || width < 0.0
        || height < 0.0
        || dpr <= 0.0
        || device_limit == 0
    {
        return Err("Invalid viewport input");
    }
    if !connected || width == 0.0 || height == 0.0 {
        return Ok(None);
    }
    let dpr = dpr.clamp(1.0, 2.0);
    let pixels = |css: f64| {
        (css.min(f64::from(device_limit) / dpr) * dpr)
            .round()
            .clamp(1.0, f64::from(device_limit)) as u32
    };
    Ok(Some(BackingSize {
        width: pixels(width),
        height: pixels(height),
        dpr,
    }))
}

/// Validate the entire payload before mutation or affected GPU work.
/// Experimental CPU contract; no device or application domain state.
pub fn validate_records(bytes: &[u8]) -> Result<u32, &'static str> {
    if bytes.is_empty()
        || !bytes.len().is_multiple_of(RECORD_BYTES)
        || bytes.len() > MAX_RECORDS as usize * RECORD_BYTES
    {
        return Err("invalid record byte length");
    }
    for record in bytes.as_chunks::<RECORD_BYTES>().0 {
        let mut values = [0.0f32; 8];
        for (slot, chunk) in values.iter_mut().zip(record.as_chunks::<4>().0) {
            *slot = f32::from_le_bytes(*chunk);
        }
        if !values.iter().all(|v| v.is_finite())
            || values[0].abs() > 1.0
            || values[1].abs() > 1.0
            || !(0.0..=1.0).contains(&values[2])
            || values[2] == 0.0
            || !(0.0..=1.0).contains(&values[3])
            || values[3] == 0.0
            || values[0].abs() + values[2] > 1.000_001
            || values[1].abs() + values[3] > 1.000_001
            || values[4..].iter().any(|v| !(0.0..=1.0).contains(v))
        {
            return Err("invalid finite bounded geometry/color");
        }
    }
    Ok((bytes.len() / RECORD_BYTES) as u32)
}

/// CSS-local input maps through bounded physical pixels into normalized scene
/// coordinates. DPR cancels in the ratio; it must never scale input twice.
#[derive(Clone, Copy)]
/// Experimental CPU contract; no device or application domain state.
pub struct PickViewport {
    /// Experimental width in this type's documented coordinate units.
    pub width: f64,
    /// Experimental height in this type's documented coordinate units.
    pub height: f64,
    /// Experimental DPR, clamped to `[1,2]` for backing allocation.
    pub dpr: f64,
}
/// Deferred selection carries both owner identity and data identity to its sink.
/// Experimental CPU contract; no device or application domain state.
pub struct PickToken {
    /// Experimental application-selected owner identity.
    pub generation: u32,
    /// Experimental checked batch data identity.
    pub version: u32,
    /// Experimental one-based rectangle slot; zero is a miss.
    pub id: u32,
}
impl PickToken {
    /// Experimental guard rejecting retired owners, stale generations or data versions.
    pub fn valid(&self, lifetime: &Lifetime, generation: u32, batch: &Batch) -> bool {
        lifetime.active()
            && self.generation == generation
            && batch.accepts_pick(self.version, self.id)
    }
}
impl PickViewport {
    fn point(self, x: f64, y: f64) -> Result<Option<[f64; 2]>, &'static str> {
        let size = backing_size(self.width, self.height, self.dpr, true, u32::MAX)?;
        if !x.is_finite() || !y.is_finite() {
            return Err("Invalid pointer input");
        }
        let Some(size) = size else { return Ok(None) };
        // Canvas exterior is excluded; rectangle edges inside it are inclusive.
        if x < 0.0 || y < 0.0 || x >= self.width || y >= self.height {
            return Ok(None);
        }
        let physical_x = x / self.width * f64::from(size.width);
        let physical_y = y / self.height * f64::from(size.height);
        Ok(Some([
            physical_x / f64::from(size.width) * 2.0 - 1.0,
            1.0 - physical_y / f64::from(size.height) * 2.0,
        ]))
    }
}

/// Bounded O(n) CPU picking on explicit selection only, never GPU readback.
/// Record slot + 1 is the returned index identity; applications own semantic IDs.
/// Updates retain slots; replacements must preserve ordering to retain identity. Later slots are drawn last and win overlaps.
/// Experimental CPU contract; no device or application domain state.
pub fn pick_records(
    bytes: &[u8],
    viewport: PickViewport,
    x: f64,
    y: f64,
) -> Result<Option<u32>, &'static str> {
    validate_records(bytes)?;
    let Some([x, y]) = viewport.point(x, y)? else {
        return Ok(None);
    };
    for (index, record) in bytes.as_chunks::<RECORD_BYTES>().0.iter().enumerate().rev() {
        let geometry: [f64; 4] =
            std::array::from_fn(|i| f64::from(f32::from_le_bytes(record.as_chunks::<4>().0[i])));
        if x >= geometry[0] - geometry[2]
            && x <= geometry[0] + geometry[2]
            && y >= geometry[1] - geometry[3]
            && y <= geometry[1] + geometry[3]
        {
            return Ok(Some(index as u32 + 1));
        }
    }
    Ok(None)
}
/// Experimental validated CPU rectangle data, independent of any device.
/// Private storage preserves validation and checked data versions. One dirty bit
/// per rectangle coalesces uploads without an event queue.
pub struct Batch {
    pub(crate) bytes: Vec<u8>,
    pub(crate) dirty: Vec<bool>,
    pub(crate) version: u32,
}
impl Batch {
    /// Experimental immutable validated packed rectangle payload.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Experimental checked data identity; successful updates/replacements advance it.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Experimental upload acknowledgement; clear only after consuming dirty ranges.
    pub fn clear_dirty(&mut self) {
        self.dirty.fill(false);
    }

    /// Experimental constructor; no GPU initialization occurs.
    pub fn new(bytes: Vec<u8>) -> Result<Self, &'static str> {
        let count = validate_records(&bytes)?;
        Ok(Self {
            bytes,
            dirty: vec![true; count as usize],
            version: 1,
        })
    }
    /// Experimental atomic full replacement; invalid input/version exhaustion leaves data intact.
    pub fn replace(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        let count = validate_records(bytes)?;
        let version = self
            .version
            .checked_add(1)
            .ok_or("data version exhausted")?;
        if self.bytes.capacity() < bytes.len() {
            self.bytes = bytes.to_vec();
        } else {
            self.bytes.clear();
            self.bytes.extend_from_slice(bytes);
        }
        if self.dirty.capacity() < count as usize {
            self.dirty = vec![true; count as usize];
        } else {
            self.dirty.resize(count as usize, true);
            self.dirty.fill(true);
        }
        self.version = version;
        Ok(())
    }
    /// Experimental validated contiguous update; invalid range/version leaves data intact.
    pub fn update(&mut self, start: u32, bytes: &[u8]) -> Result<(), &'static str> {
        let count = validate_records(bytes)?;
        let end = start.checked_add(count).ok_or("invalid update range")?;
        if end as usize > self.dirty.len() {
            return Err("invalid update range");
        }
        let version = self
            .version
            .checked_add(1)
            .ok_or("data version exhausted")?;
        self.bytes[start as usize * RECORD_BYTES..end as usize * RECORD_BYTES]
            .copy_from_slice(bytes);
        self.dirty[start as usize..end as usize].fill(true);
        self.version = version;
        Ok(())
    }
    /// Experimental guard for the current version and valid one-based slot.
    pub fn accepts_pick(&self, version: u32, id: u32) -> bool {
        version == self.version && id > 0 && id as usize <= self.dirty.len()
    }
    /// Experimental coalesced contiguous byte ranges awaiting upload.
    pub fn dirty_ranges(&self) -> impl Iterator<Item = std::ops::Range<usize>> + '_ {
        let mut index = 0;
        std::iter::from_fn(move || {
            while index < self.dirty.len() && !self.dirty[index] {
                index += 1;
            }
            if index == self.dirty.len() {
                return None;
            }
            let start = index;
            while index < self.dirty.len() && self.dirty[index] {
                index += 1;
            }
            Some(start * RECORD_BYTES..index * RECORD_BYTES)
        })
    }
}

/// Experimental browser-only owner; requires explicit `browser` feature and wasm32.
#[cfg(all(target_arch = "wasm32", feature = "browser"))]
pub mod browser;

#[cfg(test)]
mod tests;
