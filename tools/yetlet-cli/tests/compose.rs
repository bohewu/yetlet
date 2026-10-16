#![cfg(any(target_os = "linux", windows))]
use std::{fs, process::Command};
use yetlet_cli::scaffold;
#[path = "fixtures/docker_framework.rs"]
mod docker_fixture;
#[path = "fixtures/compose_app.rs"]
mod fixture;

#[test]
fn binary_compose_is_configuration_only_preserves_inputs_and_uses_usage_exit_two() {
    let f = fixture::Fixture::new();
    let secret = concat!("SYNTHETIC_COMPOSE_CLI_", "SECRET");
    fs::write(f.app.join(".env"), secret).unwrap();
    let before =
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(f.app.join(p)).unwrap());
    let output = f.parent.join("compose space 雪");
    let run = |image: &str| {
        Command::new(env!("CARGO_BIN_EXE_yetlet"))
            .args(["generate", "compose", "--manifest-path"])
            .arg(f.manifest())
            .arg("--output")
            .arg(&output)
            .arg("--image")
            .arg(image)
            .env("PATH", f.root.path().join("no-tools"))
            .env("CARGO_HOME", f.root.path().join("no-cargo-home"))
            .env("DOCKER_HOST", "invalid://must-not-connect")
            .env("KUBECONFIG", f.root.path().join("no-kube-config"))
            .env("SYNTHETIC_SECRET", secret)
            .output()
            .unwrap()
    };
    let result = run("localhost:5000/team/app:v1");
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(run("localhost:5000/team/app:v1").status.code(), Some(1));
    for invalid in [
        "",
        "ghcr.io/app:latest",
        "app:v1",
        "ghcr.io/app:v1\nprivileged: true",
        "ghcr.io/app:$(id)",
    ] {
        assert_eq!(run(invalid).status.code(), Some(2));
    }
    let mut files = fs::read_dir(&output)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, ["README.md", "compose.yaml"]);
    for file in files {
        let text = fs::read_to_string(output.join(file)).unwrap();
        for forbidden in [
            secret,
            f.root.path().to_str().unwrap(),
            f.framework.to_str().unwrap(),
        ] {
            assert!(!text.contains(forbidden));
        }
    }
    assert_eq!(
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(f.app.join(p)).unwrap()),
        before
    );
    assert!(!f.root.path().join("no-cargo-home").exists());
    assert!(!f.root.path().join("no-kube-config").exists());
    assert_eq!(
        fs::read_dir(&f.parent).unwrap().count(),
        5,
        "no staging residue"
    );
    assert_eq!(fs::read_dir(f.root.path()).unwrap().count(), 3);
}
