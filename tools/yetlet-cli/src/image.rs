//! Bounded ASCII image contract shared by deployment-file generators.

const INVALID: &str = "image requires lowercase REGISTRY/REPOSITORY with an explicit non-latest tag or @sha256:64-lowercase-hex digest; no escapes or whitespace";

pub(crate) fn validate(image: &str) -> Result<(), &'static str> {
    if image.len() > 512 || !image.is_ascii() {
        return Err(INVALID);
    }
    let repository = if let Some((repository, digest)) = image.split_once('@') {
        let hash = digest.strip_prefix("sha256:").ok_or(INVALID)?;
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(INVALID);
        }
        repository
    } else {
        let (repository, tag) = image.rsplit_once(':').ok_or(INVALID)?;
        if tag.is_empty()
            || tag.len() > 128
            || tag.eq_ignore_ascii_case("latest")
            || !tag
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            || !tag.as_bytes()[0].is_ascii_alphanumeric() && !tag.starts_with('_')
        {
            return Err(INVALID);
        }
        repository
    };
    if repository.len() > 255 {
        return Err(INVALID);
    }
    let (registry, path) = repository.split_once('/').ok_or(INVALID)?;
    let host = if let Some((host, port)) = registry.split_once(':') {
        if port.is_empty()
            || !port.bytes().all(|b| b.is_ascii_digit())
            || port.starts_with('0')
            || port.parse::<u16>().ok().is_none_or(|n| n == 0)
        {
            return Err(INVALID);
        }
        host
    } else {
        registry
    };
    if host != "localhost" && !host.contains('.') {
        return Err(INVALID);
    }
    if host.len() > 253
        || host.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || label.starts_with('-')
                || label.ends_with('-')
        })
    {
        return Err(INVALID);
    }
    for component in path.split('/') {
        // OCI repository components: alphanumeric groups separated by '.',
        // one/two underscores or one/more hyphens. No normalization/decoding.
        let mut bytes = component.as_bytes();
        while !bytes.is_empty() {
            let group = bytes
                .iter()
                .take_while(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                .count();
            if group == 0 {
                return Err(INVALID);
            }
            bytes = &bytes[group..];
            if bytes.is_empty() {
                break;
            }
            let separator = match bytes[0] {
                b'.' => 1,
                b'_' => {
                    if bytes.get(1) == Some(&b'_') {
                        2
                    } else {
                        1
                    }
                }
                b'-' => bytes.iter().take_while(|b| **b == b'-').count(),
                _ => return Err(INVALID),
            };
            bytes = &bytes[separator..];
            if bytes.is_empty() {
                return Err(INVALID);
            }
        }
        if component.is_empty() {
            return Err(INVALID);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_ascii_registry_tag_and_digest_only() {
        for value in [
            "ghcr.io/team/app:v1",
            "docker.io/library/app:1.2_ABC",
            "localhost:5000/team/a__b.c--d:_v1",
            &format!("example.com/app@sha256:{}", "a1".repeat(32)),
        ] {
            assert_eq!(validate(value), Ok(()), "{value}");
        }
        for value in [
            "",
            "app",
            "app:v1",
            "team/app:v1",
            "ghcr.io/app",
            "ghcr.io/app:latest",
            "ghcr.io/app:LaTeSt",
            "ghcr.io/app:",
            "ghcr.io/app:-v1",
            "ghcr.io/APP:v1",
            "GHCR.io/app:v1",
            "ghcr.io//app:v1",
            "ghcr.io/a/:v1",
            "ghcr.io/a..b:v1",
            "ghcr.io/a___b:v1",
            "ghcr.io/../app:v1",
            "ghcr.io/app:v1\nprivileged: true",
            "ghcr.io/app:v1\r",
            "ghcr.io/app:v1\t",
            "ghcr.io/app:v1\0",
            "ghcr.io/app:v1\u{7f}",
            "ghcr.io/app:v1\u{a0}",
            "ghcr.io/аpp:v1",
            "ghcr.io/app：v1",
            "ghcr.io/app:v1'",
            "ghcr.io/app:${SECRET}",
            "ghcr.io/app:$(id)",
            "ghcr.io/app:`id`",
            "ghcr.io/app:v1;id",
            "ghcr.io/app:v1\\n",
            "ghcr.io/app:v1%0a",
            "https://ghcr.io/app:v1",
            "user@ghcr.io/app:v1",
            "localhost:0/app:v1",
            "localhost:65536/app:v1",
            "localhost:05000/app:v1",
            "[::1]:5000/app:v1",
            "-host.test/app:v1",
            "host_.test/app:v1",
            "example.com./app:v1",
            "example.com/app@sha256:abc",
            &format!("example.com/app:v1@sha256:{}", "a".repeat(64)),
            &format!("example.com/app@sha256:{}", "A".repeat(64)),
            &format!("example.com/app@sha256:{}@x", "a".repeat(64)),
            &format!("example.com/app:{}", "a".repeat(129)),
        ] {
            assert!(validate(value).is_err(), "{value:?}");
        }
    }
}
