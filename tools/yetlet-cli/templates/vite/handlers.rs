struct AppState {
    assets: std::sync::Arc<vite::Snapshot>,
@SETTINGS_FIELD@}

async fn home(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
) -> axum::response::Response {
    state.assets.page(
        "Yetlet",
        Element::new("main")
            .child(Element::new("h1").child(View::text(@MESSAGE@)))
            .child(
                Element::new("p")
                    .attribute("id", "ssr-proof")
                    .child(View::text(
                        "Rust SSR survives without JavaScript: <safe & useful>",
                    )),
            )
            .child(
                Element::new("p")
                    .attribute("id", "@PROOF_ID@")
                    .attribute("class", "bg-blue-100 p-4 font-bold")
                    .child(View::text("Application-owned production assets")),
            )
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
                    .attribute("src", format!("/static/{}", state.assets.images()[0]))
                    .attribute("alt", "Local check mark")
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
            )
            .into(),
    )
}

async fn hello(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> axum::response::Response {
    state.assets.page(
        "Hello",
        Element::new("main")
            .child(View::text(format!("Hello, {name}!")))
            .into(),
    )
}
