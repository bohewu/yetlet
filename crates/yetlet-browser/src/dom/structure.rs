use super::DomPolicyError;
use yetlet::view::{Element, View};

fn formatting(text: &str) -> bool {
    text.bytes()
        .all(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0c))
}

fn only(element: &Element, names: &[&str]) -> bool {
    element.child_views().iter().all(|v| match v {
        View::Text(text) => formatting(text),
        View::Element(child) => names.contains(&child.tag()),
    })
}

fn table(element: &Element) -> bool {
    let mut phase = 0;
    for child in element.child_views() {
        match child {
            View::Text(s) if formatting(s) => {}
            View::Element(e) => match e.tag() {
                "caption" if phase == 0 => phase = 1,
                "colgroup" if phase <= 2 => phase = 2,
                "thead" if phase <= 2 => phase = 3,
                "tbody" if phase <= 4 => phase = 4,
                "tfoot" if phase <= 4 => phase = 5,
                _ => return false,
            },
            _ => return false,
        }
    }
    true
}

pub(super) fn validate(element: &Element, parent: Option<&str>) -> Result<(), DomPolicyError> {
    let tag = element.tag();
    let valid = match tag {
        "textarea" | "option" => element
            .child_views()
            .iter()
            .all(|v| matches!(v, View::Text(_))),
        // Preserve the old select whitespace admission; new table structures use HTML whitespace.
        "select" => element.child_views().iter().all(|v| match v {
            View::Text(s) => s.trim().is_empty(),
            View::Element(e) => matches!(e.tag(), "option" | "optgroup"),
        }),
        "optgroup" => parent == Some("select") && only(element, &["option"]),
        "datalist" => only(element, &["option"]),
        "table" => table(element),
        "caption" => parent == Some("table"),
        "colgroup" => {
            parent == Some("table")
                && only(element, &["col"])
                && !(element.attributes().iter().any(|a| a.name == "span")
                    && element
                        .child_views()
                        .iter()
                        .any(|v| matches!(v, View::Element(_))))
        }
        "col" => parent == Some("colgroup"),
        "thead" | "tbody" | "tfoot" => parent == Some("table") && only(element, &["tr"]),
        "tr" => matches!(parent, Some("thead" | "tbody" | "tfoot")) && only(element, &["td", "th"]),
        "td" | "th" => parent == Some("tr"),
        "details" => {
            let mut children = element
                .child_views()
                .iter()
                .filter(|v| !matches!(v, View::Text(s) if formatting(s)));
            matches!(children.next(), Some(View::Element(e)) if e.tag() == "summary")
                && !children.any(|v| matches!(v, View::Element(e) if e.tag() == "summary"))
        }
        "summary" => parent == Some("details"),
        "figure" => {
            let mut caption = None;
            let mut count = 0;
            let mut duplicate = false;
            for v in element
                .child_views()
                .iter()
                .filter(|v| !matches!(v, View::Text(s) if formatting(s)))
            {
                if matches!(v, View::Element(e) if e.tag() == "figcaption") {
                    duplicate |= caption.replace(count).is_some();
                }
                count += 1;
            }
            !duplicate && caption.is_none_or(|i| i == 0 || i + 1 == count)
        }
        "figcaption" => parent == Some("figure"),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(DomPolicyError::InvalidChildren(tag.into()))
    }
}
