use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Request,
    http::{StatusCode, header, request::Parts},
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::get,
};
use tower::ServiceExt;
use yetlet_auth::{
    Authenticated, Authentication, InvalidAuthenticationInput, MAX_AUTHENTICATION_NAME_BYTES,
    Principal, ResolveFuture, authenticate,
};

fn resolve<'a>(_parts: &'a mut Parts, name: &'a &'static str) -> ResolveFuture<'a> {
    Box::pin(async move {
        Ok(Some(
            Principal::from_verified("urn:test:accounts", "subject", name, None).unwrap(),
        ))
    })
}

#[test]
fn authentication_name_preserves_verified_input_and_redacts_identity() {
    let construct: fn(
        &str,
        &str,
        &str,
        Option<&str>,
    ) -> Result<Principal, InvalidAuthenticationInput> = Principal::from_verified;
    let principal = construct(
        " issuer ",
        " subject ",
        " App registration ",
        Some(" tenant "),
    )
    .unwrap();
    assert_eq!(MAX_AUTHENTICATION_NAME_BYTES, 64);
    assert_eq!(principal.authentication_name(), " App registration ");
    assert_eq!(principal.issuer(), " issuer ");
    assert_eq!(principal.subject(), " subject ");
    assert_eq!(principal.tenant(), Some(" tenant "));
    let other = construct(" issuer ", " subject ", "other", None).unwrap();
    assert!(principal.same_identity(&other));
    assert_eq!(
        format!("{principal:?}"),
        "Principal { identity: [REDACTED] }"
    );

    let configuration: Result<Authentication<&'static str>, InvalidAuthenticationInput> =
        Authentication::new(" App registration ", " App registration ", resolve, || {
            StatusCode::FORBIDDEN.into_response()
        });
    assert!(configuration.is_ok());
}

#[test]
fn construction_retains_shape_bounds_and_redacted_errors() {
    let maximum = "a".repeat(MAX_AUTHENTICATION_NAME_BYTES);
    assert!(Principal::from_verified("issuer", "subject", &maximum, None).is_ok());
    assert!(
        Authentication::new(&maximum, "name", resolve, || {
            StatusCode::FORBIDDEN.into_response()
        })
        .is_ok()
    );
    for invalid in [
        String::new(),
        " ".into(),
        "rejected\n".into(),
        "rejected\0".into(),
        "a".repeat(MAX_AUTHENTICATION_NAME_BYTES + 1),
        "é".repeat(MAX_AUTHENTICATION_NAME_BYTES / 2 + 1),
    ] {
        let principal: Result<Principal, InvalidAuthenticationInput> =
            Principal::from_verified("issuer", "subject", &invalid, None);
        let configuration: Result<Authentication<&'static str>, InvalidAuthenticationInput> =
            Authentication::new(&invalid, "name", resolve, || {
                StatusCode::FORBIDDEN.into_response()
            });
        let Err(InvalidAuthenticationInput) = principal else {
            panic!("invalid authentication name accepted by Principal");
        };
        let Err(InvalidAuthenticationInput) = configuration else {
            panic!("invalid authentication name accepted by Authentication");
        };
        assert_eq!(
            InvalidAuthenticationInput.to_string(),
            "invalid principal or authentication scheme"
        );
    }
    for values in [
        ("", "subject", None),
        ("issuer", "", None),
        ("issuer", "subject", Some("")),
    ] {
        let error = Principal::from_verified(values.0, values.1, "name", values.2).unwrap_err();
        assert_eq!(error, InvalidAuthenticationInput);
    }
}

#[tokio::test]
async fn configured_authentication_name_matching_remains_exact_and_fail_closed() {
    for (resolved_name, status, body) in [
        (" App ", StatusCode::OK, " App "),
        (
            "App",
            StatusCode::INTERNAL_SERVER_ERROR,
            "Authentication configuration error",
        ),
        (
            " app ",
            StatusCode::INTERNAL_SERVER_ERROR,
            "Authentication configuration error",
        ),
    ] {
        let authentication = Authentication::new(" App ", resolved_name, resolve, || {
            StatusCode::FORBIDDEN.into_response()
        })
        .unwrap();
        let app = Router::new()
            .route(
                "/",
                get(|identity: Authenticated| async move {
                    identity.principal().authentication_name().to_owned()
                }),
            )
            .route_layer(from_fn_with_state(
                authentication,
                authenticate::<&'static str>,
            ));
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(to_bytes(response.into_body(), 128).await.unwrap(), body);
    }
}
