//! F0-B1: a single caller-local C1 game owner with two concrete attachments.
use yetlet_composable_host_research::{
    headless_composition_f0b1::{Cleanup, HeadlessComposition, Join},
    simulation::{Command, Direction, Input, Player},
};

fn main() {
    let mut app = HeadlessComposition::new(7, 2).expect("finite caller-local source");
    app.start().expect("input and observer ready");
    app.submit_frame(vec![Input {
        generation: 7,
        tick: 1,
        sequence: 1,
        command: Command::Hold(Player::One, Direction::East),
    }])
    .expect("frame admitted");
    let snapshot = app.drive_one().expect("authoritative C1 step");
    assert_eq!(snapshot.positions()[0].x, 3);
    let report = app.stop();
    assert_eq!(report.cleanup, Cleanup::Complete);
    assert_eq!(report.join, Join::NotApplicable);
    println!(
        "F0-B1: one HeadlessRunner, finite input, read-only observer; generation={} tick={} revision={} first_x={} join=not_applicable",
        snapshot.generation(),
        snapshot.tick(),
        snapshot.revision(),
        snapshot.positions()[0].x,
    );
}
