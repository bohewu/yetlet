//! Browser DOM renderer for Yetlet's shared View tree.
//!
//! The M2 prototype intentionally uses direct DOM creation rather than
//! innerHTML or a full virtual-DOM diff. It replaces one owned subtree.
//! Only a safe, documented subset of HTML is currently supported.

use yetlet_core::View;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomPolicyError {
    UnsupportedTag(String),
    UnsupportedAttribute(String),
}

impl std::fmt::Display for DomPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedTag(tag) => write!(f, "unsupported DOM tag: {tag}"),
            Self::UnsupportedAttribute(name) => write!(f, "unsupported DOM attribute: {name}"),
        }
    }
}

/// Validate before applying any changes to the DOM.
///
/// MVP intentionally disallows script/style/inline event attributes and URL
/// attributes, so untrusted strings can only become DOM text or inert values.
pub fn validate(view: &View) -> Result<(), DomPolicyError> {
    match view {
        View::Text(_) => Ok(()),
        View::Element(element) => {
            if !matches!(
                element.tag(),
                "main"
                    | "section"
                    | "div"
                    | "article"
                    | "header"
                    | "h1"
                    | "h2"
                    | "p"
                    | "span"
                    | "ul"
                    | "li"
                    | "button"
                    | "strong"
                    | "input"
                    | "label"
            ) {
                return Err(DomPolicyError::UnsupportedTag(element.tag().to_owned()));
            }
            for attribute in element.attributes() {
                if !matches!(
                    attribute.name,
                    "id" | "class"
                        | "type"
                        | "data-id"
                        | "data-completed"
                        | "aria-live"
                        | "aria-pressed"
                        | "for"
                        | "placeholder"
                        | "maxlength"
                        | "autocomplete"
                ) {
                    return Err(DomPolicyError::UnsupportedAttribute(
                        attribute.name.to_owned(),
                    ));
                }
            }
            for child in element.child_views() {
                validate(child)?;
            }
            Ok(())
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::{View, validate};
    use wasm_bindgen::JsValue;
    use web_sys::{Document, Element, Node};

    fn create_node(document: &Document, view: &View) -> Result<Node, JsValue> {
        match view {
            View::Text(value) => Ok(document.create_text_node(value).into()),
            View::Element(element) => {
                let node = document.create_element(element.tag())?;
                for attribute in element.attributes() {
                    node.set_attribute(attribute.name, &attribute.value)?;
                }
                for child in element.child_views() {
                    node.append_child(&create_node(document, child)?)?;
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
        let replacement = create_node(&document, view)?;
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
    use yetlet_core::{Element, View};

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
