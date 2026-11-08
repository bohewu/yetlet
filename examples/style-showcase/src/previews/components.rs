use crate::style_names::*;
use yetlet::view::{View, tags::*};

pub fn preview() -> View {
    div().class("component-preview")
        .child(article().class(CARD)
            .child(span().class(BADGE).text("App-owned component"))
            .child(h3().text("Small functions. Native HTML."))
            .child(p().text("A card is an ordinary Rust function returning View. Its styles are registered separately."))
            .child(a().class("text-link").href("#stylesheet").text("Read the emitted stylesheet →")))
        .child(form().class("demo-form").attribute("aria-label", "Native form controls")
            .child(p().class("muted").text("Edit these native controls locally. Nothing is submitted."))
            .child(label().attribute("for", "demo-name").text("Example name"))
            .child(input().id("demo-name").attribute("name", "name").attribute("type", "text")
                .attribute("value", "Style Showcase").attribute("autocomplete", "off"))
            .child(label().attribute("for", "demo-format").text("Preferred format"))
            .child(select().id("demo-format").attribute("name", "format")
                .child(option().attribute("value", "rust").text("Rust source"))
                .child(option().attribute("value", "css").text("Generated CSS")))
            .child(label().class("checkbox-label").attribute("for", "demo-notes")
                .child(input().id("demo-notes").attribute("type", "checkbox").attribute("name", "notes"))
                .text("Include source notes"))
            .child(div().class("button-row")
                .child(button().class(BUTTON).attribute("type", "reset").text("Reset fields"))
                .child(button().class(BUTTON).add_class("secondary").attribute("type", "submit")
                    .attribute("disabled", "").text("Submit unavailable"))))
        .into()
}
