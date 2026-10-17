mod assets;
mod standalone;
mod vite;

use assets::Snapshot;
use axum::{
    Router,
    extract::State,
    http::{HeaderValue, StatusCode, Uri, header},
    response::{Html, IntoResponse, Response},
};
use std::{collections::BTreeMap, net::Ipv4Addr, path::PathBuf, sync::Arc};
use yetlet_core::{Element, View};
use yetlet_server_axum::{
    DEFAULT_CSP, DocumentMetadata, LanguageTag, render_document_with_metadata,
};

struct Profile {
    snapshot: Snapshot,
    html: String,
}

fn document(profile: &str, snapshot: &Snapshot) -> Result<String, &'static str> {
    let proof = if profile != "plain" {
        Element::new("p")
            .attribute("id", "tailwind-proof")
            .attribute("class", "bg-blue-100 p-4 font-bold")
            .child(View::text("Tailwind utilities scanned from Rust source"))
    } else {
        Element::new("p")
            .attribute("id", "plain-proof")
            .child(View::text("Plain Vite CSS from a TypeScript entry"))
    };
    let mut view = Element::new("main")
        .child(Element::new("h1").child(View::text("Yetlet asset contract")))
        .child(
            Element::new("p")
                .attribute("id", "ssr-proof")
                .child(View::text(
                    "Rust SSR survives without JavaScript: <safe & useful>",
                )),
        )
        .child(proof)
        .child(
            Element::new("p")
                .attribute("id", "shared-proof")
                .child(View::text(
                    "Shared dependency CSS is present before interaction.",
                )),
        );
    if let Some(svg) = snapshot.images().first() {
        view = view.child(
            Element::new("img")
                .attribute("id", "local-mark")
                .attribute("src", format!("/{profile}/{svg}"))
                .attribute("alt", "Local check mark")
                .attribute("width", "80")
                .attribute("height", "48"),
        );
    }
    if profile != "standalone" {
        view = view
            .child(
                Element::new("button")
                    .attribute("type", "button")
                    .attribute("id", "load-lazy")
                    .child(View::text("Load lazy module")),
            )
            .child(
                Element::new("p")
                    .attribute("id", "lazy-result")
                    .attribute("aria-live", "polite")
                    .child(View::text("Lazy module has not loaded.")),
            );
    }
    let metadata = DocumentMetadata::with_language(
        format!("Asset contract | {profile}"),
        LanguageTag::new("en").map_err(|_| "invalid document language")?,
    );
    let mut html = render_document_with_metadata(&metadata, &view.into())
        .map_err(|_| "SSR rendering failed")?;
    let tags = snapshot.links(profile)?;
    html = html.replacen("</head>", &format!("{tags}</head>"), 1);
    Ok(html)
}

async fn request(State(profiles): State<Arc<BTreeMap<String, Profile>>>, uri: Uri) -> Response {
    let path = uri.path();
    let mut cache = "no-store";
    let mut response = (StatusCode::NOT_FOUND, "Not found").into_response();
    if uri.query().is_none() {
        for (name, profile) in profiles.iter() {
            if path == format!("/{name}") {
                response = Html(profile.html.clone()).into_response();
                break;
            }
            if let Some(asset) = path
                .strip_prefix(&format!("/{name}/"))
                .and_then(|path| profile.snapshot.asset(path))
            {
                cache = profile.snapshot.cache_control();
                response =
                    ([(header::CONTENT_TYPE, asset.mime)], asset.bytes.clone()).into_response();
                break;
            }
        }
    }
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(DEFAULT_CSP),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}

async fn shutdown() {
    #[cfg(unix)]
    {
        let Ok(mut term) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        else {
            return;
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

async fn run() -> Result<(), &'static str> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 || args[0] != "--dist" || args[2] != "--standalone" || args[4] != "--port" {
        return Err(
            "usage: yetlet-asset-contract --dist <absolute-vite-root> --standalone <absolute-css-root> --port <1-65535>",
        );
    }
    let port = args[5]
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or("invalid loopback port")?;
    let root = PathBuf::from(&args[1]);
    let mut profiles = BTreeMap::new();
    for name in ["plain", "tailwind"] {
        let snapshot = vite::load(&root.join(name), &format!("front/{name}.ts"))?;
        let html = document(name, &snapshot)?;
        profiles.insert(name.to_string(), Profile { snapshot, html });
    }
    let snapshot = standalone::load(&PathBuf::from(&args[3]))?;
    let html = document("standalone", &snapshot)?;
    profiles.insert("standalone".into(), Profile { snapshot, html });
    let app = Router::new()
        .fallback(request)
        .with_state(Arc::new(profiles));
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .await
        .map_err(|_| "cannot bind selected loopback port")?;
    println!("ASSET_CONTRACT_READY http://127.0.0.1:{port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .map_err(|_| "HTTP server failed")
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("YT-098: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
