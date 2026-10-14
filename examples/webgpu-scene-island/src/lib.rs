//! Application-owned YT-075 browser proof; no public scene API.
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

pub const RECORDS: u32 = 120;
pub const MAX_RECORDS: u32 = 100_000;
pub const RECORD_BYTES: usize = 32;
pub const WIDTH: u32 = 600;
pub const HEIGHT: u32 = 400;

/// An initialization can commit only while its own lifetime is initializing.
#[derive(Clone)]
pub struct Lifetime(Arc<AtomicU8>);
const INERT: u8 = 0;
const INITIALIZING: u8 = 1;
const READY: u8 = 2;
const FAILED: u8 = 3;
const DISPOSED: u8 = 4;
impl Lifetime {
    pub fn new() -> Self {
        Self(Arc::new(AtomicU8::new(INERT)))
    }
    pub fn active(&self) -> bool {
        self.0.load(Ordering::SeqCst) < FAILED
    }
    pub fn begin(&self) -> bool {
        self.0
            .compare_exchange(INERT, INITIALIZING, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
    pub fn commit(&self) -> bool {
        self.0
            .compare_exchange(INITIALIZING, READY, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
    pub fn fail(&self) -> bool {
        self.0
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |state| {
                (state < FAILED).then_some(FAILED)
            })
            .is_ok()
    }
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
pub struct BackingSize {
    pub width: u32,
    pub height: u32,
    pub dpr: f64,
}

/// CSS pixels and scene coordinates stay separate from bounded physical pixels.
/// Zero/detached targets keep their previous backing allocation and suspend.
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

/// Deterministic application grid. Default 120 geometry is retained byte-for-byte.
/// Dynamic rectangles use integer CSS-pixel bounds, avoiding renderer-specific AA.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn scene_records(count: u32, seed: u32) -> Result<Vec<u8>, String> {
    if count == 0 || count > MAX_RECORDS {
        return Err("record count outside fixture bounds".into());
    }
    let mut bytes = Vec::with_capacity(count as usize * RECORD_BYTES);
    let columns = ((f64::from(count) * 1.5).sqrt().ceil() as u32).min(WIDTH);
    let rows = count.div_ceil(columns);
    for id in 0..count {
        let geometry = if count == RECORDS {
            [
                -1.0 + (id % 12) as f32 / 6.0 + 1.0 / 12.0,
                1.0 - (id / 12) as f32 / 5.0 - 0.1,
                0.06,
                0.07,
            ]
        } else {
            let left = (id % columns * WIDTH / columns) as f32;
            let top = (id / columns * HEIGHT / rows) as f32;
            let width = (WIDTH / columns * 3 / 4).max(1) as f32;
            let height = (HEIGHT / rows * 3 / 4).max(1) as f32;
            [
                (left + width / 2.0) / 300.0 - 1.0,
                1.0 - (top + height / 2.0) / 200.0,
                width / 600.0,
                height / 400.0,
            ]
        };
        for value in geometry.into_iter().chain(record_color(id, seed, 0)) {
            bytes.extend(value.to_le_bytes());
        }
    }
    validate_records(&bytes).map_err(str::to_owned)?;
    Ok(bytes)
}
pub fn instances(count: u32) -> Result<Vec<u8>, &'static str> {
    if count > RECORDS {
        return Err("record count outside fixture bounds");
    }
    scene_records(count, 0).map_err(|_| "record count outside fixture bounds")
}
fn record_color(id: u32, seed: u32, step: u32) -> [f32; 4] {
    let mut color = [0.0, 0.0, 0.0, 1.0];
    color[((id % 3 + seed % 3 + step % 3) % 3) as usize] = 1.0;
    color
}
/// One canonical preparation path for both Canvas2D and GPU payloads.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn status_patch(
    data: &[u8],
    start: u32,
    length: u32,
    seed: u32,
    step: u32,
) -> Result<Vec<u8>, String> {
    let count = validate_records(data).map_err(str::to_owned)?;
    let end = start.checked_add(length).ok_or("invalid update range")?;
    if length == 0 || end > count {
        return Err("invalid update range".into());
    }
    let mut patch = data[start as usize * RECORD_BYTES..end as usize * RECORD_BYTES].to_vec();
    for (offset, record) in patch
        .as_chunks_mut::<RECORD_BYTES>()
        .0
        .iter_mut()
        .enumerate()
    {
        for (slot, value) in record[16..]
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(record_color(start + offset as u32, seed, step))
        {
            slot.copy_from_slice(&value.to_le_bytes());
        }
    }
    Ok(patch)
}
/// Validate the entire payload before mutation or affected GPU work.
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
/// Retained CPU state lives outside LocalGraph. One dirty bit per record, no queue.
pub struct Batch {
    pub bytes: Vec<u8>,
    pub dirty: Vec<bool>,
}
impl Batch {
    pub fn new(bytes: Vec<u8>) -> Result<Self, &'static str> {
        let count = validate_records(&bytes)?;
        Ok(Self {
            bytes,
            dirty: vec![true; count as usize],
        })
    }
    pub fn replace(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        let count = validate_records(bytes)?;
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
        Ok(())
    }
    pub fn update(&mut self, start: u32, bytes: &[u8]) -> Result<(), &'static str> {
        let count = validate_records(bytes)?;
        let end = start.checked_add(count).ok_or("invalid update range")?;
        if end as usize > self.dirty.len() {
            return Err("invalid update range");
        }
        self.bytes[start as usize * RECORD_BYTES..end as usize * RECORD_BYTES]
            .copy_from_slice(bytes);
        self.dirty[start as usize..end as usize].fill(true);
        Ok(())
    }
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

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
#[cfg(target_arch = "wasm32")]
pub use browser::SceneOwner;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_scene_has_exact_finite_records() {
        assert!(instances(0).is_err());
        assert!(instances(RECORDS + 1).is_err());
        let bytes = instances(RECORDS).unwrap();
        assert_eq!(bytes.len(), 120 * 32);
        for value in bytes.as_chunks::<4>().0 {
            assert!(f32::from_le_bytes(*value).is_finite());
        }
        assert_eq!(
            &bytes[16..32],
            &[1f32, 0., 0., 1.].map(f32::to_le_bytes).concat()
        );
    }
    #[test]
    fn dynamic_counts_geometry_and_exact_colors() {
        for count in [1_000, 10_000, 50_000, MAX_RECORDS] {
            let a = scene_records(count, 7).unwrap();
            assert_eq!(a, scene_records(count, 7).unwrap());
            assert_eq!(validate_records(&a), Ok(count));
            for (id, record) in a.as_chunks::<32>().0.iter().enumerate() {
                assert_eq!(
                    &record[16..],
                    record_color(id as u32, 7, 0).map(f32::to_le_bytes).concat()
                );
            }
        }
        assert!(scene_records(MAX_RECORDS + 1, 0).is_err());
        assert!(scene_records(0, 0).is_err());
    }
    #[test]
    fn sparse_full_replace_coalesce_and_invalid_atomicity() {
        let original = scene_records(1000, 7).unwrap();
        let mut batch = Batch::new(original.clone()).unwrap();
        batch.dirty.fill(false);
        for step in 1..=8 {
            batch
                .update(0, &status_patch(&original, 0, 10, 7, step).unwrap())
                .unwrap();
        }
        assert_eq!(batch.dirty_ranges().collect::<Vec<_>>(), vec![0..320]);
        assert_eq!(&batch.bytes[320..], &original[320..]);
        assert_eq!(
            &batch.bytes[..320],
            status_patch(&original, 0, 10, 7, 8).unwrap()
        );
        let full = status_patch(&original, 0, 1000, 7, 9).unwrap();
        batch.update(0, &full).unwrap();
        assert_eq!(batch.dirty_ranges().collect::<Vec<_>>(), vec![0..32000]);
        batch.replace(&scene_records(10000, 1).unwrap()).unwrap();
        assert_eq!(batch.bytes.len(), 320000);
        let before = batch.bytes.clone();
        for bad in [vec![], vec![0; 31], vec![0; 32], vec![0; 3_200_032]] {
            assert!(batch.replace(&bad).is_err());
            assert!(batch.update(0, &bad).is_err());
            assert_eq!(batch.bytes, before);
        }
        for value in [f32::NAN, f32::INFINITY, -2.0, 2.0] {
            let mut bad = original[..32].to_vec();
            bad[..4].copy_from_slice(&value.to_le_bytes());
            assert!(batch.update(0, &bad).is_err());
        }
        let mut bad = original[..32].to_vec();
        bad[16..20].copy_from_slice(&1.1f32.to_le_bytes());
        assert!(batch.update(0, &bad).is_err());
        assert!(batch.update(u32::MAX, &original[..32]).is_err());
        assert!(batch.update(10000, &original[..32]).is_err());
        assert!(status_patch(&original, 999, 2, 0, 0).is_err());
        assert_eq!(batch.bytes, before);
    }
    #[test]
    fn viewport_finite_bounded_and_suspended() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            assert!(backing_size(bad, 400.0, 1.0, true, 8192).is_err());
            assert!(backing_size(600.0, bad, 1.0, true, 8192).is_err());
            assert!(backing_size(600.0, 400.0, bad, true, 8192).is_err());
        }
        assert!(backing_size(600.0, 400.0, 0.0, true, 8192).is_err());
        assert!(backing_size(600.0, 400.0, 1.0, true, 0).is_err());
        assert_eq!(backing_size(0.0, 400.0, 1.0, true, 8192), Ok(None));
        assert_eq!(backing_size(600.0, 0.0, 1.0, true, 8192), Ok(None));
        assert_eq!(backing_size(600.0, 400.0, 1.0, false, 8192), Ok(None));
        assert_eq!(
            backing_size(600.0, 400.0, 3.0, true, 8192).unwrap(),
            Some(BackingSize {
                width: 1200,
                height: 800,
                dpr: 2.0
            })
        );
        assert_eq!(
            backing_size(f64::MAX, f64::MAX, 2.0, true, 4096).unwrap(),
            Some(BackingSize {
                width: 4096,
                height: 4096,
                dpr: 2.0
            })
        );
        assert_eq!(
            backing_size(0.01, 0.01, 0.25, true, 16).unwrap(),
            Some(BackingSize {
                width: 1,
                height: 1,
                dpr: 1.0
            })
        );
    }
    #[test]
    fn initialization_commits_once_and_failure_requires_new_owner() {
        let owner = Lifetime::new();
        assert!(!owner.commit());
        assert!(owner.begin());
        assert!(!owner.begin());
        assert!(owner.commit());
        assert!(!owner.commit());
        assert!(owner.fail());
        assert!(!owner.active());
        assert!(!owner.begin());
        assert!(!owner.commit());
        assert!(!owner.fail());
        let recovery = Lifetime::new();
        assert!(recovery.begin());
        owner.retire();
        assert!(recovery.commit());
        let validation_pending = Lifetime::new();
        assert!(validation_pending.begin());
        validation_pending.retire();
        assert!(!validation_pending.commit());
    }
    #[test]
    fn pending_retirement_does_not_retire_replacement() {
        let old = Lifetime::new();
        let pending = old.clone();
        old.retire();
        let replacement = Lifetime::new();
        assert!(!pending.active());
        assert!(replacement.active());
        old.retire();
        assert!(replacement.active());
    }
}
