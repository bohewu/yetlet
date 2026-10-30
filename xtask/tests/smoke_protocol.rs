//! Direct tests for migrated wire/PNG/argv boundaries; no live browser claims.
use std::io::{Cursor, Write};
use yetlet_xtask::{
    Cli, parse,
    smoke::{
        self,
        fixtures::validate_base,
        http::hidden,
        png,
        static_server::fixture_path,
        websocket::{read_frame, send_frame, verify_handshake},
    },
};
#[test]
fn websocket_handshake_accept_is_verified() {
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    let valid=b"HTTP/1.1 101 Switching Protocols\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
    verify_handshake(valid, key).unwrap();
    assert!(verify_handshake(valid, "wrong").is_err());
    assert!(verify_handshake(b"HTTP/1.1 200 OK\r\n\r\n", key).is_err());
    assert!(verify_handshake(b"HTTP/1.1 101 Switching Protocols\r\n\r\n", key).is_err());
}
#[test]
fn websocket_mask_lengths_controls_and_bounded_failures() {
    let mut bytes = Vec::new();
    send_frame(&mut bytes, 1, b"Hi", [0x37, 0xfa, 0x21, 0x3d]).unwrap();
    assert_eq!(bytes, [0x81, 0x82, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x93]);
    for length in [0, 125, 126, 65535, 65536] {
        let data = vec![b'x'; length];
        let mut frame = Vec::new();
        send_frame(&mut frame, 1, &data, [1, 2, 3, 4]).unwrap();
        assert_eq!(
            frame[1],
            if length < 126 {
                0x80 | length as u8
            } else if length < 65536 {
                0xfe
            } else {
                0xff
            }
        );
        let (fin, opcode, payload) = read_frame(&mut Cursor::new(frame)).unwrap();
        assert!(fin);
        assert_eq!(opcode, 1);
        assert_eq!(payload, data);
    }
    let (fin, opcode, payload) = read_frame(&mut Cursor::new(b"\x89\x02ok")).unwrap();
    assert!(fin);
    assert_eq!(opcode, 9);
    assert_eq!(payload, b"ok");
    assert!(read_frame(&mut Cursor::new(b"\xc1\x00")).is_err());
    assert!(read_frame(&mut Cursor::new(b"\x81\x05bad")).is_err());
    assert!(
        read_frame(&mut Cursor::new(
            b"\x81\x7f\x7f\xff\xff\xff\xff\xff\xff\xff"
        ))
        .is_err()
    );
}
fn chunk(bytes: &mut Vec<u8>, name: &[u8; 4], data: &[u8]) {
    bytes.extend((data.len() as u32).to_be_bytes());
    bytes.extend(name);
    bytes.extend(data);
    bytes.extend([0; 4]);
}
fn image(color: u8, encoded: &[u8]) -> Vec<u8> {
    let mut image = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = vec![];
    header.extend(2u32.to_be_bytes());
    header.extend(2u32.to_be_bytes());
    header.extend([8, color, 0, 0, 0]);
    chunk(&mut image, b"IHDR", &header);
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    z.write_all(encoded).unwrap();
    chunk(&mut image, b"IDAT", &z.finish().unwrap());
    chunk(&mut image, b"IEND", &[]);
    image
}
#[test]
fn known_rgb_filters_and_point_checks() {
    // Fixed encoded rows all decode to [10,20,30,40,50,60],
    // [20,40,60,80,100,120]. Includes Paeth tie/left/above choices.
    let rows: [&[u8]; 5] = [
        &[0, 10, 20, 30, 40, 50, 60, 0, 20, 40, 60, 80, 100, 120],
        &[1, 10, 20, 30, 30, 30, 30, 1, 20, 40, 60, 60, 60, 60],
        &[2, 10, 20, 30, 40, 50, 60, 2, 10, 20, 30, 40, 50, 60],
        &[3, 10, 20, 30, 35, 40, 45, 3, 15, 30, 45, 50, 55, 60],
        &[4, 10, 20, 30, 30, 30, 30, 4, 10, 20, 30, 40, 50, 60],
    ];
    for row in rows {
        let p = png::decode(&image(2, row)).unwrap();
        assert_eq!((p.width, p.height, p.channels), (2, 2, 3));
        assert_eq!(p.rgb(0, 0).unwrap(), [10, 20, 30]);
        assert_eq!(p.rgb(1, 1).unwrap(), [80, 100, 120]);
        assert!(p.rgb(2, 0).is_err());
    }
    let rgba = image(
        6,
        &[
            0, 255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 0, 255, 255, 1, 2, 3, 4,
        ],
    );
    let p = png::decode(&rgba).unwrap();
    assert_eq!(p.channels, 4);
    assert_eq!(p.rgb(0, 1).unwrap(), [0, 0, 255]);
    assert!(png::decode(b"invalid").is_err());
    assert!(png::decode(&image(2, &[0, 1])).is_err());
    assert!(
        png::decode(&image(
            2,
            &[5, 10, 20, 30, 40, 50, 60, 0, 20, 40, 60, 80, 100, 120]
        ))
        .is_err()
    );
}
#[test]
fn command_options_and_unported_suites_fail_closed() {
    for suite in smoke::BROWSER_SUITES {
        assert!(matches!(
            parse(["smoke", suite].into_iter().map(Into::into)).unwrap(),
            Cli::Smoke { .. }
        ));
        if *suite != "form-css-proof-chrome"
            && !suite.starts_with("board-")
            && !suite.starts_with("scene-")
        {
            assert!(
                parse(
                    ["smoke", suite, "--profile", "release"]
                        .into_iter()
                        .map(Into::into)
                )
                .is_ok()
            );
        }
    }
    for args in [
        vec!["smoke", "browser-chrome", "--profile", "production"],
        vec!["smoke", "board-chrome", "--profile", "debug"],
        vec!["smoke", "scene-candidate-firefox", "--profile", "release"],
        vec!["smoke", "scene-unknown"],
        vec!["smoke", "hello-world", "--profile", "release"],
        vec!["fixture", "redis-cookie-browser", "https://other.example"],
    ] {
        assert!(parse(args.into_iter().map(Into::into)).is_err());
    }
    assert_eq!(
        parse(
            ["smoke", "browser-chrome", "--help"]
                .into_iter()
                .map(Into::into)
        )
        .unwrap(),
        Cli::Help
    );
    assert!(validate_base("http://127.0.0.1:8080/").is_ok());
    // Synthetic userinfo disguising a foreign host as loopback; no real credentials.
    let disguised_loopback = ["http://", "127.0.0.1", ":8080@other/"].concat();
    for url in [
        "https://127.0.0.1:8080",
        disguised_loopback.as_str(),
        "http://127.0.0.1:0/",
        "http://127.0.0.1:8080/?secret=x",
    ] {
        assert!(validate_base(url).is_err());
    }
}
#[test]
fn hidden_fields_and_static_fixture_paths_preserve_escaping_and_bounds() {
    assert_eq!(
        hidden(
            r#"<input name="csrf" value="x&amp;&quot;&#39;&lt;&gt;">"#,
            "csrf"
        )
        .unwrap(),
        "x&\"'<>"
    );
    assert!(hidden("<form>", "csrf").is_err());
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("index.html"), "fixture").unwrap();
    assert!(
        fixture_path(root.path(), "/")
            .unwrap()
            .ends_with("index.html")
    );
    for url in ["/../Cargo.toml", "/%2e%2e%2fCargo.toml", "/missing.js"] {
        assert!(fixture_path(root.path(), url).is_err());
    }
}
#[test]
fn scene_profiles_scenarios_sources_and_geometry_are_explicit() {
    use yetlet_xtask::smoke::scene::{SCENARIOS, firefox_options, validate_candidate};
    assert_eq!(SCENARIOS.len(), 6);
    let names: std::collections::BTreeSet<_> = SCENARIOS.iter().map(|s| s.1).collect();
    assert_eq!(names.len(), 6);
    assert_eq!(
        firefox_options(true, false),
        serde_json::json!({"args":[],"prefs":{}})
    );
    assert_eq!(
        firefox_options(false, true),
        serde_json::json!({"args":["-headless"],"prefs":{"dom.webgpu.enabled":true}})
    );
    let root = tempfile::tempdir().unwrap();
    let name = "crates/yetlet-browser/src/wgpu/browser.rs";
    std::fs::create_dir_all(root.path().join("crates/yetlet-browser/src/wgpu")).unwrap();
    std::fs::write(root.path().join(name), b"current").unwrap();
    let manifest =
        serde_json::json!({"candidate_sources":{name:yetlet_xtask::tooling::sha(b"current")}});
    validate_candidate(root.path(), &manifest).unwrap();
    std::fs::write(root.path().join(name), b"changed").unwrap();
    assert!(validate_candidate(root.path(), &manifest).is_err());
    assert!(validate_candidate(root.path(), &serde_json::json!({})).is_err());
    let build = yetlet_xtask::build::scene_build_sources(&yetlet_xtask::repository_root()).unwrap();
    assert!(build.get("xtask/src/build.rs").is_some());
    assert!(
        build
            .get("examples/dashboard-counter-island/src/lib.rs")
            .is_some()
    );
    for suite in [
        "board-chrome",
        "board-firefox",
        "scene-chrome",
        "scene-firefox",
        "scene-lifecycle-firefox",
        "scene-picking-firefox",
        "scene-candidate-firefox",
        "scene-support-firefox",
    ] {
        assert!(parse(["smoke", suite].into_iter().map(Into::into)).is_ok());
        for flag in ["--profile", "--gpu", "--skip", "--unknown"] {
            assert!(parse(["smoke", suite, flag].into_iter().map(Into::into)).is_err());
        }
    }
}
#[test]
fn scene_grid_pixel_mapping_uses_fixed_tolerance_and_only_full_white_seam() {
    use yetlet_xtask::smoke::scene::pixel_proof;
    let width = 600;
    let height = 400;
    let mut p = png::Pixels {
        width,
        height,
        channels: 3,
        rows: vec![vec![0; width * 3]; height],
    };
    for x in 0..width {
        p.rows[0][x * 3..x * 3 + 3].copy_from_slice(&[255, 255, 255]);
    }
    for i in 0..120 {
        let x = (i % 12) * 50 + 25;
        let y = (i / 12) * 40 + 21;
        let color = [[230, 0, 0], [0, 230, 0], [0, 0, 230]][i % 3];
        p.rows[y][x * 3..x * 3 + 3].copy_from_slice(&color);
    }
    let proof = pixel_proof(&p, None, false).unwrap();
    assert_eq!(proof["verified"], 120);
    assert_eq!(proof["capture_canvas_origin"], serde_json::json!([0, 1]));
    p.rows[21][25 * 3] = 229;
    assert_eq!(pixel_proof(&p, None, false).unwrap()["verified"], 119);
    assert!(pixel_proof(&p, Some(&[3; 120]), false).is_err());
    assert!(pixel_proof(&p, Some(&[0; 119]), false).is_err());
    p.rows[0][3] = 0;
    assert_eq!(
        pixel_proof(&p, None, false).unwrap()["capture_canvas_origin"],
        serde_json::json!([0, 0])
    );
}
#[test]
fn board_chunk_bounds_percentile_and_stopped_database_seed_preserve_history() {
    use yetlet_xtask::smoke::board::{percentile, read_chunk, seed};
    let mut wire = Cursor::new(b"3\r\nabc\r\n0\r\n\r\n");
    assert_eq!(read_chunk(&mut wire).unwrap(), b"abc");
    assert!(read_chunk(&mut wire).unwrap().is_empty());
    for bytes in [b"x\r\n".as_slice(), b"10001\r\n", b"3\r\na", b"1\r\naXX"] {
        assert!(read_chunk(&mut Cursor::new(bytes)).is_err());
    }
    assert_eq!(percentile(&[4., 1., 3., 2.], 0.5).unwrap(), 2.);
    assert_eq!(percentile(&[4., 1., 3., 2.], 0.95).unwrap(), 4.);
    assert!(percentile(&[], 0.5).is_err());
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("board.sqlite");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch("CREATE TABLE work_items(id INTEGER PRIMARY KEY,project INTEGER,title TEXT,description TEXT,status TEXT,priority TEXT,assignee TEXT,version INTEGER); CREATE TABLE history(item_id INTEGER,project INTEGER,version INTEGER,actor TEXT,action TEXT,title TEXT,status TEXT); INSERT INTO work_items VALUES(1,1,'Live update','retained','active','high','alice',4); INSERT INTO history VALUES(1,1,4,'alice','update','Live update','active');").unwrap();
    drop(db);
    assert_eq!(
        seed(&path).unwrap(),
        serde_json::json!({"items":1000,"history":9997})
    );
    assert_eq!(
        seed(&path).unwrap(),
        serde_json::json!({"items":1000,"history":9997})
    );
    let db = rusqlite::Connection::open(&path).unwrap();
    let title: String = db
        .query_row("SELECT title FROM work_items WHERE id=1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(title, "Live update");
    drop(db);
    temp.close().unwrap();
}
#[test]
fn recursive_source_reports_are_per_file_and_detect_changes() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("src/nested")).unwrap();
    std::fs::write(temp.path().join("src/nested/file.rs"), b"source").unwrap();
    let hashes = smoke::source_hashes(temp.path(), &temp.path().join("src")).unwrap();
    assert_eq!(
        hashes["src/nested/file.rs"],
        yetlet_xtask::tooling::sha(b"source")
    );
    assert!(smoke::unchanged(temp.path(), &hashes));
    std::fs::write(temp.path().join("src/nested/file.rs"), b"changed").unwrap();
    assert!(!smoke::unchanged(temp.path(), &hashes));
}
#[test]
fn scene_live_owner_comparison_keeps_lifetime_assertions_and_deadlines_fail_before_io() {
    use yetlet_xtask::smoke::{browser::Firefox, scene::same_current_owner};
    let before = serde_json::json!({"generation":2,"starts":2,"state":"ready","ownedBufferBytes":3840,"pendingInitializations":1,"lateSettled":0,"lastRetired":{"generation":1,"resourcesReleased":false}});
    let mut after = before.clone();
    after["pendingInitializations"] = 0.into();
    after["lateSettled"] = 1.into();
    after["lastRetired"]["resourcesReleased"] = true.into();
    assert!(same_current_owner(&before, &after));
    after["generation"] = 3.into();
    assert!(!same_current_owner(&before, &after));
    let driver = Firefox {
        base: "http://127.0.0.1:1".into(),
        session: None,
        capabilities: serde_json::Value::Null,
        deadline: Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
    };
    assert_eq!(driver.call("/status", "GET", None).unwrap_err().code, 124);
}
