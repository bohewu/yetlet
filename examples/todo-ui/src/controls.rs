//! A shared native-draft form: the browser owns edits and reset; GET only previews.
use yetlet::view::{Element, View};

mod content;

#[derive(Debug, Clone)]
pub struct NativeDraft {
    pub title: String,
    pub notes: String,
    pub priority: String,
    pub completed: bool,
}

impl Default for NativeDraft {
    fn default() -> Self {
        Self {
            title: "Draft task".into(),
            notes: "\nKeep this draft local.\nPreview without creating a task.".into(),
            priority: "normal".into(),
            completed: true,
        }
    }
}

/// Optional URL/image surface; the caller explicitly supplies per-mount policies.
/// This is not installed by `native_controls` and does not select browser options.
pub fn controlled_links_and_image(href: &str, src: &str) -> View {
    Element::new("section")
        .attribute("id", "controlled-resources")
        .child(Element::new("h2").child(View::text("Controlled links and images")))
        .child(
            Element::new("a")
                .attribute("id", "controlled-link")
                .attribute("href", href)
                .attribute("target", "_blank")
                .attribute("rel", "noopener noreferrer")
                .attribute("referrerpolicy", "no-referrer")
                .child(View::text("Open the selected documentation origin")),
        )
        .child(
            Element::new("img")
                .attribute("id", "controlled-image")
                .attribute("src", src)
                .attribute("alt", "Selected example asset")
                .attribute("width", "16")
                .attribute("height", "16")
                .attribute("loading", "eager")
                .attribute("decoding", "async")
                .attribute("referrerpolicy", "no-referrer"),
        )
        .into()
}

/// No graph, listener or persistence is required for native defaults and reset.
pub fn native_controls(draft: &NativeDraft) -> View {
    let mut priority = Element::new("select")
        .attribute("id", "native-priority")
        .attribute("name", "priority");
    for (value, label) in [("normal", "Normal"), ("important", "Important")] {
        let mut option = Element::new("option")
            .attribute("value", value)
            .child(View::text(label));
        if draft.priority == value {
            option = option.attribute("selected", "");
        }
        priority = priority.child(option);
    }
    let mut completed = Element::new("input")
        .attribute("id", "native-completed")
        .attribute("type", "checkbox")
        .attribute("name", "completed")
        .attribute("value", "yes");
    if draft.completed {
        completed = completed.attribute("checked", "");
    }
    Element::new("section").attribute("id", "native-controls").attribute("aria-labelledby", "native-heading")
        .child(Element::new("h2").attribute("id", "native-heading").child(View::text("Native draft preview")))
        .child(Element::new("p").attribute("id", "native-hint").child(View::text("Edit native controls, restore their defaults, or preview with a read-only GET. No task is created.")))
        .child(Element::new("form").attribute("id", "native-form").attribute("method", "get").attribute("action", "/controls")
            .child(Element::new("fieldset")
                .child(Element::new("legend").child(View::text("Draft values")))
                .child(Element::new("p")
                    .child(Element::new("label").attribute("for", "native-title").child(View::text("Title")))
                    .child(Element::new("input").attribute("id", "native-title").attribute("name", "title")
                        .attribute("type", "text").attribute("value", &draft.title).attribute("required", "")
                        .attribute("maxlength", "120").attribute("autocomplete", "off")))
                .child(Element::new("p")
                    .child(Element::new("label").attribute("for", "native-notes").child(View::text("Notes")))
                    .child(Element::new("textarea").attribute("id", "native-notes").attribute("name", "notes")
                        .attribute("rows", "3").attribute("maxlength", "500").attribute("aria-describedby", "native-hint")
                        .child(View::text(&draft.notes))))
                .child(Element::new("p")
                    .child(Element::new("label").attribute("for", "native-priority").child(View::text("Priority")))
                    .child(priority))
                .child(Element::new("p").child(Element::new("label").attribute("for", "native-completed").child(View::text("Completed"))).child(completed))
                .child(Element::new("p")
                    .child(Element::new("label").attribute("for", "native-mode").child(View::text("Mode")))
                    .child(Element::new("input").attribute("id", "native-mode").attribute("name", "mode").attribute("value", "Preview only").attribute("readonly", "")))
                .child(Element::new("p")
                    .child(Element::new("label").attribute("for", "native-disabled").child(View::text("Unavailable field")))
                    .child(Element::new("input").attribute("id", "native-disabled").attribute("name", "ignored").attribute("value", "Not submitted").attribute("disabled", "")))
                .child(Element::new("button").attribute("id", "native-submit").attribute("type", "submit").child(View::text("Preview with GET")))
                .child(Element::new("button").attribute("id", "native-reset").attribute("type", "reset").child(View::text("Restore page defaults")))))
        .child(Element::new("nav").attribute("aria-label", "Native control examples")
            .child(Element::new("a").attribute("id", "native-ssr-link").attribute("href", "/controls").child(View::text("Open the no-JS form")))
            .child(View::text(" | "))
            .child(Element::new("a").attribute("id", "native-anchor-link").attribute("href", "#native-controls").child(View::text("Jump to these controls"))))
        .child(content::view())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yt221_controlled_surface_is_separate_and_escaped() {
        let html = yetlet::html::render_to_string(&controlled_links_and_image(
            "https://docs.example.invalid/?a=1&b=2",
            "/mark.png?label=<demo>",
        ))
        .unwrap();
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains("&amp;b=2"));
        assert!(html.contains("label=&lt;demo&gt;"));
        assert!(
            !yetlet::html::render_to_string(&native_controls(&Default::default()))
                .unwrap()
                .contains("controlled-image")
        );
    }

    #[test]
    fn defaults_and_text_are_serialized_without_a_binding_runtime() {
        let html =
            yetlet::html::render_to_string(&native_controls(&NativeDraft::default())).unwrap();
        assert!(html.contains("value=\"Draft task\""));
        assert!(html.contains("value=\"normal\" selected=\"\""));
        assert!(html.contains("value=\"yes\" checked=\"\""));
        assert!(html.contains("\n\nKeep this draft local."));
        let draft = NativeDraft {
            title: "<script>not markup</script>".into(),
            completed: false,
            ..NativeDraft::default()
        };
        let html = yetlet::html::render_to_string(&native_controls(&draft)).unwrap();
        assert!(html.contains("&lt;script&gt;not markup&lt;/script&gt;"));
        assert!(!html.contains("checked="));
    }
}
