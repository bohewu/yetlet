use docs_search::markdown;
use serde::Deserialize;
use serde_json::json;
use std::{error::Error, path::PathBuf};
use yetlet::{
    html::render_to_string,
    view::{Element, View},
};

#[derive(Deserialize)]
struct Site {
    commit: String,
    #[serde(default)]
    public: bool,
    sdk: String,
    cli: String,
    published_sdk: String,
    published_cli: String,
    base: String,
    chapters: Vec<Page>,
}
#[derive(Deserialize)]
struct Page {
    source: String,
    title: String,
    route: String,
    markdown: String,
}

fn navigation(site: &Site, current: &str) -> Element {
    let links = site.chapters.iter().map(|p| {
        let mut link = Element::new("a")
            .attribute(
                "href",
                format!("{}{}", site.base, p.route.trim_start_matches('/')),
            )
            .child(View::text(&p.title));
        if p.route == current {
            link = link.attribute("aria-current", "page");
        }
        Element::new("li").child(link).into()
    });
    Element::new("nav")
        .attribute("aria-label", "Guide chapters")
        .child(Element::new("ol").children(links))
}
fn header_search() -> Element {
    Element::new("div")
        .attribute("class", "header-search")
        .attribute("id", "site-search")
        .attribute("role", "search")
        .child(
            Element::new("div")
                .attribute("class", "search-field")
                .child(
                    Element::new("span")
                        .attribute("class", "search-icon")
                        .attribute("aria-hidden", "true"),
                )
                .child(
                    Element::new("input")
                        .attribute("id", "site-search-input")
                        .attribute("type", "search")
                        .attribute("aria-label", "Search this guide")
                        .attribute("aria-controls", "site-search-results")
                        .attribute("placeholder", "Search this guide…")
                        .attribute("maxlength", "100")
                        .attribute("autocomplete", "off"),
                ),
        )
        .child(
            Element::new("p")
                .attribute("id", "site-search-status")
                .attribute("class", "search-feedback")
                .attribute("role", "status")
                .child(View::text("Enable JavaScript to search.")),
        )
        .child(
            Element::new("div")
                .attribute("id", "site-search-results")
                .attribute("class", "search-results")
                .attribute("aria-live", "polite")
                .attribute("hidden", ""),
        )
}
fn shell(site: &Site, page: &Page, content: View, island: bool) -> View {
    let mut header = Element::new("header")
        .attribute("class", "site-header")
        .child(
            Element::new("a")
                .attribute("class", "brand")
                .attribute("href", &site.base)
                .child(View::text("Yetlet"))
                .child(Element::new("span").child(View::text("Developer Guide"))),
        );
    if island {
        header = header.child(header_search());
    }
    let published = Element::new("p")
        .attribute("class", "edition")
        .child(View::text(format!(
            "Published: Experimental SDK {} · CLI {}",
            site.published_sdk, site.published_cli
        )));
    let mut edition = Element::new("p")
        .attribute("class", "edition")
        .child(View::text(format!(
            "Reader Edition: SDK {} · CLI {} · source ",
            site.sdk, site.cli
        )));
    let source_url = format!("https://github.com/bohewu/yetlet/tree/{}", site.commit);
    edition = if site.public {
        edition.child(
            Element::new("a")
                .attribute("href", &source_url)
                .attribute("title", &site.commit)
                .child(View::text(&site.commit[..7])),
        )
    } else {
        edition.child(View::text(&site.commit[..7]))
    };
    let mut footer = Element::new("footer").child(Element::new("p").child(View::text(
        if site.public {
            format!("Canonical source: {}.", page.source)
        } else {
            format!("Canonical source: {}. Local, unpublished checkout; pinned repository links may be unavailable until its commit is pushed.", page.source)
        },
    )));
    if !site.public {
        footer = footer.child(Element::new("p").child(View::text(
            "No live Pages replacement. Original validation and release records remain in the repository.",
        )));
    }
    Element::new("div")
        .attribute("class", "site")
        .child(
            Element::new("a")
                .attribute("class", "skip-link")
                .attribute("href", "#site-main")
                .child(View::text("Skip to content")),
        )
        .child(header)
        .child(
            Element::new("details")
                .attribute("class", "mobile-menu")
                .attribute("id", "site-menu")
                .child(Element::new("summary").child(View::text("Browse guide")))
                .child(navigation(site, &page.route)),
        )
        .child(
            Element::new("div")
                .attribute("class", "layout")
                .child(
                    Element::new("aside")
                        .attribute("class", "sidebar")
                        .child(
                            Element::new("p")
                                .attribute("class", "eyebrow")
                                .child(View::text("IN THIS GUIDE")),
                        )
                        .child(navigation(site, &page.route)),
                )
                .child(
                    Element::new("main")
                        .attribute("id", "site-main")
                        .attribute("tabindex", "-1")
                        .child(published)
                        .child(edition)
                        .child(content)
                        .child(footer),
                ),
        )
        .into()
}
fn directory(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new().mode(0o700).create(path)
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir(path)
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--yetlet-static-export")
        || args.next().as_deref() != Some("1")
        || args.next().as_deref() != Some("--output")
    {
        return Err("Expected static export protocol1".into());
    }
    let output = PathBuf::from(args.next().ok_or("Missing private output")?);
    if args.next().as_deref() != Some("--base-path") {
        return Err("Missing base path".into());
    }
    let base = args.next().ok_or("Missing base path")?;
    let islands = match args.next().as_deref() {
        Some("--wasm-islands") => true,
        None => false,
        _ => return Err("Unexpected export option".into()),
    };
    if args.next().is_some() {
        return Err("Unexpected argument".into());
    }
    let site: Site = serde_json::from_str(include_str!("generated/chapters.json"))?;
    if base != site.base || site.chapters.len() != 8 {
        return Err("Prepare the reviewed eight chapters at /yetlet/ first".into());
    }
    // Complete parsing/rendering before any fragment is emitted.
    let mut fragments = Vec::new();
    let mut search = Vec::new();
    for page in &site.chapters {
        let chapter =
            markdown::render(&page.markdown).map_err(|e| format!("{}: {e}", page.source))?;
        search.push(json!({"title":page.title,"href":format!("{}{}",site.base,page.route.trim_start_matches('/')),"text":chapter.text}));
        fragments.push(render_to_string(&shell(
            &site,
            page,
            chapter.view,
            islands,
        ))?);
    }
    directory(&output.join("fragments"))?;
    directory(&output.join("assets"))?;
    std::fs::write(
        output.join("assets/guide.css"),
        include_bytes!("../public/guide.css"),
    )?;
    let mut assets = vec!["assets/guide.css"];
    if islands {
        for name in [
            "search.mjs",
            "wasm-startup.mjs",
            "docs_search.js",
            "docs_search_bg.wasm",
        ] {
            let path = if name.starts_with("docs_search") {
                format!("frontend/generated/{name}")
            } else {
                format!("frontend/{name}")
            };
            std::fs::copy(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path),
                output.join("assets").join(name),
            )?;
        }
        std::fs::write(
            output.join("assets/search-data.mjs"),
            format!("export default {};\n", serde_json::to_string(&search)?),
        )?;
        assets.extend([
            "assets/search.mjs",
            "assets/wasm-startup.mjs",
            "assets/docs_search.js",
            "assets/docs_search_bg.wasm",
            "assets/search-data.mjs",
        ]);
    }
    let mut pages = Vec::new();
    for (i, (page, html)) in site.chapters.iter().zip(fragments).enumerate() {
        let path = format!("fragments/chapter-{i}.html");
        std::fs::write(output.join(&path), html)?;
        pages.push(json!({"route":page.route,"title":format!("{} — Yetlet Guide",page.title),"language":"en","fragment":path,"stylesheets":["assets/guide.css"],"island_entry":if islands {Some("assets/search.mjs")} else {None}}));
    }
    std::fs::write(
        output.join("export.json"),
        serde_json::to_vec_pretty(&json!({"schema_version":1,"pages":pages,"assets":assets}))?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_edition_links_the_full_source_without_local_preview_copy() {
        let mut site = Site {
            commit: "a".repeat(40),
            public: true,
            sdk: "0.9.0-alpha.5".into(),
            cli: "0.1.1".into(),
            published_sdk: "0.9.0-alpha.7".into(),
            published_cli: "0.3.0".into(),
            base: "/yetlet/".into(),
            chapters: vec![],
        };
        let page = Page {
            source: "README.md".into(),
            title: "Overview".into(),
            route: "/".into(),
            markdown: String::new(),
        };
        let public = render_to_string(&shell(&site, &page, View::text("Guide"), true)).unwrap();
        assert!(public.contains(&format!(
            "https://github.com/bohewu/yetlet/tree/{}",
            site.commit
        )));
        assert!(public.contains("Published: Experimental SDK 0.9.0-alpha.7 · CLI 0.3.0"));
        assert!(public.contains("Reader Edition: SDK 0.9.0-alpha.5 · CLI 0.1.1 · source "));
        assert!(!public.contains("Local, unpublished checkout"));
        assert!(!public.contains("No live Pages replacement"));
        site.public = false;
        let preview = render_to_string(&shell(&site, &page, View::text("Guide"), true)).unwrap();
        assert!(preview.contains("Local, unpublished checkout"));
        assert!(preview.contains("No live Pages replacement"));
    }
}
