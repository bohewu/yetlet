//! Native SSR and immutable, bounded application assets; no physics normal edge.
use crate::model::{Parameters, Scenario, preset};
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use yetlet_core::{Element, View};
use yetlet_server_axum::{
    DEFAULT_CSP, DocumentMetadata, LanguageTag, render_document_with_metadata,
};
const CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const NAMES: [&str; 6] = [
    "loader.js",
    "lab.js",
    "lab_bg.wasm",
    "wasm-startup.mjs",
    "wasm-startup.css",
    "frame-rate.mjs",
];
#[derive(Clone, Default)]
struct Assets {
    digest: String,
    files: BTreeMap<String, Vec<u8>>,
}
impl Assets {
    fn load(root: &Path) -> Self {
        let load = || -> Option<Self> {
            let data = std::fs::read(root.join("manifest.json")).ok()?;
            if data.len() > 256 {
                return None;
            }
            let d: serde_json::Value = serde_json::from_slice(&data).ok()?;
            let digest = d["digest"].as_str()?;
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return None;
            }
            let mut hash = Sha256::new();
            let mut files = BTreeMap::new();
            for name in NAMES {
                let p = root.join(digest).join(name);
                if std::fs::metadata(&p).ok()?.len() > 32 * 1024 * 1024 {
                    return None;
                }
                let data = std::fs::read(p).ok()?;
                hash.update(name.as_bytes());
                hash.update(&data);
                files.insert(name.to_owned(), data);
            }
            if format!("{:x}", hash.finalize()) != digest {
                return None;
            }
            Some(Self {
                digest: digest.to_owned(),
                files,
            })
        };
        load().unwrap_or_default()
    }
}
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Inputs {
    scenario: Option<String>,
    count: Option<u32>,
    gravity: Option<f32>,
    speed: Option<f64>,
}
fn text(tag: &'static str, s: impl Into<String>) -> Element {
    Element::new(tag).child(View::text(s))
}
fn button(id: &'static str, label: &str) -> Element {
    text("button", label)
        .attribute("id", id)
        .attribute("type", "button")
}
fn view(p: Parameters, assets: &Assets) -> View {
    let mut body = Element::new("tbody").attribute("id", "objects");
    for row in preset(p)
        .expect("validated parameters")
        .into_iter()
        .filter(|r| r.id < 1000)
    {
        body = body.child(
            Element::new("tr")
                .attribute("data-object", row.id.to_string())
                .child(text("td", row.id.to_string()))
                .child(text("td", if row.sphere { "Sphere" } else { "Box" }))
                .child(text(
                    "td",
                    format!(
                        "{:.2}, {:.2}, {:.2}",
                        row.center[0], row.center[1], row.center[2]
                    ),
                ))
                .child(text(
                    "td",
                    format!(
                        "{:.2}, {:.2}, {:.2}",
                        row.velocity[0], row.velocity[1], row.velocity[2]
                    ),
                )),
        );
    }
    let mut speeds = Element::new("select")
        .attribute("id", "speed")
        .attribute("name", "speed");
    for value in [0.25, 0.5, 1.0, 2.0] {
        let mut option = text("option", format!("{value}×")).attribute("value", value.to_string());
        if p.speed == value {
            option = option.attribute("selected", "")
        }
        speeds = speeds.child(option);
    }
    let mut scenarios = Element::new("select")
        .attribute("id", "scenario")
        .attribute("name", "scenario");
    for scenario in [
        Scenario::Falling,
        Scenario::Tower,
        Scenario::Dominoes,
        Scenario::Slope,
    ] {
        let mut option = text("option", scenario.label()).attribute("value", scenario.as_str());
        if p.scenario == scenario {
            option = option.attribute("selected", "");
        }
        scenarios = scenarios.child(option);
    }
    let mut slope_guide = text("p", "25° slope. Blue: lower body friction (0.1); amber: higher (0.9). Ramp friction: 0.2. Use at least 4 objects to compare balls and boxes.").attribute("id", "slope-guide");
    if p.scenario != Scenario::Slope {
        slope_guide = slope_guide.attribute("hidden", "");
    }
    let mut select = Element::new("select").attribute("id", "selection");
    for id in 1..=p.count {
        select =
            select.child(text("option", format!("Object {id}")).attribute("value", id.to_string()));
    }
    Element::new("main").attribute("id","physics-lab").attribute("data-scenario",p.scenario.as_str()).attribute("data-count",p.count.to_string()).attribute("data-gravity",p.gravity.to_string()).attribute("data-speed",p.speed.to_string()).attribute("data-state","inert").attribute("data-wasm-startup","").attribute("data-startup-state","idle")
 .child(text("p","YETLET / EXPERIMENTAL LAB").attribute("class","eyebrow"))
 .child(text("h1","Gravity, one step at a time."))
 .child(text("p","A synthetic 3D physics laboratory. Boxes and spheres; meters, seconds and kilograms. Browser simulation is optional and stays in this tab."))
 .child(text("p",if assets.files.is_empty(){"3D files are unavailable. Initial scenario and Reset remain available; ask the person running this server to restore its 3D files."}else{"Press Run to prepare 3D and start the simulation. Pause to inspect; Reset to rebuild the preset."}).attribute("id","status").attribute("role","status"))
 .child(Element::new("noscript").child(text("p","JavaScript is off. Enable it for this site to use Run; the initial scene table and native Reset work without it.")))
 .child(Element::new("details").attribute("id","browser-help")
  .child(text("summary","Browser help"))
  .child(text("p","Your usual browser can open this page. 3D requires JavaScript, WebAssembly, WebGPU and a usable graphics device. Wayland is one verified host setup, not an application requirement."))
  .child(text("p","On the server machine, open http://127.0.0.1:3016/physics-lab. On another machine, forward the server’s loopback port locally or use a separately provided trusted HTTPS deployment. A LAN HTTP address is not a secure-context substitute."))
  .child(text("p","If Run stays disabled, check that JavaScript and site modules can run and that this server has its 3D files. The initial scene and Reset remain useful without 3D."))
  .child(text("p","For WebGPU restrictions, inspect your browser’s graphics diagnostics: about:support in Firefox or chrome://gpu in Chrome. Feature exposure alone does not guarantee a device. Keep browser security and GPU blocklists intact."))
  .child(text("p","Retry is one explicit attempt after a failure; successful recovery stays paused. A disabled Retry may be waiting for cleanup. Reload only if it stays stuck: reload discards this tab’s simulation.")))
 .child(Element::new("form").attribute("id","parameters").attribute("method","get").attribute("action","/physics-lab").attribute("aria-describedby","preset-help")
  .child(text("label","Scene").attribute("for","scenario")).child(scenarios)
  .child(text("label","Dynamic objects (1–64)").attribute("for","count")).child(Element::new("input").attribute("id","count").attribute("name","count").attribute("type","number").attribute("min","1").attribute("max","64").attribute("value",p.count.to_string()))
  .child(text("label","Downward gravity (0–20 m/s²)").attribute("for","gravity")).child(Element::new("input").attribute("id","gravity").attribute("name","gravity").attribute("type","number").attribute("min","0").attribute("max","20").attribute("step","0.01").attribute("value",p.gravity.to_string()))
  .child(text("label","Simulation speed").attribute("for","speed")).child(speeds))
 .child(text("p","Choose a scene, then Reset to apply changes. First Run uses these settings.").attribute("id","preset-help"))
 .child(slope_guide)
 .child(Element::new("section").attribute("class","workspace").attribute("aria-label","Optional 3D laboratory")
  .child(Element::new("div").attribute("class","toolbar")
    .child(button("run","Run").attribute("data-available",if assets.files.is_empty(){"false"}else{"true"}).attribute("disabled",""))
    .child(button("pause","Pause").attribute("disabled",""))
    .child(text("button","Reset").attribute("id","reset").attribute("type","submit").attribute("form","parameters"))
    .child(button("retry","Retry").attribute("hidden","").attribute("disabled","")))
  .child(Element::new("div").attribute("class","scene-surface")
   .child(Element::new("canvas").attribute("id","scene").attribute("width","640").attribute("height","400").attribute("aria-label","3D objects; numeric controls and the HTML table provide an alternative").child(View::text("Initial object data is in the table below.")))
   .child(text("p",format!("{} · FPS (estimate): Not started · Objects: {}",p.scenario.label(),p.count)).attribute("id","render-rate").attribute("class","render-rate").attribute("aria-live","off"))
   .child(Element::new("div").attribute("id","loading").attribute("class","loading-overlay").attribute("hidden","").attribute("aria-hidden","true")
    .child(Element::new("span").attribute("class","loading-spinner wasm-startup-indicator"))))
  .child(text("p","Rotate with pointer drag; scroll to zoom. Pause before selecting an object in the canvas."))
  .child(Element::new("details").attribute("id","advanced").child(text("summary","Advanced controls")).child(button("step","Single Step").attribute("disabled",""))
  .child(Element::new("div").attribute("class","toolbar").child(text("label","Orbit angle").attribute("for","yaw")).child(Element::new("input").attribute("id","yaw").attribute("type","number").attribute("value","45").attribute("min","-180").attribute("max","180"))
   .child(text("label","Elevation").attribute("for","pitch")).child(Element::new("input").attribute("id","pitch").attribute("type","number").attribute("value","55").attribute("min","10").attribute("max","80"))
   .child(text("label","Distance").attribute("for","distance")).child(Element::new("input").attribute("id","distance").attribute("type","number").attribute("value","28").attribute("min","8").attribute("max","70"))
   .child(button("camera-reset","Reset camera")))
  .child(text("p","Ticks: 0 · simulated: 0.000 s · scheduler-discarded: 0.000 s").attribute("id","metrics"))))
 .child(text("h2","Inspect the objects"))
 .child(text("label","Selected object").attribute("for","selection")).child(select)
 .child(text("p","Select an object to inspect its pose and velocity.").attribute("id","detail"))
 .child(Element::new("table").child(text("caption","Initial scenario; live poses replace these values after the first Run."))
  .child(Element::new("thead").child(Element::new("tr").child(text("th","ID")).child(text("th","Shape")).child(text("th","Position (m)")).child(text("th","Velocity (m/s)")))).child(body))
 .child(text("p","Educational prototype, not engineering/safety qualification. No saved simulation, automatic retry or performance guarantee."))
 .into()
}
async fn page(State(a): State<Arc<Assets>>, Query(q): Query<Inputs>) -> Response {
    let d = Parameters::default();
    let Ok(scenario) = q.scenario.as_deref().unwrap_or("falling").parse() else {
        return (StatusCode::BAD_REQUEST, "Invalid bounded preset").into_response();
    };
    let Ok(p) = (Parameters {
        scenario,
        count: q.count.unwrap_or(d.count),
        gravity: q.gravity.unwrap_or(d.gravity),
        speed: q.speed.unwrap_or(d.speed),
    })
    .validate() else {
        return (StatusCode::BAD_REQUEST, "Invalid bounded preset").into_response();
    };
    let meta = DocumentMetadata::with_language(
        "Yetlet | Physics laboratory",
        LanguageTag::new("en").unwrap(),
    );
    let Ok(mut html) = render_document_with_metadata(&meta, &view(p, &a)) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to render laboratory",
        )
            .into_response();
    };
    let script = if a.files.is_empty() {
        String::new()
    } else {
        format!(
            "<link rel=\"stylesheet\" href=\"/physics-assets/{0}/wasm-startup.css\"><script type=\"module\" src=\"/physics-assets/{0}/loader.js\"></script>",
            a.digest
        )
    };
    html = html.replacen(
        "</head>",
        &format!("<link rel=\"stylesheet\" href=\"/physics.css\">{script}</head>"),
        1,
    );
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(if a.files.is_empty() { DEFAULT_CSP } else { CSP }),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}
async fn asset(
    State(a): State<Arc<Assets>>,
    axum::extract::Path((digest, name)): axum::extract::Path<(String, String)>,
) -> Response {
    if digest != a.digest {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(bytes) = a.files.get(&name) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut r = bytes.clone().into_response();
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(if name.ends_with(".wasm") {
            "application/wasm"
        } else if name.ends_with(".css") {
            "text/css; charset=utf-8"
        } else {
            "text/javascript; charset=utf-8"
        }),
    );
    r.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    r.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    r
}
pub fn app(root: PathBuf) -> Router {
    Router::new()
        .route("/physics-lab", get(page))
        .route("/healthz", get(|| async { "ready" }))
        .route(
            "/physics.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../style.css"),
                )
            }),
        )
        .route("/physics-assets/{digest}/{name}", get(asset))
        .with_state(Arc::new(Assets::load(&root)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;
    #[tokio::test]
    async fn yt198_slope_native_selection_and_legend() {
        let response = app(PathBuf::from("/nonexistent-yt198-assets"))
            .oneshot(
                Request::builder()
                    .uri("/physics-lab?scenario=slope&count=4")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let html = String::from_utf8(
            to_bytes(response.into_body(), 128000)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("data-scenario=\"slope\""));
        assert!(
            html.split("<option")
                .any(|s| s.split('>').next().unwrap().contains("value=\"slope\"")
                    && s.split('>').next().unwrap().contains("selected"))
        );
        let legend = html
            .split("<p")
            .find(|s| s.split('>').next().unwrap().contains("id=\"slope-guide\""))
            .unwrap();
        assert!(!legend.split('>').next().unwrap().contains("hidden"));
        assert!(
            legend.contains("25°")
                && legend.contains("0.1")
                && legend.contains("0.9")
                && legend.contains("at least 4")
        );
        assert_eq!(html.matches("data-object=").count(), 4);
        assert!(html.contains("FPS (estimate): Not started") && !html.contains("<script"));
    }
    #[tokio::test]
    async fn yt197_native_scene_selection_and_initial_hud() {
        let app = app(PathBuf::from("/nonexistent-yt197-assets"));
        for scenario in ["tower", "dominoes"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/physics-lab?scenario={scenario}&count=12"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let html = String::from_utf8(
                to_bytes(response.into_body(), 128000)
                    .await
                    .unwrap()
                    .to_vec(),
            )
            .unwrap();
            assert!(html.contains(&format!("data-scenario=\"{scenario}\"")));
            assert!(html.split("<option").any(|s| {
                s.split('>')
                    .next()
                    .unwrap()
                    .contains(&format!("value=\"{scenario}\""))
                    && s.split('>').next().unwrap().contains("selected")
            }));
            assert_eq!(html.matches("data-object=").count(), 12);
            assert!(html.contains("FPS (estimate): Not started") && html.contains("Objects: 12"));
            assert!(html.contains("aria-live=\"off\"") && html.contains("Reset to apply"));
            assert!(!html.contains("<script"));
        }
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/physics-lab?scenario=unknown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    #[tokio::test]
    async fn browser_help_and_loading_shell_survive_without_runtime_assets() {
        let response = app(PathBuf::from("/nonexistent-yt195-assets"))
            .oneshot(
                Request::builder()
                    .uri("/physics-lab")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let html = String::from_utf8(
            to_bytes(response.into_body(), 128000)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(
            html.contains("3D files are unavailable")
                && html.contains("Initial scenario and Reset remain available")
        );
        assert!(html.contains("<noscript>") && html.contains("JavaScript is off"));
        assert!(
            html.contains("id=\"browser-help\"")
                && html.contains("trusted HTTPS")
                && html.contains("about:support")
        );
        let overlay = html
            .split("<div")
            .find(|s| s.split('>').next().unwrap().contains("id=\"loading\""))
            .unwrap();
        let tag = overlay.split('>').next().unwrap();
        assert!(tag.contains("hidden") && tag.contains("aria-hidden=\"true\""));
        assert!(html.contains("loading-spinner") && html.contains("id=\"reset\""));
        assert!(!html.contains("<script"));
    }
    #[tokio::test]
    async fn primary_controls_preserve_native_reset_and_defer_optional_actions() {
        let response = app(PathBuf::from("/nonexistent-yt194-assets"))
            .oneshot(
                Request::builder()
                    .uri("/physics-lab?count=12")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let html = String::from_utf8(
            to_bytes(response.into_body(), 128000)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(!html.contains("id=\"activate\"") && !html.contains("id=\"dispose\""));
        assert!(html.contains("id=\"run\"") && html.contains("id=\"pause\""));
        let reset = html
            .split("<button")
            .find(|s| s.split('>').next().unwrap().contains("id=\"reset\""))
            .unwrap();
        let reset_tag = reset.split('>').next().unwrap();
        assert!(
            reset_tag.contains("form=\"parameters\"")
                && reset_tag.contains("type=\"submit\"")
                && !reset_tag.contains("disabled")
        );
        let advanced = html
            .split("<details")
            .nth(1)
            .unwrap()
            .split("</details>")
            .next()
            .unwrap();
        assert!(advanced.contains("Advanced controls") && advanced.contains("id=\"step\""));
        assert!(!advanced.split('>').next().unwrap().contains("open"));
        let retry = html
            .split("<button")
            .find(|s| s.split('>').next().unwrap().contains("id=\"retry\""))
            .unwrap();
        assert!(retry.split('>').next().unwrap().contains("hidden"));
        assert_eq!(html.matches("data-object=").count(), 12);
        assert!(!html.contains("<script"));
    }
    #[tokio::test]
    async fn ssr_without_assets_is_meaningful_and_rejects_bad_inputs() {
        let app = app(PathBuf::from("/nonexistent-yt193-assets"));
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/physics-lab")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
        assert!(
            !r.headers()[header::CONTENT_SECURITY_POLICY]
                .to_str()
                .unwrap()
                .contains("unsafe-inline")
        );
        let text =
            String::from_utf8(to_bytes(r.into_body(), 128000).await.unwrap().to_vec()).unwrap();
        assert!(text.contains("Gravity, one step"));
        assert!(text.contains("<canvas"));
        assert!(text.contains("Initial scenario"));
        assert!(!text.contains("<script"));
        for query in ["count=0", "count=65", "gravity=NaN", "speed=3", "unknown=1"] {
            let r = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/physics-lab?{query}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(r.status(), 400, "{query}");
        }
    }
}
