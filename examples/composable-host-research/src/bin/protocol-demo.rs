//! YT-271 D1, real in-process protocol codec/owner/replica flow.
//! No UDP/TCP socket, network runtime, OS worker or public framework API.
use yetlet_composable_host_research::{
    protocol::{
        AdmissionError, Binding, EndpointId, HostFixture, InMemoryMailbox, Message, ReplicaFixture,
        encode,
    },
    simulation::{Direction, Player, Position},
};

fn main() {
    // Trusted host assigns source provenance, peer and session mapping.
    // Two peers intentionally share one game-session ID: peer identity is
    // unique, while a shared session is allowed by the private contract.
    let mut host = HostFixture::new(
        1,
        [
            (
                EndpointId(1001),
                Binding {
                    peer: 11,
                    session: 101,
                    epoch: 7,
                    player: Player::One,
                },
            ),
            (
                EndpointId(1002),
                Binding {
                    peer: 12,
                    session: 101,
                    epoch: 7,
                    player: Player::Two,
                },
            ),
        ],
    )
    .expect("valid trusted bindings");

    host.admit_local(Player::One, Direction::East)
        .expect("local control");
    let frame = encode(&Message::HeldInput {
        peer: 12,
        session: 101,
        epoch: 7,
        sequence: 1,
        tick: 1,
        direction: Direction::West,
    })
    .expect("bounded canonical input");
    let mut mailbox = InMemoryMailbox::new();
    mailbox
        .push(EndpointId(1002), frame.clone())
        .expect("finite in-memory inbox");
    while let Some((source, bytes)) = mailbox.pop() {
        host.admit(source, &bytes).expect("verified peer binding");
    }

    assert_eq!(host.snapshot().tick(), 0); // submit is NOT apply
    let published = host.advance().expect("one existing C1 reducer");
    assert_eq!(published.positions()[0], Position { x: 3, y: 2 });
    assert_eq!(published.positions()[1], Position { x: 19, y: 12 });

    let mut replica =
        ReplicaFixture::new(EndpointId(9001), 101, 7, 1).expect("trusted Host identity");
    let snapshot_frame = host.snapshot_frame(101, 7).expect("immutable snapshot");
    let received = replica
        .admit_snapshot(EndpointId(9001), &snapshot_frame)
        .expect("new complete publication");
    assert_eq!(received.positions, published.positions());
    assert_eq!(
        replica
            .admit_snapshot(EndpointId(9001), &snapshot_frame)
            .err(),
        Some(AdmissionError::StaleSnapshot),
    );
    assert_eq!(
        host.admit(EndpointId(1002), &frame).err(),
        Some(AdmissionError::StaleSequence),
    );
    println!(
        "M28-D1: in-memory only; protocol_v1; tick={} revision={}; positions=({},{})+({},{}) replay=rejected",
        published.tick(),
        published.revision(),
        received.positions[0].x,
        received.positions[0].y,
        received.positions[1].x,
        received.positions[1].y
    );
}
