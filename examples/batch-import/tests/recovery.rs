use std::process::Command;

#[test]
fn resumed_workers_recover_the_five_weigh_ins() {
    let result = Command::new(env!("CARGO_BIN_EXE_yetlet-batch-import"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains(
        "last_row: 5, rows: [(1, 11, 240), (2, 11, 265), (3, 12, 301), (4, 13, 228), (5, 13, 251)]"
    ));
    for boundary in ["before-commit", "after-commit"] {
        assert!(output.contains(&format!(
            "Resumed {boundary}: all rows and checkpoint match."
        )));
    }
}
