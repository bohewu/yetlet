mod style_names;

use yetlet::View;
use yetlet::view::tags::{button, div};

pub fn page(is_open: bool) -> View {
    div()
        .class(style_names::SHELL)
        .child(
            div()
                .class(style_names::TOOLBAR)
                .child(button().text("Open dialog")),
        )
        .child(div().class(style_names::CONTENT).text("Grid content"))
        .child(if is_open {
            div().class(style_names::DIALOG_OPEN).text("Ready").into()
        } else {
            View::text("")
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classes(view: &View, result: &mut Vec<String>) {
        if let View::Element(element) = view {
            result.extend(
                element
                    .attributes()
                    .iter()
                    .filter(|a| a.name == "class")
                    .map(|a| a.value.clone()),
            );
            for child in element.child_views() {
                classes(child, result);
            }
        }
    }

    #[test]
    fn conditional_view_uses_shared_names_and_registered_superset() {
        let mut closed = Vec::new();
        let mut open = Vec::new();
        classes(&page(false), &mut closed);
        classes(&page(true), &mut open);
        assert!(!closed.iter().any(|c| c == style_names::DIALOG_OPEN));
        assert!(open.iter().any(|c| c == style_names::DIALOG_OPEN));
        let generated = include_str!("../assets/plain.css");
        for class in open {
            assert!(
                generated.contains(&format!(".{class} {{")),
                "missing {class}"
            );
        }
    }
}
