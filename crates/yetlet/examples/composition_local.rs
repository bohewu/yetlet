//! Caller-local public API example; no HTTP, socket, display or async runtime.
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;
use yetlet::composition::{Builder, Completion, Phase};

#[derive(Default)]
struct RevisionAudit {
    revisions: Vec<u64>,
}

impl RevisionAudit {
    fn record(&mut self, revision: u64) {
        self.revisions.push(revision);
    }
}

fn main() {
    let revision = Rc::new(Cell::new(0_u64));
    let domain_stopped = Rc::new(Cell::new(false));
    let close_receipt = domain_stopped.clone();
    let mut audit = RevisionAudit::default();
    let mut app = Builder::<&'static str>::new()
        .required_local("domain", revision.clone(), move |_| {
            close_receipt.set(true);
            Ok(())
        })
        .borrowed("revision-audit", &mut audit)
        .build();

    assert!(app.all_required_ready());
    for next_revision in 1..=3 {
        app.parts_mut()
            .previous()
            .last()
            .value_mut()
            .expect("domain remains active")
            .set(next_revision);
        app.parts_mut()
            .last()
            .value_mut()
            .expect("audit remains active")
            .record(revision.get());
    }

    assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(app.phase(), Phase::Stopped);
    assert!(app.failures().primary().is_none());
    assert!(domain_stopped.get());
    drop(app);

    assert_eq!(audit.revisions, [1, 2, 3]);
    println!(
        "composition_local: revisions={:?}, domain_stopped={}, caller_audit_retained=true",
        audit.revisions,
        domain_stopped.get()
    );
}
