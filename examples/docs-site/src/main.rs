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
    sdk: String,
    cli: String,
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
fn shell(site: &Site, page: &Page, content: View, island: bool) -> View {
    let mut search = Element::new("section").attribute("class", "search-panel").attribute("id", "site-search")
        .child(Element::new("h2").child(View::text("Search this guide")))
        .child(Element::new("p").attribute("id", "site-search-status").attribute("role", "status").child(View::text(if island {
            "Optional search requires JavaScript and Wasm. Every chapter is already readable."
        } else { "HTML-only export. Browse all eight chapters using the guide navigation." })));
    if island {
        search = search
            .child(
                Element::new("button")
                    .attribute("id", "site-search-enable")
                    .attribute("type", "button")
                    .attribute("disabled", "")
                    .child(View::text("Enable search")),
            )
            .child(Element::new("div").attribute("id", "site-search-host"));
    }
    Element::new("div").attribute("class", "site")
        .child(Element::new("a").attribute("class", "skip-link").attribute("href", "#site-main").child(View::text("Skip to content")))
        .child(Element::new("header").attribute("class", "site-header")
            .child(Element::new("a").attribute("class", "brand").attribute("href", &site.base).child(View::text("Yetlet")).child(Element::new("span").child(View::text("Developer Guide"))))
            .child(Element::new("div").attribute("class", "header-actions")
                .child(Element::new("a").attribute("href", "#site-search").child(View::text("Search")))
                .child(Element::new("span").attribute("class", "preview-badge").child(View::text("Local pilot")))))
        .child(Element::new("details").attribute("class", "mobile-menu").attribute("id", "site-menu")
            .child(Element::new("summary").child(View::text("Browse guide")))
            .child(navigation(site, &page.route)))
        .child(Element::new("div").attribute("class", "layout")
            .child(Element::new("aside").attribute("class", "sidebar")
                .child(Element::new("p").attribute("class", "eyebrow").child(View::text("IN THIS GUIDE")))
                .child(navigation(site, &page.route)))
            .child(Element::new("main").attribute("id", "site-main").attribute("tabindex", "-1")
                .child(Element::new("p").attribute("class", "edition").child(View::text(format!("Experimental SDK {} · CLI {} · source {}", site.sdk, site.cli, &site.commit[..7]))))
                .child(content)
                .child(search)
                .child(Element::new("footer")
                    .child(Element::new("p").child(View::text(format!("Canonical source: {}. Local, unpublished checkout; pinned repository links may be unavailable until its commit is pushed.", page.source))))
                    .child(Element::new("p").child(View::text("No live Pages replacement. Original validation and release records remain in the repository."))))))
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
