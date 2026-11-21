//! Real std UDP loopback evidence for the opt-in byte/socket adapter.
//! No F1 YTP1/Twin Squares protocol, simulated transport or physical LAN.
#![cfg(feature = "udp")]

use std::{
    cell::Cell,
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

use yetlet::composition::{Builder, Completion};
use yetlet_hosts::udp::{
    MAX_DATAGRAM_BYTES, MAX_PACKETS_PER_POLL, PollReport, SendOutcome, UdpEndpoint,
};

const MAX_WAIT: Duration = Duration::from_secs(2);

fn poll_until(
    receiver: &mut UdpEndpoint,
    expected: usize,
    packets: &mut Vec<(Vec<u8>, SocketAddr)>,
) -> io::Result<Vec<PollReport>> {
    let until = Instant::now() + MAX_WAIT;
    let mut reports = Vec::new();
    while packets.len() < expected {
        let report = receiver.poll_bounded(4, |payload, sender| {
            packets.push((payload.to_vec(), sender));
            Ok(())
        })?;
        reports.push(report);
        if Instant::now() >= until {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "expected real UDP loopback packets were not received",
            ));
        }
        thread::sleep(Duration::from_millis(1));
    }
    Ok(reports)
}

#[test]
fn two_owned_real_loopback_sockets_deliver_raw_bytes_with_true_peer_addresses() {
    let sender = UdpEndpoint::bind_loopback(16).unwrap();
    let mut receiver = UdpEndpoint::bind_loopback(16).unwrap();
    let peer = sender.local_addr().unwrap();
    let destination = receiver.local_addr().unwrap();
    assert_eq!(peer.ip(), Ipv4Addr::LOCALHOST);
    assert_eq!(destination.ip(), Ipv4Addr::LOCALHOST);
    assert_ne!(peer.port(), 0);
    assert_ne!(destination.port(), 0);
    assert_ne!(peer, destination);

    let raw = &[0x00, 0xff, 0x13, 0x7f]; // not UTF-8 or a bundled game schema
    assert_eq!(
        sender.send_to(destination, raw).unwrap(),
        SendOutcome::Sent(raw.len())
    );
    let mut packets = Vec::new();
    poll_until(&mut receiver, 1, &mut packets).unwrap();
    assert_eq!(packets, [(raw.to_vec(), peer)]);
    receiver.close().unwrap();
    assert!(receiver.is_closed());
    assert!(receiver.local_addr().is_err());
}

#[test]
fn finite_packet_budget_does_not_advance_a_domain_or_drain_indefinitely() {
    let sender = UdpEndpoint::bind_loopback(10).unwrap();
    let mut receiver = UdpEndpoint::bind_loopback(10).unwrap();
    let destination = receiver.local_addr().unwrap();
    for i in 0..5_u8 {
        assert_eq!(
            sender.send_to(destination, &[i]).unwrap(),
            SendOutcome::Sent(1)
        );
    }

    let mut read = Vec::new();
    let zero = receiver
        .poll_bounded(0, |bytes, _| {
            read.extend_from_slice(bytes);
            Ok(())
        })
        .unwrap();
    assert_eq!(zero.received, 0);
    assert!(zero.budget_reached);
    assert!(read.is_empty());

    let first = receiver
        .poll_bounded(2, |bytes, _| {
            read.extend_from_slice(bytes);
            Ok(())
        })
        .unwrap();
    assert_eq!(first.received, 2);
    assert_eq!(first.delivered, 2);
    assert!(first.budget_reached);
    assert_eq!(read, [0, 1]);
    let mut packets = Vec::new();
    poll_until(&mut receiver, 3, &mut packets).unwrap();
    assert_eq!(
        packets.iter().map(|(v, _)| v[0]).collect::<Vec<_>>(),
        [2, 3, 4]
    );
    let idle = receiver.poll_bounded(8, |_, _| Ok(())).unwrap();
    assert_eq!(idle.received, 0);
    assert!(idle.would_block);
    assert!(!idle.budget_reached);
}

#[test]
fn oversized_datagram_is_consumed_and_rejected_not_truncated_or_delivered() {
    let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let mut receiver = UdpEndpoint::bind_loopback(4).unwrap();
    let addr = receiver.local_addr().unwrap();
    assert_eq!(sender.send_to(&[1, 2, 3, 4, 5], addr).unwrap(), 5);
    assert_eq!(sender.send_to(&[0, 0xff, 1, 3], addr).unwrap(), 4);
    let mut delivered = Vec::new();
    let mut aggregate = PollReport::default();
    let deadline = Instant::now() + MAX_WAIT;
    while aggregate.received < 2 {
        let report = receiver
            .poll_bounded(2, |data, _| {
                delivered.push(data.to_vec());
                Ok(())
            })
            .unwrap();
        aggregate.received += report.received;
        aggregate.rejected_oversize += report.rejected_oversize;
        aggregate.delivered += report.delivered;
        assert!(Instant::now() < deadline, "missing loopback datagram");
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(aggregate.received, 2);
    assert_eq!(aggregate.rejected_oversize, 1);
    assert_eq!(aggregate.delivered, 1);
    assert_eq!(delivered, [vec![0, 0xff, 1, 3]]);
}

#[test]
fn caller_owned_peer_policy_is_not_embedded_in_transport() {
    let allowed = UdpEndpoint::bind_loopback(12).unwrap();
    let unknown = UdpEndpoint::bind_loopback(12).unwrap();
    let mut receiver = UdpEndpoint::bind_loopback(12).unwrap();
    let endpoint = receiver.local_addr().unwrap();
    allowed.send_to(endpoint, b"allowed").unwrap();
    unknown.send_to(endpoint, b"unknown").unwrap();
    let mut selected = Vec::new();
    let mut observed = 0;
    let deadline = Instant::now() + MAX_WAIT;
    while observed < 2 {
        let result = receiver
            .poll_bounded(2, |bytes, from| {
                observed += 1;
                if from == allowed.local_addr().unwrap() {
                    selected.push(bytes.to_vec());
                }
                Ok(())
            })
            .unwrap();
        assert!(result.delivered <= 2);
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(observed, 2);
    assert_eq!(selected, [b"allowed".to_vec()]);
}

#[test]
fn exact_socket_close_is_idempotent_and_rebinds_under_public_composition() {
    let already_closed = Rc::new(Cell::new(0_u32));
    let close_count = Rc::clone(&already_closed);
    let domain = Rc::new(Cell::new(0u64));
    let mut root = Builder::<io::Error>::new()
        .required_local("sole-caller-domain", domain.clone(), |_| Ok(()))
        .required_local(
            "udp",
            UdpEndpoint::bind_loopback(16).unwrap(),
            move |endpoint| {
                close_count.set(close_count.get() + 1);
                endpoint.close()
            },
        )
        .build();
    assert!(root.all_required_ready());
    let previous_addr = root.parts().last().value().local_addr().unwrap();
    assert!(!root.parts().last().value().is_closed());
    let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    sender.send_to(b"input", previous_addr).unwrap();
    let mut accepted = Vec::new();
    let deadline = Instant::now() + MAX_WAIT;
    while accepted.is_empty() {
        root.parts_mut()
            .last()
            .value_mut()
            .unwrap()
            .poll_bounded(1, |bytes, _| {
                accepted.push(bytes.to_vec());
                Ok(())
            })
            .unwrap();
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(accepted, [b"input".to_vec()]);
    assert_eq!(
        domain.get(),
        0,
        "transport must not execute application steps"
    );
    domain.set(1); // only the caller's domain owner applies the message
    assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
    assert!(root.parts().last().value().is_closed());
    assert_eq!(already_closed.get(), 1);
    assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(already_closed.get(), 1);
    assert!(root.parts().last().value().local_addr().is_err());
    drop(root);
    let rebound = UdpSocket::bind(previous_addr).expect("actual socket released");
    drop(rebound);
    assert_eq!(domain.get(), 1);
}

#[test]
fn invalid_configuration_destination_and_closed_socket_have_typed_errors() {
    assert_eq!(
        UdpEndpoint::bind_loopback(0).err().unwrap().kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(
        UdpEndpoint::bind_loopback(MAX_DATAGRAM_BYTES + 1)
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    let mut receiver = UdpEndpoint::bind_loopback(3).unwrap();
    assert_eq!(receiver.max_datagram_bytes(), 3);
    let loopback = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
    assert_eq!(
        receiver.send_to(loopback, b"a").err().unwrap().kind(),
        io::ErrorKind::InvalidInput
    );
    let unspecified = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 7));
    assert_eq!(
        receiver.send_to(unspecified, b"a").err().unwrap().kind(),
        io::ErrorKind::InvalidInput
    );
    let destination = receiver.local_addr().unwrap();
    assert_eq!(
        receiver.send_to(destination, b"over").err().unwrap().kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(
        receiver
            .poll_bounded(MAX_PACKETS_PER_POLL + 1, |_, _| Ok(()))
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    receiver.close().unwrap();
    receiver.close().unwrap();
    assert!(receiver.is_closed());
    assert_eq!(
        receiver.send_to(destination, b"ok").err().unwrap().kind(),
        io::ErrorKind::NotConnected
    );
    assert_eq!(
        receiver
            .poll_bounded(0, |_, _| Ok(()))
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::NotConnected
    );
}

#[test]
fn explicit_bind_collision_does_not_steal_the_first_socket() {
    let mut original = UdpEndpoint::bind_loopback(10).unwrap();
    let address = original.local_addr().unwrap();
    assert_eq!(
        UdpEndpoint::bind_on(address, 10).err().unwrap().kind(),
        io::ErrorKind::AddrInUse
    );
    assert_eq!(original.local_addr().unwrap(), address);
    original.close().unwrap();
    let rebound = UdpEndpoint::bind_on(address, 10).unwrap();
    assert_eq!(rebound.local_addr().unwrap(), address);
}

#[test]
fn callback_error_stops_bounded_poll_without_closing_owner() {
    let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let mut receiver = UdpEndpoint::bind_loopback(20).unwrap();
    let addr = receiver.local_addr().unwrap();
    for payload in [b"first".as_slice(), b"second".as_slice()] {
        sender.send_to(payload, addr).unwrap();
    }
    let mut called = 0;
    let deadline = Instant::now() + MAX_WAIT;
    while called == 0 {
        let result = receiver.poll_bounded(2, |_, _| {
            called += 1;
            Err(io::Error::other("application callback rejected packet"))
        });
        if called != 0 {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other);
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(called, 1);
    assert!(
        !receiver.is_closed(),
        "callback failures retain the owned socket"
    );
    let mut later = Vec::new();
    poll_until(&mut receiver, 1, &mut later).unwrap();
    assert_eq!(later[0].0, b"second");
    receiver.close().unwrap();
}
