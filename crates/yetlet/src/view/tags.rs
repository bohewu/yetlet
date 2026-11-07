//! Optional ergonomic constructors that lower to the same owned [`Element`] tree.
//!
//! This module adds no renderer, CSS policy, or browser runtime. Typed helpers
//! cover a bounded set of tags and attributes; the general `.attribute` method
//! deliberately remains append-only and renderer validation stays authoritative.
//!
//! ```
//! use yetlet::view::{View, tags::{div, h1, input, textarea}};
//!
//! let page: View = div()
//!     .id("app")
//!     .class("container")
//!     .add_class("active")
//!     .child(h1().text("Hello <Yetlet>"))
//!     .child(input().placeholder("Search..."))
//!     .child(textarea().placeholder("Notes..."))
//!     .into();
//! assert!(matches!(page, View::Element(_)));
//! ```

//! The tag and attribute helpers remain compile-time conveniences only.
//! The following combinations are intentionally unavailable:
//!
//! ```compile_fail
//! use yetlet::view::tags::div;
//! let _ = div().href("/not-a-link");
//! ```
//!
//! ```compile_fail
//! use yetlet::view::{View, tags::br};
//! let _ = br().child(View::text("void elements have no children"));
//! ```
use std::marker::PhantomData;

use super::{Attribute, Element, View};

/// Compile-time tag identity for an ordinary HTML-like element.
///
/// This trait only identifies the name. Renderers still validate supported tags,
/// attribute values and structural semantics independently.
pub trait HtmlTag {
    /// Tag name passed to the existing `Element` constructor.
    const NAME: &'static str;
}

/// Tags for which this authoring layer permits children and text.
///
/// A typed void input cannot receive child nodes:
///
/// ```compile_fail
/// use yetlet::view::{View, tags::input};
/// let _ = input().child(View::text("not allowed"));
/// ```
pub trait ChildTag: HtmlTag {}

/// Tags exposing a `placeholder` convenience setter.
///
/// The method is absent on `div`:
///
/// ```compile_fail
/// use yetlet::view::tags::div;
/// let _ = div().placeholder("not allowed");
/// ```
pub trait PlaceholderTag: HtmlTag {}
/// A consuming, typed convenience builder around the existing `Element`.
///
/// An owned `TagElement` converts directly to `Element` or `View`; no
/// `build()`, macro, or intermediate rendering format is required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagElement<T: HtmlTag> {
    element: Element,
    tag: PhantomData<T>,
}

impl<T: HtmlTag> Default for TagElement<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HtmlTag> TagElement<T> {
    /// Create an empty element using its static tag identity.
    pub fn new() -> Self {
        Self {
            element: Element::new(T::NAME),
            tag: PhantomData,
        }
    }

    /// Replace any existing `id` attributes on this typed builder.
    pub fn id(self, value: impl Into<String>) -> Self {
        self.set_attribute("id", value)
    }

    /// Replace the current class list, retaining each class token once.
    ///
    /// HTML ASCII whitespace separates tokens. This is not an inline CSS API.
    pub fn class(self, value: impl AsRef<str>) -> Self {
        self.set_attribute(
            "class",
            unique_class_tokens(std::iter::once(value.as_ref())),
        )
    }

    /// Append distinct class tokens to the existing class list.
    ///
    /// Existing `class` attributes (including values from earlier generic
    /// `attribute` calls) are consolidated into one attribute. Tokens retain
    /// their first-seen order. A blank addition on a new tag changes nothing.
    pub fn add_class(self, value: impl AsRef<str>) -> Self {
        let current_class = self
            .element
            .attributes
            .iter()
            .filter(|attribute| attribute.name == "class")
            .map(|attribute| attribute.value.as_str());
        let has_class = self
            .element
            .attributes
            .iter()
            .any(|attribute| attribute.name == "class");
        let combined = unique_class_tokens(current_class.chain(std::iter::once(value.as_ref())));
        if !has_class && combined.is_empty() {
            return self;
        }
        self.set_attribute("class", combined)
    }

    /// Append a general attribute, preserving the existing `Element` contract.
    ///
    /// This call never deduplicates. If it creates a duplicate after a
    /// convenience setter, the renderer can reject the resulting tree. A later
    /// `id`, `class`, `add_class`, or `placeholder` call consolidates only
    /// its own target attribute name.
    pub fn attribute(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.element = self.element.attribute(name, value);
        self
    }

    /// Consume this builder into its existing renderer-neutral `Element`.
    pub fn into_element(self) -> Element {
        self.element
    }

    fn set_attribute(mut self, name: &'static str, value: impl Into<String>) -> Self {
        let attributes = &mut self.element.attributes;
        let insert_at = attributes
            .iter()
            .position(|attribute| attribute.name == name)
            .unwrap_or(attributes.len());
        attributes.retain(|attribute| attribute.name != name);
        attributes.insert(
            insert_at,
            Attribute {
                name,
                value: value.into(),
            },
        );
        self
    }
}

impl<T: ChildTag> TagElement<T> {
    /// Append one child convertible to the existing `View` type.
    pub fn child(mut self, child: impl Into<View>) -> Self {
        self.element = self.element.child(child);
        self
    }

    /// Append views from an iterator, retaining their iteration order.
    pub fn children(mut self, children: impl IntoIterator<Item = View>) -> Self {
        self.element = self.element.children(children);
        self
    }

    /// Append an escaped-at-render-time text node, not raw HTML.
    pub fn text(self, value: impl Into<String>) -> Self {
        self.child(View::text(value))
    }
}

impl<T: PlaceholderTag> TagElement<T> {
    /// Set or replace the `placeholder` on input/textarea typed elements.
    pub fn placeholder(self, value: impl Into<String>) -> Self {
        self.set_attribute("placeholder", value)
    }
}

impl<T: HtmlTag> From<TagElement<T>> for Element {
    fn from(builder: TagElement<T>) -> Self {
        builder.element
    }
}

impl<T: HtmlTag> From<TagElement<T>> for View {
    fn from(builder: TagElement<T>) -> Self {
        View::from(builder.element)
    }
}

fn unique_class_tokens<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let mut tokens = Vec::new();
    for value in values {
        for token in value.split_ascii_whitespace() {
            if !tokens.contains(&token) {
                tokens.push(token);
            }
        }
    }
    tokens.join(" ")
}

macro_rules! define_tag {
    ($name:ident, $marker:ident, container) => {
        /// A static tag marker for this HTML-like element.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $marker;
        impl HtmlTag for $marker {
            const NAME: &'static str = stringify!($name);
        }
        impl ChildTag for $marker {}
        #[doc = concat!("Create a typed <", stringify!($name), "> builder.")]
        pub fn $name() -> TagElement<$marker> {
            TagElement::new()
        }
    };
    ($name:ident, $marker:ident, void) => {
        /// A static void element marker; child/text setters are not available.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $marker;
        impl HtmlTag for $marker {
            const NAME: &'static str = stringify!($name);
        }
        #[doc = concat!("Create a typed <", stringify!($name), "> builder.")]
        pub fn $name() -> TagElement<$marker> {
            TagElement::new()
        }
    };
}

macro_rules! define_tags {
    ($($name:ident, $marker:ident, $kind:ident;)+) => {
        $(define_tag!($name, $marker, $kind);)+

        #[cfg(test)]
        mod tag_coverage_tests {
            use super::*;

            #[test]
            fn all_101_constructors_preserve_tag_name_and_html_policy() {
                let elements: Vec<(&str, Element)> = vec![
                    $((stringify!($name), $name().into()),)+
                ];
                assert_eq!(elements.len(), 101);
                for (name, element) in elements {
                    assert_eq!(element.tag(), name);
                    assert!(element.attributes().is_empty());
                    assert!(element.child_views().is_empty());
                    #[cfg(feature = "html")]
                    crate::html::render_to_string(&View::from(element))
                        .expect("all ordinary View tags are admitted by SSR");
                }
            }
        }
    };
}

// Pinned YT-219 standard HTML ordinary View inventory. The 7 document/head
// tags and 5 forbidden executable/embedded tags are intentionally excluded.
// Eleven entries are SSR-only because the current browser DOM admits 89 tags
// by default, plus img with explicit ImagePolicy. Constructors alone never
// modify renderer policy or claim browser admission.
define_tags! {
    a, A, container;
    abbr, Abbr, container;
    address, Address, container;
    area, Area, void;
    article, Article, container;
    aside, Aside, container;
    audio, Audio, container;
    b, B, container;
    bdi, Bdi, container;
    bdo, Bdo, container;
    blockquote, Blockquote, container;
    br, Br, void;
    button, Button, container;
    canvas, Canvas, container;
    caption, Caption, container;
    cite, Cite, container;
    code, Code, container;
    col, Col, void;
    colgroup, ColGroup, container;
    data, Data, container;
    datalist, DataList, container;
    dd, Dd, container;
    del, Del, container;
    details, Details, container;
    dfn, Dfn, container;
    dialog, Dialog, container;
    div, Div, container;
    dl, Dl, container;
    dt, Dt, container;
    em, Em, container;
    fieldset, Fieldset, container;
    figcaption, FigCaption, container;
    figure, Figure, container;
    footer, Footer, container;
    form, Form, container;
    h1, H1, container;
    h2, H2, container;
    h3, H3, container;
    h4, H4, container;
    h5, H5, container;
    h6, H6, container;
    header, Header, container;
    hgroup, Hgroup, container;
    hr, Hr, void;
    i, I, container;
    img, Img, void;
    input, Input, void;
    ins, Ins, container;
    kbd, Kbd, container;
    label, Label, container;
    legend, Legend, container;
    li, Li, container;
    main, Main, container;
    map, Map, container;
    mark, Mark, container;
    menu, Menu, container;
    meter, Meter, container;
    nav, Nav, container;
    noscript, Noscript, container;
    ol, Ol, container;
    optgroup, OptGroup, container;
    option, Option, container;
    output, Output, container;
    p, P, container;
    picture, Picture, container;
    pre, Pre, container;
    progress, Progress, container;
    q, Q, container;
    rp, Rp, container;
    rt, Rt, container;
    ruby, Ruby, container;
    s, S, container;
    samp, Samp, container;
    search, Search, container;
    section, Section, container;
    select, Select, container;
    selectedcontent, SelectedContent, container;
    slot, Slot, container;
    small, Small, container;
    source, Source, void;
    span, Span, container;
    strong, Strong, container;
    sub, Sub, container;
    summary, Summary, container;
    sup, Sup, container;
    table, Table, container;
    tbody, Tbody, container;
    td, Td, container;
    template, Template, container;
    textarea, Textarea, container;
    tfoot, Tfoot, container;
    th, Th, container;
    thead, Thead, container;
    time, Time, container;
    tr, Tr, container;
    track, Track, void;
    u, U, container;
    ul, Ul, container;
    var, Var, container;
    video, Video, container;
    wbr, Wbr, void;
}

macro_rules! scoped_attribute {
    ($trait_name:ident, $method:ident, $attribute:literal; $($marker:ty),+ $(,)?) => {
        #[doc = concat!("Tags supporting the ", stringify!($method), " setter.")]
        pub trait $trait_name: HtmlTag {}
        $(impl $trait_name for $marker {})+
        impl<T: $trait_name> TagElement<T> {
            #[doc = concat!("Set or replace the ", $attribute, " attribute.")]
            pub fn $method(self, value: impl Into<String>) -> Self {
                self.set_attribute($attribute, value)
            }
        }
    };
}

impl PlaceholderTag for Input {}
impl PlaceholderTag for Textarea {}

// These setters only establish typed tag applicability and deduplicate earlier
// same-named typed-builder attributes; the chosen renderer still validates
// attribute values, URLs and browser navigation/image capabilities.
scoped_attribute!(HrefTag, href, "href"; A, Area);
scoped_attribute!(SrcTag, src, "src"; Img, Audio, Video, Source, Track);
scoped_attribute!(AltTag, alt, "alt"; Img, Area);
scoped_attribute!(TypeTag, type_, "type"; Input, Button, Ol, Source);
scoped_attribute!(ForTag, for_, "for"; Label, Output);
scoped_attribute!(NameTag, name, "name"; Input, Button, Textarea, Select, Form, Fieldset, Output, Details);
scoped_attribute!(ValueTag, value, "value"; Input, Button, Option, Data, Li, Meter, Progress);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_and_existing_constructors_produce_equal_owned_trees() {
        let new: View = div()
            .id("app")
            .class("container")
            .add_class("active")
            .child(h1().text("Hello <Yetlet>"))
            .child(input().placeholder("Search <anything>"))
            .into();
        let existing: View = Element::new("div")
            .attribute("id", "app")
            .attribute("class", "container active")
            .child(Element::new("h1").child(View::text("Hello <Yetlet>")))
            .child(Element::new("input").attribute("placeholder", "Search <anything>"))
            .into();
        assert_eq!(new, existing);

        let old_builder: Element = Element::new("section").child(p().text("Mixed")).clone();
        let typed_element: Element = section().child(p().text("Mixed")).into();
        assert_eq!(old_builder, typed_element);
    }

    #[test]
    fn class_operations_replace_append_deduplicate_and_preserve_position() {
        let element: Element = div()
            .attribute("title", "title")
            .attribute("class", "first first\tsecond")
            .attribute("class", "third")
            .attribute("id", "old")
            .add_class("second\r\n fourth third")
            .id("new")
            .into();
        assert_eq!(
            element.attributes(),
            &[
                Attribute {
                    name: "title",
                    value: "title".into()
                },
                Attribute {
                    name: "class",
                    value: "first second third fourth".into()
                },
                Attribute {
                    name: "id",
                    value: "new".into()
                },
            ]
        );

        let element: Element = span()
            .add_class(" ")
            .class("one two one")
            .add_class("two three")
            .class("replacement")
            .into();
        assert_eq!(
            element.attributes(),
            &[Attribute {
                name: "class",
                value: "replacement".into()
            }]
        );
        let empty: Element = div().add_class("\t ").into();
        assert!(empty.attributes().is_empty());
    }

    #[test]
    fn placeholder_applies_to_input_and_textarea_and_upserts() {
        let input_element: Element = input()
            .attribute("placeholder", "old")
            .placeholder("new")
            .into();
        assert_eq!(
            input_element.attributes(),
            &[Attribute {
                name: "placeholder",
                value: "new".into()
            }]
        );
        let textarea_view: View = textarea().placeholder("Notes").text("Draft").into();
        assert_eq!(
            textarea_view,
            View::from(
                Element::new("textarea")
                    .attribute("placeholder", "Notes")
                    .child(View::text("Draft"))
            )
        );
    }

    #[test]
    fn child_order_and_to_view_interoperability_are_retained() {
        let children = [View::text("one"), View::text("two")];
        let actual: View = ul().children(children).child(li().text("third")).into();
        let expected: View = Element::new("ul")
            .children([View::text("one"), View::text("two")])
            .child(Element::new("li").child(View::text("third")))
            .into();
        assert_eq!(actual, expected);
    }

    #[test]
    fn scoped_setters_keep_existing_tree_conversion_and_upsert_semantics() {
        let link: Element = a()
            .attribute("href", "/old")
            .attribute("href", "/also-old")
            .href("/about")
            .text("About")
            .into();
        assert_eq!(
            link,
            Element::new("a")
                .attribute("href", "/about")
                .child(View::text("About"))
        );

        let photo: Element = img()
            .attribute("src", "/old.png")
            .src("/logo.png")
            .alt("Logo <Yetlet>")
            .into();
        assert_eq!(
            photo,
            Element::new("img")
                .attribute("src", "/logo.png")
                .attribute("alt", "Logo <Yetlet>")
        );

        let controls: View = form()
            .child(label().for_("query").text("Query"))
            .child(
                input()
                    .type_("search")
                    .name("query")
                    .value("hello")
                    .placeholder("Enter a term"),
            )
            .into();
        let old: View = Element::new("form")
            .child(
                Element::new("label")
                    .attribute("for", "query")
                    .child(View::text("Query")),
            )
            .child(
                Element::new("input")
                    .attribute("type", "search")
                    .attribute("name", "query")
                    .attribute("value", "hello")
                    .attribute("placeholder", "Enter a term"),
            )
            .into();
        assert_eq!(controls, old);

        for element in [
            Element::from(area().href("/map").alt("Region")),
            Element::from(source().src("/media.m4a").type_("audio/mp4")),
            Element::from(video().src("/media.webm")),
            Element::from(track().src("/captions.vtt")),
            Element::from(button().type_("submit").name("submit").value("yes")),
            Element::from(output().for_("query").name("result")),
            Element::from(option().value("a").text("Alpha")),
            Element::from(ol().type_("a")),
            Element::from(data().value("22").text("Twenty two")),
            Element::from(li().value("3").text("Third")),
            Element::from(meter().value("0.5")),
            Element::from(progress().value("0.5")),
        ] {
            assert!(!element.attributes().is_empty());
        }
    }

    #[cfg(feature = "html")]
    #[test]
    fn nested_semantic_and_image_builder_render_without_policy_changes() {
        use crate::html::{RenderError, render_to_string};

        let page: View = div()
            .child(a().href("/about").text("About"))
            .child(img().src("/logo.png").alt("Logo <mark>"))
            .child(table().child(tbody().child(tr().child(td().text("Hello")))))
            .into();
        assert_eq!(
            render_to_string(&page).unwrap(),
            "<div><a href=\"/about\">About</a><img src=\"/logo.png\" alt=\"Logo &lt;mark&gt;\"><table><tbody><tr><td>Hello</td></tr></tbody></table></div>"
        );

        let duplicate: View = a().href("/about").attribute("href", "/again").into();
        assert_eq!(
            render_to_string(&duplicate),
            Err(RenderError::DuplicateAttribute("href".into()))
        );
        let unsafe_url: View = a().href("javascript:alert(1)").into();
        assert_eq!(
            render_to_string(&unsafe_url),
            Err(RenderError::UnsafeUrl("href".into()))
        );
    }
    #[cfg(feature = "html")]
    #[test]
    fn renderer_escapes_text_and_values_and_keeps_duplicate_errors() {
        use crate::html::render_to_string;
        let view: View = div()
            .class("ready")
            .child(p().text("Hello <world>"))
            .child(input().placeholder("Search <anything>"))
            .into();
        assert_eq!(
            render_to_string(&view).unwrap(),
            "<div class=\"ready\"><p>Hello &lt;world&gt;</p><input placeholder=\"Search &lt;anything&gt;\"></div>"
        );

        // Generic attributes deliberately preserve the original append behavior.
        let duplicate: View = div().class("ready").attribute("class", "again").into();
        assert!(render_to_string(&duplicate).is_err());
    }
}
