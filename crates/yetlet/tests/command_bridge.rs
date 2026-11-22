#![cfg(all(feature = "command-bridge", not(target_arch = "wasm32")))]

use std::{
    sync::mpsc::{self, SyncSender},
    time::Duration,
};
use yetlet::command_bridge::{AdmissionError, CommandBridge, CommandIngress};

const BUDGET: Duration = Duration::from_secs(4);

#[test]
fn bounded_pinned_snapshots_reject_stale_cross_generation_and_eviction() {
    assert!(CommandBridge::<u64, u64>::new(0, 2).is_err());
    assert!(CommandBridge::<u64, u64>::new(2, 0).is_err());
    let bridge = CommandBridge::<u64, u64>::new(1, 2).unwrap();
    let history = bridge.snapshots();
    assert!(history.latest().unwrap().is_none());
    bridge.publish_snapshot(9, 4, 44).unwrap();
    assert!(bridge.publish_snapshot(9, 4, 45).is_err());
    assert!(bridge.publish_snapshot(8, 5, 55).is_err());
    bridge.publish_snapshot(9, 5, 55).unwrap();
    bridge.publish_snapshot(9, 6, 66).unwrap();
    assert!(history.at_revision(4).unwrap().is_none());
    assert_eq!(history.at_revision(5).unwrap().unwrap().snapshot, 55);
    assert_eq!(history.latest().unwrap().unwrap().revision, 6);
    assert!(bridge.commands().discard_after_close().is_err());
    // An unrelated bridge's admission closure must not authorize draining
    // this still-open receiver. The drain now checks only its own gate.
    let other = CommandBridge::<u64, u64>::new(1, 1).unwrap();
    other.close_admission().unwrap();
    assert!(bridge.commands().discard_after_close().is_err());
    bridge.ingress().try_submit(7).unwrap();
    assert_eq!(bridge.ingress().try_submit(8), Err(AdmissionError::Full));
    bridge.close_admission().unwrap();
    assert_eq!(bridge.commands().discard_after_close().unwrap(), 1);
    assert_eq!(bridge.ingress().try_submit(9), Err(AdmissionError::Closed));
}

struct DropReentry {
    ingress: CommandIngress<DropReentry>,
    seen: SyncSender<()>,
}
impl Drop for DropReentry {
    fn drop(&mut self) {
        // A rejected/discarded message's destructor legitimately reenters
        // the very same ingress. The adapter must not hold that mutex.
        self.ingress.is_closed().unwrap();
        self.seen.send(()).unwrap();
    }
}

#[test]
fn rejected_and_discarded_destructors_can_reenter_same_gate() {
    let bridge = CommandBridge::<DropReentry, u64>::new(1, 1).unwrap();
    let (seen, returned) = mpsc::sync_channel(3);
    let new_command = || DropReentry {
        ingress: bridge.ingress(),
        seen: seen.clone(),
    };
    bridge.ingress().try_submit(new_command()).unwrap();
    assert_eq!(
        bridge.ingress().try_submit(new_command()),
        Err(AdmissionError::Full)
    );
    returned
        .recv_timeout(BUDGET)
        .expect("Full reject must drop outside lock");
    bridge.close_admission().unwrap();
    assert_eq!(bridge.commands().discard_after_close().unwrap(), 1);
    returned
        .recv_timeout(BUDGET)
        .expect("discard must drop outside lock");
    assert_eq!(
        bridge.ingress().try_submit(new_command()),
        Err(AdmissionError::Closed)
    );
    returned
        .recv_timeout(BUDGET)
        .expect("Closed reject must drop outside lock");
}
