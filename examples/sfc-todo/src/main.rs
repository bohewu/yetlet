use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    extract::Request,
    middleware::{Next, from_fn},
};
use serde::Deserialize;
use yetlet::{
    html::render_to_string,
    web::{http::header, prelude::*},
};

mod todo_form {
    include!(concat!(env!("OUT_DIR"), "/sfc-todo-form.rs"));
}

mod todo_item {
    include!(concat!(env!("OUT_DIR"), "/sfc-todo-item.rs"));
}

const MAX_TITLE_UNITS: usize = 120;
const MAX_TASKS: usize = 100;
const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

fn assets() -> impl Iterator<Item = (&'static str, &'static str, &'static str)> {
    [
        (("page.css", include_str!("page.css")), CSS),
        (todo_form::STYLE, CSS),
        (todo_item::STYLE, CSS),
        (todo_form::SCRIPT, JS),
        (todo_item::SCRIPT, JS),
    ]
    .into_iter()
    .filter(|((_, body), _)| !body.trim().is_empty())
    .map(|((name, body), kind)| (name, body, kind))
}

#[derive(Clone)]
struct Board {
    origin: String,
    todos: Arc<Mutex<Todos>>,
}

impl Board {
    fn allows_post(&self, headers: &HeaderMap) -> bool {
        headers
            .get(header::ORIGIN)
            .is_some_and(|origin| origin == self.origin.as_str())
    }
}

#[derive(Default)]
struct Todos {
    next_id: u64,
    tasks: Vec<Todo>,
}

struct Todo {
    id: u64,
    title: String,
    completed: bool,
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Change {
    Complete,
    Reopen,
    Delete,
}

#[derive(Deserialize)]
struct TodoChange {
    change: Change,
}

fn page(
    todos: &Todos,
    draft: &str,
    error: &str,
    status: StatusCode,
) -> Result<Response, HtmlPageError> {
    let form = todo_form::render(&todo_form::Props {
        draft,
        limit: &MAX_TITLE_UNITS.to_string(),
        invalid: if error.is_empty() { "false" } else { "true" },
        error,
    });
    let list = if todos.tasks.is_empty() {
        Element::new("p")
            .attribute("class", "todo-empty")
            .child(View::text("No tasks yet."))
    } else {
        Element::new("ul")
            .attribute("class", "todo-list")
            .attribute("aria-label", "Tasks")
            .children(todos.tasks.iter().map(|todo| {
                let (change, button, state) = if todo.completed {
                    ("reopen", "Reopen", "done")
                } else {
                    ("complete", "Mark done", "open")
                };
                todo_item::render(&todo_item::Props {
                    title: &todo.title,
                    action: &format!("/todos/{}", todo.id),
                    completed: todo.completed,
                    state,
                    change,
                    button,
                })
            }))
    };
    let body = render_to_string(
        &Element::new("main")
            .child(Element::new("h1").child(View::text("Todo")))
            .child(form)
            .child(list)
            .into(),
    )?;
    let head: String = assets()
        .map(|(name, _, kind)| {
            if kind == CSS {
                format!(r#"<link rel="stylesheet" href="/assets/{name}">"#)
            } else {
                format!(r#"<script src="/assets/{name}" defer></script>"#)
            }
        })
        .collect();
    let document = format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Todo · Yetlet SFC</title>
{head}
</head>
<body>{body}</body>
</html>"#
    );
    Ok((
        status,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_SECURITY_POLICY, DEFAULT_CSP),
            (header::CACHE_CONTROL, "no-store"),
        ],
        document,
    )
        .into_response())
}

async fn home(State(board): State<Board>) -> Result<Response, HtmlPageError> {
    page(&board.todos.lock().unwrap(), "", "", StatusCode::OK)
}

async fn add(
    State(board): State<Board>,
    headers: HeaderMap,
    Form(input): Form<NewTodo>,
) -> Result<Response, HtmlPageError> {
    if !board.allows_post(&headers) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    let title = input.title.trim();
    let mut todos = board.todos.lock().unwrap();
    if title.is_empty() || input.title.encode_utf16().count() > MAX_TITLE_UNITS {
        return page(
            &todos,
            &input.title,
            "Enter a nonblank task of at most 120 characters.",
            StatusCode::BAD_REQUEST,
        );
    }
    if todos.tasks.len() == MAX_TASKS {
        return page(
            &todos,
            &input.title,
            "Delete a task before adding another.",
            StatusCode::CONFLICT,
        );
    }
    let id = todos.next_id.checked_add(1).expect("task ID limit");
    todos.next_id = id;
    todos.tasks.push(Todo {
        id,
        title: title.to_owned(),
        completed: false,
    });
    Ok(Redirect::to("/").into_response())
}

async fn change(
    State(board): State<Board>,
    Path(id): Path<u64>,
    headers: HeaderMap,
    Form(input): Form<TodoChange>,
) -> Response {
    if !board.allows_post(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mut todos = board.todos.lock().unwrap();
    let Some(index) = todos.tasks.iter().position(|todo| todo.id == id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match input.change {
        Change::Complete => todos.tasks[index].completed = true,
        Change::Reopen => todos.tasks[index].completed = false,
        Change::Delete => {
            todos.tasks.remove(index);
        }
    }
    Redirect::to("/").into_response()
}

fn app(address: SocketAddr) -> Router {
    let authority = address.to_string();
    let implicit_host = (address.port() == 80).then(|| address.ip().to_string());
    let origin = format!("http://{}", implicit_host.as_deref().unwrap_or(&authority));
    let mut routes = App::new()
        .get("/", home)
        .post("/todos", add)
        .post("/todos/{id}", change);
    for (name, body, mime) in assets() {
        routes = routes.get(&format!("/assets/{name}"), move || async move {
            (
                [
                    (header::CONTENT_TYPE, mime),
                    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                ],
                body,
            )
        });
    }
    routes
        .route_layer(DefaultBodyLimit::max(2048))
        .with_state(Board {
            origin,
            todos: Default::default(),
        })
        .into_router()
        .layer(from_fn(move |request: Request, next: Next| {
            let is_expected =
                |host: &str| host == authority || implicit_host.as_deref() == Some(host);
            let mut hosts = request.headers().get_all(header::HOST).iter();
            let allowed = hosts
                .next()
                .and_then(|host| host.to_str().ok())
                .is_some_and(is_expected)
                && hosts.next().is_none()
                && request
                    .uri()
                    .authority()
                    .is_none_or(|target| is_expected(target.as_str()));
            async move {
                if allowed {
                    next.run(request).await
                } else {
                    StatusCode::FORBIDDEN.into_response()
                }
            }
        }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = match std::env::var("YETLET_SFC_TODO_PORT") {
        Ok(port) => port.parse()?,
        Err(std::env::VarError::NotPresent) => 3012,
        Err(error) => return Err(error.into()),
    };
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await?;
    let address = listener.local_addr()?;
    println!("Yetlet SFC Todo at http://{address}/");
    axum::serve(listener, app(address)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    use super::*;

    const HOST: &str = "127.0.0.1:3012";
    const ORIGIN: &str = "http://127.0.0.1:3012";

    async fn post(router: &Router, path: &str, origin: &str, body: &str) -> Response {
        router
            .clone()
            .oneshot(
                Request::post(path)
                    .header(header::HOST, HOST)
                    .header(header::ORIGIN, origin)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body.to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    async fn read(router: &Router) -> String {
        let response = router
            .clone()
            .oneshot(
                Request::get("/")
                    .header(header::HOST, HOST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        let body = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn adds_completes_reopens_and_deletes_tasks() {
        let router = app(HOST.parse().unwrap());
        let added = post(&router, "/todos", ORIGIN, "title=Read+%3CView%3E+%26+try").await;
        assert_eq!(added.status(), StatusCode::SEE_OTHER);
        assert_eq!(added.headers()[header::LOCATION], "/");
        assert!(read(&router).await.contains("Read &lt;View&gt; &amp; try"));

        for (action, completed) in [("complete", true), ("reopen", false)] {
            let response = post(&router, "/todos/1", ORIGIN, &format!("change={action}")).await;
            assert_eq!(response.status(), StatusCode::SEE_OTHER);
            assert_eq!(read(&router).await.contains(">Completed</span>"), completed);
        }

        let removed = post(&router, "/todos/1", ORIGIN, "change=delete").await;
        assert_eq!(removed.status(), StatusCode::SEE_OTHER);
        assert!(read(&router).await.contains("No tasks yet."));
    }

    #[tokio::test]
    async fn rejects_blank_cross_origin_and_missing_tasks() {
        let router = app(HOST.parse().unwrap());
        for (path, origin, body, expected) in [
            ("/todos", ORIGIN, "title=+++", StatusCode::BAD_REQUEST),
            (
                "/todos",
                "http://example.test",
                "title=Unexpected",
                StatusCode::FORBIDDEN,
            ),
            ("/todos/99", ORIGIN, "change=delete", StatusCode::NOT_FOUND),
        ] {
            assert_eq!(post(&router, path, origin, body).await.status(), expected);
        }
        assert!(read(&router).await.contains("No tasks yet."));
    }

    #[tokio::test]
    async fn requires_the_bound_host_before_serving_the_page_or_assets() {
        let router = app(HOST.parse().unwrap());
        for path in ["/", "/assets/page.css"] {
            for (hosts, expected) in [
                (&[HOST][..], StatusCode::OK),
                (&[][..], StatusCode::FORBIDDEN),
                (&["unexpected.test:3012"][..], StatusCode::FORBIDDEN),
                (&["127.0.0.1:3013"][..], StatusCode::FORBIDDEN),
                (&[HOST, HOST][..], StatusCode::FORBIDDEN),
            ] {
                let mut request = Request::get(path);
                for host in hosts {
                    request = request.header(header::HOST, *host);
                }
                let response = router
                    .clone()
                    .oneshot(request.body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                assert_eq!(response.status(), expected, "{path} {hosts:?}");
            }
        }
    }

    #[tokio::test]
    async fn checks_the_request_target_and_accepts_the_implicit_http_port() {
        let router = app("127.0.0.1:80".parse().unwrap());
        for (target, host, expected) in [
            ("/", "127.0.0.1", StatusCode::OK),
            ("http://127.0.0.1/", "127.0.0.1:80", StatusCode::OK),
            (
                "http://unexpected.test/",
                "127.0.0.1",
                StatusCode::FORBIDDEN,
            ),
        ] {
            let request = Request::get(target)
                .header(header::HOST, host)
                .body(Body::empty())
                .unwrap();
            let response = router.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), expected, "{target} {host}");
        }
    }
}
