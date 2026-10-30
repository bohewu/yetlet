//! YT-156: actual before/current products with identical fresh HTTP SSR work.
#[cfg(not(feature = "ssr-context-study"))]
use axum::{Router, extract::Path, http::StatusCode};
use yetlet::view::{Element, View};
#[cfg(not(feature = "ssr-context-study"))]
use yetlet::server_axum::App;
use yetlet::server_axum::{DocumentMetadata, LanguageTag, Page};

#[cfg(all(feature = "ssr-stage-probe", feature = "ssr-context-study"))]
compile_error!("select only one SSR diagnostic feature");

#[cfg(feature = "ssr-context-study")]
#[path = "ssr/context_study.rs"]
mod context_study;

#[cfg(feature = "ssr-stage-probe")]
#[path = "ssr/stage_probe.rs"]
mod stage_probe;

#[cfg(not(feature = "ssr-context-study"))]
const CASES: [(&str, usize); 3] = [("small", 8), ("medium", 128), ("large", 1024)];
const TITLE: &str = "YT145 public report <&> \"quoted\" 'café'";
#[cfg(any(test, feature = "ssr-context-study"))]
const PREFIX: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>YT145 public report &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;</title></head><body>";
#[cfg(any(test, feature = "ssr-context-study"))]
const BODY_START: &str = "<main id=\"ssr-report\"><header><h1>SSR report &lt;&amp;&gt; &quot;title&quot; &#39;café&#39;</h1><p>Public synthetic rows</p></header><section aria-label=\"Records\"><ul>";
#[cfg(any(test, feature = "ssr-context-study"))]
const BODY_END: &str = "</ul></section><footer><p>End of public report</p></footer></main>";

fn row(index: usize) -> View {
    Element::new("li")
        .attribute("data-row", format!("{index:04}"))
        .attribute("title", format!("Row {index:04} <&> \"quoted\" 'café'"))
        .child(Element::new("h2").child(View::text(format!("Record {index:04}"))))
        .child(Element::new("p").child(View::text(format!(
            "Public <&> \"value\" 'café' {index:04}"
        ))))
        .child(
            Element::new("a")
                .attribute("href", format!("/records/{index:04}?view=full&public=1"))
                .child(View::text("Open record")),
        )
        .into()
}

fn page(rows: usize) -> (DocumentMetadata, View) {
    let view = Element::new("main")
        .attribute("id", "ssr-report")
        .child(
            Element::new("header")
                .child(Element::new("h1").child(View::text("SSR report <&> \"title\" 'café'")))
                .child(Element::new("p").child(View::text("Public synthetic rows"))),
        )
        .child(
            Element::new("section")
                .attribute("aria-label", "Records")
                .child(Element::new("ul").children((0..rows).map(row))),
        )
        .child(
            Element::new("footer")
                .child(Element::new("p").child(View::text("End of public report"))),
        )
        .into();
    (
        DocumentMetadata::with_language(TITLE, LanguageTag::new("en").unwrap()),
        view,
    )
}

// Independent already-escaped literal oracle, not a second general serializer.
#[cfg(any(test, feature = "ssr-context-study"))]
fn expected(rows: usize) -> (String, String) {
    let mut body = BODY_START.to_owned();
    for i in 0..rows {
        body.push_str(&format!("<li data-row=\"{i:04}\" title=\"Row {i:04} &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;\"><h2>Record {i:04}</h2><p>Public &lt;&amp;&gt; &quot;value&quot; &#39;café&#39; {i:04}</p><a href=\"/records/{i:04}?view=full&amp;public=1\">Open record</a></li>"));
    }
    body.push_str(BODY_END);
    let document = format!("{PREFIX}{body}</body></html>");
    (body, document)
}

#[cfg(not(any(feature = "ssr-stage-probe", feature = "ssr-context-study")))]
async fn ssr(Path(size): Path<String>) -> Result<Page, StatusCode> {
    let rows = CASES
        .iter()
        .find(|(name, _)| *name == size)
        .map(|(_, rows)| *rows)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let (metadata, view) = page(rows);
    Ok(Page::with_metadata(metadata, view))
}

#[cfg(not(any(feature = "ssr-stage-probe", feature = "ssr-context-study")))]
fn router() -> Router {
    App::new().get("/ssr/{size}", ssr).into_router()
}

#[cfg(not(feature = "ssr-context-study"))]
#[cfg(feature = "ssr-stage-probe")]
fn diagnostic_bind_ipv4(value: Option<&str>) -> Result<std::net::Ipv4Addr, &'static str> {
    let Some(value) = value else {
        return Ok(std::net::Ipv4Addr::LOCALHOST);
    };
    let address: std::net::Ipv4Addr = value.parse().map_err(|_| "invalid YT164_BIND_IPV4")?;
    if address.to_string() == value
        && (address == std::net::Ipv4Addr::LOCALHOST || address.is_private())
    {
        Ok(address)
    } else {
        Err("YT164_BIND_IPV4 requires loopback or private IPv4")
    }
}

#[cfg(not(feature = "ssr-context-study"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let label = args
        .next()
        .ok_or("expected variant and high loopback port")?;
    let port: u16 = args.next().ok_or("expected port")?.parse()?;
    if !matches!(label.as_str(), "reference" | "current") || args.next().is_some() || port < 1024 {
        return Err("expected reference|current and high loopback port".into());
    }
    #[cfg(feature = "ssr-stage-probe")]
    let bind_ip = match std::env::var("YT164_BIND_IPV4") {
        Ok(value) => diagnostic_bind_ipv4(Some(&value))?,
        Err(std::env::VarError::NotPresent) => diagnostic_bind_ipv4(None)?,
        Err(std::env::VarError::NotUnicode(_)) => return Err("invalid YT164_BIND_IPV4".into()),
    };
    #[cfg(not(feature = "ssr-stage-probe"))]
    let bind_ip = std::net::Ipv4Addr::LOCALHOST;
    #[cfg(feature = "ssr-stage-probe")]
    let probe = std::sync::Arc::new(stage_probe::Probe::new());
    #[cfg(feature = "ssr-stage-probe")]
    let app = stage_probe::router(probe.clone());
    #[cfg(not(any(feature = "ssr-stage-probe", feature = "ssr-context-study")))]
    let app = router();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?
        .block_on(async move {
            let listener = tokio::net::TcpListener::bind((bind_ip, port)).await?;
            println!("listening {label} {bind_ip}:{port}");
            let mut terminate =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    tokio::select! { _ = terminate.recv() => {}, _ = tokio::signal::ctrl_c() => {} }
                })
                .await?;
            #[cfg(feature = "ssr-stage-probe")]
            probe.dump();
            println!("stopped");
            Ok::<(), Box<dyn std::error::Error>>(())
        })
}

#[cfg(all(test, feature = "ssr-stage-probe", not(feature = "ssr-context-study")))]
mod private_address_tests {
    use super::diagnostic_bind_ipv4;

    #[test]
    fn yt164_private_address_admission() {
        assert_eq!(diagnostic_bind_ipv4(None).unwrap().to_string(), "127.0.0.1");
        for value in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "172.31.255.254",
            "192.168.0.1",
        ] {
            assert_eq!(
                diagnostic_bind_ipv4(Some(value)).unwrap().to_string(),
                value
            );
        }
        for value in [
            "",
            "0.0.0.0",
            "127.0.0.2",
            "172.15.0.1",
            "172.32.0.1",
            "8.8.8.8",
            "localhost",
            "10.0.0.1:80",
            "192.168.001.1",
            " 10.0.0.1",
            "::1",
        ] {
            assert!(diagnostic_bind_ipv4(Some(value)).is_err(), "{value}");
        }
    }
}

#[cfg(all(
    test,
    not(any(feature = "ssr-stage-probe", feature = "ssr-context-study"))
))]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, header},
    };
    use tower::ServiceExt;
    #[tokio::test]
    async fn yt156_fresh_routes_match_literal_documents_and_security_headers() {
        const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
        for ((name, rows), length) in CASES.into_iter().zip([2383, 31063, 245207]) {
            let expected = expected(rows).1;
            assert_eq!(expected.len(), length);
            let response = router()
                .oneshot(
                    Request::builder()
                        .uri(format!("/ssr/{name}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let headers = response.headers();
            assert_eq!(headers[header::CONTENT_TYPE], "text/html; charset=utf-8");
            assert_eq!(headers[header::CONTENT_LENGTH], expected.len().to_string());
            assert_eq!(headers[header::CONTENT_SECURITY_POLICY], CSP);
            for h in [header::CONTENT_ENCODING, header::SET_COOKIE, header::SERVER] {
                assert!(!headers.contains_key(h));
            }
            assert_eq!(
                to_bytes(response.into_body(), 262144)
                    .await
                    .unwrap()
                    .as_ref(),
                expected.as_bytes()
            );
        }
    }
}

#[cfg(feature = "ssr-context-study")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    context_study::run()
}
