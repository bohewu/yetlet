use std::{fs, path::Path};
use yetlet_xtask::docs;
fn put(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn fixture(root: &Path) {
    for name in docs::REQUIRED {
        put(root, name, "English documentation.\n");
    }
    put(
        root,
        "docs/BACKLOG.md",
        "| YT-001 | P1 | S | M1 | Task | None | Proof | Todo |\n",
    );
}
fn errors(root: &Path) -> Vec<String> {
    docs::check(root).unwrap()
}
#[test]
fn required_files_english_links_eof_whitespace_fences() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    assert!(errors(root).is_empty());
    for (text, needle) in [
        ("No newline", "EOF newline"),
        ("Trailing \n", "trailing whitespace"),
        ("```rust\n", "unbalanced code fences"),
        ("[missing](missing.md)\n", "broken link"),
        ("\u{4e2d}\u{6587}\n", "non-English CJK"),
    ] {
        put(root, "README.md", text);
        assert!(errors(root).iter().any(|s| s.contains(needle)), "{needle}");
    }
    fixture(root);
    fs::remove_file(root.join("docs/MVP.md")).unwrap();
    assert!(errors(root).iter().any(|s| s == "missing: docs/MVP.md"));
    fixture(root);
    put(root, "new.md", "\u{4e2d}\n");
    assert!(errors(root).iter().any(|s| s.contains("new.md")));
    fs::remove_file(root.join("new.md")).unwrap();
    for skip in [".git", ".codex", "target", "dist", "node_modules", ".cache"] {
        put(root, &format!("{skip}/skip.md"), "\u{4e2d}\n");
    }
    assert!(errors(root).is_empty());
}
#[test]
fn duplicate_unknown_self_cycle_malformed_and_empty_backlogs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    for (rows, needle) in [
        (
            "| YT-001 | P1 | S | M1 | Task | None | Proof | Todo |\n| YT-001 | P1 | S | M1 | Task | None | Proof | Todo |\n",
            "duplicate backlog ID",
        ),
        (
            "| YT-001 | P1 | S | M1 | Task | YT-999 | Proof | Todo |\n",
            "unknown backlog dependency",
        ),
        (
            "| YT-001 | P1 | S | M1 | Task | YT-001 | Proof | Todo |\n",
            "self dependency",
        ),
        (
            "| YT-001 | P1 | S | M1 | Task | YT-002 | Proof | Todo |\n| YT-002 | P1 | S | M1 | Task | YT-001 | Proof | Todo |\n",
            "dependency cycle",
        ),
        ("| YT-001 | short |\n", "malformed backlog row"),
        ("No rows\n", "no backlog work items"),
    ] {
        put(root, "docs/BACKLOG.md", rows);
        assert!(errors(root).iter().any(|s| s.contains(needle)), "{needle}");
    }
}
