//! Build identity consumed only by the explicit repository browser builder.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use yetlet_style_showcase::{CSS_SHA256, SOURCE_SHA256};
    println!(
        "{}",
        serde_json::json!({"css_sha256": CSS_SHA256, "source_sha256": SOURCE_SHA256})
    );
}
#[cfg(target_arch = "wasm32")]
fn main() {}
