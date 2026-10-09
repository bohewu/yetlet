//! M6c uses ordinary typed setup functions, not a plugin registry or DI.
use axum::{
    Router,
    body::{Body, to_bytes},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tower::ServiceExt;
use yetlet_server_axum::{
    App,
    extract::Extension,
    http::{Request, StatusCode},
};

#[derive(Clone)]
struct Enabled(&'static str);
async fn selected(value: Option<Extension<Enabled>>) -> &'static str {
    value.map_or("absent", |Extension(Enabled(label))| label)
}
async fn label(app: Router, path: &str) -> String {
    let response = app
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    String::from_utf8(to_bytes(response.into_body(), 1024).await.unwrap().to_vec()).unwrap()
}

#[tokio::test]
async fn configuration_executes_once_and_capability_data_is_scoped() {
    let calls = Arc::new(AtomicUsize::new(0));
    let setup_calls = Arc::clone(&calls);
    let app: Router = App::new()
        .get("/plain", selected)
        .group("/selected", |group| {
            group.get("/item", selected).configure(move |group| {
                setup_calls.fetch_add(1, Ordering::Relaxed);
                group.route_layer(Extension(Enabled("configured")))
            })
        })
        .into_router();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    for _ in 0..3 {
        assert_eq!(label(app.clone(), "/selected/item").await, "configured");
        assert_eq!(label(app.clone(), "/plain").await, "absent");
    }
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[derive(Debug, PartialEq, Eq)]
struct BadConfiguration;

#[test]
fn setup_errors_propagate_without_serving_a_partly_configured_application() {
    let result = App::<()>::new().try_configure(|_| Err::<App, _>(BadConfiguration));
    assert!(matches!(result, Err(BadConfiguration)));
    let configured =
        App::<()>::new().try_configure(|app| Ok::<_, BadConfiguration>(app.get("/", selected)));
    assert!(configured.is_ok());
}

#[tokio::test]
async fn no_configuration_means_no_capability_or_optional_headers() {
    let app: Router = App::new().get("/", selected).into_router();
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(!response.headers().contains_key("x-yetlet-request-id"));
    assert_eq!(
        to_bytes(response.into_body(), 1024).await.unwrap(),
        "absent"
    );
}
