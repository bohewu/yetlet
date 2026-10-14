//! Application-owned synthetic work records, SSR and thin Experimental renderer binding.
pub use yetlet_wgpu::{
    BackingSize, Batch, Lifetime, MAX_RECORDS, PickToken, PickViewport, RECORD_BYTES, backing_size,
    pick_records, validate_records,
};
pub const RECORDS: u32 = 120;
pub const WIDTH: u32 = 600;
pub const HEIGHT: u32 = 400;

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
    fn picking_mapping_ids_invalids_bounds_and_last_drawn_overlap() {
        let bytes = instances(RECORDS).unwrap();
        for (width, height, dpr) in [
            (600.0, 400.0, 1.0),
            (300.0, 200.0, 2.0),
            (240.0, 320.0, 3.0),
        ] {
            let viewport = PickViewport { width, height, dpr };
            for id in [1, 2, 13, 120] {
                let x = (f64::from((id - 1) % 12) + 0.5) * width / 12.0;
                let y = (f64::from((id - 1) / 12) + 0.5) * height / 10.0;
                assert_eq!(pick_records(&bytes, viewport, x, y), Ok(Some(id)));
                let changed = status_patch(&bytes, 0, RECORDS, 2, 1).unwrap();
                assert_eq!(pick_records(&changed, viewport, x, y), Ok(Some(id)));
            }
            for (x, y) in [
                (-1.0, 0.0),
                (width, 0.0),
                (0.0, height),
                (0.0, -1.0),
                (0.0, 0.0),
            ] {
                assert_eq!(pick_records(&bytes, viewport, x, y), Ok(None));
            }
            assert!(pick_records(&bytes, viewport, f64::NAN, 10.0).is_err());
            assert!(pick_records(&bytes, viewport, 10.0, f64::INFINITY).is_err());
        }
        let viewport = PickViewport {
            width: 600.0,
            height: 400.0,
            dpr: 1.0,
        };
        let rectangle = [0f32, 0., 0.5, 0.5, 1., 0., 0., 1.]
            .map(f32::to_le_bytes)
            .concat();
        let overlap = [rectangle.clone(), rectangle.clone()].concat();
        for (x, y) in [(150., 100.), (450., 300.), (300., 200.)] {
            assert_eq!(pick_records(&overlap, viewport, x, y), Ok(Some(2)));
        }
        assert_eq!(pick_records(&overlap, viewport, 149.0, 100.0), Ok(None));
        assert!(pick_records(&[], viewport, 300., 200.).is_err());
        for bad in [f64::NAN, f64::INFINITY, -1.0] {
            assert!(
                pick_records(
                    &rectangle,
                    PickViewport {
                        width: bad,
                        ..viewport
                    },
                    0.,
                    0.
                )
                .is_err()
            );
            assert!(
                pick_records(
                    &rectangle,
                    PickViewport {
                        dpr: bad,
                        ..viewport
                    },
                    0.,
                    0.
                )
                .is_err()
            );
        }
        assert_eq!(
            pick_records(
                &rectangle,
                PickViewport {
                    height: 0.,
                    ..viewport
                },
                0.,
                0.
            ),
            Ok(None)
        );
        let maximum = scene_records(MAX_RECORDS, 0).unwrap();
        assert_eq!(pick_records(&maximum, viewport, 0.5, 0.5), Ok(Some(1)));
    }
    #[test]
    fn picks_reject_stale_data_and_retired_generations() {
        let mut batch = Batch::new(instances(RECORDS).unwrap()).unwrap();
        let lifetime = Lifetime::new();
        let generation = 7;
        let token = PickToken {
            generation,
            version: batch.version(),
            id: 1,
        };
        assert!(token.valid(&lifetime, generation, &batch));
        assert!(!token.valid(&lifetime, generation + 1, &batch));
        assert!(!batch.accepts_pick(token.version, 0));
        assert!(!batch.accepts_pick(token.version, RECORDS + 1));
        assert!(batch.replace(&[]).is_err());
        assert!(token.valid(&lifetime, generation, &batch));
        batch.replace(&instances(RECORDS).unwrap()).unwrap();
        assert!(!token.valid(&lifetime, generation, &batch));
        let version = batch.version();
        batch.update(0, &instances(1).unwrap()).unwrap();
        assert!(!batch.accepts_pick(version, 1));
        let fresh = PickToken {
            generation,
            version: batch.version(),
            id: 1,
        };
        assert!(fresh.valid(&lifetime, generation, &batch));
        lifetime.retire();
        assert!(!fresh.valid(&lifetime, generation, &batch));
        // Version exhaustion and atomicity are exercised inside the extracted crate.
        let before = batch.bytes().to_vec();
        assert!(batch.replace(&[]).is_err());
        assert_eq!(batch.bytes(), before);
    }
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
        batch.clear_dirty();
        for step in 1..=8 {
            batch
                .update(0, &status_patch(&original, 0, 10, 7, step).unwrap())
                .unwrap();
        }
        assert_eq!(batch.dirty_ranges().collect::<Vec<_>>(), vec![0..320]);
        assert_eq!(&batch.bytes()[320..], &original[320..]);
        assert_eq!(
            &batch.bytes()[..320],
            status_patch(&original, 0, 10, 7, 8).unwrap()
        );
        let full = status_patch(&original, 0, 1000, 7, 9).unwrap();
        batch.update(0, &full).unwrap();
        assert_eq!(batch.dirty_ranges().collect::<Vec<_>>(), vec![0..32000]);
        batch.replace(&scene_records(10000, 1).unwrap()).unwrap();
        assert_eq!(batch.bytes().len(), 320000);
        let before = batch.bytes().to_vec();
        for bad in [vec![], vec![0; 31], vec![0; 32], vec![0; 3_200_032]] {
            assert!(batch.replace(&bad).is_err());
            assert!(batch.update(0, &bad).is_err());
            assert_eq!(batch.bytes(), before);
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
        assert_eq!(batch.bytes(), before);
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
