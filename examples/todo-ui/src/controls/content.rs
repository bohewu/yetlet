//! Ordinary document elements shared by SSR and the optional DOM mount.
use yetlet::view::{Element, View};

fn text(tag: &'static str, value: &str) -> Element {
    Element::new(tag).child(View::text(value))
}

pub(super) fn view() -> View {
    Element::new("section")
        .attribute("id", "native-document-content")
        .child(text("h2", "Document elements"))
        .child(
            Element::new("p")
                .child(text("abbr", "SSR").attribute("title", "Server-rendered HTML"))
                .child(View::text(
                    " and browser Views share code, tables and native controls. ",
                ))
                .child(text("mark", "Optional interactivity")),
        )
        .child(
            Element::new("figure")
                .child(
                    Element::new("pre")
                        .attribute("id", "native-pre")
                        .child(View::text("\r"))
                        .child(View::text("\nlet value = "))
                        .child(text("code", "<escaped>&\r\n")),
                )
                .child(text(
                    "figcaption",
                    "Text stays text; line endings stay readable.",
                )),
        )
        .child(
            Element::new("table")
                .attribute("id", "native-table")
                .child(text("caption", "A small capability reference"))
                .child(Element::new("colgroup").child(Element::new("col").attribute("span", "2")))
                .child(
                    Element::new("thead").child(
                        Element::new("tr")
                            .child(text("th", "Surface").attribute("scope", "col"))
                            .child(text("th", "Owner").attribute("scope", "col")),
                    ),
                )
                .child(
                    Element::new("tbody").child(
                        Element::new("tr")
                            .child(text("th", "HTML").attribute("scope", "row"))
                            .child(text("td", "Application")),
                    ),
                )
                .child(
                    Element::new("tfoot").child(Element::new("tr").child(
                        text("td", "No binding runtime required").attribute("colspan", "2"),
                    )),
                ),
        )
        .child(
            Element::new("details")
                .attribute("id", "native-details")
                .child(text("summary", "Native behavior"))
                .child(text(
                    "p",
                    "Open this disclosure with a pointer or keyboard.",
                )),
        )
        .child(
            Element::new("p")
                .child(text("label", "Suggested tag ").attribute("for", "native-tag"))
                .child(
                    Element::new("input")
                        .attribute("id", "native-tag")
                        .attribute("list", "native-tags"),
                )
                .child(
                    Element::new("datalist")
                        .attribute("id", "native-tags")
                        .child(
                            Element::new("option")
                                .attribute("value", "HTML")
                                .attribute("label", "HTML"),
                        )
                        .child(Element::new("option").attribute("value", "Wasm")),
                ),
        )
        .child(
            Element::new("p")
                .child(text("label", "Related surfaces ").attribute("for", "native-surfaces"))
                .child(
                    Element::new("select")
                        .attribute("id", "native-surfaces")
                        .attribute("multiple", "")
                        .attribute("size", "2")
                        .child(
                            Element::new("optgroup")
                                .attribute("label", "Rendering")
                                .child(text("option", "HTML").attribute("selected", ""))
                                .child(text("option", "DOM")),
                        ),
                ),
        )
        .child(
            Element::new("p")
                .child(View::text("Example completion: "))
                .child(
                    text("progress", "50%")
                        .attribute("id", "native-progress")
                        .attribute("value", "0.5"),
                )
                .child(View::text(". Example gauge: "))
                .child(
                    text("meter", "0.75")
                        .attribute("id", "native-meter")
                        .attribute("value", "0.75"),
                ),
        )
        .child(text("output", "No background work is running.").attribute("for", "native-tag"))
        .child(
            text("canvas", "Canvas host; no drawing context is acquired.")
                .attribute("width", "0")
                .attribute("height", "0"),
        )
        .child(
            Element::new("dialog")
                .attribute("id", "native-dialog")
                .child(text(
                    "p",
                    "A dialog host; the application owns modal methods and focus.",
                )),
        )
        .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn yt220_reference_content_is_escaped_and_has_explicit_table_sections() {
        let html = yetlet::html::render_to_string(&super::view()).unwrap();
        assert!(html.contains("<pre id=\"native-pre\">\n\r\n"));
        assert!(html.contains("&lt;escaped&gt;&amp;"));
        assert!(html.contains("<thead><tr>"));
        assert!(html.contains("<tbody><tr>"));
        assert!(html.contains("<details id=\"native-details\"><summary>"));
    }
}
