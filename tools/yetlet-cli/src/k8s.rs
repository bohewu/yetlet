//! Configuration-only Linux Kubernetes manifests; no Kubernetes client or tools.
use crate::{compose, docker, image};
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn parse_replicas(value: &str) -> Result<u32, &'static str> {
    let invalid = "--replicas requires an ASCII decimal integer 1-100";
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid);
    }
    let replicas = value.parse().map_err(|_| invalid)?;
    if !(1..=100).contains(&replicas) {
        return Err(invalid);
    }
    Ok(replicas)
}

fn validate_name(name: &str) -> Result<(), &'static str> {
    // Identity mapping avoids underscore/hyphen and truncation collisions.
    // The letter prefix also satisfies the portable Service DNS label rule.
    if name.is_empty()
        || name.len() > 63
        || !name.as_bytes()[0].is_ascii_lowercase()
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || name.ends_with('-')
    {
        return Err(
            "Kubernetes requires an unchanged package name of 1-63 lowercase ASCII letters/digits/hyphens, starting with a letter and ending with a letter/digit; underscores and name translation are unsupported",
        );
    }
    Ok(())
}

pub(crate) fn generate(
    manifest: &Path,
    output: &Path,
    image: &str,
    replicas: u32,
) -> Result<(), String> {
    generate_inner(manifest, output, image, replicas, || {})
}

fn generate_inner(
    manifest: &Path,
    output: &Path,
    image: &str,
    replicas: u32,
    after_inspection: impl FnOnce(),
) -> Result<(), String> {
    image::validate(image).map_err(str::to_owned)?;
    if !(1..=100).contains(&replicas) {
        return Err("--replicas requires an ASCII decimal integer 1-100".into());
    }
    docker::generate_checked(
        manifest,
        output,
        |snapshot| {
            compose::validate_lock(snapshot)?;
            let name = snapshot.manifest["package"]["name"].as_str().unwrap();
            validate_name(name).map_err(str::to_owned)?;
            Ok(vec![
                (
                    "deployment.yaml",
                    document(deployment(name, image, replicas))?,
                ),
                ("service.yaml", document(service(name))?),
                (
                    "README.md",
                    include_str!("../templates/k8s/README.md").into(),
                ),
            ])
        },
        after_inspection,
    )
}

fn document(value: Value) -> Result<String, String> {
    // JSON is YAML 1.2 syntax. Use the existing serializer for typed scalars,
    // deterministic ordering and escaping, without a YAML/SDK dependency.
    serde_json::to_string_pretty(&value)
        .map(|text| format!("{text}\n"))
        .map_err(|_| "cannot render Kubernetes manifest".into())
}

fn deployment(name: &str, image: &str, replicas: u32) -> Value {
    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {"name": name, "labels": {"app.kubernetes.io/name": name}},
        "spec": {
            "replicas": replicas,
            "selector": {"matchLabels": {"app.kubernetes.io/name": name}},
            "template": {
                "metadata": {"labels": {"app.kubernetes.io/name": name}},
                "spec": {
                    "automountServiceAccountToken": false,
                    "terminationGracePeriodSeconds": 30,
                    "nodeSelector": {"kubernetes.io/os": "linux", "kubernetes.io/arch": "amd64"},
                    "securityContext": {
                        "runAsNonRoot": true,
                        "runAsUser": 10001,
                        "runAsGroup": 10001,
                        "seccompProfile": {"type": "RuntimeDefault"}
                    },
                    "containers": [{
                        "name": "app",
                        "image": image,
                        "imagePullPolicy": "IfNotPresent",
                        "ports": [{"name": "http", "containerPort": 8080, "protocol": "TCP"}],
                        "env": [
                            {"name": "YETLET_HOST", "value": "0.0.0.0"},
                            {"name": "YETLET_PORT", "value": "8080"}
                        ],
                        "startupProbe": {
                            "httpGet": {"path": "/health/live", "port": "http"},
                            "periodSeconds": 2, "timeoutSeconds": 1, "failureThreshold": 30
                        },
                        "livenessProbe": {
                            "httpGet": {"path": "/health/live", "port": "http"},
                            "periodSeconds": 10, "timeoutSeconds": 1, "failureThreshold": 3
                        },
                        "readinessProbe": {
                            "httpGet": {"path": "/health/ready", "port": "http"},
                            "periodSeconds": 5, "timeoutSeconds": 1, "failureThreshold": 3
                        },
                        "resources": {
                            "requests": {"cpu": "100m", "memory": "64Mi"},
                            "limits": {"cpu": "500m", "memory": "128Mi"}
                        },
                        "securityContext": {
                            "readOnlyRootFilesystem": true,
                            "allowPrivilegeEscalation": false,
                            "capabilities": {"drop": ["ALL"]}
                        }
                    }]
                }
            }
        }
    })
}

fn service(name: &str) -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {"name": name, "labels": {"app.kubernetes.io/name": name}},
        "spec": {
            "type": "ClusterIP",
            "selector": {"app.kubernetes.io/name": name},
            "ports": [{"name": "http", "port": 80, "targetPort": "http", "protocol": "TCP"}]
        }
    })
}

#[cfg(test)]
mod tests;
