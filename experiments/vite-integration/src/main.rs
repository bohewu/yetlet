mod manifest;

use axum::{
    Router,
    extract::State,
    http::{HeaderValue, StatusCode, Uri, header},
    response::{Html, IntoResponse, Response},
};
use manifest::Snapshot;
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
    let proof = if profile == "tailwind" {
        Element::new("p")
            .attribute("id", "tailwind-proof")
            .attribute("class", "bg-blue-100 p-4 font-bold")
            .child(View::text("Tailwind utilities scanned from Rust source"))
    } else {
        Element::new("p")
            .attribute("id", "plain-proof")
            .child(View::text("Plain Vite CSS from a TypeScript entry"))
    };
    let view = Element::new("main")
        .child(Element::new("h1").child(View::text("Yetlet Vite feasibility")))
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
        )
        .child(
            Element::new("img")
                .attribute("id", "local-mark")
                .attribute("src", format!("/{profile}/{}", snapshot.svg))
                .attribute("alt", "Locally served check mark")
                .attribute("width", "80")
                .attribute("height", "48"),
        )
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
    let metadata = DocumentMetadata::with_language(
        format!("Vite spike | {profile}"),
        LanguageTag::new("en").map_err(|_| "invalid document language")?,
    );
    let mut html = render_document_with_metadata(&metadata, &view.into())
        .map_err(|_| "SSR rendering failed")?;
    // URLs contain only the validated ASCII path alphabet, never raw manifest HTML.
    let mut tags = String::new();
    for path in &snapshot.styles {
        tags.push_str(&format!(
            "<link rel=\"stylesheet\" href=\"/{profile}/{path}\">"
        ));
    }
    tags.push_str(&format!(
        "<link rel=\"icon\" href=\"/{profile}/{}\">",
        snapshot.svg
    ));
    tags.push_str(&format!(
        "<script type=\"module\" src=\"/{profile}/{}\"></script>",
        snapshot.entry
    ));
    html = html.replacen("</head>", &format!("{tags}</head>"), 1);
    Ok(html)
}

async fn request(State(profiles): State<Arc<BTreeMap<String, Profile>>>, uri: Uri) -> Response {
    let path = uri.path();
    let mut response = (StatusCode::NOT_FOUND, "Not found").into_response();
    if uri.query().is_none() {
        for (name, profile) in profiles.iter() {
            if path == format!("/{name}") {
                response = Html(profile.html.clone()).into_response();
                break;
            }
            if let Some(asset) = path
                .strip_prefix(&format!("/{name}/"))
                .and_then(|path| profile.snapshot.files.get(path))
            {
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
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
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
    if args.len() != 4 || args[0] != "--dist" || args[2] != "--port" {
        return Err("usage: yetlet-vite-spike --dist <built-dist-directory> --port <1-65535>");
    }
    let port = args[3]
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or("invalid loopback port")?;
    let root = PathBuf::from(&args[1]);
    let mut profiles = BTreeMap::new();
    for name in ["plain", "tailwind"] {
        let snapshot = Snapshot::load(&root.join(name), &format!("front/{name}.ts"))?;
        let html = document(name, &snapshot)?;
        profiles.insert(name.to_string(), Profile { snapshot, html });
    }
    let app = Router::new()
        .fallback(request)
        .with_state(Arc::new(profiles));
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .await
        .map_err(|_| "cannot bind selected loopback port")?;
    println!("VITE_SPIKE_READY http://127.0.0.1:{port}");
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
            eprintln!("VITE-SPIKE: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}
