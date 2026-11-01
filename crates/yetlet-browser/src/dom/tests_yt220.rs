use super::{DomPolicyError, validate};
use yetlet::view::{Element, View};

fn invalid_children(view: Element) {
    assert!(matches!(
        validate(&view.into()),
        Err(DomPolicyError::InvalidChildren(_))
    ));
}

#[test]
fn yt220_ordinary_content_and_reserved_boundaries() {
    for tag in [
        "abbr",
        "address",
        "b",
        "bdi",
        "blockquote",
        "canvas",
        "cite",
        "data",
        "del",
        "dfn",
        "dialog",
        "figure",
        "hgroup",
        "i",
        "ins",
        "kbd",
        "mark",
        "menu",
        "output",
        "pre",
        "progress",
        "q",
        "rp",
        "rt",
        "ruby",
        "s",
        "samp",
        "search",
        "sub",
        "sup",
        "time",
        "u",
        "var",
        "wbr",
    ] {
        assert_eq!(validate(&Element::new(tag).into()), Ok(()), "{tag}");
    }
    assert_eq!(
        validate(&Element::new("bdo").attribute("dir", "rtl").into()),
        Ok(())
    );
    assert!(validate(&Element::new("bdo").into()).is_err());
    for tag in [
        "img",
        "picture",
        "source",
        "audio",
        "video",
        "track",
        "map",
        "area",
        "html",
        "head",
        "body",
        "title",
        "meta",
        "link",
        "base",
        "script",
        "style",
        "iframe",
        "object",
        "embed",
        "template",
        "slot",
        "noscript",
        "selectedcontent",
        "svg",
        "math",
        "custom-card",
        "marquee",
    ] {
        assert!(
            matches!(
                validate(&Element::new(tag).into()),
                Err(DomPolicyError::UnsupportedTag(_))
            ),
            "{tag}"
        );
    }
    for name in [
        "style",
        "onclick",
        "src",
        "srcset",
        "cite",
        "poster",
        "autofocus",
        "is",
        "data-yetlet-owner",
        "data-arbitrary",
        "aria-arbitrary",
        "popover",
    ] {
        assert!(
            matches!(
                validate(&Element::new("div").attribute(name, "x").into()),
                Err(DomPolicyError::UnsupportedAttribute(_))
            ),
            "{name}"
        );
    }
}

#[test]
fn yt220_table_shape_and_span_limits() {
    let table = Element::new("table")
        .child(Element::new("caption").child(View::text("Caption")))
        .child(Element::new("colgroup").child(Element::new("col").attribute("span", "2")))
        .child(
            Element::new("thead").child(
                Element::new("tr").child(
                    Element::new("th")
                        .attribute("scope", "col")
                        .child(View::text("Name")),
                ),
            ),
        )
        .child(
            Element::new("tbody").child(
                Element::new("tr").child(
                    Element::new("td")
                        .attribute("rowspan", "0")
                        .attribute("colspan", "1000")
                        .child(View::text("Cell")),
                ),
            ),
        )
        .child(
            Element::new("tfoot")
                .child(Element::new("tr").child(Element::new("td").child(View::text("End")))),
        );
    assert_eq!(validate(&table.into()), Ok(()));
    for view in [
        Element::new("table").child(Element::new("tr")),
        Element::new("table")
            .child(Element::new("thead"))
            .child(Element::new("thead")),
        Element::new("table")
            .child(Element::new("tbody"))
            .child(Element::new("caption")),
        Element::new("table")
            .child(Element::new("tfoot"))
            .child(Element::new("tbody")),
        Element::new("table").child(View::text("\u{a0}")),
        Element::new("table").child(Element::new("tbody").child(View::text("foster text"))),
        Element::new("table").child(
            Element::new("colgroup")
                .attribute("span", "1")
                .child(Element::new("col")),
        ),
        Element::new("div").child(Element::new("td")),
    ] {
        invalid_children(view);
    }
    for (name, value) in [
        ("colspan", "0"),
        ("colspan", "1001"),
        ("rowspan", "65535"),
        ("rowspan", "-1"),
    ] {
        let view = Element::new("table").child(
            Element::new("tbody")
                .child(Element::new("tr").child(Element::new("td").attribute(name, value))),
        );
        assert_eq!(
            validate(&view.into()),
            Err(DomPolicyError::InvalidAttributeValue(name.into()))
        );
    }
    for tag in ["col", "wbr"] {
        let leaf = Element::new(tag).child(View::text("invalid"));
        let view = if tag == "col" {
            Element::new("table").child(Element::new("colgroup").child(leaf))
        } else {
            leaf
        };
        assert!(matches!(
            validate(&view.into()),
            Err(DomPolicyError::VoidElementHasChildren(_))
        ));
    }
}

#[test]
fn yt220_disclosure_figure_and_classic_controls() {
    let view = Element::new("section")
        .child(
            Element::new("details")
                .attribute("open", "false")
                .child(Element::new("summary").child(View::text("Read"))),
        )
        .child(
            Element::new("figure")
                .child(Element::new("pre").child(View::text("code")))
                .child(Element::new("figcaption").child(View::text("Caption"))),
        )
        .child(
            Element::new("select").attribute("multiple", "false").child(
                Element::new("optgroup")
                    .attribute("label", "Group")
                    .attribute("disabled", "false")
                    .child(Element::new("option").child(View::text("One"))),
            ),
        )
        .child(
            Element::new("datalist").child(
                Element::new("option")
                    .attribute("label", "Two")
                    .child(View::text("2")),
            ),
        );
    assert_eq!(validate(&view.into()), Ok(()));
    for view in [
        Element::new("details"),
        Element::new("details")
            .child(View::text("Before"))
            .child(Element::new("summary")),
        Element::new("details")
            .child(Element::new("summary"))
            .child(Element::new("summary")),
        Element::new("div").child(Element::new("summary")),
        Element::new("figure")
            .child(View::text("Before"))
            .child(Element::new("figcaption"))
            .child(View::text("After")),
        Element::new("figure")
            .child(Element::new("figcaption"))
            .child(Element::new("figcaption")),
        Element::new("datalist").child(Element::new("span")),
        Element::new("select").child(Element::new("optgroup").child(Element::new("div"))),
    ] {
        invalid_children(view);
    }
}

#[test]
fn yt220_attribute_scopes_keywords_and_legacy_values() {
    for (tag, name, value) in [
        ("div", "contenteditable", "plaintext-only"),
        ("div", "inert", "false"),
        ("input", "min", "2026-01-01"),
        ("input", "step", "any"),
        ("input", "minlength", "0"),
        ("input", "pattern", "[a-z]+"),
        ("input", "form", "owned-form"),
        ("output", "for", "a b"),
        ("form", "enctype", "multipart/form-data"),
        ("form", "novalidate", "false"),
        ("textarea", "wrap", "hard"),
        ("dialog", "open", "false"),
        ("ol", "start", "-2"),
        ("li", "value", "-3"),
        ("canvas", "width", "640"),
        ("time", "datetime", "2026-10-31"),
        ("data", "value", "record-a"),
        // Existing global admission and native boolean presence are unchanged.
        ("div", "type", "arbitrary"),
        ("div", "for", "anything"),
        ("input", "maxlength", "historically-unchecked"),
        ("input", "required", "false"),
    ] {
        assert_eq!(
            validate(&Element::new(tag).attribute(name, value).into()),
            Ok(()),
            "{tag}/{name}"
        );
    }
    for (tag, name, value) in [
        ("pre", "for", "x"),
        ("div", "multiple", ""),
        ("div", "datetime", "x"),
        ("div", "width", "2"),
        ("input", "formaction", "/other"),
    ] {
        assert_eq!(
            validate(&Element::new(tag).attribute(name, value).into()),
            Err(DomPolicyError::UnsupportedAttribute(name.into()))
        );
    }
    for (tag, name, value) in [
        ("div", "contenteditable", "script"),
        ("div", "draggable", "auto"),
        ("input", "minlength", "-1"),
        ("select", "size", "0"),
        ("textarea", "wrap", "invalid"),
        ("li", "value", "+1"),
        ("canvas", "width", "4294967296"),
    ] {
        assert_eq!(
            validate(&Element::new(tag).attribute(name, value).into()),
            Err(DomPolicyError::InvalidAttributeValue(name.into()))
        );
    }
}

#[test]
fn yt220_meter_progress_finite_values_and_ordering() {
    for view in [
        Element::new("progress"),
        Element::new("progress")
            .attribute("max", "10")
            .attribute("value", "1e1"),
        Element::new("meter")
            .attribute("value", "-.2")
            .attribute("min", "-1")
            .attribute("max", "0"),
        Element::new("meter")
            .attribute("value", "1")
            .attribute("min", "1")
            .attribute("max", "1"),
        Element::new("meter")
            .attribute("value", ".5")
            .attribute("low", ".25")
            .attribute("high", ".75")
            .attribute("optimum", ".5"),
    ] {
        assert_eq!(validate(&view.into()), Ok(()));
    }
    for view in [
        Element::new("meter"),
        Element::new("progress").attribute("max", "0"),
        Element::new("progress").attribute("value", "2"),
        Element::new("meter")
            .attribute("value", ".5")
            .attribute("low", ".8")
            .attribute("high", ".2"),
        Element::new("meter")
            .attribute("value", ".5")
            .attribute("optimum", "2"),
    ] {
        assert!(matches!(
            validate(&view.into()),
            Err(DomPolicyError::InvalidAttributeValue(_))
        ));
    }
    for value in ["NaN", "inf", "+.5", "1.", "1e", "1e999", " .5", ".5 "] {
        assert!(
            validate(&Element::new("progress").attribute("value", value).into()).is_err(),
            "{value}"
        );
    }
}
