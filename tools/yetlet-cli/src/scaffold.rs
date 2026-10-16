use std::path::Path;

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    const RESERVED: &[&str] = &[
        "as",
        "async",
        "await",
        "break",
        "const",
        "continue",
        "crate",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
        "abstract",
        "become",
        "box",
        "do",
        "final",
        "gen",
        "macro",
        "override",
        "priv",
        "try",
        "typeof",
        "unsized",
        "virtual",
        "yield",
        "con",
        "prn",
        "aux",
        "nul",
        "test",
        "build",
        "deps",
        "examples",
        "incremental",
        // Cargo package IDs must not collide with our path dependencies at 0.1.0.
        "yetlet-core",
        "yetlet-html",
        "yetlet-server-axum",
    ];
    if name.len() > 64
        || !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        || RESERVED.contains(&name)
        || ["com", "lpt"].iter().any(|prefix| {
            name.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
        })
    {
        return Err("invalid or reserved project NAME (see --help)");
    }
    Ok(())
}

// Preserve the accepted YT-087/YT-088 public scaffold API and its default profile.
pub fn create(name: &str, framework: &Path, dir: &Path) -> Result<(), String> {
    create_with_profile(name, framework, dir, false)
}

// The cloud profile is an explicit opt-in. No existing callers need a new argument.
pub fn create_cloud(name: &str, framework: &Path, dir: &Path) -> Result<(), String> {
    create_with_profile(name, framework, dir, true)
}

fn create_with_profile(
    name: &str,
    framework: &Path,
    dir: &Path,
    cloud: bool,
) -> Result<(), String> {
    validate_name(name).map_err(str::to_owned)?;
    crate::platform::fs::ensure_create_supported()?;
    let framework = crate::framework::validate(framework).map_err(str::to_owned)?;
    let entries = files(name, &framework, cloud)?;
    crate::platform::fs::create_project(dir, name, &entries)
}

pub(crate) fn files(
    name: &str,
    framework: &Path,
    cloud: bool,
) -> Result<Vec<(&'static str, String)>, String> {
    let quoted = |path: &Path| -> Result<String, String> {
        Ok(toml::Value::String(
            path.to_str()
                .ok_or("framework path must be UTF-8 for Cargo.toml")?
                .into(),
        )
        .to_string())
    };
    let core = quoted(&framework.join("crates/yetlet-core"))?;
    let server = quoted(&framework.join("crates/yetlet-server-axum"))?;
    let signal = if cloud { ", \"signal\"" } else { "" };
    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.99\"\ndescription = \"Native Yetlet SSR application preview\"\nlicense = \"MIT OR Apache-2.0\"\npublish = false\n\n[workspace]\nresolver = \"3\"\n\n[dependencies]\naxum = \"0.8\"\ntokio = {{ version = \"1\", features = [\"macros\", \"net\", \"rt-multi-thread\"{signal}] }}\nyetlet-core = {{ path = {core} }}\nyetlet-server-axum = {{ path = {server} }}\n"
    );
    let (manifest, readme, main) = if cloud {
        (
            format!("{manifest}\n[package.metadata.yetlet]\ncloud-profile = \"native-ssr-v1\"\n"),
            include_str!("../templates/cloud/README.md"),
            include_str!("../templates/cloud/main.rs"),
        )
    } else {
        (
            manifest,
            include_str!("../templates/README.md"),
            include_str!("../templates/main.rs"),
        )
    };
    Ok(vec![
        ("Cargo.toml", manifest),
        ("README.md", readme.replace("@NAME@", name)),
        (".gitignore", "/target/\n.env\n.env.*\n".into()),
        ("src/main.rs", main.into()),
    ])
}
