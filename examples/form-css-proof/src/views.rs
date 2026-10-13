use crate::{Draft, Errors};
use yetlet_core::{Element, View};

fn flag(element: Element, name: &'static str, enabled: bool) -> Element {
    if enabled {
        element.attribute(name, "")
    } else {
        element
    }
}
fn text(tag: &'static str, value: &str) -> Element {
    Element::new(tag).child(View::text(value))
}
fn field(
    label: &str,
    id: &'static str,
    control: Element,
    error: Option<&str>,
    help: &str,
) -> Element {
    let control = if error.is_some() {
        control
            .attribute("aria-invalid", "true")
            .attribute("aria-describedby", format!("{id}-help {id}-error"))
    } else {
        control.attribute("aria-describedby", format!("{id}-help"))
    };
    let mut field = Element::new("div")
        .attribute("class", "mb-4")
        .child(
            text("label", label)
                .attribute("for", id)
                .attribute("class", "form-label"),
        )
        .child(control)
        .child(
            text("p", help)
                .attribute("id", format!("{id}-help"))
                .attribute("class", "proof-help mt-2 mb-0"),
        );
    if let Some(error) = error {
        field = field.child(
            text("p", error)
                .attribute("id", format!("{id}-error"))
                .attribute("class", "text-danger mt-2"),
        );
    }
    field
}

pub(crate) fn form(draft: &Draft, errors: &Errors, enhanced: bool, assets_ready: bool) -> View {
    let title = flag(
        Element::new("input")
            .attribute("id", "title")
            .attribute("name", "title")
            .attribute("type", "text")
            .attribute("class", "form-control")
            .attribute("maxlength", "160")
            .attribute("value", &draft.title),
        "required",
        true,
    );
    // HTML removes one leading newline inside textarea; this sentinel preserves a user's leading newline.
    let notes = Element::new("textarea")
        .attribute("id", "notes")
        .attribute("name", "notes")
        .attribute("rows", "4")
        .attribute("maxlength", "4000")
        .attribute("class", "form-control")
        .child(View::text(format!("\n{}", draft.notes)));
    let mut priority = Element::new("select")
        .attribute("id", "priority")
        .attribute("name", "priority")
        .attribute("class", "form-select");
    if errors.priority {
        priority = priority.child(flag(
            text("option", "Choose a valid priority").attribute("value", ""),
            "selected",
            true,
        ));
    }
    for (value, label) in [("low", "Low"), ("normal", "Normal"), ("high", "High")] {
        priority = priority.child(flag(
            text("option", label).attribute("value", value),
            "selected",
            draft.priority == value,
        ));
    }
    let check = flag(
        Element::new("input")
            .attribute("type", "checkbox")
            .attribute("id", "notify")
            .attribute("name", "notify")
            .attribute("value", "yes")
            .attribute("class", "form-check-input"),
        "checked",
        draft.notify.as_deref() == Some("yes"),
    );
    let mut notify = Element::new("div")
        .attribute("class", "form-check mb-3")
        .child(check)
        .child(
            text("label", "Notify me about status changes")
                .attribute("for", "notify")
                .attribute("class", "form-check-label"),
        );
    if errors.notify {
        notify = notify.child(
            text("p", "Choose a valid notification option.").attribute("class", "text-danger"),
        );
    }
    let mut native_form = Element::new("form")
        .attribute("method", "post")
        .attribute("action", "/proof/submit")
        .attribute("novalidate", "")
        .attribute("class", "proof-form");
    if errors.any() {
        native_form = native_form.child(
            text(
                "p",
                "Correct the highlighted fields. Your draft is retained; nothing was saved.",
            )
            .attribute("role", "alert")
            .attribute("class", "alert alert-danger")
            .attribute("tabindex", "-1"),
        );
    }
    native_form = native_form
        .child(field(
            "Title",
            "title",
            title,
            errors
                .title
                .then_some("Enter 1 to 160 characters, without control characters."),
            "A short description of the work. Required.",
        ))
        .child(field(
            "Notes",
            "notes",
            notes,
            errors
                .notes
                .then_some("Use at most 4,000 characters and ordinary line breaks."),
            "Plain text only. Markup is displayed literally.",
        ))
        .child(field(
            "Priority",
            "priority",
            priority,
            errors.priority.then_some("Choose Low, Normal or High."),
            "Priority is validated again on the server.",
        ))
        .child(notify)
        .child(
            Element::new("div")
                .attribute("class", "proof-actions")
                .child(
                    text("button", "Validate form")
                        .attribute("type", "submit")
                        .attribute("class", "btn btn-primary"),
                )
                .child(
                    text("button", "Reset draft")
                        .attribute("type", "reset")
                        .attribute("class", "btn btn-outline-secondary"),
                )
                .child(flag(
                    text("button", "Save unavailable in proof")
                        .attribute("type", "button")
                        .attribute("class", "btn btn-secondary"),
                    "disabled",
                    true,
                )),
        );
    let mut main = Element::new("main")
        .attribute("id", "main")
        .attribute("class", "proof-main")
        .attribute("data-form-proof", "v1")
        .child(text("h1", "New work item"))
        .child(
            text(
                "p",
                "Isolated form proof: validation only. No records or notifications are saved.",
            )
            .attribute("class", "proof-help"),
        );
    if !assets_ready {
        main = main.child(
            text(
                "p",
                "Optional assets are unavailable. The native form remains usable.",
            )
            .attribute("role", "status"),
        );
    }
    main = main.child(native_form);
    if enhanced && assets_ready {
        main = main.child(Element::new("section").attribute("aria-labelledby", "preview-heading")
            .child(text("h2", "Local preview").attribute("id", "preview-heading"))
            .child(text("p", "Preview listens to the existing controls. It never replaces them or sends their values to a server.").attribute("class", "proof-help"))
            .child(Element::new("div").attribute("class", "proof-actions mb-3")
                .child(text("button", "Activate preview").attribute("type", "button").attribute("data-preview-activate", "").attribute("class", "btn btn-outline-primary"))
                .child(text("button", "Dispose preview").attribute("type", "button").attribute("data-preview-dispose", "").attribute("class", "btn btn-outline-secondary"))
                .child(text("button", "Queue preview update").attribute("type", "button").attribute("data-preview-later", "").attribute("class", "btn btn-outline-secondary")))
            .child(text("p", "Preview inactive. The native form works without it.").attribute("data-preview-summary", "").attribute("class", "proof-summary").attribute("aria-live", "polite")));
    }
    let mut tbody = Element::new("tbody");
    for (name, priority) in [
        ("Inspect lab backup", "Normal"),
        ("Replace test fixture", "Low"),
        ("Review failed check", "High"),
    ] {
        tbody = tbody.child(
            Element::new("tr")
                .child(text("th", name).attribute("scope", "row"))
                .child(text("td", priority)),
        );
    }
    main = main.child(
        Element::new("section")
            .attribute("class", "mt-5")
            .child(text("h2", "Template examples"))
            .child(
                Element::new("div")
                    .attribute("class", "table-responsive")
                    .child(
                        Element::new("table")
                            .attribute("class", "table")
                            .child(text(
                                "caption",
                                "Static fixture templates, not persisted work records.",
                            ))
                            .child(
                                Element::new("thead").child(
                                    Element::new("tr")
                                        .child(text("th", "Template").attribute("scope", "col"))
                                        .child(text("th", "Priority").attribute("scope", "col")),
                                ),
                            )
                            .child(tbody),
                    ),
            ),
    );
    Element::new("div")
        .child(
            text("a", "Skip to the form")
                .attribute("href", "#main")
                .attribute("class", "skip-link"),
        )
        .child(
            Element::new("header")
                .attribute("class", "proof-header")
                .child(
                    text("a", "Lab Operations")
                        .attribute("href", "/proof/native")
                        .attribute("class", "brand"),
                ),
        )
        .child(
            Element::new("div")
                .attribute("class", "proof-shell")
                .child(
                    Element::new("nav")
                        .attribute("aria-label", "Proof modes")
                        .attribute("class", "proof-nav")
                        .child(text("a", "Native form").attribute("href", "/proof/native"))
                        .child(text("a", "Optional preview").attribute("href", "/proof/enhanced")),
                )
                .child(main),
        )
        .into()
}
