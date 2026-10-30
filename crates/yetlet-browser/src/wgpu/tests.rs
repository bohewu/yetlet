use super::*;

fn rectangle() -> Vec<u8> {
    [0f32, 0., 0.5, 0.5, 1., 0., 0., 1.]
        .map(f32::to_le_bytes)
        .concat()
}

#[test]
fn typed_cpu_errors_preserve_bounded_messages_and_atomicity() {
    fn assert_std_error<E: std::error::Error>() {}
    assert_std_error::<SceneError>();
    let bytes = rectangle();
    assert_eq!(validate_records(&[]), Err(SceneError::InvalidRecordLength));
    assert_eq!(
        validate_records(&[0; 32]),
        Err(SceneError::InvalidRecordData)
    );
    let viewport = PickViewport {
        width: 600.,
        height: 400.,
        dpr: 1.,
    };
    assert_eq!(
        backing_size(f64::NAN, 400., 1., true, 4096),
        Err(SceneError::InvalidViewport)
    );
    assert_eq!(
        pick_records(&bytes, viewport, f64::INFINITY, 0.),
        Err(SceneError::InvalidPointer)
    );
    let mut batch = Batch::new(bytes.clone()).unwrap();
    let initial = batch.bytes().to_vec();
    assert_eq!(batch.update(1, &bytes), Err(SceneError::InvalidUpdateRange));
    assert_eq!(batch.bytes(), initial);
    batch.version = u32::MAX;
    assert_eq!(batch.replace(&bytes), Err(SceneError::VersionExhausted));
    assert_eq!(batch.bytes(), initial);
    assert_eq!(batch.version(), u32::MAX);

    for (error, message) in [
        (
            SceneError::InvalidRecordLength,
            "invalid record byte length",
        ),
        (
            SceneError::InvalidRecordData,
            "invalid finite bounded geometry/color",
        ),
        (SceneError::InvalidViewport, "Invalid viewport input"),
        (SceneError::InvalidPointer, "Invalid pointer input"),
        (SceneError::InvalidUpdateRange, "invalid update range"),
        (SceneError::VersionExhausted, "data version exhausted"),
    ] {
        assert_eq!(error.as_str(), message);
        assert_eq!(error.to_string(), message);
    }
}

#[test]
fn retirement_signals_first_transition_once_even_after_failure() {
    let lifetime = Lifetime::new();
    let cloned = lifetime.clone();
    assert!(lifetime.begin());
    assert!(cloned.retire());
    assert!(!lifetime.retire());
    assert!(!cloned.commit());
    assert!(!lifetime.active());
    let failed = Lifetime::new();
    assert!(failed.begin());
    assert!(failed.fail());
    assert!(failed.retire());
    assert!(!failed.retire());
    assert!(!failed.fail());
    assert!(!failed.commit());
}

#[test]
fn bounded_validation_and_atomic_version_exhaustion() {
    let bytes = rectangle();
    assert_eq!(validate_records(&bytes), Ok(1));
    let maximum = bytes.repeat(MAX_RECORDS as usize);
    assert_eq!(validate_records(&maximum), Ok(MAX_RECORDS));
    for bad in [
        vec![],
        vec![0; 31],
        vec![0; 32],
        bytes.repeat(MAX_RECORDS as usize + 1),
    ] {
        assert!(Batch::new(bad).is_err());
    }
    for (offset, value) in [(0, f32::NAN), (0, f32::INFINITY), (8, 0.), (16, 1.1)] {
        let mut bad = bytes.clone();
        bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(validate_records(&bad).is_err());
    }
    let mut batch = Batch::new(bytes.clone()).unwrap();
    batch.version = u32::MAX;
    assert!(batch.replace(&bytes).is_err());
    assert!(batch.update(0, &bytes).is_err());
    assert_eq!(batch.bytes(), bytes);
    assert_eq!(batch.version(), u32::MAX);
}

#[test]
fn coalesced_batches_and_stale_picks() {
    let bytes = rectangle();
    let mut batch = Batch::new(bytes.repeat(3)).unwrap();
    let owner = Lifetime::new();
    let token = PickToken {
        generation: 7,
        version: batch.version(),
        id: 1,
    };
    assert!(token.valid(&owner, 7, &batch));
    assert!(!token.valid(&owner, 8, &batch));
    batch.clear_dirty();
    batch.update(0, &bytes).unwrap();
    batch.update(1, &bytes).unwrap();
    assert_eq!(batch.dirty_ranges().collect::<Vec<_>>(), vec![0..64]);
    assert!(!token.valid(&owner, 7, &batch));
    let before = batch.bytes().to_vec();
    let version = batch.version();
    assert!(batch.update(u32::MAX, &bytes).is_err());
    assert!(batch.replace(&[]).is_err());
    assert_eq!(batch.bytes(), before);
    assert_eq!(batch.version(), version);
    batch.replace(&bytes).unwrap();
    let token = PickToken {
        generation: 7,
        version: batch.version(),
        id: 1,
    };
    assert!(token.valid(&owner, 7, &batch));
    owner.retire();
    assert!(!token.valid(&owner, 7, &batch));
}

#[test]
fn viewport_and_reverse_pick_bounds() {
    let bytes = rectangle().repeat(2);
    for (width, height, dpr) in [(600., 400., 1.), (300., 200., 2.), (240., 320., 3.)] {
        let viewport = PickViewport { width, height, dpr };
        assert_eq!(
            pick_records(&bytes, viewport, width / 2., height / 2.),
            Ok(Some(2))
        );
        assert_eq!(
            pick_records(&bytes, viewport, width / 4., height / 4.),
            Ok(Some(2))
        );
        assert_eq!(pick_records(&bytes, viewport, width, height / 2.), Ok(None));
        assert!(pick_records(&bytes, viewport, f64::NAN, 0.).is_err());
    }
    assert_eq!(backing_size(0., 400., 1., true, 4096), Ok(None));
    assert_eq!(backing_size(600., 400., 1., false, 4096), Ok(None));
    assert!(backing_size(f64::INFINITY, 400., 1., true, 4096).is_err());
    assert_eq!(
        backing_size(f64::MAX, f64::MAX, 3., true, 4096),
        Ok(Some(BackingSize {
            width: 4096,
            height: 4096,
            dpr: 2.
        }))
    );
}

#[test]
fn retired_initialization_cannot_commit_or_fail_replacement() {
    let old = Lifetime::new();
    assert!(old.begin());
    assert!(!old.begin());
    let pending = old.clone();
    old.retire();
    assert!(!pending.commit());
    assert!(!pending.fail());
    let replacement = Lifetime::new();
    assert!(replacement.begin());
    assert!(replacement.commit());
    assert!(!replacement.commit());
    assert!(replacement.fail());
    assert!(!replacement.active());
}
