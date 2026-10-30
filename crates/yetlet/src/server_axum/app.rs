//! Minimal handler-first application composition over Axum Router.
//!
//! Axum owns routing, request extraction, responses, state and Tower execution.
//! No MVC controller hierarchy, model binder, or parallel router is created.

use crate::server_axum::Page;
use crate::view::View;
use std::{borrow::Cow, convert::Infallible, future::Future};

use axum::{
    Router,
    extract::Request,
    handler::Handler,
    response::IntoResponse,
    routing::{MethodRouter, Route, get, post},
};
use tower::{Layer, Service};

/// Thin application facade.
///
/// `S` is Axum's missing-state type, not a dependency-injection container.
/// Register routes before calling `with_state`. Call `into_router` when ordinary
/// Axum APIs, `axum::serve` or pre-routing global middleware are needed.
///
/// # Example
///
/// Ordinary handler functions return native Axum results, without Controllers.
///
/// ```rust
/// use axum::{Json, Router, response::Response};
/// use yetlet::view::View;
/// use yetlet::server_axum::{App, HtmlPageError, render_page};
///
/// async fn home() -> Result<Response, HtmlPageError> {
///     render_page("Home", &View::text("Hello"))
/// }
/// async fn ping() -> Json<&'static str> {
///     Json("ready")
/// }
///
/// let router: Router = App::new()
///     .get("/", home)
///     .group("/api", |api| api.get("/ping", ping))
///     .into_router();
/// ```
pub struct App<S = ()> {
    router: Router<S>,
}

impl<S> Default for App<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> App<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Start an empty application; unmatched requests return Axum's 404.
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    /// Keep an existing native Axum Router without reconstructing routes.
    pub fn from_router(router: Router<S>) -> Self {
        Self { router }
    }

    /// Return the underlying Router without introducing another HTTP engine.
    pub fn into_router(self) -> Router<S> {
        self.router
    }

    /// Escape hatch for Router APIs such as fallback, nest_service and merge.
    pub fn map_router(self, configure: impl FnOnce(Router<S>) -> Router<S>) -> Self {
        Self {
            router: configure(self.router),
        }
    }

    /// Compose an explicitly selected module at setup, not on each request.
    ///
    /// A module is an ordinary Rust function/closure `App<S> -> App<S>`. It can
    /// install native Tower layers, routes and typed extensions. No scanning,
    /// global registry, implicit initialization or capability field is added.
    pub fn configure(self, configure: impl FnOnce(Self) -> Self) -> Self {
        configure(self)
    }

    /// Fallible configuration: return the module's typed setup error unchanged.
    /// Nothing is served until the caller obtains a successfully built Router.
    pub fn try_configure<E>(
        self,
        configure: impl FnOnce(Self) -> Result<Self, E>,
    ) -> Result<Self, E> {
        configure(self)
    }

    /// Register a no-argument async View-producing page with an explicit title.
    ///
    /// For handlers needing extractors or application errors, use get() with
    /// a Page or Result<Page, E> return value. Native Axum binding is retained.
    ///
    /// ```
    /// use yetlet::view::View;
    /// use yetlet::server_axum::App;
    /// async fn home() -> View { View::text("Hello") }
    /// let app: axum::Router = App::new().page("/", "Home", home).into_router();
    /// ```
    pub fn page<H, F>(self, path: &str, title: impl Into<Cow<'static, str>>, handler: H) -> Self
    where
        H: Fn() -> F + Clone + Send + Sync + 'static,
        F: Future<Output = View> + Send + 'static,
    {
        let title = title.into();
        self.get(path, move || {
            let title = title.clone();
            let view = handler();
            async move { Page::new(title, view.await) }
        })
    }

    /// Accept a native MethodRouter with its own method-level middleware.
    pub fn route(self, path: &str, methods: MethodRouter<S>) -> Self {
        Self {
            router: self.router.route(path, methods),
        }
    }

    /// Register an ordinary Rust GET handler with native Axum extractors.
    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.route(path, get(handler))
    }

    /// Register an ordinary Rust POST handler.
    pub fn post<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.route(path, post(handler))
    }

    /// Nest a group using Axum Router::nest; prefix must not be root.
    ///
    /// The callback registers routes and optional group-scoped middleware.
    pub fn group(self, prefix: &str, configure: impl FnOnce(Self) -> Self) -> Self {
        let child = configure(Self::new());
        Self {
            router: self.router.nest(prefix, child.router),
        }
    }

    /// Apply a native Tower layer only to already registered routes.
    ///
    /// Unmatched 404 fallbacks are not covered. Register group routes before
    /// this method. For pre-routing global middleware, call
    /// middleware::with_global(app.into_router(), layer) after construction.
    pub fn route_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        Self {
            router: self.router.route_layer(layer),
        }
    }

    /// Supply explicit application state after registering handlers, as in Axum.
    pub fn with_state<S2>(self, state: S) -> App<S2>
    where
        S2: Clone + Send + Sync + 'static,
    {
        App {
            router: self.router.with_state(state),
        }
    }
}
