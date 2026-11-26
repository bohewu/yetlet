//! Build-script compilation of trusted single-file components into ordinary Yetlet views.

use roxmltree::{Document, Node};
use std::{fmt::Write, fs, io, path::Path};

/// Compile the named components into an existing build output directory (usually `OUT_DIR`).
///
/// Each (ID, source path) pair produces `sfc-ID.rs`, `sfc-ID.css` and `sfc-ID.js`.
/// Give each component sharing an output directory a distinct lowercase asset ID. Include
/// each Rust file in its own module for its `Props`, `render` and `STYLE`/`SCRIPT` pairs
/// (filename, contents). The caller owns URL paths, serving, and document references.
/// Source paths are registered with Cargo for build-script change detection.
pub fn compile(components: &[(&str, &str)], out_dir: impl AsRef<Path>) -> io::Result<()> {
    let mut ids = std::collections::BTreeSet::new();
    for &(id, path) in components {
        if !id.starts_with(|c: char| c.is_ascii_lowercase())
            || !id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || !ids.insert(id)
        {
            return Err(io::Error::other(format!(
                "{path}: invalid or duplicate component ID {id:?}"
            )));
        }
        if path.contains(['\r', '\n']) {
            return Err(io::Error::other(
                "component path must fit on one Cargo directive line",
            ));
        }
        println!("cargo:rerun-if-changed={path}");
        let source = fs::read_to_string(path)
            .map_err(|e| io::Error::new(e.kind(), format!("{path}: {e}")))?;
        let input = Input {
            path,
            source: &source,
        };
        let (template, style, script) = input.sections()?;
        let rust = input.generate(id, template)?;
        for (extension, contents) in [("css", style), ("js", script), ("rs", &rust)] {
            fs::write(
                out_dir.as_ref().join(format!("sfc-{id}.{extension}")),
                contents,
            )?;
        }
    }
    Ok(())
}

struct Input<'a> {
    path: &'a str,
    source: &'a str,
}

impl<'a> Input<'a> {
    fn error(&self, offset: usize, message: impl std::fmt::Display) -> io::Error {
        let prefix = &self.source[..offset];
        let line = prefix.bytes().filter(|&c| c == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}:{line}:{column}: {message}", self.path),
        )
    }

    // Delimit raw blocks on complete lines, so CSS/JS never passes through an XML parser.
    // Retain the original template byte offset for authored-file diagnostics, in any order.
    fn sections(&self) -> io::Result<(std::ops::Range<usize>, &'a str, &'a str)> {
        let mut template = None;
        let mut style = None;
        let mut script = None;
        let mut offset = 0;
        let mut lines = self.source.split_inclusive('\n');
        while let Some(line) = lines.next() {
            let start = offset;
            offset += line.len();
            let tag = line.trim();
            if tag.is_empty() {
                continue;
            }
            let closing = match tag {
                "<style>" if style.is_none() => "</style>",
                "<script>" if script.is_none() => "</script>",
                t if template.is_none()
                    && (t == "<template>"
                        || t.strip_prefix("<template").is_some_and(|tail| {
                            tail.starts_with(char::is_whitespace) && tail.ends_with('>')
                        })) =>
                {
                    "</template>"
                }
                _ => {
                    return Err(self.error(
                        start + line.len() - line.trim_start().len(),
                        "expected one opening <template>, <style> or <script> tag per section",
                    ));
                }
            };
            let content = offset;
            loop {
                let Some(line) = lines.next() else {
                    return Err(self.error(start, format!("missing {closing} on its own line")));
                };
                let end = offset;
                offset += line.len();
                if line.trim() == closing {
                    match closing {
                        "</style>" => style = Some(&self.source[content..end]),
                        "</script>" => script = Some(&self.source[content..end]),
                        _ => template = Some(start..offset),
                    }
                    break;
                }
            }
        }
        match (template, style, script) {
            (Some(template), Some(style), Some(script)) => Ok((template, style, script)),
            _ => Err(self.error(offset, "expected one template, style and script section")),
        }
    }

    fn generate(&self, id: &str, template: std::ops::Range<usize>) -> io::Result<String> {
        let start = template.start;
        let xml = &self.source[template];
        let doc = Document::parse(xml).map_err(|e| {
            let line =
                self.source[..start].bytes().filter(|&b| b == b'\n').count() + e.pos().row as usize;
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}:{line}:{}: {e}", self.path, e.pos().col),
            )
        })?;
        let template = doc.root_element();
        let mut props = std::collections::BTreeMap::new();
        for attr in template.attributes() {
            let name = attr.name();
            if !name.starts_with(|c: char| c.is_ascii_lowercase())
                || !name
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
                || matches!(name, "self" | "super" | "crate")
            {
                return Err(self.error(
                    start + attr.range().start,
                    format!("invalid prop name {name:?}"),
                ));
            }
            let ty = match attr.value() {
                "text" => "&'a str",
                "bool" => "bool",
                _ => {
                    return Err(self.error(
                        start + attr.range_value().start,
                        "expected prop type text or bool",
                    ));
                }
            };
            props.insert(name, ty);
        }
        let mut rust = String::new();
        let lifetime = if props.values().any(|&t| t == "&'a str") {
            "<'a>"
        } else {
            ""
        };
        writeln!(rust, "pub struct Props{lifetime} {{").unwrap();
        for (name, ty) in &props {
            writeln!(rust, "pub r#{name}: {ty},").unwrap();
        }
        rust.push_str("}\n");
        let borrow = if lifetime.is_empty() { "" } else { "<'_>" };
        writeln!(
            rust,
            "pub fn render(_props: &Props{borrow}) -> ::yetlet::View {{"
        )
        .unwrap();
        let mut root = None;
        for node in template.children() {
            if node.is_element() && root.is_none() {
                root = Some(node);
            } else if !(node.is_comment()
                || node.is_text() && node.text().unwrap().trim().is_empty())
            {
                return Err(self.error(
                    start + node.range().start,
                    "expected one template root element",
                ));
            }
        }
        let root = root.ok_or_else(|| self.error(start, "missing template root element"))?;
        if let Some(attr) = root.attribute_node("when") {
            return Err(self.error(start + attr.range().start, "when needs a parent element"));
        }
        if let Some(node) = template.descendants().find(|n| {
            n.namespaces().next().is_some() || n.attributes().any(|a| a.namespace().is_some())
        }) {
            return Err(self.error(
                start + node.range().start,
                "template namespaces are unsupported",
            ));
        }
        let emitter = Emitter {
            input: self,
            start,
            props,
        };
        emitter.element(root, &mut rust)?;
        rust.push_str(".into()\n}\n");
        for (kind, ext) in [("STYLE", "css"), ("SCRIPT", "js")] {
            let file = format!("sfc-{id}.{ext}");
            writeln!(
                rust,
                "pub const {kind}: (&str, &str) = ({file:?}, include_str!({file:?}));"
            )
            .unwrap();
        }
        Ok(rust)
    }
}

struct Emitter<'a, 's> {
    input: &'a Input<'s>,
    start: usize,
    props: std::collections::BTreeMap<&'a str, &'static str>,
}

impl Emitter<'_, '_> {
    fn binding(&self, name: &str, boolean: bool, offset: impl Fn() -> usize) -> io::Result<String> {
        let ty = self
            .props
            .get(name)
            .ok_or_else(|| self.input.error(offset(), format!("unknown prop {name:?}")))?;
        if (*ty == "bool") != boolean {
            return Err(self.input.error(
                offset(),
                format!(
                    "expected {} prop at {name:?}",
                    if boolean { "bool" } else { "text" }
                ),
            ));
        }
        Ok(format!("_props.r#{name}"))
    }

    // Map a prefix of the decoded XML value back to its authored extent. Without a DTD,
    // each reference contributes one character. Adjacent CDATA is part of the same value.
    fn value_offset(&self, start: usize, prefix: &str) -> usize {
        let mut offset = self.start + start;
        let mut chars = prefix.chars();
        let mut cdata = false;
        loop {
            let raw = &self.input.source[offset..];
            if !cdata && raw.starts_with("<![CDATA[") {
                offset += "<![CDATA[".len();
                cdata = true;
            } else if cdata && raw.starts_with("]]>") {
                offset += "]]>".len();
                cdata = false;
            } else if chars.next().is_none() {
                return offset;
            } else if !cdata && raw.starts_with('&') {
                offset += raw.find(';').unwrap() + 1;
            } else if raw.starts_with("\r\n") {
                offset += 2;
            } else {
                offset += raw.chars().next().unwrap().len_utf8();
            }
        }
    }

    // Each text/attribute value accepts one {text_prop}, with optional surrounding literals.
    fn value(&self, value: &str, start: usize) -> io::Result<String> {
        let offset = |index| self.value_offset(start, &value[..index]);
        let mut braces = value.match_indices(['{', '}']);
        let open = match braces.next() {
            Some((i, "{")) => i,
            Some((i, _)) => return Err(self.input.error(offset(i), "unmatched closing brace")),
            None => return Ok(format!("{value:?}")),
        };
        let close = match braces.next() {
            Some((i, "}")) => i,
            Some((i, _)) => return Err(self.input.error(offset(i), "unexpected opening brace")),
            None => return Err(self.input.error(offset(open), "unclosed prop binding")),
        };
        if let Some((extra, _)) = braces.next() {
            return Err(self.input.error(
                offset(extra),
                "expected one prop binding per text or attribute value",
            ));
        }
        let before = &value[..open];
        let after = &value[close + 1..];
        let prop = self.binding(&value[open + 1..close], false, || offset(open))?;
        if before.is_empty() && after.is_empty() {
            Ok(prop)
        } else {
            Ok(format!(
                "::std::format!(\"{{}}{{}}{{}}\", {before:?}, {prop}, {after:?})"
            ))
        }
    }

    fn element(&self, node: Node, rust: &mut String) -> io::Result<()> {
        let tag = node.tag_name().name();
        let attributes = || node.attributes().filter(|attr| attr.name() != "when");
        if attributes().next().is_none() && node.children().all(|child| child.is_comment()) {
            write!(rust, "::yetlet::Element::new({tag:?})").unwrap();
            return Ok(());
        }
        writeln!(rust, "{{ let mut el = ::yetlet::Element::new({tag:?});").unwrap();
        for attr in attributes() {
            let name = attr.name();
            let value = self.value(attr.value(), attr.range_value().start)?;
            writeln!(rust, "el = el.attribute({name:?}, {value});").unwrap();
        }
        for child in node.children() {
            if child.is_text() {
                let value = self.value(child.text().unwrap(), child.range().start)?;
                writeln!(rust, "el = el.child(::yetlet::View::text({value}));").unwrap();
            } else if child.is_element() {
                let when = child.attribute_node("when");
                if let Some(attr) = when {
                    let condition =
                        self.binding(attr.value(), true, || self.start + attr.range_value().start)?;
                    write!(rust, "if {condition} {{ ").unwrap();
                }
                rust.push_str("el = el.child(");
                self.element(child, rust)?;
                rust.push_str(");\n");
                if when.is_some() {
                    rust.push_str("}\n");
                }
            } else if !child.is_comment() {
                return Err(self.input.error(
                    self.start + child.range().start,
                    "unexpected template instruction",
                ));
            }
        }
        rust.push_str("el }");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &str = "<script>\nif (x < y && z) f();\n</script>\n<style>\na > b {}\n</style>\n";

    fn output(template: &str) -> io::Result<String> {
        source_output(&format!("{RAW}{template}"))
    }

    fn source_output(source: &str) -> io::Result<String> {
        let input = Input {
            path: "src/Card.yetlet",
            source,
        };
        let (template, style, script) = input.sections()?;
        assert_eq!(style, "a > b {}\n");
        assert_eq!(script, "if (x < y && z) f();\n");
        input.generate("card", template)
    }

    #[test]
    fn locates_second_binding_on_its_own_line() {
        let template = "<template label='text'>\n<p>{label}\n{extra}</p>\n</template>\n";
        let error = source_output(&format!("{template}{RAW}")).unwrap_err();
        assert_eq!(
            error.to_string(),
            "src/Card.yetlet:3:1: expected one prop binding per text or attribute value"
        );
    }

    #[test]
    fn locates_binding_at_authored_character_reference() {
        let template = "<template>\n<p>Line 1\nBad prop &#123;typo}</p>\n</template>\n";
        let error = source_output(&format!("{template}{RAW}")).unwrap_err();
        assert_eq!(
            error.to_string(),
            "src/Card.yetlet:3:10: unknown prop \"typo\""
        );
    }

    #[test]
    fn locates_unknown_binding_after_entities_and_multibyte_text() {
        let error =
            output("<template>\n<p title='Café &amp; {missing}'/>\n</template>").unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("src/Card.yetlet:8:22: unknown prop"),
            "{error}"
        );
    }

    #[test]
    fn locates_malformed_markup_at_authored_line() {
        let error = output("<template>\n<div>\n  </span>\n</template>").unwrap_err();
        assert!(
            error.to_string().starts_with("src/Card.yetlet:9:3:"),
            "{error}"
        );
    }

    #[test]
    fn emits_typed_props_escaped_rust_literals_and_external_assets() {
        let rust = output("<template type=\"text\" visible=\"bool\">\n<p title='&quot;\\'>{type}<span when='visible'>OK</span><br/></p>\n</template>").unwrap();
        assert!(rust.contains("pub r#type: &'a str,"));
        assert!(rust.contains("if _props.r#visible"));
        assert!(rust.contains(r#"attribute("title", "\"\\")"#));
        assert!(rust.contains("::yetlet::View::text(_props.r#type)"));
        assert!(rust.contains(r#"el.child(::yetlet::Element::new("br"))"#));
        assert!(rust.contains(r#"("sfc-card.css", include_str!("sfc-card.css"))"#));
    }

    #[test]
    fn rejects_wrong_prop_type_and_invalid_names() {
        assert!(
            output("<template label='text'>\n<p><b when='label'/></p>\n</template>")
                .unwrap_err()
                .to_string()
                .contains("expected bool prop")
        );
        assert!(
            output("<template self='text'>\n<p/>\n</template>")
                .unwrap_err()
                .to_string()
                .contains("invalid prop name")
        );
        assert!(
            compile(&[("../bad", "Card.yetlet")], ".")
                .unwrap_err()
                .to_string()
                .contains("invalid or duplicate component ID")
        );
    }
}
