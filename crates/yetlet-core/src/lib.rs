//! Minimal renderer-neutral Yetlet UI representation.
//!
//! This is an MVP spike, not a stable public component or template API.

/// Validated renderer-neutral identity, key, and dynamic-slot vocabulary.
pub mod binding;

/// A UI tree that can be consumed by either a native HTML or a browser renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Text(String),
    Element(Element),
}

impl View {
    /// Construct a text node. Renderers must treat this value as untrusted text.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }
}

/// An HTML-like element description, not an already-escaped HTML string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    tag: &'static str,
    attributes: Vec<Attribute>,
    children: Vec<View>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: &'static str,
    pub value: String,
}

impl Element {
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn attribute(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            name,
            value: value.into(),
        });
        self
    }

    pub fn child(mut self, child: impl Into<View>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = View>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn tag(&self) -> &'static str {
        self.tag
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn child_views(&self) -> &[View] {
        &self.children
    }
}

impl From<Element> for View {
    fn from(element: Element) -> Self {
        Self::Element(element)
    }
}

#[cfg(test)]
mod tests {
    use super::{Element, View};

    #[test]
    fn view_tree_is_renderer_neutral() {
        let tree: View = Element::new("ul")
            .attribute("class", "todos")
            .child(Element::new("li").child(View::text("first")))
            .into();

        assert_eq!(
            tree,
            View::Element(
                Element::new("ul")
                    .attribute("class", "todos")
                    .child(Element::new("li").child(View::Text("first".into())),)
            )
        );
    }
}
