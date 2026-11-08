use crate::style_names::*;
use yetlet::view::{View, tags::*};

pub fn preview() -> View {
    div()
        .class("layout-preview")
        .child(
            div()
                .class(TOOLBAR)
                .id("flex-preview")
                .child(
                    div()
                        .child(strong().text("A toolbar that makes room"))
                        .child(
                            p().class("muted")
                                .text("Flex alignment + a shared spacing token."),
                        ),
                )
                .child(
                    a().href("#states")
                        .class("demo-button secondary")
                        .text("Explore states →"),
                ),
        )
        .child(
            div().class(GRID).id("grid-preview").children(
                [
                    ("01", "One shared View"),
                    ("02", "Registered Rust styles"),
                    ("03", "External CSS"),
                ]
                .map(|(number, title)| {
                    div()
                        .class(CARD)
                        .child(span().class("step-number").text(number))
                        .child(h3().text(title))
                        .into()
                }),
            ),
        )
        .child(
            div()
                .class(SPACING)
                .id("spacing-preview")
                .child(span().text("Padding"))
                .child(span().text("Gap"))
                .child(span().text("One token")),
        )
        .into()
}
