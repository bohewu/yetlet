use serde_json::{Value, json};
use yetlet_xtask::{
    measure::{self, Options},
    qualify::board,
    release::private,
};
#[test]
fn frozen_nearest_rank_median_and_raw_counts() {
    let values = [4., 1., 3., 2.];
    let s = measure::distribution(&values).unwrap();
    assert_eq!(s["samples"], 4);
    assert_eq!(s["median_ms"], 2.5);
    assert_eq!(s["p95_ms"], 4.);
    assert_eq!(s["raw_ms"], json!(values));
    assert!(measure::distribution(&[]).is_err());
    assert!(measure::distribution(&[f64::NAN]).is_err());
    let samples: Vec<_> = (1..=30)
        .map(|n| json!({"prepareMs":n,"completionWaitMs":null}))
        .collect();
    let d = measure::dynamic::distributions(&samples).unwrap();
    assert_eq!(
        d["prepareMs"],
        json!({"p50":15.,"p95":29.,"p99":30.,"n":30})
    );
    assert!(d.get("completionWaitMs").is_none());
}
#[test]
fn exact_board_profiles_and_strict_gate_boundaries() {
    for (name, w, r, s, rate, stagger, trace) in [
        ("bounded", 30, vec![60, 60, 60], 300, 2., false, false),
        ("full", 120, vec![600, 600, 600], 1800, 0.5, true, false),
        (
            "full-sync",
            120,
            vec![600, 600, 600],
            1800,
            0.5,
            false,
            true,
        ),
        ("sync-12m", 120, vec![600], 0, 0.5, false, true),
        ("staggered-12m", 120, vec![600], 0, 0.5, true, true),
    ] {
        let p = board::profile(name).unwrap();
        assert_eq!(
            (p.warmup, p.rounds, p.soak, p.rps, p.stagger, p.trace),
            (w, r, s, rate, stagger, trace)
        );
    }
    let rows = vec![json!(["read", 249.9, 200]), json!(["write", 499.9, 303])];
    assert_eq!(
        board::summarize(&rows, 1, 1.).unwrap()["thresholds_pass"],
        true
    );
    for rows in [
        vec![json!(["read", 250., 200]), json!(["write", 1., 303])],
        vec![json!(["read", 1., 200]), json!(["write", 500., 303])],
        vec![json!(["read", 1., 200]), json!(["write", 1., 503])],
    ] {
        assert_eq!(
            board::summarize(&rows, 1, 1.).unwrap()["thresholds_pass"],
            false
        );
    }
}
#[test]
fn metrics_production_test_boundary_and_dependency_closure_inputs() {
    let t = tempfile::tempdir().unwrap();
    std::fs::create_dir(t.path().join("tests")).unwrap();
    std::fs::write(
        t.path().join("app.rs"),
        "pub async fn a() {}\n\n#[cfg(test)]\nasync fn b() {}\n",
    )
    .unwrap();
    std::fs::write(t.path().join("tests/test.rs"), "async fn c() {}\n").unwrap();
    let m = measure::dx::source_metrics(t.path()).unwrap();
    assert_eq!(
        m,
        json!({"rust_files":2,"production_nonblank_lines":1,"test_nonblank_lines":3,"production_async_functions":1})
    );
    let t:toml::Value="[dependencies]\na='1'\n[dev-dependencies]\nb='1'\n[target.'cfg(unix)'.build-dependencies]\nc='1'".parse().unwrap();
    assert_eq!(
        measure::dx::dependencies(&t)
            .into_iter()
            .collect::<Vec<_>>(),
        ["a", "c"]
    );
}
fn report() -> Value {
    json!({"config":{"rows":3,"updates_per_round":1,"rounds":1},"strategies":(["subtree_replacement","view_tree_patch","keyed_direct_dom","static_template_slots"].into_iter().map(|n|json!({"strategy":n,"final_state_equivalent_all_rounds":true,"outside_focus_preserved_all_rounds":true,"setup":{"samples":1},"update":{"samples":1},"structural":{"samples":3},"final_state_checksums":[17],"inside_input_identity_preserved_all_rounds":n!="subtree_replacement","inside_focus_preserved_all_rounds":n!="subtree_replacement","untouched_row_identity_preserved_all_rounds":!["subtree_replacement","view_tree_patch"].contains(&n)})).collect::<Vec<_>>())})
}
#[test]
fn render_all_suite_specific_identity_count_and_equivalence_assertions() {
    let r = report();
    measure::render::validate(&r, 3, 1, 1).unwrap();
    for field in [
        "final_state_equivalent_all_rounds",
        "outside_focus_preserved_all_rounds",
        "inside_input_identity_preserved_all_rounds",
        "inside_focus_preserved_all_rounds",
        "untouched_row_identity_preserved_all_rounds",
    ] {
        let mut r = report();
        r["strategies"][2][field] = json!(false);
        assert!(measure::render::validate(&r, 3, 1, 1).is_err());
    }
    let mut r = report();
    r["strategies"][0]["update"]["samples"] = json!(0);
    assert!(measure::render::validate(&r, 3, 1, 1).is_err());
    let mut r = report();
    r["strategies"][0]["final_state_checksums"] = json!([18]);
    assert!(measure::render::validate(&r, 3, 1, 1).is_err());
}
#[test]
fn options_bounds_and_unknown_flags() {
    for (suite, argv) in [
        ("baseline", vec!["--samples", "4"]),
        ("baseline", vec!["--browser-runs", "31"]),
        ("render-strategies", vec!["--rows", "2"]),
        ("render-strategies", vec!["--rounds", "31"]),
        ("sample-dx", vec!["--profile", "release"]),
        ("scene-dynamic", vec!["--skip-gpu", "true"]),
    ] {
        assert!(
            Options::parse(suite, &argv.into_iter().map(Into::into).collect::<Vec<_>>()).is_err()
        );
    }
}
#[test]
fn private_exact_package_alias_allowlist_and_paths() {
    assert_eq!(private::NAMES.len(), 9);
    let a = private::aliases();
    assert_eq!(a.len(), 18);
    assert!(
        a.values()
            .all(|s| s == "../../LICENSE-MIT" || s == "../../LICENSE-APACHE")
    );
    for p in ["../x", "a/../x", "/x", "a\\x", "a\0x", "a//x", "a/./x"] {
        assert!(!private::archive_path(p));
    }
    assert!(private::archive_path("crates/yetlet-core/src/lib.rs"));
}

#[test]
fn signed_and_arbitrary_width_seed_remains_a_decimal_js_argument() {
    for (raw, literal) in [
        ("-1", "-1"),
        ("4294967296", "4294967296"),
        ("9007199254740993", "9007199254740993"),
        ("+0007", "7"),
        ("-１２_٣", "-123"),
        ("-0", "0"),
        ("  -1_000  ", "-1000"),
    ] {
        let o = Options::parse("render-strategies", &["--seed".into(), raw.into()]).unwrap();
        assert_eq!(o.seed_literal().unwrap(), literal);
    }
    let wide = "9".repeat(400);
    let o = Options::parse("render-strategies", &["--seed".into(), wide.clone().into()]).unwrap();
    assert_eq!(o.seed_literal().unwrap(), wide);
    for bad in [
        "",
        "--1",
        "1.5",
        "1e2",
        "0x10",
        "1;alert(1)",
        "_1",
        "1__2",
        "1_",
    ] {
        assert!(Options::parse("render-strategies", &["--seed".into(), bad.into()]).is_err());
    }
}

#[test]
fn dynamic_hard_failures_never_become_deadline_partial() {
    use measure::dynamic::measurement_status;
    use yetlet_xtask::tooling::{Error, fail};
    for message in [
        "Actual non-fallback GPU required",
        "fixed pixel tolerance",
        "Unequal renderer final state",
    ] {
        assert_eq!(
            measurement_status(&Err(fail(message)), true, true),
            "failed"
        );
    }
    assert_eq!(
        measurement_status(
            &Err(Error {
                code: 124,
                message: "deadline".into()
            }),
            true,
            true
        ),
        "bounded_partial"
    );
    assert_eq!(measurement_status(&Ok(()), true, true), "bounded_partial");
    assert_eq!(
        measurement_status(&Ok(()), false, true),
        "passed_yt077_measurement"
    );
    assert_eq!(measurement_status(&Ok(()), true, false), "failed");
}

#[test]
fn board_and_scene_recheck_selected_application_and_lock_mutations() {
    use std::{fs, process::Command};
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .arg(root)
            .status()
            .unwrap()
            .success()
    );
    let files = [
        "xtask/src/lib.rs",
        "xtask/Cargo.toml",
        "xtask/Cargo.lock",
        ".cargo/config.toml",
        "rust-toolchain.toml",
        "Cargo.lock",
        "Cargo.toml",
        "crates/fixture/src/lib.rs",
        "scripts/smoke_browser.py",
        "examples/lab-operations-board/src/lib.rs",
        "examples/lab-operations-board/style.css",
        "examples/webgpu-scene-island/src/lib.rs",
        "examples/webgpu-scene-island/Cargo.toml",
        "examples/webgpu-scene-island/Cargo.lock",
        "examples/webgpu-scene-island/comparison.js",
        "examples/webgpu-scene-island/verify_dynamic.py",
        "examples/webgpu-scene-island/target/debug/yetlet-webgpu-scene-island",
        "target/webgpu-scene-island-dist/manifest.json",
        "target/webgpu-scene-island-dist/assets/app.wasm",
    ];
    for p in files {
        let p = root.join(p);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "original").unwrap();
    }
    let before_board = board::input_hashes(root).unwrap();
    let before_scene = measure::dynamic::input_hashes(root).unwrap();
    for p in [
        "Cargo.lock",
        "examples/lab-operations-board/src/lib.rs",
        "crates/fixture/src/lib.rs",
        "examples/lab-operations-board/style.css",
    ] {
        assert!(before_board.get(p).is_some(), "{p}");
        fs::write(root.join(p), "changed").unwrap();
        assert_ne!(before_board, board::input_hashes(root).unwrap());
        fs::write(root.join(p), "original").unwrap();
    }
    for p in [
        "Cargo.lock",
        "examples/webgpu-scene-island/src/lib.rs",
        "examples/webgpu-scene-island/Cargo.lock",
        "examples/webgpu-scene-island/comparison.js",
    ] {
        assert!(before_scene.get(p).is_some(), "{p}");
        fs::write(root.join(p), "changed").unwrap();
        assert_ne!(before_scene, measure::dynamic::input_hashes(root).unwrap());
        fs::write(root.join(p), "original").unwrap();
    }
    assert_eq!(before_board, board::input_hashes(root).unwrap());
    assert_eq!(before_scene, measure::dynamic::input_hashes(root).unwrap());
}
