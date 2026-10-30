#![cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
use yetlet::rate_limit::{InvalidRateLimitInput, Key, Policy};

#[test]
fn invalid_policy_and_key_return_value_free_public_error() {
    let policy: Result<Policy, InvalidRateLimitInput> = Policy::new(Duration::ZERO, 1, 1);
    let key: Result<Key, InvalidRateLimitInput> = Key::new([]);
    for result in [policy.map(|_| ()), key.map(|_| ())] {
        let Err(error @ InvalidRateLimitInput) = result else {
            panic!("invalid input accepted");
        };
        assert_eq!(error.to_string(), "invalid rate-limit configuration or key");
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn positional_policy_and_key_construction_keep_existing_bounds_and_redaction() {
    for (refill_interval, burst, max_keys) in [
        (Duration::ZERO, 1, 1),
        (Duration::from_secs(1), 0, 1),
        (Duration::from_secs(1), 1, 0),
        (Duration::from_secs(1), 1, 4097),
        (Duration::from_secs(86_400), 2, 1),
        (Duration::MAX, u32::MAX, 1),
    ] {
        assert_eq!(
            Policy::new(refill_interval, burst, max_keys).unwrap_err(),
            InvalidRateLimitInput
        );
    }
    assert!(Policy::new(Duration::from_secs(86_400), 1, 4096).is_ok());
    assert!(Policy::new(Duration::from_nanos(1), u32::MAX, 1).is_ok());
    assert_eq!(Key::new([]).unwrap_err(), InvalidRateLimitInput);
    assert!(Key::new([0; 1]).is_ok());
    assert!(Key::new([0; 512]).is_ok());
    assert_eq!(Key::new([0; 513]).unwrap_err(), InvalidRateLimitInput);
    assert_eq!(
        format!("{:?}", Key::new("private-account").unwrap()),
        "Key([REDACTED])"
    );
}
