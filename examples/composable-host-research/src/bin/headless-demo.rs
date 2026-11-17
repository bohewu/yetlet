//! C1: the same reducer, explicit 60 Hz logical ticks, no display or sockets.
use yetlet_composable_host_research::simulation::{
    Command, Direction, HeadlessRunner, Input, Player, Position, TICKS_PER_SECOND,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = |tick, sequence, command| Input {
        generation: 7,
        tick,
        sequence,
        command,
    };
    let frames = [
        vec![
            input(1, 2, Command::Hold(Player::Two, Direction::North)),
            input(1, 1, Command::Hold(Player::One, Direction::East)),
        ],
        vec![],
        vec![input(3, 3, Command::Pause)],
        vec![input(4, 4, Command::Hold(Player::One, Direction::West))],
        vec![],
        vec![input(6, 5, Command::Resume)],
        vec![
            input(7, 7, Command::Hold(Player::Two, Direction::West)),
            input(7, 6, Command::Hold(Player::One, Direction::South)),
        ],
        vec![],
    ];
    let mut first = HeadlessRunner::new(7)?;
    let mut replay = HeadlessRunner::new(7)?;
    for frame in &frames {
        assert_eq!(first.tick(frame)?, replay.tick(frame)?);
    }
    let final_state = first.snapshot();
    assert_eq!(
        final_state.positions(),
        [Position { x: 4, y: 4 }, Position { x: 18, y: 10 }]
    );
    assert_eq!(
        (
            final_state.tick(),
            final_state.active_ticks(),
            final_state.revision()
        ),
        (8, 5, 8)
    );
    println!(
        "M28-C1 Twin Squares: logical_hz={} tick={} active_ticks={} revision={} positions={:?}; replay_equal=true",
        TICKS_PER_SECOND,
        final_state.tick(),
        final_state.active_ticks(),
        final_state.revision(),
        final_state.positions()
    );
    Ok(())
}
