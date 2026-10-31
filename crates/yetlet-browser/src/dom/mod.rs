//! Browser DOM renderer for Yetlet's shared View tree.
//!
//! The M2 prototype intentionally uses direct DOM creation rather than
//! innerHTML or a full virtual-DOM diff. It replaces one owned subtree.
//! Only a safe, documented subset of HTML is currently supported.
//!
//! `hidden` is passed to the browser without value coercion. Use an empty value
//! to hide an element and omit/remove the attribute to reveal it. In HTML this
//! is an enumerated attribute: the string `"false"` does not mean visible.
//! `until-found` retains native browser semantics; no reveal lifecycle is added.

use yetlet::view::View;

mod structure;
mod vocabulary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomPolicyError {
    UnsupportedTag(String),
    UnsupportedAttribute(String),
    DuplicateAttribute(String),
    VoidElementHasChildren(String),
    InvalidChildren(String),
    InvalidAttributeValue(String),
}

impl std::fmt::Display for DomPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedTag(tag) => write!(f, "unsupported DOM tag: {tag}"),
            Self::UnsupportedAttribute(name) => write!(f, "unsupported DOM attribute: {name}"),
            Self::DuplicateAttribute(name) => write!(f, "duplicate DOM attribute: {name}"),
            Self::VoidElementHasChildren(tag) => write!(f, "void DOM element has children: {tag}"),
            Self::InvalidChildren(tag) => write!(f, "unsupported DOM children for: {tag}"),
            Self::InvalidAttributeValue(name) => {
                write!(f, "unsupported DOM attribute value: {name}")
            }
        }
    }
}

/// Validate before applying any changes to the DOM.
///
/// Executable markup, inline handlers/styles and resource URLs remain excluded.
/// Navigation is limited to root-relative paths and fragments. Native validation
/// checks that spelling; mount additionally resolves URLs with the browser parser
/// and requires HTTP(S), no credentials and the owner document's origin.
pub fn validate(view: &View) -> Result<(), DomPolicyError> {
    validate_inside(view, false, None)
}

fn validate_inside(
    view: &View,
    inside_form: bool,
    parent: Option<&str>,
) -> Result<(), DomPolicyError> {
    let View::Element(element) = view else {
        return Ok(());
    };
    let tag = element.tag();
    if !vocabulary::supported_tag(tag) {
        return Err(DomPolicyError::UnsupportedTag(tag.into()));
    }
    if matches!(tag, "input" | "br" | "hr" | "col" | "wbr") && !element.child_views().is_empty() {
        return Err(DomPolicyError::VoidElementHasChildren(tag.into()));
    }
    if tag == "form" && inside_form {
        return Err(DomPolicyError::InvalidChildren(tag.into()));
    }
    structure::validate(element, parent)?;
    for (index, attribute) in element.attributes().iter().enumerate() {
        let name = attribute.name;
        if element.attributes()[..index].iter().any(|a| a.name == name) {
            return Err(DomPolicyError::DuplicateAttribute(name.into()));
        }
        if !vocabulary::attribute_allowed(tag, name) {
            return Err(DomPolicyError::UnsupportedAttribute(name.into()));
        }
        if !vocabulary::attribute_value_allowed(tag, name, &attribute.value) {
            return Err(DomPolicyError::InvalidAttributeValue(name.into()));
        }
        if (matches!(name, "href" | "action")
            && !navigation_spelling(&attribute.value, name == "href"))
            || (name == "method" && !matches!(attribute.value.as_str(), "get" | "post"))
        {
            return Err(DomPolicyError::InvalidAttributeValue(name.into()));
        }
    }
    vocabulary::validate_element_values(element)?;
    for child in element.child_views() {
        validate_inside(child, inside_form || tag == "form", Some(tag))?;
    }
    Ok(())
}

// This is a deliberately narrow input grammar, not a URL parser or an origin check.
fn navigation_spelling(value: &str, fragments: bool) -> bool {
    !value
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
        && ((value.starts_with('/') && !value.starts_with("//"))
            || (fragments && value.starts_with('#')))
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::{View, validate};
    use wasm_bindgen::JsValue;
    use web_sys::{Document, Element, Node, Url};

    fn navigation_url(document: &Document, value: &str) -> Result<String, JsValue> {
        let page = Url::new(&document.url()?)?;
        let base = document
            .base_uri()?
            .ok_or_else(|| JsValue::from_str("missing document base URL"))?;
        let target = Url::new_with_base(value, &base)?;
        if !matches!(page.protocol().as_str(), "http:" | "https:")
            || !matches!(target.protocol().as_str(), "http:" | "https:")
            || target.origin() != page.origin()
            || !target.username().is_empty()
            || !target.password().is_empty()
        {
            return Err(JsValue::from_str(
                "DOM navigation must stay on the HTTP(S) document origin",
            ));
        }
        // Freeze the accepted destination; later <base> changes cannot redirect it.
        Ok(target.href())
    }

    fn create_node(document: &Document, view: &View, in_pre: bool) -> Result<Node, JsValue> {
        match view {
            View::Text(value) => Ok(document.create_text_node(value).into()),
            View::Element(element) => {
                let node = document.create_element(element.tag())?;
                for attribute in element.attributes() {
                    let value = if matches!(attribute.name, "href" | "action") {
                        navigation_url(document, &attribute.value)?
                    } else {
                        attribute.value.clone()
                    };
                    node.set_attribute(attribute.name, &value)?;
                }
                if element.tag() == "textarea" {
                    // Join first so a CR/LF split across adjacent text nodes normalizes once.
                    let text: String = element
                        .child_views()
                        .iter()
                        .filter_map(|v| match v {
                            View::Text(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .collect();
                    node.set_text_content(Some(&text.replace("\r\n", "\n").replace('\r', "\n")));
                } else {
                    let children = element.child_views();
                    let normalize = in_pre || element.tag() == "pre";
                    let mut index = 0;
                    while index < children.len() {
                        if normalize && matches!(children[index], View::Text(_)) {
                            // HTML parses adjacent text as one stream, including split CR/LF.
                            let mut text = String::new();
                            while let Some(View::Text(value)) = children.get(index) {
                                text.push_str(value);
                                index += 1;
                            }
                            let text = text.replace("\r\n", "\n").replace('\r', "\n");
                            node.append_child(&document.create_text_node(&text))?;
                        } else {
                            node.append_child(&create_node(
                                document,
                                &children[index],
                                normalize,
                            )?)?;
                            index += 1;
                        }
                    }
                }
                Ok(node.into())
            }
        }
    }

    /// Mount a fully-built shared View into an owned browser DOM subtree.
    ///
    /// The View is validated and constructed before clearing the old subtree.
    /// Event listeners must live outside the replaced subtree.
    pub fn mount(target: &Element, view: &View) -> Result<(), JsValue> {
        validate(view).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let document = target
            .owner_document()
            .ok_or_else(|| JsValue::from_str("DOM mount target has no document"))?;
        let replacement = create_node(&document, view, false)?;
        target.set_text_content(None);
        target.append_child(&replacement)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::mount;

#[cfg(test)]
mod tests {
    use super::{DomPolicyError, validate};
    use yetlet::view::{Element, View};

    #[test]
    fn rejects_duplicate_names_and_void_children() {
        let duplicate = Element::new("div")
            .attribute("id", "first")
            .attribute("id", "last")
            .into();
        assert_eq!(
            validate(&duplicate),
            Err(DomPolicyError::DuplicateAttribute("id".into()))
        );
        for tag in ["input", "br", "hr"] {
            let view = Element::new(tag).child(View::text("invalid")).into();
            assert_eq!(
                validate(&view),
                Err(DomPolicyError::VoidElementHasChildren(tag.into()))
            );
        }
    }

    #[test]
    fn accepts_selected_native_control_defaults_and_semantics() {
        let fields = Element::new("fieldset")
            .child(Element::new("legend").child(View::text("Draft")))
            .child(
                Element::new("input")
                    .attribute("name", "title")
                    .attribute("value", "Draft")
                    .attribute("required", "false"),
            )
            .child(
                Element::new("textarea")
                    .attribute("name", "notes")
                    .attribute("rows", "3")
                    .attribute("readonly", "")
                    .child(View::text("\nnotes")),
            )
            .child(
                Element::new("select").attribute("name", "status").child(
                    Element::new("option")
                        .attribute("value", "new")
                        .attribute("selected", "false")
                        .child(View::text("New")),
                ),
            );
        let navigation = Element::new("nav").attribute("aria-label", "Links").child(
            Element::new("a")
                .attribute("href", "#draft")
                .child(View::text("Draft")),
        );
        let view = Element::new("form")
            .attribute("action", "/controls")
            .attribute("method", "get")
            .child(fields)
            .child(navigation)
            .into();
        assert_eq!(validate(&view), Ok(()));
    }

    #[test]
    fn rejects_invalid_control_children_and_misplaced_attributes() {
        for view in [
            Element::new("textarea").child(Element::new("div")).into(),
            Element::new("option").child(Element::new("span")).into(),
            Element::new("select").child(Element::new("div")).into(),
            Element::new("form")
                .child(Element::new("section").child(Element::new("form")))
                .into(),
        ] {
            assert!(matches!(
                validate(&view),
                Err(DomPolicyError::InvalidChildren(_))
            ));
        }
        for (tag, name) in [
            ("textarea", "value"),
            ("div", "href"),
            ("input", "selected"),
            ("div", "multiple"),
            ("button", "formaction"),
            ("a", "target"),
            ("p", "data-yetlet-owner"),
            ("p", "aria-arbitrary"),
        ] {
            assert_eq!(
                validate(&Element::new(tag).attribute(name, "x").into()),
                Err(DomPolicyError::UnsupportedAttribute(name.into()))
            );
        }
    }

    #[test]
    fn navigation_has_a_narrow_spelling_and_separate_submission_boundary() {
        for value in ["/", "/controls?title=a%20b", "#draft", "/a/../b"] {
            assert_eq!(
                validate(&Element::new("a").attribute("href", value).into()),
                Ok(())
            );
        }
        for value in [
            "",
            "//other.example/a",
            "/\\other.example/a",
            "https://example.com",
            "javascript:alert(1)",
            "data:text/html,test",
            "relative",
            " /path",
            "/a\nb",
            "/a b",
        ] {
            assert_eq!(
                validate(&Element::new("a").attribute("href", value).into()),
                Err(DomPolicyError::InvalidAttributeValue("href".into()))
            );
        }
        assert!(validate(&Element::new("form").attribute("action", "#draft").into()).is_err());
        assert!(validate(&Element::new("form").attribute("method", "dialog").into()).is_err());
        assert!(validate(&Element::new("input").attribute("src", "/image.png").into()).is_err());
    }

    #[test]
    fn accepts_semantic_component_subtree() {
        let view = Element::new("main")
            .attribute("id", "todo-app")
            .child(
                Element::new("ul").child(
                    Element::new("li")
                        .attribute("data-completed", "false")
                        .child(View::text("<not markup>")),
                ),
            )
            .into();
        assert_eq!(validate(&view), Ok(()));
    }

    #[test]
    fn accepts_hidden_without_coercing_its_value() {
        assert_eq!(validate(&Element::new("p").into()), Ok(()));
        for value in ["", "hidden", "false", "until-found"] {
            let view = Element::new("p").attribute("hidden", value).into();
            assert_eq!(validate(&view), Ok(()), "hidden={value:?}");
        }
    }

    #[test]
    fn hidden_does_not_relax_other_attribute_or_child_restrictions() {
        for name in [
            "onclick",
            "style",
            "srcdoc",
            "href",
            "src",
            "data-arbitrary",
        ] {
            let view = Element::new("div")
                .attribute("hidden", "")
                .attribute(name, "not allowed")
                .into();
            assert_eq!(
                validate(&view),
                Err(DomPolicyError::UnsupportedAttribute(name.into())),
            );
        }
        let view = Element::new("div")
            .attribute("hidden", "")
            .child(Element::new("script"))
            .into();
        assert_eq!(
            validate(&view),
            Err(DomPolicyError::UnsupportedTag("script".into()))
        );
    }

    #[test]
    fn rejects_executable_markup_and_inline_handlers() {
        let script = Element::new("script").child(View::text("danger")).into();
        assert_eq!(
            validate(&script),
            Err(DomPolicyError::UnsupportedTag("script".into()))
        );
        let on_click = Element::new("button").attribute("onclick", "evil()").into();
        assert_eq!(
            validate(&on_click),
            Err(DomPolicyError::UnsupportedAttribute("onclick".into()))
        );
        let link = Element::new("div").attribute("srcdoc", "<script>").into();
        assert!(validate(&link).is_err());
    }
}

#[cfg(test)]
mod tests_yt220;
