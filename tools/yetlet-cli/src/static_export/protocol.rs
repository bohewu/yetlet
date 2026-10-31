use crate::platform::fs::static_output as files;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use yetlet::html::escape_text;

const MIB: usize = 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema_version: u32,
    pages: Vec<Page>,
    assets: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Page {
    route: String,
    title: String,
    language: String,
    fragment: String,
    stylesheets: Vec<String>,
    #[serde(deserialize_with = "required_optional")]
    island_entry: Option<String>,
}
fn required_optional<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Option::deserialize(d)
}
fn error() -> String {
    "static descriptor, path, metadata or resource selection is invalid".into()
}
fn device(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or("");
    matches!(stem, "con" | "prn" | "aux" | "nul")
        || ["com", "lpt"].iter().any(|p| {
            stem.strip_prefix(p)
                .is_some_and(|s| s.len() == 1 && matches!(s.as_bytes()[0], b'1'..=b'9'))
        })
}
fn component(value: &str, dots: bool) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && (value.as_bytes()[0].is_ascii_lowercase() || value.as_bytes()[0].is_ascii_digit())
        && !value.ends_with('.')
        && !device(value)
        && value.bytes().all(|b| {
            b.is_ascii_lowercase()
                || b.is_ascii_digit()
                || matches!(b, b'_' | b'-')
                || (dots && b == b'.')
        })
}
// Keep grammar explicit instead of filesystem normalization.
pub(super) fn base(value: &str) -> bool {
    value == "/"
        || (value.len() <= 240 && value.starts_with('/') && value.ends_with('/') && {
            let parts: Vec<_> = value[1..value.len() - 1].split('/').collect();
            parts.len() <= 8 && parts.iter().all(|s| component(s, false))
        })
}
fn route(value: &str) -> Result<String, String> {
    if value == "/404.html" {
        return Ok("404.html".into());
    }
    if !base(value) || value.starts_with("/assets/") {
        return Err(error());
    }
    Ok(if value == "/" {
        "index.html".into()
    } else {
        format!("{}index.html", &value[1..])
    })
}
fn artifact(value: &str, root: &str) -> bool {
    value.len() <= 240 && value.starts_with(root) && {
        let parts: Vec<_> = value.split('/').collect();
        parts.len() <= 8 && parts.iter().all(|s| component(s, true))
    }
}
fn language(value: &str) -> bool {
    if value.is_empty() || value.len() > 35 {
        return false;
    }
    let mut p = value.split('-');
    p.next().is_some_and(|s| {
        !s.is_empty() && s.len() <= 8 && s.bytes().all(|b| b.is_ascii_alphabetic())
    }) && p.all(|s| !s.is_empty() && s.len() <= 8 && s.bytes().all(|b| b.is_ascii_alphanumeric()))
}
fn insert(set: &mut BTreeSet<String>, path: &str) -> Result<(), String> {
    let folded = path.to_ascii_lowercase();
    if set.iter().any(|x| {
        x == &folded
            || x.starts_with(&(folded.clone() + "/"))
            || folded.starts_with(&(x.clone() + "/"))
    }) {
        return Err(error());
    }
    set.insert(folded);
    Ok(())
}
fn public_url(base: &str, path: &str) -> String {
    format!("{base}{path}")
}
fn document(page: &Page, body: &str, base: &str, islands: bool) -> String {
    let scripts = if islands {
        "script-src 'self' 'wasm-unsafe-eval'; connect-src 'self'"
    } else {
        "script-src 'none'; connect-src 'none'"
    };
    let csp = format!(
        "default-src 'none'; {scripts}; style-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'none'"
    );
    let mut out = format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"{}\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title>",
        escape_text(&page.language),
        escape_text(&csp),
        escape_text(&page.title)
    );
    for css in &page.stylesheets {
        out.push_str(&format!(
            "<link rel=\"stylesheet\" href=\"{}\">",
            escape_text(&public_url(base, css))
        ));
    }
    if let Some(entry) = &page.island_entry {
        out.push_str(&format!(
            "<script type=\"module\" src=\"{}\"></script>",
            escape_text(&public_url(base, entry))
        ));
    }
    out.push_str("</head><body>");
    out.push_str(body);
    out.push_str("</body></html>");
    out
}
pub(super) struct Prepared {
    pub files: BTreeMap<String, Vec<u8>>,
    pub routes: BTreeMap<String, String>,
}
pub(super) fn prepare(root: &Path, base_path: &str, islands: bool) -> Result<Prepared, String> {
    if !base(base_path) {
        return Err(error());
    }
    let data = files::read(root, "export.json", MIB)?;
    let desc: Descriptor = serde_json::from_slice(&data).map_err(|_| error())?;
    if desc.schema_version != 1
        || desc.pages.is_empty()
        || desc.pages.len() > 256
        || desc.assets.len() > 1024
        || desc.pages.iter().filter(|p| p.route == "/").count() != 1
    {
        return Err(error());
    }
    let mut expected = BTreeSet::from(["export.json".to_owned()]);
    let mut output_names = BTreeSet::new();
    let mut fragments = BTreeSet::new();
    let mut assets = BTreeSet::new();
    let allowed = if islands {
        &[
            "css", "png", "jpg", "jpeg", "webp", "ico", "js", "mjs", "wasm",
        ][..]
    } else {
        &["css", "png", "jpg", "jpeg", "webp", "ico"][..]
    };
    for a in &desc.assets {
        if !artifact(a, "assets/")
            || !allowed.contains(&a.rsplit('.').next().unwrap_or(""))
            || !assets.insert(a.clone())
        {
            return Err(error());
        }
        insert(&mut output_names, a)?;
        expected.insert(a.clone());
    }
    let mut routes = BTreeMap::new();
    for page in &desc.pages {
        let path = route(&page.route)?;
        insert(&mut output_names, &path)?;
        if !language(&page.language)
            || !artifact(&page.fragment, "fragments/")
            || !page.fragment.ends_with(".html")
            || !fragments.insert(page.fragment.clone())
        {
            return Err(error());
        }
        expected.insert(page.fragment.clone());
        let mut css = BTreeSet::new();
        if page
            .stylesheets
            .iter()
            .any(|s| !assets.contains(s) || !s.ends_with(".css") || !css.insert(s))
        {
            return Err(error());
        }
        if page.island_entry.as_ref().is_some_and(|s| {
            !islands || !assets.contains(s) || !matches!(s.rsplit('.').next(), Some("js" | "mjs"))
        }) {
            return Err(error());
        }
        routes.insert(page.route.clone(), path);
    }
    // Include directory names so unlisted empty directories are also refused.
    for file in expected.clone() {
        let mut path = Path::new(&file).parent();
        while let Some(p) = path.filter(|p| !p.as_os_str().is_empty()) {
            expected.insert(format!("{}/", p.to_str().ok_or_else(error)?));
            path = p.parent();
        }
    }
    if files::inventory(root)?.into_iter().collect::<BTreeSet<_>>() != expected {
        return Err(error());
    }
    let mut total = data.len();
    let mut output_total = 0;
    let mut output = BTreeMap::new();
    for a in &desc.assets {
        let bytes = files::read(root, a, 16 * MIB)?;
        total += bytes.len();
        output_total += bytes.len();
        if total > 64 * MIB {
            return Err(error());
        }
        output.insert(a.clone(), bytes);
    }
    for page in &desc.pages {
        let bytes = files::read(root, &page.fragment, 2 * MIB)?;
        total += bytes.len();
        if total > 64 * MIB {
            return Err(error());
        }
        let body = std::str::from_utf8(&bytes).map_err(|_| error())?;
        let html = document(page, body, base_path, islands).into_bytes();
        output_total += html.len();
        if output_total > 64 * MIB {
            return Err(error());
        }
        output.insert(routes[&page.route].clone(), html);
    }
    output.insert(".nojekyll".into(), Vec::new());
    Ok(Prepared {
        files: output,
        routes,
    })
}
