use super::*;

const WIRE: &str = r#"{"protocol_version":1,"handler_id":"counter.add","handler_version":1,"capture":{"count":40,"step":2}}"#;

#[test]
fn stable_wire_identity_and_roundtrip_dispatch() {
    for (handler, expected) in [(Handler::Add, 42), (Handler::Subtract, 38)] {
        let original = Descriptor::new(handler, Capture::new(40, 2).unwrap());
        let bytes = original.encode().unwrap();
        let restored = Descriptor::decode(&bytes).unwrap();
        assert_eq!(restored, original);
        assert_eq!(restored.dispatch(), Ok(expected));
        assert_eq!(restored.dispatch(), original.dispatch());
        if handler == Handler::Add {
            assert_eq!(bytes, WIRE.as_bytes());
        }
        // The next event uses newly selected application state, not a resumed stack.
        let next = Descriptor::new(handler, Capture::new(expected, 2).unwrap());
        assert_eq!(
            next.dispatch(),
            Ok(if handler == Handler::Add { 44 } else { 36 })
        );
    }
}

#[test]
fn unknown_ids_and_incompatible_versions_fail_closed() {
    for (old, new, error) in [
        ("counter.add", "counter.evil", Error::UnknownHandler),
        ("counter.add", "counter.add.v2", Error::UnknownHandler),
        (
            "\"protocol_version\":1",
            "\"protocol_version\":2",
            Error::ProtocolVersion,
        ),
        (
            "\"protocol_version\":1",
            "\"protocol_version\":0",
            Error::ProtocolVersion,
        ),
        (
            "\"handler_version\":1",
            "\"handler_version\":2",
            Error::HandlerVersion,
        ),
        (
            "\"handler_version\":1",
            "\"handler_version\":0",
            Error::HandlerVersion,
        ),
    ] {
        assert_eq!(
            Descriptor::decode(WIRE.replace(old, new).as_bytes()),
            Err(error)
        );
    }
    // Even a descriptor obtained by bypassing decode with serde cannot dispatch.
    let invalid: Descriptor =
        serde_json::from_str(&WIRE.replace("counter.add", "counter.evil")).unwrap();
    assert_eq!(invalid.dispatch(), Err(Error::UnknownHandler));
    assert_eq!(invalid.encode(), Err(Error::UnknownHandler));
}

#[test]
fn numeric_capture_and_result_bounds_are_enforced() {
    for (old, new) in [
        ("40", "-1"),
        ("40", "1000001"),
        ("\"step\":2", "\"step\":0"),
        ("\"step\":2", "\"step\":1001"),
    ] {
        assert_eq!(
            Descriptor::decode(WIRE.replace(old, new).as_bytes()),
            Err(Error::InvalidCapture)
        );
    }
    for (handler, count, step, expected) in [
        (Handler::Add, 999_000, 1000, Ok(1_000_000)),
        (Handler::Subtract, 1000, 1000, Ok(0)),
        (Handler::Add, 1_000_000, 1, Err(Error::InvalidResult)),
        (Handler::Subtract, 0, 1, Err(Error::InvalidResult)),
    ] {
        let descriptor = Descriptor::new(handler, Capture::new(count, step).unwrap());
        assert_eq!(
            Descriptor::decode(&descriptor.encode().unwrap())
                .unwrap()
                .dispatch(),
            expected
        );
    }
}

#[test]
fn invalid_shapes_extra_fields_and_code_are_rejected() {
    for bytes in [
        WIRE.replace("\"count\":40", "\"count\":\"40\""),
        WIRE.replace("\"count\":40", "\"count\":40.5"),
        WIRE.replace("\"count\":40", "\"count\":2147483648"),
        WIRE.replace("\"step\":2", "\"step\":-1"),
        WIRE.replace("\"step\":2", "\"step\":65536"),
        WIRE.replace("\"step\":2", "\"step\":null"),
        WIRE.replace("\"count\":40,", ""),
        WIRE.replace("\"count\":40", "\"count\":40,\"count\":41"),
        WIRE.replace("\"count\":40", "\"count\":40,\"token\":\"synthetic\""),
        WIRE.replace("\"count\":40", "\"count\":40,\"server_handle\":1"),
        WIRE.replace("\"count\":40", "\"count\":40,\"code\":\"eval()\""),
        WIRE.replace(
            "\"protocol_version\":1",
            "\"protocol_version\":1,\"closure\":{}",
        ),
        WIRE.replace(
            "\"protocol_version\":1",
            "\"protocol_version\":1,\"protocol_version\":1",
        ),
        WIRE.replace("{\"count\":40,\"step\":2}", "[40,2]"),
        WIRE.replace(
            "{\"count\":40,\"step\":2}",
            "{\"nested\":{\"count\":40,\"step\":2}}",
        ),
        WIRE.replace("\"handler_version\":1", "\"handler_version\":\"1\""),
        format!("{WIRE} {{}}"),
        r#"[1,"counter.add",1,{"count":40,"step":2}]"#.to_owned(),
        "{}".to_owned(),
        "[]".to_owned(),
    ] {
        assert_eq!(
            Descriptor::decode(bytes.as_bytes()),
            Err(Error::Malformed),
            "{bytes}"
        );
    }
    assert_eq!(Descriptor::decode(&[0xff]), Err(Error::Malformed));
}

#[test]
fn size_bound_precedes_parsing_and_bounds_handler_strings() {
    let exactly = format!("{WIRE}{}", " ".repeat(MAX_ENCODED_BYTES - WIRE.len()));
    assert!(Descriptor::decode(exactly.as_bytes()).is_ok());
    assert_eq!(
        Descriptor::decode(format!("{exactly} ").as_bytes()),
        Err(Error::TooLarge)
    );
    let long_id = WIRE.replace("counter.add", &"x".repeat(MAX_ENCODED_BYTES));
    assert_eq!(Descriptor::decode(long_id.as_bytes()), Err(Error::TooLarge));
    let bounded_unknown = WIRE.replace("counter.add", &"x".repeat(80));
    assert_eq!(
        Descriptor::decode(bounded_unknown.as_bytes()),
        Err(Error::UnknownHandler)
    );
}

#[test]
fn snapshot_is_untrusted_data_not_authorization_or_integrity() {
    // Valid numeric tampering is allowed: no authentication/integrity is implied.
    let tampered = Descriptor::decode(WIRE.replace("40", "100").as_bytes()).unwrap();
    assert_eq!(tampered.dispatch(), Ok(102));
    // Unknown secret/capability fields are rejected rather than silently emitted.
    // Numeric client-safe state still requires deliberate application selection.
    let capture = Capture::new(40, 2).unwrap();
    assert_eq!(
        serde_json::to_string(&capture).unwrap(),
        r#"{"count":40,"step":2}"#
    );
}
