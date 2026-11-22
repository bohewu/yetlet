//! Actual B2 worker-ownership demonstration; separate from the unchanged B1 demo.
use std::{sync::Arc, time::Duration};
use yetlet_composable_host_research::worker::{CallerScheduler, Cleanup, Group, Options};

fn main() {
    let scheduler = CallerScheduler::default();
    let mut group = Group::start(Options::default(), &scheduler).expect("both hosts ready");
    group.panel_credit(8).expect("panel admission");
    group.rule_debit(3).expect("rule admission");

    let snapshot = group
        .wait_revision(2, Duration::from_secs(1))
        .expect("worker published two commands");
    assert!(Arc::ptr_eq(&snapshot, &group.panel_snapshot()));
    assert!(Arc::ptr_eq(&snapshot, &group.rule_snapshot()));

    let stop = group.stop(Duration::from_secs(1));
    assert_eq!(stop.cleanup, Cleanup::Joined);
    assert_eq!(scheduler.pulse(), 1);
    println!(
        "M28-B2: worker joined; generation={} revision={} balance={}; scheduler_alive=true",
        snapshot.generation, snapshot.revision, snapshot.balance,
    );
}
