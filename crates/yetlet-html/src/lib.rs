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
    DuplicateAttribute(String),
    InvalidChildren(String),
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
            Self::DuplicateAttribute(name) => write!(f, "duplicate HTML attribute: {name}"),
            Self::InvalidChildren(tag) => write!(f, "unsupported HTML children for: {tag}"),
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
    render_into(view, &mut output, false)?;
    Ok(output)
}

fn render_into(view: &View, output: &mut String, inside_form: bool) -> Result<(), RenderError> {
    match view {
        View::Text(value) => output.push_str(&escape_text(value)),
        View::Element(element) => {
            let tag = element.tag();
            if !valid_identifier(tag) || FORBIDDEN_TAGS.contains(&tag) {
                return Err(RenderError::InvalidTag(tag.to_owned()));
            }

            if (tag == "form" && inside_form)
                || (matches!(tag, "textarea" | "option")
                    && element
                        .child_views()
                        .iter()
                        .any(|v| !matches!(v, View::Text(_))))
                || (tag == "select"
                    && element.child_views().iter().any(|v| {
                        !matches!(v, View::Element(e) if matches!(e.tag(), "option" | "optgroup"))
                            && !matches!(v, View::Text(s) if s.trim().is_empty())
                    }))
            {
                return Err(RenderError::InvalidChildren(tag.into()));
            }
            write!(output, "<{tag}").expect("writing to String is infallible");
            for (index, attribute) in element.attributes().iter().enumerate() {
                let name = attribute.name;
                if element.attributes()[..index].iter().any(|a| a.name == name) {
                    return Err(RenderError::DuplicateAttribute(name.into()));
                }
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
            if tag == "textarea" {
                let text: String = element
                    .child_views()
                    .iter()
                    .filter_map(|v| match v {
                        View::Text(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .collect();
                let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                // HTML parsing discards one LF immediately after the start tag.
                if normalized.starts_with('\n') {
                    output.push('\n');
                }
                output.push_str(&escape_text(&normalized));
            } else {
                for child in element.child_views() {
                    render_into(child, output, inside_form || tag == "form")?;
                }
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
    fn rejects_duplicate_names_and_invalid_control_children() {
        let view = Element::new("div")
            .attribute("id", "first")
            .attribute("id", "last")
            .into();
        assert_eq!(
            render_to_string(&view),
            Err(RenderError::DuplicateAttribute("id".into()))
        );
        for view in [
            Element::new("textarea").child(Element::new("p")).into(),
            Element::new("option").child(Element::new("span")).into(),
            Element::new("select").child(Element::new("div")).into(),
            Element::new("form")
                .child(Element::new("section").child(Element::new("form")))
                .into(),
        ] {
            assert!(matches!(
                render_to_string(&view),
                Err(RenderError::InvalidChildren(_))
            ));
        }
    }

    #[test]
    fn textarea_defaults_preserve_initial_newline_and_escape_text() {
        let view = Element::new("textarea")
            .child(View::text("\r"))
            .child(View::text("\n<draft>\rnext"))
            .into();
        assert_eq!(
            render_to_string(&view).unwrap(),
            "<textarea>\n\n&lt;draft&gt;\nnext</textarea>"
        );
        let view = Element::new("textarea").child(View::text("normal")).into();
        assert_eq!(
            render_to_string(&view).unwrap(),
            "<textarea>normal</textarea>"
        );
    }

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
    fn serializes_hidden_presence_and_values_without_boolean_coercion() {
        assert_eq!(
            render_to_string(&Element::new("p").into()).unwrap(),
            "<p></p>"
        );
        for value in ["", "hidden", "false", "until-found"] {
            let view = Element::new("p").attribute("hidden", value).into();
            assert_eq!(
                render_to_string(&view).unwrap(),
                format!("<p hidden=\"{value}\"></p>"),
            );
        }
        let view = Element::new("p")
            .attribute("hidden", "\" onclick=\"evil()&")
            .into();
        assert_eq!(
            render_to_string(&view).unwrap(),
            "<p hidden=\"&quot; onclick=&quot;evil()&amp;\"></p>",
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
