//! Explicit product asset builder, never run by HTTP requests.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    use std::{fs, path::PathBuf, process::Command};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tool = std::env::var_os("WASM_BINDGEN")
        .ok_or("Set WASM_BINDGEN to the explicit 0.2.129 executable")?;
    let v = Command::new(&tool).arg("--version").output()?;
    if !v.status.success() || String::from_utf8_lossy(&v.stdout).trim() != "wasm-bindgen 0.2.129" {
        return Err("wasm-bindgen version mismatch".into());
    }
    let status = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .current_dir(&root)
        .args([
            "build",
            "--frozen",
            "--lib",
            "--target",
            "wasm32-unknown-unknown",
            "--features",
            "browser",
            "--release",
        ])
        .arg("--target-dir")
        .arg(root.join("target"))
        .status()?;
    if !status.success() {
        return Err("Wasm build failed; assets not published".into());
    }
    let dist = root.join("target/dist");
    fs::create_dir_all(&dist)?;
    let temp = dist.join(format!(".build-{}", std::process::id()));
    fs::create_dir(&temp)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new(tool)
            .arg(root.join("target/wasm32-unknown-unknown/release/yetlet_physics_lab.wasm"))
            .args(["--target", "web", "--out-name", "lab", "--out-dir"])
            .arg(&temp)
            .status()?;
        if !status.success() {
            return Err("bindgen failed; assets not published".into());
        }
        fs::write(temp.join("loader.js"), include_bytes!("../../loader.js"))?;
        fs::write(
            temp.join("wasm-startup.mjs"),
            include_bytes!("../../../../assets/wasm-startup.mjs"),
        )?;
        fs::write(
            temp.join("wasm-startup.css"),
            include_bytes!("../../../../assets/wasm-startup.css"),
        )?;
        fs::write(
            temp.join("frame-rate.mjs"),
            include_bytes!("../../frame-rate.mjs"),
        )?;
        let mut hash = Sha256::new();
        for n in [
            "loader.js",
            "lab.js",
            "lab_bg.wasm",
            "wasm-startup.mjs",
            "wasm-startup.css",
            "frame-rate.mjs",
        ] {
            hash.update(n.as_bytes());
            hash.update(fs::read(temp.join(n))?);
        }
        let digest = format!("{:x}", hash.finalize());
        let dest = dist.join(&digest);
        if !dest.exists() {
            fs::rename(&temp, &dest)?;
        } else {
            fs::remove_dir_all(&temp)?;
        }
        let manifest = dist.join("manifest.next.json");
        fs::write(
            &manifest,
            serde_json::to_vec(&serde_json::json!({"digest":digest}))?,
        )?;
        fs::rename(manifest, dist.join("manifest.json"))?;
        println!("Published source-hashed lab assets: {digest}");
        Ok(())
    })();
    if temp.exists() {
        fs::remove_dir_all(temp)?;
    }
    result
}
#[cfg(target_arch = "wasm32")]
fn main() {}
