//! Public synthetic comparison fixture; no application data or optional providers.
use axum::{
    Json, Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use yetlet_core::{Element, View};
use yetlet_server_axum::{App, DocumentMetadata, LanguageTag, Page};

const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

async fn plaintext() -> &'static str {
    "Hello, World!"
}

async fn json() -> Json<Message> {
    Json(Message {
        message: "Hello, World!",
    })
}

struct Fixture {
    number: String,
    title: String,
    attribute: String,
    heading: String,
    paragraph: String,
}

fn fixture(id: u8) -> Result<Fixture, StatusCode> {
    if !(1..=20).contains(&id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let number = format!("{id:02}");
    Ok(Fixture {
        title: format!("YT143 {number} <&> \"quoted\" 'title' café"),
        attribute: format!("Item {number} <&> \"quoted\" 'attribute' café"),
        heading: format!("Item {number} <&> \"quoted\" 'text' café"),
        paragraph: format!("Dynamic request {number}"),
        number,
    })
}

// Independent native baseline: fixed structure and explicit dynamic escaping.
fn escape(input: &str) -> String {
    let mut output = String::new();
    for character in input.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '\"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
    output
}

async fn axum_ssr(Path(id): Path<u8>) -> Result<Response, StatusCode> {
    let fixture = fixture(id)?;
    let mut body = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>",
    );
    body.push_str(&escape(&fixture.title));
    body.push_str("</title></head><body><main data-id=\"");
    body.push_str(&escape(&fixture.number));
    body.push_str("\" title=\"");
    body.push_str(&escape(&fixture.attribute));
    body.push_str("\"><h1>");
    body.push_str(&escape(&fixture.heading));
    body.push_str("</h1><p>");
    body.push_str(&escape(&fixture.paragraph));
    body.push_str("</p></main></body></html>");
    Ok(([(header::CONTENT_SECURITY_POLICY, CSP)], Html(body)).into_response())
}

async fn yetlet_ssr(Path(id): Path<u8>) -> Result<Page, StatusCode> {
    let fixture = fixture(id)?;
    let view = Element::new("main")
        .attribute("data-id", fixture.number)
        .attribute("title", fixture.attribute)
        .child(Element::new("h1").child(View::text(fixture.heading)))
        .child(Element::new("p").child(View::text(fixture.paragraph)));
    Ok(Page::with_metadata(
        DocumentMetadata::with_language(
            fixture.title,
            LanguageTag::new("en").expect("fixed valid language"),
        ),
        view,
    ))
}

fn router(implementation: &str) -> Result<Router, &'static str> {
    match implementation {
        "axum" => Ok(Router::new()
            .route("/plaintext", get(plaintext))
            .route("/json", get(json))
            .route("/ssr/{id}", get(axum_ssr))),
        "yetlet" => Ok(App::new()
            .get("/plaintext", plaintext)
            .get("/json", json)
            .get("/ssr/{id}", yetlet_ssr)
            .into_router()),
        _ => Err("implementation must be axum or yetlet"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let implementation = args.next().ok_or("expected implementation and port")?;
    let port: u16 = args.next().ok_or("expected port")?.parse()?;
    if args.next().is_some() || port < 1024 {
        return Err("expected implementation and high loopback port".into());
    }
    let app = router(&implementation)?;
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?
        .block_on(async move {
            let listener =
                tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
            println!("listening {implementation} 127.0.0.1:{port}");
            let mut terminate =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    tokio::select! {
                        _ = terminate.recv() => {},
                        _ = tokio::signal::ctrl_c() => {},
                    }
                })
                .await?;
            println!("stopped");
            Ok::<(), Box<dyn std::error::Error>>(())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    async fn response(mode: &str, path: &str, expected: &str, content_type: &str, csp: bool) {
        let response = router(mode)
            .unwrap()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(headers[header::CONTENT_TYPE], content_type);
        assert_eq!(headers[header::CONTENT_LENGTH], expected.len().to_string());
        assert_eq!(
            headers
                .get(header::CONTENT_SECURITY_POLICY)
                .map(|v| v.to_str().unwrap()),
            csp.then_some(CSP)
        );
        for forbidden in [header::CONTENT_ENCODING, header::SET_COOKIE, header::SERVER] {
            assert!(!headers.contains_key(forbidden));
        }
        assert_eq!(
            to_bytes(response.into_body(), 4096).await.unwrap().as_ref(),
            expected.as_bytes()
        );
    }

    #[tokio::test]
    async fn plaintext_and_fresh_json_have_exact_native_responses() {
        for mode in ["axum", "yetlet"] {
            response(
                mode,
                "/plaintext",
                "Hello, World!",
                "text/plain; charset=utf-8",
                false,
            )
            .await;
            response(
                mode,
                "/json",
                r#"{"message":"Hello, World!"}"#,
                "application/json",
                false,
            )
            .await;
        }
    }

    #[tokio::test]
    async fn all_dynamic_ssr_fixtures_escape_title_text_and_attributes() {
        // Literal independent oracle from the reviewed contract, never a renderer call.
        const FIRST: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>YT143 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;title&#39; café</title></head><body><main data-id="01" title="Item 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;attribute&#39; café"><h1>Item 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;text&#39; café</h1><p>Dynamic request 01</p></main></body></html>"#;
        for id in 1..=20 {
            let expected = FIRST.replace("01", &format!("{id:02}"));
            assert_eq!(expected.len(), 428);
            if id != 1 {
                assert_ne!(expected, FIRST);
            }
            for mode in ["axum", "yetlet"] {
                response(
                    mode,
                    &format!("/ssr/{id}"),
                    &expected,
                    "text/html; charset=utf-8",
                    true,
                )
                .await;
            }
        }
    }
}
