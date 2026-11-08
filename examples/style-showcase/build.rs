//! App-owned host generation: no recursive Cargo, View traversal or browser tools.
#[path = "../../tools/yetlet-cli/templates/styles/builder.rs"]
#[allow(dead_code, clippy::collapsible_if)]
mod builder;
#[path = "src/style_names.rs"]
#[allow(dead_code)]
mod style_names;
#[path = "styles/mod.rs"]
mod styles;

use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let inputs = [
        "build.rs",
        "src/lib.rs",
        "src/style_names.rs",
        "src/selection.rs",
        "src/previews/layout.rs",
        "src/previews/tokens.rs",
        "src/previews/components.rs",
        "src/previews/states.rs",
        "styles/mod.rs",
        "styles/layout.rs",
        "styles/tokens.rs",
        "styles/components.rs",
        "styles/states.rs",
        "styles/app.css",
        "../../tools/yetlet-cli/templates/styles/builder.rs",
        "../style-showcase-browser/src/lib.rs",
        "../style-showcase-browser/loader.js",
    ];
    let mut source = Sha256::new();
    for input in inputs {
        println!("cargo:rerun-if-changed={input}");
        source.update(input.as_bytes());
        source.update([0]);
        source.update(fs::read(root.join(input)).unwrap());
        source.update([0]);
    }
    let css = format!(
        "/* Registered Rust styles: styles/mod.rs */\n{}\n/* Ordinary app CSS: styles/app.css */\n{}",
        styles::stylesheet().unwrap().css().unwrap(),
        include_str!("styles/app.css")
    );
    let hash = format!("{:x}", Sha256::digest(css.as_bytes()));
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("showcase.css"), css).unwrap();
    fs::write(out.join("identity.rs"), format!(
        "pub const CSS_SHA256: &str = {hash:?};\npub const CSS_URL: &str = \"/showcase-assets/{}/showcase.css\";\npub const SOURCE_SHA256: &str = {:?};\n",
        &hash[..16], format!("{:x}", source.finalize()))).unwrap();
}
