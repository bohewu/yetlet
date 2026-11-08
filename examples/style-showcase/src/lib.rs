//! Shared gallery View. Styles are compiled by the separate host build script.
pub mod selection;
pub mod style_names;
pub use selection::Selection;
use yetlet::view::{View, tags::*};

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

mod previews {
    pub mod components;
    pub mod layout;
    pub mod states;
    pub mod tokens;
}

include!(concat!(env!("OUT_DIR"), "/identity.rs"));
pub const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/showcase.css"));

pub fn choice(selection: Selection, path: &str, kind: &str, value: &str, label: &str) -> View {
    a().class("choice")
        .href(selection.change(kind, value).href(path, "tokens"))
        .attribute("data-choice", kind)
        .attribute("data-value", value)
        .attribute(
            "aria-current",
            if selection.value(kind) == value {
                "true"
            } else {
                "false"
            },
        )
        .text(label)
        .into()
}

fn source(label: &str, contents: &str) -> View {
    details()
        .class("source-panel")
        .child(summary().text(label))
        .child(pre().child(code().text(contents)))
        .into()
}

fn example(
    id: &str,
    number: &str,
    title: &str,
    description: &str,
    preview: View,
    sources: (&str, &str),
) -> View {
    section()
        .id(id)
        .class("gallery-section")
        .child(
            div()
                .class("section-heading")
                .child(span().class("section-number").text(number))
                .child(
                    div()
                        .child(h2().text(title))
                        .child(p().class("muted").text(description)),
                ),
        )
        .child(
            div()
                .class("example-panel")
                .child(div().class("preview").child(preview))
                .child(
                    div()
                        .class("source-heading")
                        .text("From this build · actual compiled sources"),
                )
                .child(source(&format!("View · src/previews/{id}.rs"), sources.0))
                .child(source(&format!("Style · styles/{id}.rs"), sources.1))
                .child(
                    p().class("css-reference").child(
                        a().href("#stylesheet")
                            .text("Produced CSS + ordinary CSS source ↓"),
                    ),
                ),
        )
        .into()
}

/// Both routes render this exact tree; enhancement never replaces it.
pub fn gallery(selection: Selection, interactive: bool, available: bool) -> View {
    let path = if interactive {
        "/showcase/interactive"
    } else {
        "/showcase"
    };
    let mode_text = if interactive && available {
        "Optional Rust/WASM enhancement: theme, density and selection change locally. Links still work without scripts."
    } else if interactive {
        "Browser assets are unavailable or stale. The complete gallery works through native links; run the explicit browser build to enable local switching."
    } else {
        "Native SSR · no JavaScript or WASM required. Theme, density and selection use ordinary page links."
    };
    div().id("showcase").class(selection.classes())
        .attribute("data-theme", selection.theme()).attribute("data-density", selection.density())
        .attribute("data-selected", selection.selected())
        .child(a().href("#content").class("skip-link").text("Skip to examples"))
        .child(header().class("site-header")
            .child(a().class("brand").href("/showcase").text("yetlet").child(span().text(" / style showcase")))
            .child(a().class("mode-link").href(selection.href(if interactive { "/showcase" } else { "/showcase/interactive" }, "content"))
                .text(if interactive { "Native version ↗" } else { "Try optional enhancement ↗" })))
        .child(div().class("page-layout")
            .child(nav().class("category-nav").attribute("aria-label", "Gallery categories")
                .child(p().class("nav-label").text("ON THIS PAGE"))
                .children([("layout", "01", "Layout"), ("tokens", "02", "Tokens"),
                    ("components", "03", "Components"), ("states", "04", "States")]
                    .map(|(id, n, label)| a().href(format!("#{id}"))
                        .child(span().text(n)).text(label).into()))
                .child(a().class("output-link").href("#stylesheet").text("Build sources & CSS ↓")))
            .child(main().id("content")
                .child(div().class("intro")
                    .child(p().class("eyebrow").text("RUST → HTML + CSS"))
                    .child(h1().text("Styles you can trace."))
                    .child(p().class("lede").text("Explore real previews, then follow each one back to its Rust View, registered styles and generated stylesheet."))
                    .child(p().class("mode-note").id("enhancement-status").text(mode_text)))
                .child(example("layout", "01", "Layout", "Flex, Grid and spacing. Ordinary browser layout, authored in Rust.",
                    previews::layout::preview(), (include_str!("previews/layout.rs"), include_str!("../styles/layout.rs"))))
                .child(example("tokens", "02", "Tokens", "Two complete color themes. Two densities. All registered before the first request.",
                    previews::tokens::preview(selection, path), (include_str!("previews/tokens.rs"), include_str!("../styles/tokens.rs"))))
                .child(example("components", "03", "Components", "Buttons, a card, a badge and labeled controls, composed as normal View functions.",
                    previews::components::preview(), (include_str!("previews/components.rs"), include_str!("../styles/components.rs"))))
                .child(example("states", "04", "States", "Focus, disabled, selected and disclosed. State has words and semantics as well as color.",
                    previews::states::preview(selection, path), (include_str!("previews/states.rs"), include_str!("../styles/states.rs"))))
                .child(section().id("stylesheet").class("gallery-section output-section")
                    .child(h2().text("Follow the build"))
                    .child(p().class("muted").text("The host composes four modules explicitly. Ordinary CSS supplies unsupported selectors, backgrounds, borders, wrapping and focus rings. Both routes link this one complete stylesheet."))
                    .child(source("Registration · styles/mod.rs", include_str!("../styles/mod.rs")))
                    .child(source("Shared class names · src/style_names.rs", include_str!("style_names.rs")))
                    .child(source("Shared selection · src/selection.rs", include_str!("selection.rs")))
                    .child(source("Gallery composition and source inclusion · src/lib.rs", include_str!("lib.rs")))
                    .child(source("Host generation · build.rs", include_str!("../build.rs")))
                    .child(source("Ordinary app CSS · styles/app.css", include_str!("../styles/app.css")))
                    .child(source("Complete produced stylesheet · Rust output + ordinary app CSS", CSS))
                    .child(p().class("build-identity").text(format!("CSS SHA-256: {CSS_SHA256}"))))
                .child(footer().class("page-footer").text("Yetlet · application-owned styles · source-only prototype"))))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yetlet::html::render_to_string;

    #[test]
    fn real_sources_are_included_as_escaped_text() {
        let html = render_to_string(&gallery(Selection::default(), false, false)).unwrap();
        assert!(html.contains("pub fn preview() -&gt; View"));
        assert!(html.contains(".attribute(&quot;disabled&quot;, &quot;&quot;)"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<style"));
        assert!(
            html.split('<').all(|fragment| !fragment
                .split('>')
                .next()
                .unwrap_or("")
                .contains(" style="))
        );
        for category in ["layout", "tokens", "components", "states"] {
            assert!(html.contains(&format!("id=\"{category}\"")));
        }
    }

    #[test]
    fn selections_are_finite_and_conditional_styles_are_registered() {
        assert_eq!(
            Selection::from_values("evil", "<script>", "other"),
            Selection::default()
        );
        assert_eq!(
            Selection::default().change("theme", "<script>"),
            Selection::default()
        );
        for class in [
            style_names::DARK,
            style_names::COMPACT,
            style_names::SELECTED,
            style_names::UNSELECTED,
        ] {
            assert!(CSS.contains(&format!(".{class} {{")));
        }
        let selection = Selection::from_values("dark", "compact", "second");
        let html = render_to_string(&gallery(selection, false, false)).unwrap();
        assert!(html.contains("showcase theme-dark density-compact"));
        assert!(html.contains("id=\"select-second\" class=\"demo-card is-selected\""));
        assert!(html.contains("Current example: Optional enhancement."));
        assert!(html.contains("selected=second#states"));
    }
}
