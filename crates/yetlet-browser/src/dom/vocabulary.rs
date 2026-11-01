//! Finite HTML admission; URL/resource capabilities remain separate.
use super::DomPolicyError;
use yetlet::view::Element;

fn legacy_tag(tag: &str) -> bool {
    matches!(
        tag,
        "main"
            | "section"
            | "div"
            | "article"
            | "header"
            | "footer"
            | "nav"
            | "aside"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "p"
            | "span"
            | "ul"
            | "ol"
            | "li"
            | "dl"
            | "dt"
            | "dd"
            | "strong"
            | "em"
            | "small"
            | "code"
            | "br"
            | "hr"
            | "button"
            | "input"
            | "label"
            | "form"
            | "fieldset"
            | "legend"
            | "textarea"
            | "select"
            | "option"
            | "a"
    )
}

pub(super) fn supported_tag(tag: &str) -> bool {
    legacy_tag(tag)
        || matches!(
            tag,
            "abbr"
                | "address"
                | "b"
                | "bdi"
                | "bdo"
                | "blockquote"
                | "canvas"
                | "caption"
                | "cite"
                | "col"
                | "colgroup"
                | "data"
                | "datalist"
                | "del"
                | "details"
                | "dfn"
                | "dialog"
                | "figcaption"
                | "figure"
                | "hgroup"
                | "i"
                | "ins"
                | "kbd"
                | "mark"
                | "menu"
                | "meter"
                | "optgroup"
                | "output"
                | "pre"
                | "progress"
                | "q"
                | "rp"
                | "rt"
                | "ruby"
                | "s"
                | "samp"
                | "search"
                | "sub"
                | "summary"
                | "sup"
                | "table"
                | "tbody"
                | "td"
                | "tfoot"
                | "th"
                | "thead"
                | "time"
                | "tr"
                | "u"
                | "var"
                | "wbr"
        )
}

pub(super) fn attribute_allowed(tag: &str, name: &str) -> bool {
    let global = matches!(
        name,
        "id" | "class"
            | "hidden"
            | "title"
            | "lang"
            | "dir"
            | "role"
            | "tabindex"
            | "data-id"
            | "data-completed"
            | "data-select"
            | "data-title"
            | "data-meta"
            | "data-selected"
            | "aria-live"
            | "aria-pressed"
            | "aria-label"
            | "aria-labelledby"
            | "aria-describedby"
            | "aria-invalid"
            | "aria-busy"
            | "aria-hidden"
            | "aria-current"
            | "aria-expanded"
            | "aria-controls"
            | "accesskey"
            | "autocapitalize"
            | "autocorrect"
            | "contenteditable"
            | "draggable"
            | "enterkeyhint"
            | "inert"
            | "inputmode"
            | "spellcheck"
            | "translate"
            | "writingsuggestions"
    );
    // These five old globally admitted names remain accepted on old tags.
    let legacy = legacy_tag(tag)
        && matches!(
            name,
            "type" | "for" | "placeholder" | "maxlength" | "autocomplete"
        );
    let scoped = match name {
        "href" => tag == "a",
        "action" | "method" | "accept-charset" | "enctype" | "novalidate" => tag == "form",
        "name" => matches!(
            tag,
            "input" | "button" | "textarea" | "select" | "form" | "fieldset" | "output" | "details"
        ),
        "value" => matches!(
            tag,
            "input" | "button" | "option" | "data" | "li" | "meter" | "progress"
        ),
        "checked" | "accept" | "list" => tag == "input",
        "selected" => tag == "option",
        "disabled" => matches!(
            tag,
            "input" | "button" | "textarea" | "select" | "fieldset" | "option" | "optgroup"
        ),
        "required" => matches!(tag, "input" | "textarea" | "select"),
        "readonly" | "dirname" | "minlength" => matches!(tag, "input" | "textarea"),
        "rows" | "cols" | "wrap" => tag == "textarea",
        "form" => matches!(
            tag,
            "input" | "button" | "fieldset" | "output" | "select" | "textarea"
        ),
        "multiple" | "size" => matches!(tag, "input" | "select"),
        "min" => matches!(tag, "input" | "meter"),
        "max" => matches!(tag, "input" | "meter" | "progress"),
        "step" | "pattern" => tag == "input",
        "label" => matches!(tag, "option" | "optgroup"),
        "for" => tag == "output",
        "datetime" => matches!(tag, "time" | "del" | "ins"),
        "start" | "reversed" | "type" => tag == "ol",
        "span" => matches!(tag, "col" | "colgroup"),
        "colspan" | "rowspan" | "headers" => matches!(tag, "td" | "th"),
        "scope" | "abbr" => tag == "th",
        "open" => matches!(tag, "details" | "dialog"),
        "low" | "high" | "optimum" => tag == "meter",
        "width" | "height" => tag == "canvas",
        _ => false,
    };
    global || legacy || scoped
}

fn keyword(value: &str, choices: &[&str]) -> bool {
    choices
        .iter()
        .any(|choice| value.eq_ignore_ascii_case(choice))
}

fn nonnegative(value: &str) -> Option<u32> {
    (!value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn integer(value: &str) -> bool {
    let digits = value.strip_prefix('-').unwrap_or(value);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) && value.parse::<i32>().is_ok()
}

// HTML floating-point spelling, not Rust's NaN/inf or leading-plus extensions.
fn number(value: &str) -> Option<f64> {
    let bytes = value.as_bytes();
    let mut i = usize::from(bytes.first() == Some(&b'-'));
    let before = i;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let digits = i - before;
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return None;
        }
    } else if digits == 0 {
        return None;
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return None;
        }
    }
    if i != bytes.len() {
        return None;
    }
    value.parse::<f64>().ok().filter(|n| n.is_finite())
}

pub(super) fn attribute_value_allowed(tag: &str, name: &str, value: &str) -> bool {
    match name {
        "autocapitalize" => keyword(
            value,
            &["on", "off", "none", "sentences", "words", "characters"],
        ),
        "autocorrect" => keyword(value, &["", "on", "off"]),
        "contenteditable" => keyword(value, &["", "true", "false", "plaintext-only"]),
        "draggable" => keyword(value, &["true", "false"]),
        "enterkeyhint" => keyword(
            value,
            &["enter", "done", "go", "next", "previous", "search", "send"],
        ),
        "inputmode" => keyword(
            value,
            &[
                "none", "text", "tel", "email", "url", "numeric", "decimal", "search",
            ],
        ),
        "spellcheck" | "writingsuggestions" => keyword(value, &["", "true", "false"]),
        "translate" => keyword(value, &["", "yes", "no"]),
        "enctype" => keyword(
            value,
            &[
                "application/x-www-form-urlencoded",
                "multipart/form-data",
                "text/plain",
            ],
        ),
        "wrap" => keyword(value, &["soft", "hard"]),
        "scope" => keyword(value, &["row", "col", "rowgroup", "colgroup"]),
        "dir" if tag == "bdo" => keyword(value, &["ltr", "rtl"]),
        "span" | "colspan" => nonnegative(value).is_some_and(|n| (1..=1000).contains(&n)),
        "rowspan" => nonnegative(value).is_some_and(|n| n <= 65534),
        "size" => nonnegative(value).is_some_and(|n| n > 0),
        "minlength" => nonnegative(value).is_some_and(|n| n <= i32::MAX as u32),
        "width" | "height" => nonnegative(value).is_some(),
        "start" => integer(value),
        "value" if tag == "li" => integer(value),
        "value" | "min" | "max" | "low" | "high" | "optimum"
            if matches!(tag, "meter" | "progress") =>
        {
            number(value).is_some()
        }
        _ => true, // Includes old permissive control/default values and presence booleans.
    }
}

fn attribute<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
    element
        .attributes()
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.value.as_str())
}

pub(super) fn validate_element_values(element: &Element) -> Result<(), DomPolicyError> {
    let invalid = |name: &str| DomPolicyError::InvalidAttributeValue(name.into());
    if element.tag() == "bdo" && attribute(element, "dir").is_none() {
        return Err(invalid("dir"));
    }
    if !matches!(element.tag(), "meter" | "progress") {
        return Ok(());
    }
    // Individual numeric spellings have already been validated.
    let get = |name, default| attribute(element, name).and_then(number).unwrap_or(default);
    let min = get("min", 0.0);
    let max = get("max", 1.0);
    if element.tag() == "progress" {
        if max <= 0.0 {
            return Err(invalid("max"));
        }
        if let Some(value) = attribute(element, "value").and_then(number)
            && !(0.0..=max).contains(&value)
        {
            return Err(invalid("value"));
        }
    } else {
        if max < min {
            return Err(invalid("max"));
        }
        let value = attribute(element, "value")
            .and_then(number)
            .ok_or_else(|| invalid("value"))?;
        if !(min..=max).contains(&value) {
            return Err(invalid("value"));
        }
        let low = get("low", min);
        let high = get("high", max);
        if !(min..=max).contains(&low) {
            return Err(invalid("low"));
        }
        if !(low..=max).contains(&high) {
            return Err(invalid("high"));
        }
        if attribute(element, "optimum")
            .and_then(number)
            .is_some_and(|n| !(min..=max).contains(&n))
        {
            return Err(invalid("optimum"));
        }
    }
    Ok(())
}
