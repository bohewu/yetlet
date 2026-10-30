use super::*;
use crate::{Cli, parse};
use std::ffi::OsString;

const IMAGE: &str = "localhost:5000/team/app:v1";

#[test]
fn k8s_parser_keeps_literal_targets_and_moves_replicas_to_settings() {
    let args = |parts: &[&str]| parts.iter().map(OsString::from).collect::<Vec<_>>();
    let base = [
        "generate",
        "k8s",
        "--manifest-path",
        "應 用/Cargo.toml",
        "--output",
        "設定 ;$(literal)",
        "--image",
        IMAGE,
    ];
    assert_eq!(
        parse(&args(&base)),
        Ok(Cli::GenerateK8s {
            manifest: "應 用/Cargo.toml".into(),
            output: "設定 ;$(literal)".into(),
            image: IMAGE.into(),
        })
    );
    for tail in [
        vec!["--replicas", "1"],
        vec!["--replicas=2"],
        vec!["--output", "x"],
        vec!["--image", IMAGE],
        vec!["--apply"],
        vec!["--help"],
        vec!["--"],
        vec!["extra"],
    ] {
        let mut input = base.to_vec();
        input.extend(tail);
        assert!(parse(&args(&input)).is_err(), "{input:?}");
    }
    assert!(
        matches!(parse(&args(&["generate", "k8s", "--output=out", "--image=localhost:5000/team/app:v1"])), Ok(Cli::GenerateK8s {manifest, ..}) if manifest == std::path::Path::new("Cargo.toml"))
    );
    for bad in [
        "app:v1",
        "ghcr.io/app:latest",
        "ghcr.io/app:v1\nkind: Secret",
        "ghcr.io/app:'",
        "ghcr.io/app:${SECRET}",
    ] {
        let mut input = base;
        input[7] = bad;
        assert!(parse(&args(&input)).is_err());
    }
    assert!(Cli::Help(crate::HELP).generate_k8s().is_err());
}

#[test]
fn names_have_identity_mapping_without_underscore_or_truncation_collisions() {
    for name in ["a", "native-ssr", "a1", "a--b", &"a".repeat(63)] {
        assert_eq!(validate_name(name), Ok(()));
        assert_eq!(deployment(name, IMAGE, 1)["metadata"]["name"], name);
        assert_eq!(service(name)["metadata"]["name"], name);
    }
    for name in [
        "",
        "1app",
        "demo_app",
        "UPPER",
        "app-",
        "-app",
        "a.b",
        "a/b",
        "a\nkind: Secret",
        "應用",
        &"a".repeat(64),
    ] {
        assert!(validate_name(name).is_err(), "{name:?}");
    }
}

#[test]
fn manifest_semantics_select_ports_probes_limits_and_linux_security() {
    let d: Value =
        serde_json::from_str(&document(deployment("cloudapp", IMAGE, 2)).unwrap()).unwrap();
    let s: Value = serde_json::from_str(&document(service("cloudapp")).unwrap()).unwrap();
    assert_eq!(d["apiVersion"], "apps/v1");
    assert_eq!(d["kind"], "Deployment");
    assert_eq!(s["apiVersion"], "v1");
    assert_eq!(s["kind"], "Service");
    assert_eq!(d["spec"]["replicas"], 2);
    let labels = json!({"app.kubernetes.io/name": "cloudapp"});
    assert_eq!(d["metadata"]["labels"], labels);
    assert_eq!(s["metadata"]["labels"], labels);
    assert_eq!(d["spec"]["selector"]["matchLabels"], labels);
    assert_eq!(d["spec"]["template"]["metadata"]["labels"], labels);
    assert_eq!(s["spec"]["selector"], labels);
    assert_eq!(s["spec"]["type"], "ClusterIP");
    assert_eq!(
        s["spec"]["ports"],
        json!([{"name":"http","port":80,"targetPort":"http","protocol":"TCP"}])
    );
    let pod = &d["spec"]["template"]["spec"];
    assert_eq!(pod["automountServiceAccountToken"], false);
    assert_eq!(pod["terminationGracePeriodSeconds"], 30);
    assert_eq!(
        pod["nodeSelector"],
        json!({"kubernetes.io/os":"linux","kubernetes.io/arch":"amd64"})
    );
    assert_eq!(
        pod["securityContext"],
        json!({"runAsNonRoot":true,"runAsUser":10001,"runAsGroup":10001,"seccompProfile":{"type":"RuntimeDefault"}})
    );
    assert_eq!(pod["containers"].as_array().unwrap().len(), 1);
    let app = &pod["containers"][0];
    assert_eq!(app["name"], "app");
    assert_eq!(app["image"], IMAGE);
    assert_eq!(app["imagePullPolicy"], "IfNotPresent");
    assert_eq!(
        app["ports"],
        json!([{"name":"http","containerPort":8080,"protocol":"TCP"}])
    );
    assert_eq!(
        app["env"],
        json!([{"name":"YETLET_HOST","value":"0.0.0.0"},{"name":"YETLET_PORT","value":"8080"}])
    );
    for (probe, path, period, failures) in [
        ("startupProbe", "/health/live", 2, 30),
        ("livenessProbe", "/health/live", 10, 3),
        ("readinessProbe", "/health/ready", 5, 3),
    ] {
        assert_eq!(
            app[probe],
            json!({"httpGet":{"path":path,"port":"http"},"periodSeconds":period,"timeoutSeconds":1,"failureThreshold":failures})
        );
    }
    assert_eq!(
        app["resources"],
        json!({"requests":{"cpu":"100m","memory":"64Mi"},"limits":{"cpu":"500m","memory":"128Mi"}})
    );
    assert_eq!(
        app["securityContext"],
        json!({"readOnlyRootFilesystem":true,"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]}})
    );
}

#[test]
fn output_property_retains_exact_images_counts_and_absence_of_sensitive_fields() {
    fn absent(value: &Value) {
        match value {
            Value::Object(fields) => {
                for (key, value) in fields {
                    assert!(
                        ![
                            "namespace",
                            "data",
                            "stringData",
                            "envFrom",
                            "valueFrom",
                            "imagePullSecrets",
                            "serviceAccountName",
                            "volumes",
                            "volumeMounts",
                            "hostPath",
                            "hostNetwork",
                            "hostPort",
                            "nodePort",
                            "privileged",
                            "command",
                            "args"
                        ]
                        .contains(&key.as_str()),
                        "{key}"
                    );
                    absent(value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    absent(value);
                }
            }
            Value::String(text) => assert!(!text.contains(['\n', '\r', '\0', '$', '`', '\\'])),
            _ => (),
        }
    }
    let digest = format!("example.com/app@sha256:{}", "ab".repeat(32));
    for name in ["a", "on", "native-ssr"] {
        for image in [IMAGE, digest.as_str(), "ghcr.io/team/a__b:V1"] {
            image::validate(image).unwrap();
            for replicas in [1, 2, 100] {
                let output = document(deployment(name, image, replicas)).unwrap();
                let parsed: Value = serde_json::from_str(&output).unwrap();
                assert_eq!(
                    parsed["spec"]["template"]["spec"]["containers"][0]["image"],
                    image
                );
                assert_eq!(parsed["spec"]["replicas"], replicas);
                assert_eq!(output, document(deployment(name, image, replicas)).unwrap());
                assert!(output.ends_with('\n'));
                assert!(!output.contains('\r'));
                absent(&parsed);
                absent(&service(name));
            }
        }
    }
}

#[test]
fn source_lock_framework_substitution_and_late_destinations_fail_closed() {
    use crate::cloud_fixture::Fixture;
    use std::fs;
    for mutation in 0..4 {
        let f = Fixture::new();
        let output = f.parent.join("out");
        assert!(
            generate_inner(&f.manifest(), &output, IMAGE, 1, || {
                match mutation {
                    0 => fs::write(f.app.join("src/main.rs"), "fn main() {}\n").unwrap(),
                    1 | 2 => {
                        let path = if mutation == 1 {
                            f.app.join("Cargo.lock")
                        } else {
                            f.framework.join("crates/yetlet/src/view/mod.rs")
                        };
                        let before = fs::read(&path).unwrap();
                        if let Err(error) = fs::write(&path, "changed\n") {
                            // Accepted Windows handles may prevent the mutation
                            // itself. Preserve that guarantee and still test a
                            // snapshot mismatch using the unpinned app source.
                            assert!(
                                cfg!(windows) && error.raw_os_error() == Some(32),
                                "unexpected protected mutation failure: {error}"
                            );
                            assert_eq!(fs::read(path).unwrap(), before);
                            fs::write(f.app.join("src/main.rs"), "fn main() {}\n").unwrap();
                        }
                    }
                    _ => fs::write(&output, "foreign").unwrap(),
                }
            })
            .is_err()
        );
        if mutation == 3 {
            assert_eq!(fs::read(output).unwrap(), b"foreign");
        } else {
            assert!(!output.exists());
        }
        assert_eq!(
            fs::read_dir(&f.parent).unwrap().count(),
            4 + usize::from(mutation == 3)
        );
    }
    let f = Fixture::new();
    for replicas in [0, 101, u32::MAX] {
        assert!(generate(&f.manifest(), &f.parent.join("invalid"), IMAGE, replicas).is_err());
        assert!(!f.parent.join("invalid").exists());
    }
}
