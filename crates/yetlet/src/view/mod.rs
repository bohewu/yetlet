//! Owned UI values shared by the HTML and browser renderers.
//!
//! A component can be an ordinary function returning [`View`]. Names are static
//! strings; dynamic text and attribute values become owned strings. Construction
//! does not sanitize content or select a renderer. See the
//! [View guide](https://github.com/bohewu/yetlet/blob/master/docs/VIEW-COMPONENTS.md)
//! for application composition. This API remains Experimental.

/// Validated renderer-neutral identity, key, and dynamic-slot vocabulary.
pub mod binding;

/// A UI tree that can be consumed by either a native HTML or a browser renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// Untrusted text; a renderer must handle it as text, not executable markup.
    Text(String),
    /// An element and its owned attributes/children, awaiting renderer validation.
    Element(Element),
}

impl View {
    /// Own a text value without treating it as markup or pre-escaped output.
    ///
    /// Renderers must treat the value as untrusted text. Do not HTML-escape it
    /// before construction: the HTML renderer performs escaping when rendering.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }
}

/// An HTML-like element description, not an already-escaped HTML string.
///
/// Builder methods consume and return the element; convert the final value into
/// [`View`] with `.into()`. A View does not borrow its text or attribute inputs.
/// Tag/attribute names and child structure are checked by the selected renderer,
/// not by these constructors. HTML and browser DOM have different accepted subsets.
///
/// ```
/// use yetlet::{Element, View};
///
/// let heading = String::from("My tasks");
/// let view: View = Element::new("section")
///     .attribute("id", "tasks")
///     .child(Element::new("h1").child(View::text(&heading)))
///     .children([View::text("Read"), View::text("Build")])
///     .into();
/// drop(heading); // The tree owns the copied text.
/// assert!(matches!(view, View::Element(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    tag: &'static str,
    attributes: Vec<Attribute>,
    children: Vec<View>,
}

/// One attribute with a static name and an owned, unescaped value.
///
/// This value alone establishes no renderer policy. Prefer [`Element::attribute`]
/// to append it to an element; duplicate names are not merged by the builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Attribute name, checked by the selected renderer.
    pub name: &'static str,
    /// Literal value; output escaping and URL policy belong to the renderer.
    pub value: String,
}

impl Element {
    /// Start an element with a static tag name and no attributes or children.
    ///
    /// Construction is infallible; an unsupported name can be rejected later by
    /// the selected renderer. This does not provide a raw-HTML escape hatch.
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Append an owned attribute value, preserving insertion order.
    ///
    /// Existing names are not replaced or deduplicated. The renderer can reject
    /// duplicate/forbidden attributes or unsafe URLs; escaping happens at output.
    pub fn attribute(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            name,
            value: value.into(),
        });
        self
    }

    /// Append one child convertible into [`View`], after the existing children.
    ///
    /// An [`Element`] converts directly; use [`View::text`] for literal text.
    pub fn child(mut self, child: impl Into<View>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Append Views from an iterator in iteration order, keeping existing children.
    ///
    /// Unlike [`Self::child`], iterator items must already be Views; map child
    /// elements with `View::from` when necessary.
    pub fn children(mut self, children: impl IntoIterator<Item = View>) -> Self {
        self.children.extend(children);
        self
    }

    /// Return the tag name as supplied, without renderer validation.
    pub fn tag(&self) -> &'static str {
        self.tag
    }

    /// Borrow attributes in insertion order; no normalization or filtering occurs.
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    /// Borrow the children in their construction order.
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
    fn yt152_child_conversion_order_clone_and_unwind() {
        use std::cell::Cell;

        struct Converted<'a>(&'a Cell<usize>);
        impl From<Converted<'_>> for View {
            fn from(value: Converted<'_>) -> Self {
                value.0.set(value.0.get() + 1);
                View::text("first")
            }
        }
        let conversions = Cell::new(0);
        let tree = Element::new("div")
            .child(Converted(&conversions))
            .children([View::text("second"), View::text("third")])
            .child(View::text("fourth"))
            .child(View::text("fifth"));
        assert_eq!(conversions.get(), 1);
        assert_eq!(
            tree.child_views(),
            ["first", "second", "third", "fourth", "fifth"].map(View::text)
        );
        let cloned = tree.clone().child(View::text("sixth"));
        assert_eq!(tree.child_views().len(), 5);
        assert_eq!(cloned.child_views().last(), Some(&View::text("sixth")));
        let bulk_first = Element::new("div")
            .children([View::text("bulk")])
            .child(View::text("after"));
        assert_eq!(bulk_first.child_views(), ["bulk", "after"].map(View::text));

        struct Panicking<'a>(&'a Cell<usize>);
        impl Drop for Panicking<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        impl From<Panicking<'_>> for View {
            fn from(_: Panicking<'_>) -> Self {
                panic!("intentional conversion unwind")
            }
        }
        let drops = Cell::new(0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Element::new("div").child(Panicking(&drops))
        }));
        assert!(result.is_err());
        assert_eq!(drops.get(), 1);
    }

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
