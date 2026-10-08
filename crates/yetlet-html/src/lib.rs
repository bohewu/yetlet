//! Native HTML renderer for Yetlet's renderer-neutral views.
//!
//! The serializer always escapes dynamic text/attribute values and validates
//! tag/attribute names. No arbitrary raw HTML escape hatch exists in the MVP.

use std::error::Error;
use std::fmt::{self, Write as _};
use yetlet_core::View;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    InvalidTag(String),
    InvalidAttribute(String),
    ForbiddenAttribute(String),
    UnsafeUrl(String),
    VoidElementHasChildren(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTag(tag) => write!(f, "invalid HTML tag: {tag}"),
            Self::InvalidAttribute(attribute) => {
                write!(f, "invalid HTML attribute: {attribute}")
            }
            Self::ForbiddenAttribute(attribute) => {
                write!(f, "forbidden HTML attribute: {attribute}")
            }
            Self::UnsafeUrl(attribute) => {
                write!(f, "unsafe URL scheme for attribute: {attribute}")
            }
            Self::VoidElementHasChildren(tag) => {
                write!(f, "void HTML element cannot have children: {tag}")
            }
        }
    }
}

impl Error for RenderError {}

pub fn escape_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
    output
}

/// Serialize one view without allowing markup injection through its text/attrs.
pub fn render_to_string(view: &View) -> Result<String, RenderError> {
    let mut output = String::new();
    render_into(view, &mut output)?;
    Ok(output)
}

fn render_into(view: &View, output: &mut String) -> Result<(), RenderError> {
    match view {
        View::Text(value) => output.push_str(&escape_text(value)),
        View::Element(element) => {
            let tag = element.tag();
            if !valid_identifier(tag) || FORBIDDEN_TAGS.contains(&tag) {
                return Err(RenderError::InvalidTag(tag.to_owned()));
            }

            write!(output, "<{tag}").expect("writing to String is infallible");
            for attribute in element.attributes() {
                let name = attribute.name;
                if !valid_identifier(name) {
                    return Err(RenderError::InvalidAttribute(name.to_owned()));
                }
                if name.starts_with("on") || matches!(name, "style" | "srcdoc") {
                    return Err(RenderError::ForbiddenAttribute(name.to_owned()));
                }
                if URL_ATTRIBUTES.contains(&name) && unsafe_scheme(&attribute.value) {
                    return Err(RenderError::UnsafeUrl(name.to_owned()));
                }

                write!(output, " {name}=\"{}\"", escape_text(&attribute.value))
                    .expect("writing to String is infallible");
            }
            output.push('>');

            if VOID_ELEMENTS.contains(&tag) {
                if !element.child_views().is_empty() {
                    return Err(RenderError::VoidElementHasChildren(tag.to_owned()));
                }
                return Ok(());
            }
            for child in element.child_views() {
                render_into(child, output)?;
            }
            write!(output, "</{tag}>").expect("writing to String is infallible");
        }
    }
    Ok(())
}

const FORBIDDEN_TAGS: &[&str] = &["script", "style", "iframe", "object", "embed"];
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];
const URL_ATTRIBUTES: &[&str] = &["href", "src", "action", "formaction", "xlink:href"];

fn valid_identifier(identifier: &str) -> bool {
    let mut bytes = identifier.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z'))
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn unsafe_scheme(value: &str) -> bool {
    // Whitespace/control characters can be ignored by URL parsers; strip them
    // before checking common script-capable schemes. This is intentionally
    // conservative for MVP and is not a full URL policy engine.
    let normalized: String = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && !character.is_ascii_control())
        .flat_map(char::to_lowercase)
        .collect();
    normalized.starts_with("javascript:")
        || normalized.starts_with("vbscript:")
        || normalized.starts_with("data:")
}

#[cfg(test)]
mod tests {
    use super::{RenderError, render_to_string};
    use yetlet_core::{Element, View};

    #[test]
    fn escapes_dynamic_text_and_attributes() {
        let tree: View = Element::new("p")
            .attribute("title", "\" onclick='evil()' &")
            .child(View::text("<img src=x onerror=1> & ' \""))
            .into();
        let html = render_to_string(&tree).unwrap();
        assert_eq!(
            html,
            "<p title=\"&quot; onclick=&#39;evil()&#39; &amp;\">&lt;img src=x onerror=1&gt; &amp; &#39; &quot;</p>"
        );
    }

    #[test]
    fn rejects_event_handler_attribute() {
        let tree = Element::new("button").attribute("onclick", "evil()").into();
        assert_eq!(
            render_to_string(&tree),
            Err(RenderError::ForbiddenAttribute("onclick".to_string()))
        );
    }

    #[test]
    fn rejects_script_urls_and_inline_styles() {
        let link = Element::new("a")
            .attribute("href", "ja\nvascript:alert(1)")
            .into();
        assert_eq!(
            render_to_string(&link),
            Err(RenderError::UnsafeUrl("href".to_string()))
        );

        let style = Element::new("div").attribute("style", "color:red").into();
        assert_eq!(
            render_to_string(&style),
            Err(RenderError::ForbiddenAttribute("style".to_string()))
        );
    }

    #[test]
    fn rejects_invalid_tag_and_void_children() {
        let invalid = Element::new("div onload=evil").into();
        assert!(matches!(
            render_to_string(&invalid),
            Err(RenderError::InvalidTag(_))
        ));
        let invalid = Element::new("input").child(View::text("unsafe")).into();
        assert!(matches!(
            render_to_string(&invalid),
            Err(RenderError::VoidElementHasChildren(_))
        ));
    }
}
