use crate::domain::{Fields, History, ListQuery, Listing, Principal, Snapshot, WorkItem};
use yetlet_core::{Element, View};

pub(crate) fn text(tag: &'static str, value: impl Into<String>) -> Element {
    Element::new(tag).child(View::text(value))
}
pub(crate) fn hidden(name: &'static str, value: impl Into<String>) -> Element {
    Element::new("input")
        .attribute("type", "hidden")
        .attribute("name", name)
        .attribute("value", value)
}
fn link(label: &str, path: &str) -> Element {
    text("a", label).attribute("href", path)
}
fn flag(element: Element, name: &'static str, value: bool) -> Element {
    if value {
        element.attribute(name, "")
    } else {
        element
    }
}
pub(crate) fn shell(
    user: Option<Principal>,
    csrf: Option<&str>,
    title: &str,
    body: Element,
) -> View {
    let mut header = Element::new("header")
        .attribute("class", "board-header")
        .child(link("Lab Operations", "/board").attribute("class", "board-brand"));
    if let Some(user) = user {
        header = header.child(text(
            "span",
            format!(
                "{} / {} / Project {}",
                user.name,
                if user.write { "Operator" } else { "Viewer" },
                user.project
            ),
        ));
        if let Some(csrf) = csrf {
            header = header.child(
                Element::new("form")
                    .attribute("method", "post")
                    .attribute("action", "/board/logout")
                    .child(hidden("csrf", csrf))
                    .child(
                        text("button", "Sign out")
                            .attribute("type", "submit")
                            .attribute("class", "btn btn-outline-secondary btn-sm"),
                    ),
            );
        }
    }
    let mut layout = Element::new("div").attribute(
        "class",
        if user.is_some() {
            "board-layout"
        } else {
            "board-single"
        },
    );
    if let Some(user) = user {
        let root = format!("/board/projects/{}/items", user.project);
        let mut items_link = link("Work items", &root);
        if title != "New work item" && title != "Triage workspace" {
            items_link = items_link.attribute(
                "aria-current",
                if title == "Work items" {
                    "page"
                } else {
                    "location"
                },
            );
        }
        let mut navigation = Element::new("nav")
            .attribute("aria-label", "Project navigation")
            .attribute("class", "board-nav")
            .child(items_link);
        if user.write {
            let mut new_link = link("New work item", &format!("{root}/new"));
            if title == "New work item" {
                new_link = new_link.attribute("aria-current", "page");
            }
            let mut triage = link(
                "Triage workspace",
                &format!("/board/projects/{}/triage", user.project),
            );
            if title == "Triage workspace" {
                triage = triage.attribute("aria-current", "page");
            }
            navigation = navigation.child(new_link).child(triage);
        }
        layout = layout.child(navigation);
    }
    layout = layout.child(
        Element::new("main")
            .attribute("id", "main")
            .attribute("class", "board-main")
            .child(text("h1", title))
            .child(body),
    );
    Element::new("div")
        .child(link("Skip to content","#main").attribute("class","skip-link"))
        .child(header).child(layout)
        .child(text("footer","Local noncritical lab demonstration. Synthetic identities; records persist in SQLite. No remote machine control.").attribute("class","board-footer"))
        .into()
}
pub(crate) fn login(csrf: &str, message: Option<&str>) -> View {
    let mut body = Element::new("section").child(text("p","Use a terminal-only synthetic login code. Alice operates Project 1; Bob operates Project 2; Viewer reads Project 1."));
    if let Some(message) = message {
        body = body.child(
            text("p", message)
                .attribute("role", "alert")
                .attribute("class", "alert alert-danger"),
        );
    }
    let form = Element::new("form")
        .attribute("method", "post")
        .attribute("action", "/board/login")
        .attribute("class", "board-form")
        .child(hidden("csrf", csrf))
        .child(field(
            "Identity",
            "username",
            Element::new("input")
                .attribute("type", "text")
                .attribute("name", "username")
                .attribute("id", "username")
                .attribute("class", "form-control")
                .attribute("required", "")
                .attribute("autocomplete", "username"),
            None,
        ))
        .child(field(
            "Terminal code",
            "code",
            Element::new("input")
                .attribute("type", "password")
                .attribute("name", "code")
                .attribute("id", "code")
                .attribute("class", "form-control")
                .attribute("required", "")
                .attribute("autocomplete", "current-password"),
            None,
        ))
        .child(
            text("button", "Sign in")
                .attribute("type", "submit")
                .attribute("class", "btn btn-primary"),
        );
    shell(None, None, "Sign in", body.child(form))
}
pub(crate) fn failure(message: &str) -> View {
    shell(
        None,
        None,
        "Unable to complete the request",
        Element::new("section")
            .attribute("class", "board-error")
            .child(text("p", message).attribute("role", "alert"))
            .child(link("Return to the board", "/board").attribute("class", "btn btn-primary")),
    )
}
fn field(label: &str, id: &'static str, control: Element, error: Option<&str>) -> Element {
    let mut control = control;
    if error.is_some() {
        control = control
            .attribute("aria-invalid", "true")
            .attribute("aria-describedby", format!("{id}-error"));
    }
    let mut wrapper = Element::new("div")
        .attribute("class", "mb-4")
        .child(
            text("label", label)
                .attribute("for", id)
                .attribute("class", "form-label"),
        )
        .child(control);
    if let Some(message) = error {
        wrapper = wrapper.child(
            text("p", message)
                .attribute("id", format!("{id}-error"))
                .attribute("class", "text-danger mt-2"),
        );
    }
    wrapper
}
fn select(name: &'static str, selected: &str, options: &[(&str, &str)]) -> Element {
    let mut select = Element::new("select")
        .attribute("name", name)
        .attribute("id", name)
        .attribute("class", "form-select");
    if !options.iter().any(|(value, _)| *value == selected) {
        select = select.child(
            text("option", "Choose a valid option")
                .attribute("value", "")
                .attribute("selected", ""),
        );
    }
    for (value, label) in options {
        select = select.child(flag(
            text("option", *label).attribute("value", *value),
            "selected",
            *value == selected,
        ));
    }
    select
}
pub(crate) fn edit(
    user: Principal,
    csrf: &str,
    id: Option<i64>,
    expected: i64,
    value: &Fields,
    message: Option<&str>,
) -> View {
    let root = format!("/board/projects/{}/items", user.project);
    let action = id.map_or_else(|| root.clone(), |id| format!("{root}/{id}"));
    let errors = if message.is_some() {
        value.errors(user.project)
    } else {
        Vec::new()
    };
    let err = |name| {
        errors
            .iter()
            .find(|(field, _)| *field == name)
            .map(|(_, msg)| *msg)
    };
    let mut body = Element::new("section").child(link("Back to work items", &root));
    if let Some(message) = message {
        body = body.child(
            text("p", message)
                .attribute("role", "alert")
                .attribute("class", "alert alert-warning mt-3"),
        );
        if let Some(id) = id {
            body = body.child(link(
                "Open the latest stored version before reapplying this draft",
                &format!("{root}/{id}"),
            ));
        }
    }
    let operator = if user.project == 1 { "alice" } else { "bob" };
    let form = Element::new("form").attribute("method","post").attribute("action",action).attribute("novalidate","").attribute("class","board-form mt-4")
        .child(hidden("csrf",csrf)).child(hidden("expected_version",expected.to_string()))
        .child(field("Title","title",Element::new("input").attribute("type","text").attribute("name","title").attribute("id","title").attribute("value",&value.title).attribute("maxlength","160").attribute("required","").attribute("class","form-control"),err("title")))
        .child(field("Description (plain text)","description",Element::new("textarea").attribute("name","description").attribute("id","description").attribute("rows","6").attribute("maxlength","4000").attribute("class","form-control").child(View::text(format!("\n{}",value.description))),err("description")))
        .child(field("Status","status",select("status",&value.status,&[("open","Open"),("active","Active"),("blocked","Blocked"),("closed","Closed")]),err("status")))
        .child(field("Priority","priority",select("priority",&value.priority,&[("low","Low"),("normal","Normal"),("high","High")]),err("priority")))
        .child(field("Assignee","assignee",select("assignee",&value.assignee,&[("","Unassigned"),(operator,operator)]),err("assignee")))
        .child(text("p","Closing preserves the record and its history. Reopen a closed item before moving it to Active or Blocked.").attribute("class","text-body-secondary"))
        .child(Element::new("div").attribute("class","board-actions")
            .child(text("button",if id.is_some() { "Save changes" } else { "Create work item" }).attribute("type","submit").attribute("class","btn btn-primary"))
            .child(text("button","Reset draft").attribute("type","reset").attribute("class","btn btn-outline-secondary")));
    shell(
        Some(user),
        Some(csrf),
        if id.is_some() {
            "Edit work item"
        } else {
            "New work item"
        },
        body.child(form),
    )
}
fn summary(snapshot: &Snapshot) -> String {
    format!(
        "{} total · {} open · {} active · {} blocked · {} closed",
        snapshot.total, snapshot.open, snapshot.active, snapshot.blocked, snapshot.closed
    )
}
pub(crate) fn listing(user: Principal, csrf: &str, query: &ListQuery, values: &Listing) -> View {
    let root = format!("/board/projects/{}/items", user.project);
    let mut body = Element::new("section")
        .child(text("p", summary(&values.snapshot)).attribute("id", "record-counts"));
    if query.live {
        body = body.child(
            Element::new("section")
                .attribute(
                    "data-board-stream",
                    format!("/board/projects/{}/events", user.project),
                )
                .attribute(
                    "data-initial-revision",
                    values.snapshot.revision.to_string(),
                )
                .child(
                    // Orca treats status-bar text as UI and coalesces adjacent
                    // paragraph events. An explicit generic live child keeps
                    // this message in its live-region event path.
                    Element::new("div").attribute("role", "status").child(
                        text("p", "Stored snapshot shown. Connecting to live status...")
                            .attribute("data-stream-message", "")
                            .attribute("role", "generic")
                            .attribute("aria-live", "polite")
                            .attribute("aria-atomic", "true"),
                    ),
                )
                .child(
                    text("button", "Stop live status")
                        .attribute("type", "button")
                        .attribute("data-stream-stop", "")
                        .attribute("class", "btn btn-outline-secondary btn-sm"),
                )
                .child(
                    text("button", "Reconnect live status")
                        .attribute("type", "button")
                        .attribute("data-stream-retry", "")
                        .attribute("class", "btn btn-outline-secondary btn-sm"),
                )
                .child(link(
                    "Refresh authoritative records",
                    &query.url(user.project, query.page),
                )),
        );
    } else {
        let mut live = query.clone();
        live.live = true;
        body = body.child(link(
            "Enable optional live status",
            &live.url(user.project, query.page),
        ));
    }
    body = body.child(
        Element::new("form")
            .attribute("method", "get")
            .attribute("action", &root)
            .attribute("class", "board-filters my-4")
            .child(hidden("live", query.live.to_string()))
            .child(field(
                "Title contains (case-sensitive)",
                "q",
                Element::new("input")
                    .attribute("type", "search")
                    .attribute("id", "q")
                    .attribute("name", "q")
                    .attribute("value", &query.q)
                    .attribute("maxlength", "100")
                    .attribute("class", "form-control"),
                None,
            ))
            .child(field(
                "Filter status",
                "status",
                select(
                    "status",
                    &query.status,
                    &[
                        ("", "All"),
                        ("open", "Open"),
                        ("active", "Active"),
                        ("blocked", "Blocked"),
                        ("closed", "Closed"),
                    ],
                ),
                None,
            ))
            .child(field(
                "Sort",
                "sort",
                select(
                    "sort",
                    &query.sort,
                    &[
                        ("updated", "Latest update"),
                        ("title", "Title"),
                        ("priority", "Priority"),
                    ],
                ),
                None,
            ))
            .child(
                text("button", "Apply filters")
                    .attribute("type", "submit")
                    .attribute("class", "btn btn-outline-secondary"),
            ),
    );
    if user.write {
        body = body.child(
            link("New work item", &format!("{root}/new"))
                .attribute("class", "btn btn-primary mb-3"),
        );
    }
    if values.items.is_empty() {
        body=body.child(text("p","No work items match this page and filter. Change the filters or create the first item."));
    } else {
        let mut rows = Element::new("tbody");
        for item in &values.items {
            rows = rows.child(
                Element::new("tr")
                    .child(
                        Element::new("th")
                            .attribute("scope", "row")
                            .child(link(&item.fields.title, &format!("{root}/{}", item.id))),
                    )
                    .child(text("td", &item.fields.status))
                    .child(text("td", &item.fields.priority))
                    .child(text(
                        "td",
                        if item.fields.assignee.is_empty() {
                            "Unassigned"
                        } else {
                            &item.fields.assignee
                        },
                    ))
                    .child(text("td", item.version.to_string())),
            );
        }
        let headings = Element::new("tr").children(
            ["Work item", "Status", "Priority", "Assignee", "Version"]
                .map(|v| text("th", v).attribute("scope", "col").into()),
        );
        body = body.child(
            Element::new("div")
                .attribute("class", "table-responsive")
                .attribute("tabindex", "0")
                .attribute("role", "region")
                .attribute(
                    "aria-label",
                    "Work items table; scroll horizontally for more columns",
                )
                .child(
                    Element::new("table")
                        .attribute("class", "table board-records align-middle")
                        .child(text(
                            "caption",
                            format!(
                                "Project {} — {} matching items; 50 per page.",
                                user.project, values.matching
                            ),
                        ))
                        .child(Element::new("thead").child(headings))
                        .child(rows),
                ),
        );
    }
    let mut pages = Element::new("nav")
        .attribute("aria-label", "Work item pages")
        .attribute("class", "board-actions")
        .child(text("span", format!("Page {}", query.page)));
    if query.page > 1 {
        pages = pages.child(link("Previous", &query.url(user.project, query.page - 1)));
    }
    if i64::from(query.page) * 50 < values.matching {
        pages = pages.child(link("Next", &query.url(user.project, query.page + 1)));
    }
    shell(Some(user), Some(csrf), "Work items", body.child(pages))
}
pub(crate) fn detail(user: Principal, csrf: &str, value: &WorkItem, history: &[History]) -> View {
    let root = format!("/board/projects/{}/items", user.project);
    let mut body = Element::new("section")
        .child(link("Back to work items", &root))
        .child(text("h2", &value.fields.title).attribute("class", "mt-4"))
        .child(text(
            "p",
            format!(
                "{} / {} / assignee {} / version {}",
                value.fields.status,
                value.fields.priority,
                if value.fields.assignee.is_empty() {
                    "Unassigned"
                } else {
                    &value.fields.assignee
                },
                value.version
            ),
        ))
        .child(text("p", &value.fields.description).attribute("class", "board-description"))
        .child(
            text(
                "p",
                format!(
                    "Created {} · Updated {} (Unix seconds)",
                    value.created_at, value.updated_at
                ),
            )
            .attribute("class", "text-body-secondary"),
        );
    if user.write {
        body = body.child(
            link("Edit work item", &format!("{root}/{}/edit", value.id))
                .attribute("class", "btn btn-primary"),
        );
    }
    let mut rows = Element::new("tbody");
    for entry in history {
        rows = rows.child(
            Element::new("tr")
                .child(text("th", entry.version.to_string()).attribute("scope", "row"))
                .child(text("td", &entry.actor))
                .child(text("td", &entry.action))
                .child(text("td", &entry.title))
                .child(text("td", &entry.status))
                .child(text("td", entry.occurred_at.to_string())),
        );
    }
    let headings = Element::new("tr").children(
        [
            "Version",
            "Actor",
            "Action",
            "Title at change",
            "Status",
            "Unix seconds",
        ]
        .map(|s| text("th", s).attribute("scope", "col").into()),
    );
    body = body
        .child(text("h2", "History").attribute("class", "mt-5"))
        .child(
            Element::new("div")
                .attribute("class", "table-responsive")
                .attribute("tabindex", "0")
                .attribute("role", "region")
                .attribute(
                    "aria-label",
                    "History table; scroll horizontally for more columns",
                )
                .child(
                    Element::new("table")
                        .attribute("class", "table")
                        .child(text(
                            "caption",
                            "Most recent 100 changes, committed with the work record.",
                        ))
                        .child(Element::new("thead").child(headings))
                        .child(rows),
                ),
        );
    shell(Some(user), Some(csrf), "Work item", body)
}
