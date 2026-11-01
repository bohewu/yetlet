//! Application-owned CommonMark events -> escaped Yetlet Views. No raw HTML sink.
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::{collections::BTreeMap, iter::Peekable};
use yetlet::view::{Element, View};

pub struct Chapter {
    pub view: View,
    pub text: String,
}
#[derive(Default)]
struct Context {
    text: String,
    ids: BTreeMap<String, usize>,
}

pub fn render(markdown: &str) -> Result<Chapter, String> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let mut events = Parser::new_ext(markdown, options).peekable();
    let mut context = Context::default();
    let children = nodes(&mut events, None, &mut context, false)?;
    Ok(Chapter {
        view: Element::new("article")
            .attribute("class", "prose")
            .children(children)
            .into(),
        text: context
            .text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
    })
}

fn nodes<'a>(
    events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
    end: Option<TagEnd>,
    context: &mut Context,
    header: bool,
) -> Result<Vec<View>, String> {
    let mut output = Vec::new();
    while let Some(event) = events.next() {
        let view = match event {
            Event::End(actual) if Some(actual) == end => return Ok(output),
            Event::End(_) => return Err("Unexpected Markdown container end".into()),
            Event::Text(text) => {
                context.text.push_str(&text);
                View::text(text.into_string())
            }
            Event::Code(text) => {
                context.text.push_str(&text);
                Element::new("code")
                    .child(View::text(text.into_string()))
                    .into()
            }
            Event::SoftBreak => {
                context.text.push(' ');
                View::text("\n")
            }
            Event::HardBreak => {
                context.text.push(' ');
                Element::new("br").into()
            }
            Event::Rule => Element::new("hr").into(),
            Event::Html(_) | Event::InlineHtml(_) => {
                return Err("Raw HTML is not accepted by this pilot".into());
            }
            Event::Start(tag) => {
                let end = tag.to_end();
                let before = context.text.len();
                let element = match tag {
                    Tag::HtmlBlock
                    | Tag::Image { .. }
                    | Tag::FootnoteDefinition(_)
                    | Tag::DefinitionList
                    | Tag::DefinitionListTitle
                    | Tag::DefinitionListDefinition
                    | Tag::MetadataBlock(_)
                    | Tag::Superscript
                    | Tag::Subscript => {
                        return Err(
                            "Unsupported Markdown construct; content was not exported".into()
                        );
                    }
                    Tag::Heading { level, .. } => {
                        let children = nodes(events, Some(end), context, header)?;
                        let stem: String = context.text[before..]
                            .trim()
                            .to_lowercase()
                            .chars()
                            .filter_map(|c| {
                                if c.is_whitespace() {
                                    Some('-')
                                } else if c.is_alphanumeric() || matches!(c, '-' | '_') {
                                    Some(c)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let stem = if stem.is_empty() {
                            "section".into()
                        } else {
                            stem
                        };
                        let count = context.ids.entry(stem.clone()).or_default();
                        let id = if *count == 0 {
                            stem
                        } else {
                            format!("{stem}-{count}")
                        };
                        *count += 1;
                        context.text.push(' ');
                        let tag = match level as u8 {
                            1 => "h1",
                            2 => "h2",
                            3 => "h3",
                            4 => "h4",
                            5 => "h5",
                            _ => "h6",
                        };
                        Element::new(tag).attribute("id", id).children(children)
                    }
                    Tag::Table(alignments) => {
                        if alignments
                            .iter()
                            .any(|a| *a != pulldown_cmark::Alignment::None)
                        {
                            return Err(
                                "Explicit Markdown table alignment is not selected in this pilot"
                                    .into(),
                            );
                        }
                        let mut children = nodes(events, Some(end), context, false)?.into_iter();
                        let head = children.next().ok_or("Missing table header")?;
                        Element::new("table")
                            .child(head)
                            .child(Element::new("tbody").children(children))
                    }
                    Tag::TableHead => Element::new("thead").child(
                        Element::new("tr").children(nodes(events, Some(end), context, true)?),
                    ),
                    Tag::CodeBlock(kind) => {
                        let language = match kind {
                            CodeBlockKind::Fenced(info) => {
                                info.split([',', ' ']).next().unwrap_or("").to_owned()
                            }
                            _ => String::new(),
                        };
                        let mut code = Element::new("code").children(nodes(
                            events,
                            Some(end),
                            context,
                            header,
                        )?);
                        if !language.is_empty()
                            && language.len() <= 32
                            && language
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                        {
                            code = code.attribute("class", format!("language-{language}"));
                        }
                        context.text.push(' ');
                        Element::new("pre").child(code)
                    }
                    Tag::Link {
                        dest_url, title, ..
                    } => {
                        let mut link = Element::new("a").attribute("href", dest_url.into_string());
                        if !title.is_empty() {
                            link = link.attribute("title", title.into_string());
                        }
                        link.children(nodes(events, Some(end), context, header)?)
                    }
                    tag => {
                        let block =
                            !matches!(tag, Tag::Emphasis | Tag::Strong | Tag::Strikethrough);
                        let mut element = match tag {
                            Tag::Paragraph => Element::new("p"),
                            Tag::BlockQuote(_) => Element::new("blockquote"),
                            Tag::List(Some(start)) => {
                                Element::new("ol").attribute("start", start.to_string())
                            }
                            Tag::List(None) => Element::new("ul"),
                            Tag::Item => Element::new("li"),
                            Tag::TableRow => Element::new("tr"),
                            Tag::TableCell => Element::new(if header { "th" } else { "td" }),
                            Tag::Emphasis => Element::new("em"),
                            Tag::Strong => Element::new("strong"),
                            Tag::Strikethrough => Element::new("del"),
                            _ => return Err("Unsupported Markdown tag".into()),
                        };
                        element = element.children(nodes(events, Some(end), context, header)?);
                        if block {
                            context.text.push(' ');
                        }
                        element
                    }
                };
                element.into()
            }
            _ => return Err("Unsupported Markdown event".into()),
        };
        output.push(view);
    }
    if end.is_some() {
        return Err("Unclosed Markdown container".into());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn html(input: &str) -> Result<String, String> {
        yetlet::html::render_to_string(&render(input)?.view).map_err(|e| e.to_string())
    }
    #[test]
    fn raw_html_and_active_links_have_no_escape_hatch() {
        assert!(html("<script>alert(1)</script>").is_err());
        assert!(html("x <b>raw</b>").is_err());
        assert!(html("[bad](javascript:alert%281%29)").is_err());
        let code = html("```html\n<script>alert(1)</script>\n```\n").unwrap();
        assert!(code.contains("&lt;script&gt;"));
        assert!(!code.contains("<script"));
    }
    #[test]
    fn headings_tables_and_unicode_use_semantic_views() {
        let output = html("# Hello `World` 界\n\n# Hello `World` 界\n\n| Name | Value |\n| --- | --- |\n| **Safe** | `<&>` |\n").unwrap();
        assert!(output.contains("id=\"hello-world-界\""));
        assert!(output.contains("id=\"hello-world-界-1\""));
        assert!(output.contains("<thead><tr><th>"));
        assert!(output.contains("<tbody><tr><td>"));
        assert!(output.contains("&lt;&amp;&gt;"));
    }
    #[test]
    fn unsupported_content_is_explicit_and_search_text_is_plain() {
        assert!(html("![image](https://example.invalid/a.png)").is_err());
        assert!(html("| Left |\n| :--- |\n| value |").is_err());
        let chapter = render("Read **Rust** with `View` and [guide](/yetlet/views/).").unwrap();
        assert!(chapter.text.contains("Rust"));
        assert!(chapter.text.contains("View"));
        assert!(!chapter.text.contains("**"));
    }
}
