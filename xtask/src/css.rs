//! Pinned Bootstrap transform; no network except the explicit mutually exclusive --fetch.
use crate::{
    build::{Build, Options},
    tooling::*,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use percent_encoding::percent_decode_str;
use regex::Regex;
use serde_json::json;
use sha2::{Digest, Sha384};
use std::{collections::BTreeMap, fs, path::Path, time::Duration};
use xml::reader::{EventReader, XmlEvent};
const UPSTREAM: &str = "https://raw.githubusercontent.com/twbs/bootstrap/v5.3.8/";
const SRI: &str = "sRIl4kxILFvY47J16cr9ZwB07vP4J8+LH7qKQnuqkuIAvNWLzeN8tE5YBujZqJLB";
const LIMIT: usize = 1024 * 1024;
const INPUTS: [(&str, &str); 2] = [
    ("bootstrap.min.css", "dist/css/bootstrap.min.css"),
    ("LICENSE.bootstrap", "LICENSE"),
];
fn validate_source(name: &str, data: &[u8]) -> Result<()> {
    if data.len() > LIMIT {
        return Err(fail("upstream asset exceeds bound"));
    }
    if name.ends_with(".css") && STANDARD.encode(Sha384::digest(data)) != SRI {
        return Err(fail("CSS source digest mismatch"));
    }
    if name.starts_with("LICENSE")
        && !String::from_utf8_lossy(data).contains("Permission is hereby granted")
    {
        return Err(fail("unexpected license content"));
    }
    Ok(())
}
pub fn validate_svg(data: &[u8]) -> Result<()> {
    let text = std::str::from_utf8(data).map_err(|_| fail("SVG not UTF-8"))?;
    if text.contains("<!") || text.contains("<?") {
        return Err(fail("SVG declarations/entities rejected"));
    }
    let mut root = false;
    for event in EventReader::new(data) {
        match event.map_err(|_| fail("invalid SVG XML"))? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if !root && name.local_name != "svg" {
                    return Err(fail("SVG root required"));
                }
                root = true;
                if name
                    .namespace
                    .as_deref()
                    .is_some_and(|n| n != "http://www.w3.org/2000/svg")
                    || ![
                        "svg", "g", "path", "polyline", "polygon", "line", "circle", "ellipse",
                        "rect",
                    ]
                    .contains(&name.local_name.as_str())
                {
                    return Err(fail("unexpected SVG element"));
                }
                for attr in attributes {
                    let key = attr.name.local_name.as_str();
                    let value = attr.value.to_ascii_lowercase();
                    if attr.name.namespace.is_some()
                        || ![
                            "viewBox",
                            "width",
                            "height",
                            "fill",
                            "fill-rule",
                            "stroke",
                            "stroke-width",
                            "stroke-linecap",
                            "stroke-linejoin",
                            "stroke-miterlimit",
                            "d",
                            "points",
                            "x",
                            "y",
                            "x1",
                            "x2",
                            "y1",
                            "y2",
                            "cx",
                            "cy",
                            "r",
                            "rx",
                            "ry",
                            "transform",
                            "opacity",
                            "fill-opacity",
                            "stroke-opacity",
                        ]
                        .contains(&key)
                        || value.contains("url(")
                        || value.contains("data:")
                        || value.contains("://")
                    {
                        return Err(fail("unexpected SVG active/external attribute"));
                    }
                }
            }
            XmlEvent::ProcessingInstruction { .. } => {
                return Err(fail("SVG processing instruction rejected"));
            }
            _ => {}
        }
    }
    if !root {
        return Err(fail("missing SVG root"));
    }
    Ok(())
}
/// Pure transform is tested with unsafe fixtures independently of the pinned digest.
pub type AssetOutputs = BTreeMap<String, (Vec<u8>, String)>;
pub fn externalize(css: &str, prefix: &str) -> Result<(String, AssetOutputs, usize)> {
    let expression = Regex::new(r#"url\("data:image/svg\+xml,([^"]*)"\)"#).unwrap();
    let mut outputs = BTreeMap::new();
    let mut count = 0;
    let mut transformed = String::new();
    let mut end = 0;
    for cap in expression.captures_iter(css) {
        let found = cap.get(0).unwrap();
        transformed.push_str(&css[end..found.start()]);
        let data = percent_decode_str(&cap[1]).collect::<Vec<_>>();
        validate_svg(&data)?;
        let name = format!("icon-{}.svg", sha(&data));
        transformed.push_str(&format!("url(\"{prefix}{name}\")"));
        outputs.insert(name, (data, "image/svg+xml".into()));
        count += 1;
        end = found.end();
    }
    transformed.push_str(&css[end..]);
    transformed = Regex::new(r"/\*# sourceMappingURL=.*?\*/")
        .unwrap()
        .replace_all(&transformed, "")
        .into_owned();
    validate_css_urls(&transformed, prefix, &outputs)?;
    Ok((transformed, outputs, count))
}
fn validate_css_urls(
    css: &str,
    prefix: &str,
    outputs: &BTreeMap<String, (Vec<u8>, String)>,
) -> Result<()> {
    let lower = css.to_ascii_lowercase();
    if lower.contains("data:")
        || lower.contains("@import")
        || css.contains('\\')
        || lower.contains("expression(")
    {
        return Err(fail("unconverted embedded/external CSS source"));
    }
    for cap in Regex::new(r"(?i)url\(([^)]+)\)")
        .unwrap()
        .captures_iter(css)
    {
        let url = cap[1].trim_matches(['\'', '"']);
        if url
            .strip_prefix(prefix)
            .is_none_or(|n| !outputs.contains_key(n))
        {
            return Err(fail("CSS references an unregistered asset"));
        }
    }
    Ok(())
}
pub fn expected(root: &Path, folder: &str, prefix: &str) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut originals = BTreeMap::new();
    for (name, _) in INPUTS {
        let data = read(root, format!("{folder}/assets/upstream/{name}"))?;
        validate_source(name, &data)?;
        originals.insert(name, data);
    }
    let css =
        std::str::from_utf8(&originals["bootstrap.min.css"]).map_err(|_| fail("CSS not UTF-8"))?;
    let (css, mut outputs, count) = externalize(css, prefix)?;
    let css_name = format!("bootstrap-{}.css", sha(css.as_bytes()));
    outputs.insert(
        css_name.clone(),
        (css.into_bytes(), "text/css; charset=utf-8".into()),
    );
    let mut locals = BTreeMap::new();
    for (name, mime) in [
        ("app.css", "text/css; charset=utf-8"),
        ("enhance.js", "text/javascript; charset=utf-8"),
    ] {
        let data = read(root, format!("{folder}/assets/{name}"))?;
        let s = std::str::from_utf8(&data).map_err(|_| fail("local asset not UTF-8"))?;
        if name.ends_with(".css") {
            validate_css_urls(s, prefix, &outputs)?;
        } else {
            // These fixed application loaders have no imports or network asset URLs.
            // Fail closed when modified to introduce embedded or remote executable assets.
            let lower = s.to_ascii_lowercase();
            if [
                "data:", "https:", "http:", "import(", "import ", "import\n", "import\t",
            ]
            .iter()
            .any(|v| lower.contains(v))
            {
                return Err(fail(
                    "local JS contains embedded/remote asset or unregistered import",
                ));
            }
        }
        let (stem, suffix) = name.rsplit_once('.').unwrap();
        let hashed = format!("{stem}-{}.{suffix}", sha(&data));
        locals.insert(name, hashed.clone());
        outputs.insert(hashed, (data, mime.into()));
    }
    let manifest = json!({"schema_version":1,"stylesheets":[format!("{prefix}{css_name}"),format!("{prefix}{}",locals["app.css"])],"enhancement":format!("{prefix}{}",locals["enhance.js"])});
    let source:Vec<_>=INPUTS.iter().map(|(name,relative)|json!({"file":format!("upstream/{name}"),"url":format!("{UPSTREAM}{relative}"),"sha256":sha(&originals[name])})).collect();
    let output_records:Vec<_>=outputs.iter().map(|(name,(data,mime))|json!({"file":format!("generated/{name}"),"sha256":sha(data),"bytes":data.len(),"content_type":mime})).collect();
    let inventory = json!({"bootstrap_version":"5.3.8","license":"MIT","source_css_sri":format!("sha384-{SRI}"),"source":source,"externalized_svg_occurrences":count,"outputs":output_records});
    let mut expected = BTreeMap::new();
    for (name, (data, _)) in &outputs {
        expected.insert(format!("{folder}/assets/generated/{name}"), data.clone());
    }
    expected.insert(
        format!("{folder}/assets/manifest.json"),
        json_bytes(&manifest),
    );
    expected.insert(
        format!("{folder}/assets/inventory.json"),
        json_bytes(&inventory),
    );
    // Retain the original provenance comment to preserve every generated byte.
    let mut rust="// Generated by build_assets.py; never edit by hand.\n#[rustfmt::skip]\npub struct EmbeddedAsset { pub name: &'static str, pub content_type: &'static str, pub bytes: &'static [u8] }\n#[rustfmt::skip]\npub static ASSETS: &[EmbeddedAsset] = &[\n".to_owned();
    for (name, (_, mime)) in &outputs {
        rust.push_str(&format!("    EmbeddedAsset {{ name: \"{name}\", content_type: \"{mime}\", bytes: include_bytes!(\"../assets/generated/{name}\") }},\n"));
    }
    rust.push_str("];\n");
    expected.insert(format!("{folder}/src/asset_data.rs"), rust.into_bytes());
    Ok(expected)
}
pub fn run(root: &Path, kind: Build, options: Options) -> Result<()> {
    if options.fetch && options.check {
        return Err(fail("fetch and check are separate operations"));
    }
    let (folder, prefix) = if kind == Build::FormCssProof {
        ("examples/form-css-proof", "/form-proof-assets/")
    } else {
        ("examples/lab-operations-board", "/board-assets/")
    };
    if options.fetch {
        // Exact fixed HTTPS URLs only; redirects cannot leave the allowlist.
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .max_redirects(0)
            .build()
            .new_agent();
        for (name, relative) in INPUTS {
            check_cancel()?;
            let target = format!("{folder}/assets/upstream/{name}");
            let path = safe(root, &target, true)?;
            if path.exists() {
                validate_source(name, &fs::read(path)?)?;
                continue;
            }
            let mut response = agent
                .get(format!("{UPSTREAM}{relative}"))
                .call()
                .map_err(|_| fail("pinned HTTPS acquisition failed"))?;
            if response.status() != 200 {
                return Err(fail("pinned HTTPS status is not 200"));
            }
            let data = response
                .body_mut()
                .with_config()
                .limit((LIMIT + 1) as u64)
                .read_to_vec()
                .map_err(|_| fail("upstream read failed or exceeded bound"))?;
            validate_source(name, &data)?;
            write(root, target, &data)?;
        }
    }
    let expected = expected(root, folder, prefix)?;
    let generated = format!("{folder}/assets/generated");
    let directory = safe(root, &generated, true)?;
    if directory.exists() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| fail("unexpected generated asset"))?;
            let relative = format!("{generated}/{name}");
            safe(root, &relative, true)?;
            if !expected.contains_key(&relative) {
                return Err(fail(
                    "unexpected/stale generated asset; inspect and remove deliberately",
                ));
            }
        }
    }
    for (name, data) in expected {
        if options.check {
            if read(root, &name).ok().as_ref() != Some(&data) {
                return Err(fail(format!("generated artifact mismatch: {name}")));
            }
        } else {
            write(root, name, &data)?;
        }
    }
    println!(
        "PASS pinned CSS, license, SVG and hashed outputs; mode={}",
        if options.check { "check" } else { "build" }
    );
    check_cancel()
}
