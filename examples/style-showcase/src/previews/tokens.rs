use crate::{Selection, choice, style_names::*};
use yetlet::view::{View, tags::*};

pub fn preview(selection: Selection, path: &str) -> View {
    div()
        .child(div().class("appearance-controls")
            .child(div().child(p().class("control-label").text("Color theme"))
                .child(div().class("choices").attribute("role", "group").attribute("aria-label", "Color theme")
                    .child(choice(selection, path, "theme", "light", "Light"))
                    .child(choice(selection, path, "theme", "dark", "Dark"))))
            .child(div().child(p().class("control-label").text("Spacing density"))
                .child(div().class("choices").attribute("role", "group").attribute("aria-label", "Spacing density")
                    .child(choice(selection, path, "density", "comfortable", "Comfortable"))
                    .child(choice(selection, path, "density", "compact", "Compact")))))
        .child(div().class("token-swatches")
            .children([("swatch-surface", "Surface", "--app-surface"),
                ("swatch-ink", "Text", "--app-ink"),
                ("swatch-accent", "Accent", "--app-accent"),
                ("swatch-border", "Border", "--app-border")]
                .map(|(class, name, token)| div().class("token")
                    .child(span().class(class).add_class("swatch").attribute("aria-hidden", "true"))
                    .child(strong().text(name)).child(code().text(token)).into())))
        .child(div().class(CARD).id("density-preview")
            .child(h3().text("A complete set, on one root"))
            .child(p().text("Theme changes every color token. Density changes the same spacing used by the toolbar, grid and cards.")))
        .into()
}
