use std::collections::HashMap;
use yetlet::{
    html::render_to_string,
    web::{http::header, prelude::*},
};

fn page(message: Option<String>) -> Result<Response, HtmlPageError> {
    let result = match message {
        Some(message) => render_to_string(
            &Element::new("p")
                .attribute("id", "result")
                .child(View::text(format!("You submitted: {message}")))
                .into(),
        )?,
        None => String::new(),
    };
    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Message form</title>
  <link rel="stylesheet" href="/assets/app.css">
  <script src="/assets/app.js" defer></script>
</head>
<body>
  <main>
    <h1>Message form</h1>
    <form method="post" action="/">
      <label for="message">Message (required)</label>
      <input id="message" name="message" required>
      <p id="preview" hidden>Preview: <span id="preview-text"></span></p>
      <button type="submit">Submit message</button>
    </form>
    {result}
  </main>
</body>
</html>"#
    );
    Ok(with_headers("text/html; charset=utf-8", html))
}

async fn home() -> Result<Response, HtmlPageError> {
    page(None)
}

async fn submit(
    Form(mut fields): Form<HashMap<String, String>>,
) -> Result<Response, HtmlPageError> {
    match fields.remove("message") {
        Some(message) if !message.trim().is_empty() => page(Some(message)),
        _ => Ok((StatusCode::BAD_REQUEST, "Enter a message.").into_response()),
    }
}

fn with_headers(content_type: &'static str, body: impl IntoResponse) -> Response {
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_SECURITY_POLICY, DEFAULT_CSP),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

async fn stylesheet() -> Response {
    with_headers("text/css; charset=utf-8", include_str!("app.css"))
}

async fn script() -> Response {
    with_headers("text/javascript; charset=utf-8", include_str!("app.js"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .get("/", home)
        .post("/", submit)
        .get("/assets/app.css", stylesheet)
        .get("/assets/app.js", script)
        .serve()
        .await?;
    Ok(())
}
