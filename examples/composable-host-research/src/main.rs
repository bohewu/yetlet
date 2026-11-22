use std::rc::Rc;
use yetlet_composable_host_research::{PanelHost, RuleHost, StateOwner};

fn main() {
    // One composition root, one OS thread; no host-managed scheduler or IO.
    let mut owner = StateOwner::new(1, 2);
    let (commands, snapshots) = owner.ports();
    let mut panel = PanelHost::new(commands.clone(), snapshots.clone());
    let mut rule = RuleHost::new(commands, snapshots);

    panel.start().expect("panel starts");
    rule.start().expect("rule starts");
    owner.open();

    panel.credit(8).expect("panel command admitted");
    rule.debit(3).expect("rule command admitted");
    assert_eq!(owner.apply_pending().expect("single state owner"), 2);

    let a = panel.snapshot().expect("panel reads");
    let b = rule.snapshot().expect("rule reads");
    assert!(Rc::ptr_eq(&a, &b));
    println!(
        "M28-B1: two caller-thread hosts; generation={} revision={} balance={}",
        a.generation, a.revision, a.balance
    );

    panel.stop().expect("panel stops independently");
    rule.stop().expect("rule stops independently");
    assert_eq!(owner.close(), 0);
}
