use crate::{Selection, style_names::*};
use yetlet::view::{View, tags::*};

pub fn preview(selection: Selection, path: &str) -> View {
    div()
        .child(p().class("muted").text("Use Tab to see focus. Choose a card to change its text and semantic current state."))
        .child(div().class("selection-grid").attribute("role", "group").attribute("aria-label", "Example selection")
            .children([("first", "Native HTML", "Read the complete page before any browser code loads."),
                ("second", "Optional enhancement", "Change predefined classes locally with Rust/WASM.")]
                .map(|(value, title, description)| {
                    let selected = selection.selected() == value;
                    a().id(format!("select-{value}")).class(CARD)
                        .add_class(if selected { SELECTED } else { UNSELECTED })
                        .href(selection.change("selected", value).href(path, "states"))
                        .attribute("data-choice", "selected").attribute("data-value", value)
                        .attribute("aria-current", if selected { "true" } else { "false" })
                        .child(span().class("selection-label").text(if selected { "Selected" } else { "Select this example" }))
                        .child(h3().text(title)).child(p().text(description)).into()
                })))
        .child(p().id("selection-status").attribute("role", "status")
            .text(format!("Current example: {}.", if selection.second { "Optional enhancement" } else { "Native HTML" })))
        .child(details().id("native-details").class("native-note")
            .child(summary().text("Why native disclosure?"))
            .child(p().text("This content opens with the keyboard and works without JavaScript. Focus, disabled controls and selection remain native HTML semantics.")))
        .into()
}
