//! YT-271 / M28-D1: private, std-only bounded wire/session research.
//!
//! Neither this module nor its InMemory fixtures opens a socket. A trusted
//! future NetworkHost supplies endpoint provenance, assigns a Binding, and
//! owns one C1 Simulation; a packet's peer ID alone grants no authority.
//! Epoch/session IDs are freshness/routing markers, NOT authentication.

use crate::simulation::{
    self, ARENA_HEIGHT, ARENA_WIDTH, Command, Direction, Input, MAX_COMMANDS_PER_TICK, Player,
    Position, SQUARE_SIZE, Simulation, Snapshot,
};
use std::{collections::VecDeque, rc::Rc};

pub const WIRE_VERSION: u8 = 1;
pub const HEADER_BYTES: usize = 12;
pub const MAX_WIRE_BYTES: usize = 512;
pub const MAX_INPUT_SLOTS: usize = 2;
pub const MAX_INMEMORY_FRAMES: usize = 4;
const MAGIC: &[u8; 4] = b"YTP1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecError {
    TooLarge,
    Truncated,
    Trailing,
    BadMagic,
    UnsupportedVersion,
    UnknownKind,
    InvalidBits,
    InvalidLength,
    InvalidField,
    ZeroIdentifier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireSnapshot {
    pub session: u64,
    pub epoch: u64,
    pub generation: u64,
    pub revision: u64,
    pub tick: u64,
    pub positions: [Position; 2],
    pub held: [Direction; 2],
    pub paused: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Hello {
        nonce: u64,
    },
    Welcome {
        peer: u64,
        session: u64,
        epoch: u64,
        generation: u64,
        tick: u64,
        player: Player,
    },
    HeldInput {
        peer: u64,
        session: u64,
        epoch: u64,
        sequence: u64,
        tick: u64,
        direction: Direction,
    },
    Snapshot(WireSnapshot),
    Leave {
        peer: u64,
        session: u64,
        epoch: u64,
        sequence: u64,
    },
}

impl Message {
    fn kind(&self) -> u8 {
        match self {
            Self::Hello { .. } => 1,
            Self::Welcome { .. } => 2,
            Self::HeldInput { .. } => 3,
            Self::Snapshot(_) => 4,
            Self::Leave { .. } => 5,
        }
    }
}

fn nonzero(value: u64) -> Result<u64, CodecError> {
    if value == 0 {
        Err(CodecError::ZeroIdentifier)
    } else {
        Ok(value)
    }
}
fn direction_byte(direction: Direction) -> u8 {
    match direction {
        Direction::Still => 0,
        Direction::North => 1,
        Direction::East => 2,
        Direction::South => 3,
        Direction::West => 4,
    }
}
fn direction_from(value: u8) -> Result<Direction, CodecError> {
    match value {
        0 => Ok(Direction::Still),
        1 => Ok(Direction::North),
        2 => Ok(Direction::East),
        3 => Ok(Direction::South),
        4 => Ok(Direction::West),
        _ => Err(CodecError::InvalidField),
    }
}
fn player_byte(player: Player) -> u8 {
    match player {
        Player::One => 1,
        Player::Two => 2,
    }
}
fn player_from(value: u8) -> Result<Player, CodecError> {
    match value {
        1 => Ok(Player::One),
        2 => Ok(Player::Two),
        _ => Err(CodecError::InvalidField),
    }
}
fn append_u64(dst: &mut Vec<u8>, value: u64) {
    dst.extend_from_slice(&value.to_le_bytes());
}
fn append_i16(dst: &mut Vec<u8>, value: i16) {
    dst.extend_from_slice(&value.to_le_bytes());
}
fn bounded_position(position: Position) -> Result<(i16, i16), CodecError> {
    if position.x < 0
        || position.y < 0
        || position.x > ARENA_WIDTH - SQUARE_SIZE
        || position.y > ARENA_HEIGHT - SQUARE_SIZE
    {
        return Err(CodecError::InvalidField);
    }
    Ok((
        i16::try_from(position.x).map_err(|_| CodecError::InvalidField)?,
        i16::try_from(position.y).map_err(|_| CodecError::InvalidField)?,
    ))
}

/// Canonical little-endian encoding. Fixed bounded payloads, no allocations
/// proportional to the header's untrusted length field.
pub fn encode(message: &Message) -> Result<Vec<u8>, CodecError> {
    let mut payload = Vec::with_capacity(64);
    match message {
        Message::Hello { nonce } => append_u64(&mut payload, nonzero(*nonce)?),
        Message::Welcome {
            peer,
            session,
            epoch,
            generation,
            tick,
            player,
        } => {
            for id in [*peer, *session, *epoch, *generation] {
                append_u64(&mut payload, nonzero(id)?);
            }
            append_u64(&mut payload, *tick);
            payload.push(player_byte(*player));
        }
        Message::HeldInput {
            peer,
            session,
            epoch,
            sequence,
            tick,
            direction,
        } => {
            for value in [
                nonzero(*peer)?,
                nonzero(*session)?,
                nonzero(*epoch)?,
                nonzero(*sequence)?,
                nonzero(*tick)?,
            ] {
                append_u64(&mut payload, value);
            }
            payload.push(direction_byte(*direction));
        }
        Message::Snapshot(snapshot) => {
            for value in [
                nonzero(snapshot.session)?,
                nonzero(snapshot.epoch)?,
                nonzero(snapshot.generation)?,
                snapshot.revision,
                snapshot.tick,
            ] {
                append_u64(&mut payload, value);
            }
            for position in snapshot.positions {
                let (x, y) = bounded_position(position)?;
                append_i16(&mut payload, x);
                append_i16(&mut payload, y);
            }
            payload.push(direction_byte(snapshot.held[0]));
            payload.push(direction_byte(snapshot.held[1]));
            payload.push(u8::from(snapshot.paused));
        }
        Message::Leave {
            peer,
            session,
            epoch,
            sequence,
        } => {
            for id in [
                nonzero(*peer)?,
                nonzero(*session)?,
                nonzero(*epoch)?,
                nonzero(*sequence)?,
            ] {
                append_u64(&mut payload, id);
            }
        }
    }
    let payload_len = u16::try_from(payload.len()).map_err(|_| CodecError::TooLarge)?;
    if HEADER_BYTES + payload.len() > MAX_WIRE_BYTES {
        return Err(CodecError::TooLarge);
    }
    let mut result = Vec::with_capacity(HEADER_BYTES + payload.len());
    result.extend_from_slice(MAGIC);
    result.extend_from_slice(&[WIRE_VERSION, message.kind(), 0, 0]);
    result.extend_from_slice(&payload_len.to_le_bytes());
    result.extend_from_slice(&[0, 0]);
    result.extend_from_slice(&payload);
    Ok(result)
}

struct Reader<'a> {
    payload: &'a [u8],
    index: usize,
}
impl Reader<'_> {
    fn u8(&mut self) -> u8 {
        let value = self.payload[self.index];
        self.index += 1;
        value
    }
    fn u64(&mut self) -> u64 {
        let value = u64::from_le_bytes(
            self.payload[self.index..self.index + 8]
                .try_into()
                .expect("length checked by decoder"),
        );
        self.index += 8;
        value
    }
    fn i16(&mut self) -> i16 {
        let value = i16::from_le_bytes(
            self.payload[self.index..self.index + 2]
                .try_into()
                .expect("length checked by decoder"),
        );
        self.index += 2;
        value
    }
}

/// Fail closed on truncation, trailing bytes, oversized declared payloads,
/// reserved header bits and invalid enum values before using any field.
pub fn decode(bytes: &[u8]) -> Result<Message, CodecError> {
    if bytes.len() > MAX_WIRE_BYTES {
        return Err(CodecError::TooLarge);
    }
    if bytes.len() < HEADER_BYTES {
        return Err(CodecError::Truncated);
    }
    if &bytes[..4] != MAGIC {
        return Err(CodecError::BadMagic);
    }
    if bytes[4] != WIRE_VERSION {
        return Err(CodecError::UnsupportedVersion);
    }
    let kind = bytes[5];
    let expected_payload = match kind {
        1 => 8,
        2 => 41,
        3 => 41,
        4 => 51,
        5 => 32,
        _ => return Err(CodecError::UnknownKind),
    };
    if bytes[6] != 0 || bytes[7] != 0 || bytes[10] != 0 || bytes[11] != 0 {
        return Err(CodecError::InvalidBits);
    }
    let declared = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
    if HEADER_BYTES + declared > MAX_WIRE_BYTES {
        return Err(CodecError::TooLarge);
    }
    if declared != expected_payload {
        return Err(CodecError::InvalidLength);
    }
    let needed = HEADER_BYTES + declared;
    if bytes.len() < needed {
        return Err(CodecError::Truncated);
    }
    if bytes.len() > needed {
        return Err(CodecError::Trailing);
    }
    let mut input = Reader {
        payload: &bytes[HEADER_BYTES..needed],
        index: 0,
    };
    let message = match kind {
        1 => Message::Hello {
            nonce: nonzero(input.u64())?,
        },
        2 => Message::Welcome {
            peer: nonzero(input.u64())?,
            session: nonzero(input.u64())?,
            epoch: nonzero(input.u64())?,
            generation: nonzero(input.u64())?,
            tick: input.u64(),
            player: player_from(input.u8())?,
        },
        3 => Message::HeldInput {
            peer: nonzero(input.u64())?,
            session: nonzero(input.u64())?,
            epoch: nonzero(input.u64())?,
            sequence: nonzero(input.u64())?,
            tick: nonzero(input.u64())?,
            direction: direction_from(input.u8())?,
        },
        4 => {
            let session = nonzero(input.u64())?;
            let epoch = nonzero(input.u64())?;
            let generation = nonzero(input.u64())?;
            let revision = input.u64();
            let tick = input.u64();
            let mut positions = [Position { x: 0, y: 0 }; 2];
            for position in &mut positions {
                *position = Position {
                    x: i32::from(input.i16()),
                    y: i32::from(input.i16()),
                };
                bounded_position(*position)?;
            }
            let held = [direction_from(input.u8())?, direction_from(input.u8())?];
            let paused = match input.u8() {
                0 => false,
                1 => true,
                _ => return Err(CodecError::InvalidField),
            };
            Message::Snapshot(WireSnapshot {
                session,
                epoch,
                generation,
                revision,
                tick,
                positions,
                held,
                paused,
            })
        }
        5 => Message::Leave {
            peer: nonzero(input.u64())?,
            session: nonzero(input.u64())?,
            epoch: nonzero(input.u64())?,
            sequence: nonzero(input.u64())?,
        },
        _ => unreachable!("kind validated"),
    };
    debug_assert_eq!(input.index, expected_payload);
    Ok(message)
}

/// Opaque in-memory provenance supplied by a trusted future NetworkHost.
/// This is NOT a socket endpoint or an authentication credential.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndpointId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Binding {
    pub peer: u64,
    pub session: u64,
    pub epoch: u64,
    pub player: Player,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    Codec(CodecError),
    InvalidBinding,
    QueueFull,
    GenerationMismatch,
    StaleSnapshot,
    UnknownEndpoint,
    PeerMismatch,
    SessionMismatch,
    EpochMismatch,
    WrongKind,
    StaleSequence,
    ExhaustedSequence,
    WrongTick,
    TickOverflow,
    Closed,
    Owner(simulation::Error),
}
impl From<CodecError> for AdmissionError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}
#[derive(Clone, Copy, Debug)]
struct Slot {
    endpoint: EndpointId,
    binding: Binding,
    last_sequence: u64,
    last_tick: u64,
    pending: Option<Direction>,
}
impl Slot {
    fn new(endpoint: EndpointId, binding: Binding) -> Self {
        Self {
            endpoint,
            binding,
            last_sequence: 0,
            last_tick: 0,
            pending: None,
        }
    }
}

/// One in-memory authority fixture. The packet is decoded and its trusted
/// sender endpoint mapped before committing its latest held-input slot.
/// A successful admission does NOT mutate Simulation or publish a Snapshot.
pub struct HostFixture {
    owner: Simulation,
    slots: [Slot; MAX_INPUT_SLOTS],
    accepting: bool,
}
impl HostFixture {
    pub fn new(
        generation: u64,
        peers: [(EndpointId, Binding); MAX_INPUT_SLOTS],
    ) -> Result<Self, AdmissionError> {
        let [(endpoint_one, one), (endpoint_two, two)] = peers;
        if endpoint_one.0 == 0
            || endpoint_two.0 == 0
            || endpoint_one == endpoint_two
            || one.player != Player::One
            || two.player != Player::Two
            || one.peer == two.peer
            || one.peer == 0
            || two.peer == 0
            || one.session == 0
            || two.session == 0
            || one.epoch == 0
            || two.epoch == 0
        {
            return Err(AdmissionError::InvalidBinding);
        }
        Ok(Self {
            owner: Simulation::new(generation).map_err(AdmissionError::Owner)?,
            slots: [Slot::new(endpoint_one, one), Slot::new(endpoint_two, two)],
            accepting: true,
        })
    }

    pub fn snapshot(&self) -> Rc<Snapshot> {
        self.owner.snapshot()
    }

    pub fn next_tick(&self) -> Result<u64, AdmissionError> {
        self.owner
            .next_tick()
            .map_err(|_| AdmissionError::TickOverflow)
    }

    pub fn pending_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.pending.is_some())
            .count()
    }

    /// Invalid input (including rejected session/epoch/sequence/tick) leaves
    /// both slots/ledgers and the authoritative immutable State unchanged.
    pub fn admit(&mut self, source: EndpointId, frame: &[u8]) -> Result<(), AdmissionError> {
        if !self.accepting {
            return Err(AdmissionError::Closed);
        }
        let message = decode(frame)?;
        let Some(index) = self.slots.iter().position(|slot| slot.endpoint == source) else {
            return Err(AdmissionError::UnknownEndpoint);
        };
        let Message::HeldInput {
            peer,
            session,
            epoch,
            sequence,
            tick,
            direction,
        } = message
        else {
            return Err(AdmissionError::WrongKind);
        };
        let slot = self.slots[index];
        if peer != slot.binding.peer {
            return Err(AdmissionError::PeerMismatch);
        }
        if session != slot.binding.session {
            return Err(AdmissionError::SessionMismatch);
        }
        if epoch != slot.binding.epoch {
            return Err(AdmissionError::EpochMismatch);
        }
        if sequence == u64::MAX {
            return Err(AdmissionError::ExhaustedSequence);
        }
        if sequence <= slot.last_sequence {
            return Err(AdmissionError::StaleSequence);
        }
        if tick != self.next_tick()? || tick < slot.last_tick {
            return Err(AdmissionError::WrongTick);
        }
        // One bounded, replaceable latest input per server-mapped player.
        // No Player supplied on the wire can override that mapping.
        let slot = &mut self.slots[index];
        slot.pending = Some(direction);
        slot.last_sequence = sequence;
        slot.last_tick = tick;
        Ok(())
    }

    /// A trusted caller's local controls use the same two per-player input
    /// slots and the same single authoritative Simulation as remote controls.
    /// The D1 fixture does not decide same-player local/remote priority;
    /// a future NetworkHost must choose ownership of each player.
    pub fn admit_local(
        &mut self,
        player: Player,
        direction: Direction,
    ) -> Result<(), AdmissionError> {
        if !self.accepting {
            return Err(AdmissionError::Closed);
        }
        let index = match player {
            Player::One => 0,
            Player::Two => 1,
        };
        self.slots[index].pending = Some(direction);
        Ok(())
    }

    /// The single existing C1 reducer handles *all* local/remote game
    /// commands. No independent network-side mutable game state exists.
    /// Failure preserves the exact prior Rc publication and pending slots.
    pub fn advance(&mut self) -> Result<Rc<Snapshot>, AdmissionError> {
        if !self.accepting {
            return Err(AdmissionError::Closed);
        }
        let snapshot = self.owner.snapshot();
        let tick = self.next_tick()?;
        let mut sequence = snapshot.last_sequence();
        let mut inputs = Vec::with_capacity(MAX_INPUT_SLOTS);
        for slot in &self.slots {
            if let Some(direction) = slot.pending {
                sequence = sequence
                    .checked_add(1)
                    .ok_or(AdmissionError::ExhaustedSequence)?;
                inputs.push(Input {
                    generation: snapshot.generation(),
                    tick,
                    sequence,
                    command: Command::Hold(slot.binding.player, direction),
                });
            }
        }
        debug_assert!(inputs.len() <= MAX_COMMANDS_PER_TICK);
        let next = self
            .owner
            .step(tick, &inputs)
            .map_err(AdmissionError::Owner)?;
        for slot in &mut self.slots {
            slot.pending = None;
        }
        Ok(next)
    }

    pub fn stop(&mut self) {
        self.accepting = false;
        for slot in &mut self.slots {
            slot.pending = None;
        }
    }

    pub fn snapshot_frame(&self, session: u64, epoch: u64) -> Result<Vec<u8>, CodecError> {
        let state = self.owner.snapshot();
        encode(&Message::Snapshot(WireSnapshot {
            session,
            epoch,
            generation: state.generation(),
            revision: state.revision(),
            tick: state.tick(),
            positions: state.positions(),
            held: state.held(),
            paused: state.paused(),
        }))
    }
}

/// Read-only in-memory client projection: complete Snapshot messages from
/// the trusted Host endpoint replace only strictly newer publications.
/// It never owns or modifies Simulation, and is not a network client.
pub struct ReplicaFixture {
    host_endpoint: EndpointId,
    session: u64,
    epoch: u64,
    generation: u64,
    latest: Option<Rc<WireSnapshot>>,
}
impl ReplicaFixture {
    pub fn new(
        host_endpoint: EndpointId,
        session: u64,
        epoch: u64,
        generation: u64,
    ) -> Result<Self, AdmissionError> {
        if host_endpoint.0 == 0 || session == 0 || epoch == 0 || generation == 0 {
            return Err(AdmissionError::InvalidBinding);
        }
        Ok(Self {
            host_endpoint,
            session,
            epoch,
            generation,
            latest: None,
        })
    }

    pub fn current(&self) -> Option<Rc<WireSnapshot>> {
        self.latest.as_ref().map(Rc::clone)
    }

    /// Reconnect constructs a fresh fixture with a new trusted session/epoch.
    /// This does not implement handshake, retransmission or UDP.
    pub fn admit_snapshot(
        &mut self,
        source: EndpointId,
        frame: &[u8],
    ) -> Result<Rc<WireSnapshot>, AdmissionError> {
        if source != self.host_endpoint {
            return Err(AdmissionError::UnknownEndpoint);
        }
        let Message::Snapshot(incoming) = decode(frame)? else {
            return Err(AdmissionError::WrongKind);
        };
        if incoming.session != self.session {
            return Err(AdmissionError::SessionMismatch);
        }
        if incoming.epoch != self.epoch {
            return Err(AdmissionError::EpochMismatch);
        }
        if incoming.generation != self.generation {
            return Err(AdmissionError::GenerationMismatch);
        }
        if let Some(existing) = &self.latest
            && (incoming.revision <= existing.revision || incoming.tick <= existing.tick)
        {
            return Err(AdmissionError::StaleSnapshot);
        }
        let published = Rc::new(incoming);
        self.latest = Some(Rc::clone(&published));
        Ok(published)
    }
}

pub fn next_epoch(epoch: u64) -> Result<u64, AdmissionError> {
    if epoch == 0 {
        return Err(AdmissionError::InvalidBinding);
    }
    epoch
        .checked_add(1)
        .filter(|value| *value != 0)
        .ok_or(AdmissionError::ExhaustedSequence)
}

/// A finite local-memory fixture only; no I/O, socket, OS thread or
/// universal network/Transport trait. Carries source provenance out of band.
pub struct InMemoryMailbox {
    frames: VecDeque<(EndpointId, Vec<u8>)>,
}
impl Default for InMemoryMailbox {
    fn default() -> Self {
        Self::new()
    }
}
impl InMemoryMailbox {
    pub fn new() -> Self {
        Self {
            frames: VecDeque::with_capacity(MAX_INMEMORY_FRAMES),
        }
    }
    pub fn push(&mut self, source: EndpointId, frame: Vec<u8>) -> Result<(), AdmissionError> {
        if frame.len() > MAX_WIRE_BYTES {
            return Err(AdmissionError::Codec(CodecError::TooLarge));
        }
        if self.frames.len() >= MAX_INMEMORY_FRAMES {
            return Err(AdmissionError::QueueFull);
        }
        self.frames.push_back((source, frame));
        Ok(())
    }
    pub fn pop(&mut self) -> Option<(EndpointId, Vec<u8>)> {
        self.frames.pop_front()
    }
    pub fn len(&self) -> usize {
        self.frames.len()
    }
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding_one() -> (EndpointId, Binding) {
        (
            EndpointId(4001),
            Binding {
                peer: 11,
                session: 101,
                epoch: 7,
                player: Player::One,
            },
        )
    }
    fn binding_two() -> (EndpointId, Binding) {
        (
            EndpointId(4002),
            Binding {
                peer: 12,
                session: 102,
                epoch: 8,
                player: Player::Two,
            },
        )
    }
    fn host() -> HostFixture {
        HostFixture::new(1, [binding_one(), binding_two()]).unwrap()
    }
    fn input(source: usize, sequence: u64, tick: u64, direction: Direction) -> Vec<u8> {
        let binding = if source == 0 {
            binding_one().1
        } else {
            binding_two().1
        };
        encode(&Message::HeldInput {
            peer: binding.peer,
            session: binding.session,
            epoch: binding.epoch,
            sequence,
            tick,
            direction,
        })
        .unwrap()
    }
    fn mutate_valid_payload<F>(f: F) -> Vec<u8>
    where
        F: FnOnce(&mut [u8]),
    {
        let mut bytes = input(0, 1, 1, Direction::East);
        f(&mut bytes);
        bytes
    }
    #[test]
    fn all_five_wire_message_types_roundtrip_with_canonical_bytes() {
        let game = host();
        let messages = [
            Message::Hello { nonce: 33 },
            Message::Welcome {
                peer: 11,
                session: 101,
                epoch: 7,
                generation: 1,
                tick: 0,
                player: Player::One,
            },
            Message::HeldInput {
                peer: 11,
                session: 101,
                epoch: 7,
                sequence: 1,
                tick: 1,
                direction: Direction::West,
            },
            decode(&game.snapshot_frame(101, 7).unwrap()).unwrap(),
            Message::Leave {
                peer: 11,
                session: 101,
                epoch: 7,
                sequence: 9,
            },
        ];
        let expected_lengths = [20, 53, 53, 63, 44];
        for (message, expected_length) in messages.into_iter().zip(expected_lengths) {
            let encoded = encode(&message).unwrap();
            assert_eq!(encoded.len(), expected_length);
            assert!(encoded.len() <= MAX_WIRE_BYTES);
            assert_eq!(encoded[..4], *MAGIC);
            assert_eq!(encoded[4], WIRE_VERSION);
            assert_eq!(decode(&encoded), Ok(message.clone()));
            assert_eq!(encode(&decode(&encoded).unwrap()), Ok(encoded));
        }
    }

    #[test]
    fn malformed_header_version_kind_flags_and_reserved_bits_rejected() {
        assert_eq!(
            decode(&mutate_valid_payload(|b| b[0] = b'X')),
            Err(CodecError::BadMagic)
        );
        assert_eq!(
            decode(&mutate_valid_payload(|b| b[4] = 255)),
            Err(CodecError::UnsupportedVersion)
        );
        assert_eq!(
            decode(&mutate_valid_payload(|b| b[5] = 255)),
            Err(CodecError::UnknownKind)
        );
        for index in [6, 7, 10, 11] {
            assert_eq!(
                decode(&mutate_valid_payload(|b| b[index] = 1)),
                Err(CodecError::InvalidBits)
            );
        }
    }

    #[test]
    fn truncation_declared_length_extra_bytes_and_oversize_fail_closed() {
        let data = input(0, 1, 1, Direction::East);
        for i in 0..HEADER_BYTES {
            assert_eq!(decode(&data[..i]), Err(CodecError::Truncated));
        }
        assert_eq!(decode(&data[..data.len() - 1]), Err(CodecError::Truncated));
        let mut extra = data.clone();
        extra.push(0);
        assert_eq!(decode(&extra), Err(CodecError::Trailing));
        assert_eq!(
            decode(&mutate_valid_payload(|b| b[8] = 42)),
            Err(CodecError::InvalidLength)
        );
        assert_eq!(
            decode(&mutate_valid_payload(|b| {
                b[8] = 0xFF;
                b[9] = 0x7F;
            })),
            Err(CodecError::TooLarge)
        );
        assert_eq!(
            decode(&vec![0; MAX_WIRE_BYTES + 1]),
            Err(CodecError::TooLarge)
        );
    }

    #[test]
    fn invalid_field_bytes_and_zero_identity_never_decode() {
        assert_eq!(
            decode(&mutate_valid_payload(|b| *b.last_mut().unwrap() = 254)),
            Err(CodecError::InvalidField)
        );
        assert_eq!(
            decode(&mutate_valid_payload(
                |b| b[HEADER_BYTES..HEADER_BYTES + 8].fill(0)
            )),
            Err(CodecError::ZeroIdentifier)
        );
        let mut welcome = encode(&Message::Welcome {
            peer: 11,
            session: 101,
            epoch: 7,
            generation: 1,
            tick: 0,
            player: Player::Two,
        })
        .unwrap();
        *welcome.last_mut().unwrap() = 3;
        assert_eq!(decode(&welcome), Err(CodecError::InvalidField));
        let game = host();
        let mut snapshot = game.snapshot_frame(101, 7).unwrap();
        *snapshot.last_mut().unwrap() = 8; // invalid bool bits
        assert_eq!(decode(&snapshot), Err(CodecError::InvalidField));
        let mut snapshot = game.snapshot_frame(101, 7).unwrap();
        snapshot[HEADER_BYTES + 40] = 255; // invalid negative x
        snapshot[HEADER_BYTES + 41] = 255;
        assert_eq!(decode(&snapshot), Err(CodecError::InvalidField));
    }

    #[test]
    fn new_host_bindings_require_unique_trusted_ids_and_player_mapping() {
        let mut peers = [binding_one(), binding_two()];
        peers[1].0 = peers[0].0;
        assert!(matches!(
            HostFixture::new(1, peers),
            Err(AdmissionError::InvalidBinding)
        ));
        let mut peers = [binding_one(), binding_two()];
        peers[1].1.peer = peers[0].1.peer;
        assert!(matches!(
            HostFixture::new(1, peers),
            Err(AdmissionError::InvalidBinding)
        ));
        let mut peers = [binding_one(), binding_two()];
        peers[1].1.session = 0; // zero identity is always invalid
        assert!(matches!(
            HostFixture::new(1, peers),
            Err(AdmissionError::InvalidBinding)
        ));
        let mut peers = [binding_one(), binding_two()];
        peers[1].1.player = Player::One;
        assert!(matches!(
            HostFixture::new(1, peers),
            Err(AdmissionError::InvalidBinding)
        ));
        assert!(matches!(
            HostFixture::new(0, [binding_one(), binding_two()]),
            Err(AdmissionError::Owner(simulation::Error::InvalidGeneration))
        ));
    }

    #[test]
    fn wrong_endpoint_peer_session_epoch_and_kind_preserve_authority_and_slot() {
        let mut host = host();
        let initial = host.snapshot();
        let base = input(0, 1, 1, Direction::East);
        let candidates = [
            (
                EndpointId(9999),
                base.clone(),
                AdmissionError::UnknownEndpoint,
            ),
            (binding_two().0, base.clone(), AdmissionError::PeerMismatch),
            (
                binding_one().0,
                mutate_valid_payload(|b| b[HEADER_BYTES] = 12),
                AdmissionError::PeerMismatch,
            ),
            (
                binding_one().0,
                mutate_valid_payload(|b| b[HEADER_BYTES + 8] = 102),
                AdmissionError::SessionMismatch,
            ),
            (
                binding_one().0,
                mutate_valid_payload(|b| b[HEADER_BYTES + 16] = 8),
                AdmissionError::EpochMismatch,
            ),
            (
                binding_one().0,
                encode(&Message::Hello { nonce: 44 }).unwrap(),
                AdmissionError::WrongKind,
            ),
        ];
        for (source, packet, error) in candidates {
            assert_eq!(host.admit(source, &packet), Err(error));
            assert!(Rc::ptr_eq(&initial, &host.snapshot()));
            assert_eq!(host.pending_count(), 0);
            assert_eq!(host.slots[0].last_sequence, 0);
        }
    }

    #[test]
    fn stale_replay_foreign_tick_and_overflow_do_not_consume_owner_or_slot() {
        let mut host = host();
        let original = host.snapshot();
        assert_eq!(
            host.admit(binding_one().0, &input(0, u64::MAX, 1, Direction::East)),
            Err(AdmissionError::ExhaustedSequence)
        );
        assert_eq!(
            host.admit(binding_one().0, &input(0, 1, u64::MAX, Direction::East)),
            Err(AdmissionError::WrongTick)
        );
        assert_eq!(
            host.admit(binding_one().0, &input(0, 1, 2, Direction::East)),
            Err(AdmissionError::WrongTick)
        );
        assert!(Rc::ptr_eq(&original, &host.snapshot()));
        assert_eq!(host.slots[0].last_sequence, 0);
        host.admit(binding_one().0, &input(0, 2, 1, Direction::East))
            .unwrap();
        assert_eq!(
            host.admit(binding_one().0, &input(0, 2, 1, Direction::West)),
            Err(AdmissionError::StaleSequence)
        );
        assert_eq!(
            host.admit(binding_one().0, &input(0, 1, 1, Direction::West)),
            Err(AdmissionError::StaleSequence)
        );
        assert_eq!(host.slots[0].pending, Some(Direction::East));
        assert_eq!(host.slots[0].last_sequence, 2);
        assert_eq!(host.pending_count(), 1);
        assert!(Rc::ptr_eq(&original, &host.snapshot()));
        assert_eq!(next_epoch(u64::MAX), Err(AdmissionError::ExhaustedSequence));
        assert_eq!(next_epoch(0), Err(AdmissionError::InvalidBinding));
        assert_eq!(next_epoch(7), Ok(8));
    }

    #[test]
    fn per_peer_latest_slot_and_canonical_owner_tick_are_deterministic() {
        let mut one = host();
        let mut two = host();
        let mut queue = InMemoryMailbox::new();
        let one_1 = input(0, 1, 1, Direction::North);
        let one_2 = input(0, 2, 1, Direction::East);
        let two_1 = input(1, 10, 1, Direction::West);
        // Same per-peer order but different cross-peer arrival order.
        for (source, frame) in [
            (binding_two().0, two_1.clone()),
            (binding_one().0, one_1.clone()),
            (binding_one().0, one_2.clone()),
        ] {
            queue.push(source, frame).unwrap();
        }
        while let Some((endpoint, bytes)) = queue.pop() {
            one.admit(endpoint, &bytes).unwrap();
        }
        for (source, frame) in [
            (binding_one().0, one_1),
            (binding_one().0, one_2),
            (binding_two().0, two_1),
        ] {
            two.admit(source, &frame).unwrap();
        }
        assert_eq!(one.pending_count(), 2);
        let old = one.snapshot();
        assert!(Rc::ptr_eq(&old, &one.snapshot()));
        assert_eq!(*one.advance().unwrap(), *two.advance().unwrap());
        let result = one.snapshot();
        assert_eq!(result.positions()[0], Position { x: 3, y: 2 });
        assert_eq!(result.positions()[1], Position { x: 19, y: 12 });
        assert_eq!(result.last_sequence(), 2);
        assert_eq!(one.pending_count(), 0);
        assert_eq!(old.tick(), 0);
    }

    #[test]
    fn same_game_session_allows_two_independently_mapped_peers() {
        let mut peers = [binding_one(), binding_two()];
        peers[1].1.session = peers[0].1.session;
        let game = HostFixture::new(1, peers).unwrap();
        assert_eq!(game.snapshot().tick(), 0);
        assert_eq!(game.pending_count(), 0);
    }

    #[test]
    fn local_and_verified_remote_directions_share_one_c1_owner() {
        let mut game = host();
        let before = game.snapshot();
        game.admit_local(Player::One, Direction::East).unwrap();
        game.admit(binding_two().0, &input(1, 5, 1, Direction::West))
            .unwrap();
        assert_eq!(game.pending_count(), 2);
        assert!(Rc::ptr_eq(&before, &game.snapshot()));
        let next = game.advance().unwrap();
        assert_eq!(next.positions()[0], Position { x: 3, y: 2 });
        assert_eq!(next.positions()[1], Position { x: 19, y: 12 });
        assert_eq!(next.last_sequence(), 2);
        assert_eq!(game.pending_count(), 0);
        assert!(Rc::ptr_eq(&before, &before));
    }

    #[test]
    fn replica_rejects_foreign_endpoint_session_epoch_generation_and_stale_snapshot() {
        let mut game = host();
        let mut replica = ReplicaFixture::new(EndpointId(9001), 101, 7, 1).unwrap();
        let original = game.snapshot_frame(101, 7).unwrap();
        let first = replica.admit_snapshot(EndpointId(9001), &original).unwrap();
        assert_eq!(first.tick, 0);
        assert_eq!(
            replica.current().unwrap().positions,
            game.snapshot().positions()
        );

        game.admit(binding_one().0, &input(0, 1, 1, Direction::East))
            .unwrap();
        game.advance().unwrap();
        let latest = game.snapshot_frame(101, 7).unwrap();
        let next = replica.admit_snapshot(EndpointId(9001), &latest).unwrap();
        assert_eq!(next.tick, 1);
        assert_eq!(next.revision, 1);
        let expected = replica.current().unwrap();
        assert!(Rc::ptr_eq(&next, &expected));

        let error_cases = [
            (
                EndpointId(9999),
                latest.clone(),
                AdmissionError::UnknownEndpoint,
            ),
            (EndpointId(9001), original, AdmissionError::StaleSnapshot),
            (EndpointId(9001), latest, AdmissionError::StaleSnapshot),
            (
                EndpointId(9001),
                game.snapshot_frame(102, 7).unwrap(),
                AdmissionError::SessionMismatch,
            ),
            (
                EndpointId(9001),
                game.snapshot_frame(101, 8).unwrap(),
                AdmissionError::EpochMismatch,
            ),
        ];
        for (source, frame, error) in error_cases {
            assert_eq!(replica.admit_snapshot(source, &frame).err(), Some(error));
            assert!(Rc::ptr_eq(&expected, &replica.current().unwrap()));
        }

        let mut frame = game.snapshot_frame(101, 7).unwrap();
        // Snapshot wire generation is the third u64 field in its payload.
        frame[HEADER_BYTES + 16] = 2;
        assert_eq!(
            replica.admit_snapshot(EndpointId(9001), &frame).err(),
            Some(AdmissionError::GenerationMismatch),
        );
        assert!(Rc::ptr_eq(&expected, &replica.current().unwrap()));
        assert_eq!(
            replica
                .admit_snapshot(
                    EndpointId(9001),
                    &encode(&Message::Hello { nonce: 8 }).unwrap()
                )
                .err(),
            Some(AdmissionError::WrongKind),
        );
        assert!(Rc::ptr_eq(&expected, &replica.current().unwrap()));
        assert!(matches!(
            ReplicaFixture::new(EndpointId(0), 101, 7, 1),
            Err(AdmissionError::InvalidBinding)
        ));
    }

    #[test]
    fn replica_new_epoch_fences_stale_full_snapshot_after_reconnect() {
        let game = host();
        let old = game.snapshot_frame(101, 7).unwrap();
        let mut replaced = ReplicaFixture::new(EndpointId(9001), 102, 8, 1).unwrap();
        assert_eq!(
            replaced.admit_snapshot(EndpointId(9001), &old).err(),
            Some(AdmissionError::SessionMismatch),
        );
        assert!(replaced.current().is_none());
        let correct = game.snapshot_frame(102, 8).unwrap();
        replaced.admit_snapshot(EndpointId(9001), &correct).unwrap();
        assert_eq!(replaced.current().unwrap().epoch, 8);
        assert_eq!(replaced.current().unwrap().session, 102);
    }
    #[test]
    fn invalid_frames_do_not_affect_c1_immutable_publication_or_tick() {
        let mut host = host();
        let original = host.snapshot();
        for bad in [
            vec![0; MAX_WIRE_BYTES + 1],
            vec![0],
            mutate_valid_payload(|b| b[4] = 2),
            mutate_valid_payload(|b| b[5] = 254),
            mutate_valid_payload(|b| b[7] = 1),
            mutate_valid_payload(|b| *b.last_mut().unwrap() = 50),
        ] {
            assert!(host.admit(binding_one().0, &bad).is_err());
            assert_eq!(host.pending_count(), 0);
            assert!(Rc::ptr_eq(&original, &host.snapshot()));
        }
        let next = host.advance().unwrap();
        assert_eq!(next.tick(), 1);
        assert_eq!(next.positions(), original.positions());
        assert_eq!(next.last_sequence(), 0);
        assert!(Rc::ptr_eq(&original, &original));
    }

    #[test]
    fn host_close_rejects_late_input_and_preserves_last_snapshot() {
        let mut host = host();
        host.admit(binding_one().0, &input(0, 1, 1, Direction::East))
            .unwrap();
        let original = host.snapshot();
        host.stop();
        assert_eq!(host.pending_count(), 0);
        assert_eq!(
            host.admit(binding_one().0, &input(0, 2, 1, Direction::West)),
            Err(AdmissionError::Closed)
        );
        assert!(matches!(host.advance(), Err(AdmissionError::Closed)));
        assert!(Rc::ptr_eq(&original, &host.snapshot()));
    }

    #[test]
    fn finite_inmemory_queue_rejects_overflow_without_losing_retained_frame() {
        let mut queue = InMemoryMailbox::new();
        let first = input(0, 1, 1, Direction::North);
        for i in 0..MAX_INMEMORY_FRAMES {
            queue.push(EndpointId(i as u64 + 1), first.clone()).unwrap();
        }
        assert_eq!(queue.len(), MAX_INMEMORY_FRAMES);
        assert!(queue.push(EndpointId(9), first.clone()).is_err());
        assert_eq!(queue.len(), MAX_INMEMORY_FRAMES);
        assert!(
            queue
                .push(EndpointId(9), vec![1; MAX_WIRE_BYTES + 1])
                .is_err()
        );
        assert_eq!(queue.len(), MAX_INMEMORY_FRAMES);
        assert_eq!(queue.pop(), Some((EndpointId(1), first)));
        assert_eq!(queue.len(), MAX_INMEMORY_FRAMES - 1);
    }

    #[test]
    fn snapshot_encoding_projects_only_immutable_owner_values() {
        let mut host = host();
        let before = host.snapshot();
        let initial = host.snapshot_frame(101, 7).unwrap();
        let Message::Snapshot(initial) = decode(&initial).unwrap() else {
            panic!("snapshot kind");
        };
        assert_eq!(initial.tick, before.tick());
        assert_eq!(initial.positions, before.positions());
        host.admit(binding_one().0, &input(0, 1, 1, Direction::East))
            .unwrap();
        let after = host.advance().unwrap();
        let Message::Snapshot(later) = decode(&host.snapshot_frame(101, 7).unwrap()).unwrap()
        else {
            panic!("snapshot kind");
        };
        assert_eq!(later.tick, after.tick());
        assert_eq!(later.positions, after.positions());
        assert_eq!(before.tick(), 0);
        assert_eq!(before.positions()[0], Position { x: 2, y: 2 });
    }
}
