use serde_json::json;
use std::{
    error::Error,
    path::{Path, PathBuf},
};
use yetlet::{
    html::render_to_string,
    view::{Element, View},
};

fn directory(path: &Path) -> std::io::Result<()> {
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
fn page(base: &str, label: &str, guide: bool, island: bool) -> View {
    let heading = Element::new("h1").child(View::text(label));
    let nav = Element::new("nav")
        .child(
            Element::new("a")
                .attribute("href", base)
                .child(View::text("Home")),
        )
        .child(View::text(" · "))
        .child(
            Element::new("a")
                .attribute("href", format!("{base}guide/install/#steps"))
                .child(View::text("Installation guide")),
        );
    let mut main = Element::new("main")
        .child(heading)
        .child(nav)
        .child(Element::new("p").child(View::text(
            "Readable HTML without JavaScript. Escaped content: <safe> & 世界.",
        )))
        .child(
            Element::new("img")
                .attribute("src", format!("{base}assets/diagram.png"))
                .attribute("alt", "Blue static asset tile")
                .attribute("width", "24")
                .attribute("height", "24"),
        );
    if guide {
        main = main.child(
            Element::new("section")
                .attribute("id", "steps")
                .child(Element::new("h2").child(View::text("Build steps")))
                .child(
                    Element::new("ol")
                        .child(
                            Element::new("li")
                                .child(View::text("Prepare the app lock and native toolchain.")),
                        )
                        .child(
                            Element::new("li")
                                .child(View::text("Export to a new output directory.")),
                        ),
                ),
        );
    }
    if island {
        main=main.child(Element::new("section").attribute("id","island-panel")
        .child(Element::new("h2").child(View::text("Optional local counter")))
        .child(Element::new("p").attribute("id","island-status").attribute("role","status").child(View::text("The guide works without this counter. Enable it to load Wasm.")))
        .child(Element::new("button").attribute("id","enable-counter").attribute("type","button").child(View::text("Enable counter")))
        .child(Element::new("div").attribute("id","island-host").child(View::text("Static counter placeholder; no download until activation.")))
        .child(Element::new("p").child(View::text("If activation is unavailable, keep reading the guide or reload this page. Reload discards the local counter."))));
    }
    main.into()
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some("--yetlet-static-export".as_ref())
        || args.next().as_deref() != Some("1".as_ref())
        || args.next().as_deref() != Some("--output".as_ref())
    {
        return Err("expected static export protocol1".into());
    }
    let output = PathBuf::from(args.next().ok_or("missing private output")?);
    if args.next().as_deref() != Some("--base-path".as_ref()) {
        return Err("missing public base".into());
    }
    let base = args
        .next()
        .ok_or("missing public base")?
        .into_string()
        .map_err(|_| "base must be UTF-8")?;
    let island = match args.next() {
        Some(x) if x == "--wasm-islands" => true,
        None => false,
        _ => return Err("unexpected export argument".into()),
    };
    if args.next().is_some() {
        return Err("unexpected export argument".into());
    }
    directory(&output.join("fragments"))?;
    directory(&output.join("assets"))?;
    std::fs::write(
        output.join("assets/site.css"),
        include_bytes!("../public/site.css"),
    )?;
    std::fs::write(
        output.join("assets/diagram.png"),
        include_bytes!("../public/diagram.png"),
    )?;
    let mut assets = vec!["assets/site.css", "assets/diagram.png"];
    if island {
        for (name, bytes) in [
            (
                "counter.mjs",
                include_bytes!("../frontend/counter.mjs").as_slice(),
            ),
            (
                "wasm-startup.mjs",
                include_bytes!("../frontend/wasm-startup.mjs").as_slice(),
            ),
        ] {
            std::fs::write(output.join("assets").join(name), bytes)?;
        }
        for name in ["static_island.js", "static_island_bg.wasm"] {
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("frontend/generated")
                    .join(name),
                output.join("assets").join(name),
            )?;
        }
        assets.extend([
            "assets/counter.mjs",
            "assets/wasm-startup.mjs",
            "assets/static_island.js",
            "assets/static_island_bg.wasm",
        ]);
    }
    let mut pages = Vec::new();
    for (route, name, title, guide, interactive) in [
        ("/", "home", "Static home <Yetlet>", false, island),
        (
            "/guide/install/",
            "install",
            "Installation guide",
            true,
            false,
        ),
        ("/404.html", "not-found", "Page not found", false, false),
    ] {
        let fragment = format!("fragments/{name}.html");
        std::fs::write(
            output.join(&fragment),
            render_to_string(&page(&base, title, guide, interactive))?,
        )?;
        pages.push(json!({"route":route,"title":title,"language":"en","fragment":fragment,"stylesheets":["assets/site.css"],"island_entry":if interactive{Some("assets/counter.mjs")}else{None}}));
    }
    std::fs::write(
        output.join("export.json"),
        serde_json::to_vec_pretty(&json!({"schema_version":1,"pages":pages,"assets":assets}))?,
    )?;
    Ok(())
}
