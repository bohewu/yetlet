use super::*;
use governor::clock::Clock;
use std::{sync::Barrier, thread};

fn key(value: &str) -> Key {
    Key::new(value).unwrap()
}
fn policy(burst: u32, capacity: usize) -> Policy {
    Policy::new(Duration::from_secs(2), burst, capacity).unwrap()
}
fn check(state: &mut State, clock: &FakeRelativeClock, value: &str) -> Result<(), Denial> {
    state.check(key(value), || Some(clock.now().into()))
}

#[test]
fn configuration_bounds_and_redaction() {
    for (interval, burst, cap) in [
        (Duration::ZERO, 1, 1),
        (Duration::from_secs(1), 0, 1),
        (Duration::from_secs(1), 1, 0),
        (Duration::from_secs(1), 1, 4097),
        (MAX_REFILL, 2, 1),
        (Duration::MAX, u32::MAX, 1),
    ] {
        assert!(Policy::new(interval, burst, cap).is_err());
    }
    assert!(Policy::new(MAX_REFILL, 1, 4096).is_ok());
    assert!(Policy::new(Duration::from_nanos(1), u32::MAX, 1).is_ok());
    assert!(Key::new([]).is_err());
    assert!(Key::new([0; 513]).is_err());
    assert!(Key::new([0; 512]).is_ok());
    assert_eq!(format!("{:?}", key("private-account")), "Key([REDACTED])");
    assert!(
        !Key::new([])
            .unwrap_err()
            .to_string()
            .contains("private-account")
    );
}

#[test]
fn burst_refill_and_safe_eviction_without_rejection_refresh() {
    let clock = FakeRelativeClock::default();
    let mut state = State::new(policy(5, 1));
    for _ in 0..5 {
        assert_eq!(check(&mut state, &clock, "a"), Ok(()));
    }
    assert_eq!(
        check(&mut state, &clock, "a"),
        Err(Denial::Limited(Duration::from_secs(2)))
    );
    assert_eq!(check(&mut state, &clock, "b"), Err(Denial::Unavailable));
    clock.advance(Duration::from_secs(2));
    assert_eq!(check(&mut state, &clock, "a"), Ok(()));
    assert!(matches!(
        check(&mut state, &clock, "a"),
        Err(Denial::Limited(_))
    ));
    clock.advance(Duration::from_secs(9));
    assert_eq!(check(&mut state, &clock, "b"), Err(Denial::Unavailable));
    clock.advance(Duration::from_secs(1));
    assert_eq!(check(&mut state, &clock, "b"), Ok(()));
    assert!(!state.entries.contains_key(&key("a")));

    let mut state = State::new(policy(1, 1));
    assert_eq!(check(&mut state, &clock, "a"), Ok(()));
    clock.advance(Duration::from_millis(1999));
    assert!(matches!(
        check(&mut state, &clock, "a"),
        Err(Denial::Limited(_))
    ));
    clock.advance(Duration::from_millis(1));
    assert_eq!(check(&mut state, &clock, "b"), Ok(()));
}

#[test]
fn retention_uses_post_success_clock_sample() {
    let clock = FakeRelativeClock::default();
    let mut state = State::new(policy(1, 1));
    let mut samples = 0;
    assert_eq!(
        state.check(key("a"), || {
            samples += 1;
            if samples == 2 {
                clock.advance(Duration::from_secs(1));
            }
            Some(clock.now().into())
        }),
        Ok(())
    );
    clock.advance(Duration::from_secs(1));
    assert_eq!(check(&mut state, &clock, "b"), Err(Denial::Unavailable));
    clock.advance(Duration::from_secs(1));
    assert_eq!(check(&mut state, &clock, "b"), Ok(()));
}

#[test]
fn guarded_clock_horizon_regression_and_postcheck_failure_fail_closed() {
    let clock = FakeRelativeClock::default();
    let mut state = State::new(policy(1, 1));
    clock.advance(MAX_AGE);
    assert_eq!(check(&mut state, &clock, "a"), Ok(()));
    clock.advance(Duration::from_nanos(1));
    assert_eq!(check(&mut state, &clock, "a"), Err(Denial::Unavailable));
    assert_eq!(
        state.check(key("a"), || Some(Duration::ZERO)),
        Err(Denial::Unavailable)
    );
    let mut state = State::new(policy(1, 1));
    assert_eq!(
        state.check(key("a"), || Some(Duration::from_secs(1))),
        Ok(())
    );
    assert_eq!(
        state.check(key("b"), || Some(Duration::ZERO)),
        Err(Denial::Unavailable)
    );
    let mut state = State::new(policy(1, 1));
    let mut samples = [Some(Duration::ZERO), None].into_iter();
    assert_eq!(
        state.check(key("a"), || samples.next().unwrap()),
        Err(Denial::Unavailable)
    );
    assert_eq!(
        state.check(key("b"), || Some(MAX_REFILL)),
        Err(Denial::Unavailable)
    );
}

#[test]
fn same_policy_independent_budgets_and_shared_clones() {
    let config = policy(1, 1);
    let one = Budget::new(config);
    let two = Budget::new(config);
    let shared = one.clone();
    assert_eq!(one.check(key("a")), Ok(()));
    assert!(matches!(shared.check(key("a")), Err(Denial::Limited(_))));
    assert_eq!(two.check(key("a")), Ok(()));
    let weak = Arc::downgrade(&one.0);
    drop(one);
    assert!(weak.upgrade().is_some());
    drop(shared);
    assert!(weak.upgrade().is_none());
}

#[test]
fn concurrent_same_key_charges_once_and_capacity_cannot_overfill() {
    for distinct in [false, true] {
        let budget = Budget::new(Policy::new(MAX_REFILL, 1, 4).unwrap());
        let barrier = Arc::new(Barrier::new(16));
        let handles: Vec<_> = (0..16)
            .map(|n| {
                let budget = budget.clone();
                let barrier = barrier.clone();
                thread::spawn(move || {
                    barrier.wait();
                    budget
                        .check(key(&if distinct {
                            n.to_string()
                        } else {
                            "shared".into()
                        }))
                        .is_ok()
                })
            })
            .collect();
        let admitted = handles
            .into_iter()
            .map(|h| usize::from(h.join().unwrap()))
            .sum::<usize>();
        assert_eq!(admitted, if distinct { 4 } else { 1 });
        assert_eq!(budget.0.state.lock().unwrap().entries.len(), admitted);
    }
}

#[test]
fn poisoned_storage_and_http_denials_are_private_and_uncacheable() {
    let budget = Budget::new(policy(1, 1));
    let other = budget.clone();
    assert!(
        thread::spawn(move || {
            let _guard = other.0.state.lock().unwrap();
            panic!("test poison");
        })
        .join()
        .is_err()
    );
    assert_eq!(budget.check(key("secret")), Err(Denial::Unavailable));
    for (wait, expected) in [
        (Duration::ZERO, "1"),
        (Duration::from_millis(1001), "2"),
        (MAX_REFILL, "86400"),
    ] {
        let response = rejection(Denial::Limited(wait));
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], expected);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert!(!response.headers().contains_key(header::SET_COOKIE));
    }
    let response = rejection(Denial::Unavailable);
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(!response.headers().contains_key(header::RETRY_AFTER));
}
