//! YT-146 sample-only attribution and component probes; no supported renderer API.
use super::*;

#[cfg(any(
    feature = "serializer-attribution",
    feature = "escape-comparison",
    feature = "url-probes"
))]
mod url_check {
    pub(super) const URL_ATTRIBUTES: &[&str] =
        &["href", "src", "action", "formaction", "xlink:href"];

    pub(super) fn unsafe_scheme(value: &str) -> bool {
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
}

#[cfg(any(feature = "serializer-attribution", feature = "escape-comparison"))]
mod shadow {
    use super::url_check::{URL_ATTRIBUTES, unsafe_scheme};
    use super::*;
    #[cfg(feature = "serializer-attribution")]
    use crate::counts::attribution::{Bucket, tagged};
    #[cfg(not(feature = "serializer-attribution"))]
    #[derive(Clone, Copy)]
    enum Bucket {
        Body,
        Text,
        Attribute,
        Other,
        Url,
        Envelope,
        Title,
    }
    #[cfg(not(feature = "serializer-attribution"))]
    #[inline]
    fn tagged<T>(_: Bucket, work: impl FnOnce() -> T) -> T {
        work()
    }

    use std::fmt::Write as _;
    use yetlet_html::{RenderError, escape_text};

    // Source-derived from the pinned HTML serializer. Only scoped event tags added.
    pub fn render_to_string(view: &View) -> Result<String, RenderError> {
        tagged(Bucket::Body, || {
            let mut output = String::new();
            render_into(view, &mut output, false)?;
            Ok(output)
        })
    }

    fn render_into(view: &View, output: &mut String, inside_form: bool) -> Result<(), RenderError> {
        match view {
            View::Text(value) => output.push_str(&tagged(Bucket::Text, || escape_text(value))),
            View::Element(element) => {
                let tag = element.tag();
                if !valid_identifier(tag) || FORBIDDEN_TAGS.contains(&tag) {
                    return tagged(Bucket::Other, || {
                        Err(RenderError::InvalidTag(tag.to_owned()))
                    });
                }

                if (tag == "form" && inside_form)
                    || (matches!(tag, "textarea" | "option")
                        && element
                            .child_views()
                            .iter()
                            .any(|v| !matches!(v, View::Text(_))))
                    || (tag == "select" && element.child_views().iter().any(|v| {
                        !matches!(v, View::Element(e) if matches!(e.tag(), "option" | "optgroup"))
                            && !matches!(v, View::Text(s) if s.trim().is_empty())
                    }))
                {
                    return tagged(Bucket::Other, || {
                        Err(RenderError::InvalidChildren(tag.into()))
                    });
                }
                write!(output, "<{tag}").expect("writing to String is infallible");
                for (index, attribute) in element.attributes().iter().enumerate() {
                    let name = attribute.name;
                    if element.attributes()[..index].iter().any(|a| a.name == name) {
                        return tagged(Bucket::Other, || {
                            Err(RenderError::DuplicateAttribute(name.into()))
                        });
                    }
                    if !valid_identifier(name) {
                        return tagged(Bucket::Other, || {
                            Err(RenderError::InvalidAttribute(name.to_owned()))
                        });
                    }
                    if name.starts_with("on") || matches!(name, "style" | "srcdoc") {
                        return tagged(Bucket::Other, || {
                            Err(RenderError::ForbiddenAttribute(name.to_owned()))
                        });
                    }
                    if URL_ATTRIBUTES.contains(&name)
                        && tagged(Bucket::Url, || unsafe_scheme(&attribute.value))
                    {
                        return tagged(Bucket::Other, || {
                            Err(RenderError::UnsafeUrl(name.to_owned()))
                        });
                    }

                    write!(
                        output,
                        " {name}=\"{}\"",
                        tagged(Bucket::Attribute, || escape_text(&attribute.value))
                    )
                    .expect("writing to String is infallible");
                }
                output.push('>');

                if VOID_ELEMENTS.contains(&tag) {
                    if !element.child_views().is_empty() {
                        return tagged(Bucket::Other, || {
                            Err(RenderError::VoidElementHasChildren(tag.to_owned()))
                        });
                    }
                    return Ok(());
                }
                if tag == "textarea" {
                    let (_text, normalized) = tagged(Bucket::Other, || {
                        let text: String = element
                            .child_views()
                            .iter()
                            .filter_map(|v| match v {
                                View::Text(s) => Some(s.as_str()),
                                _ => None,
                            })
                            .collect();
                        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                        (text, normalized)
                    });
                    // HTML parsing discards one LF immediately after the start tag.
                    if normalized.starts_with('\n') {
                        output.push('\n');
                    }
                    output.push_str(&tagged(Bucket::Text, || escape_text(&normalized)));
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
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];

    fn valid_identifier(identifier: &str) -> bool {
        let mut bytes = identifier.bytes();
        matches!(bytes.next(), Some(b'a'..=b'z'))
            && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    }

    // Source-derived adapter composition; retains the body temporary until append.
    pub fn render_document_with_metadata(
        metadata: &DocumentMetadata,
        view: &View,
    ) -> Result<String, RenderError> {
        tagged(Bucket::Envelope, || {
            let mut html = String::from("<!doctype html><html lang=\"");
            html.push_str(metadata.language().as_str());
            html.push_str("\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>");
            html.push_str(&tagged(Bucket::Title, || escape_text(metadata.title())));
            html.push_str("</title></head><body>");
            html.push_str(&render_to_string(view)?);
            html.push_str("</body></html>");
            Ok(html)
        })
    }
}

#[cfg(feature = "serializer-attribution")]
pub fn output(directory: &Path) -> Result<(), Box<dyn Error>> {
    for (name, rows) in CASES {
        let (metadata, view) = page(rows);
        let body = shadow::render_to_string(&view)?;
        let document = shadow::render_document_with_metadata(&metadata, &view)?;
        let oracle = expected(rows);
        assert_eq!(body, oracle.0);
        assert_eq!(document, oracle.1);
        assert_eq!(body, yetlet_html::render_to_string(&view)?);
        assert_eq!(document, render_document_with_metadata(&metadata, &view)?);
        std::fs::write(directory.join(format!("{name}-body.html")), body)?;
        std::fs::write(directory.join(format!("{name}-document.html")), document)?;
    }
    Ok(())
}

#[cfg(feature = "serializer-attribution")]
pub fn allocation_counts(phase: &str) -> Result<(), Box<dyn Error>> {
    use counts::attribution as tags;
    if !matches!(phase, "body" | "document") {
        return Err("expected body|document".into());
    }
    let (metadata, view) = page(1024);
    let mut records = [(
        counts::snapshot(),
        counts::snapshot(),
        counts::snapshot(),
        tags::snapshot(),
        tags::snapshot(),
    ); 3];
    for record in &mut records {
        // Only the outer operation resets PEAK. Tags never call begin().
        let start = counts::begin();
        let bucket_start = tags::snapshot();
        let output = black_box(if phase == "body" {
            shadow::render_to_string(black_box(&view))?
        } else {
            shadow::render_document_with_metadata(black_box(&metadata), black_box(&view))?
        });
        let held = counts::snapshot();
        let bucket_held = tags::snapshot();
        drop(output);
        let released = counts::snapshot();
        *record = (start, held, released, bucket_start, bucket_held);
    }
    println!(
        "kind,phase,operation,point,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (operation, (start, held, released, _, _)) in records.iter().enumerate() {
        for (point, s) in [("start", start), ("held", held), ("released", released)] {
            println!(
                "global,{phase},{operation},{point},{},{},{},{},{},{},{},{},{},{},{},{}",
                s.allocations,
                s.zeroed,
                s.reallocations,
                s.deallocations,
                s.allocated_bytes,
                s.realloc_old_bytes,
                s.realloc_new_bytes,
                s.deallocated_bytes,
                s.failed_allocations,
                s.failed_reallocations,
                s.live,
                s.peak
            );
        }
    }
    println!(
        "kind,phase,operation,point,bucket,allocations,zeroed,reallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,failed_allocations,failed_reallocations"
    );
    for (operation, (_, _, _, start, held)) in records.iter().enumerate() {
        for (b, name) in tags::NAMES.iter().enumerate() {
            let d: [usize; 8] = std::array::from_fn(|f| held[b][f] - start[b][f]);
            println!(
                "bucket,{phase},{operation},creation,{name},{},{},{},{},{},{},{},{}",
                d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]
            );
        }
    }
    // All raw observations are emitted before consistency assertions.
    for (start, held, released, bucket_start, bucket_held) in &records {
        let sum: [usize; 8] =
            std::array::from_fn(|f| (0..7).map(|b| bucket_held[b][f] - bucket_start[b][f]).sum());
        let global = [
            held.allocations - start.allocations,
            held.zeroed - start.zeroed,
            held.reallocations - start.reallocations,
            held.allocated_bytes - start.allocated_bytes,
            held.realloc_old_bytes - start.realloc_old_bytes,
            held.realloc_new_bytes - start.realloc_new_bytes,
            held.failed_allocations - start.failed_allocations,
            held.failed_reallocations - start.failed_reallocations,
        ];
        assert_eq!(sum, global, "bucket/global reconciliation");
        assert_eq!(
            bucket_held[6], bucket_start[6],
            "unclassified creation calls"
        );
        assert_eq!(released.live, start.live, "release balance");
        assert_eq!(released.failed_allocations, start.failed_allocations);
        assert_eq!(released.failed_reallocations, start.failed_reallocations);
    }
    drop(view);
    drop(metadata);
    Ok(())
}

#[cfg(all(feature = "component-probes", not(feature = "allocation-counts")))]
fn corpus<'a>(title: &'a str, view: &'a View) -> Vec<&'a str> {
    fn visit<'a>(view: &'a View, result: &mut Vec<&'a str>) {
        match view {
            View::Text(text) => result.push(text),
            View::Element(element) => {
                result.extend(element.attributes().iter().map(|a| a.value.as_str()));
                for child in element.child_views() {
                    visit(child, result);
                }
            }
        }
    }
    let mut result = Vec::with_capacity(6150);
    result.push(title);
    visit(view, &mut result);
    result
}

// Fixed escaped literals for the component corpus; no general escape algorithm.
#[cfg(all(feature = "component-probes", not(feature = "allocation-counts")))]
fn expected_corpus() -> Vec<String> {
    let mut result = vec![
        "YT145 public report &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;".into(),
        "ssr-report".into(),
        "SSR report &lt;&amp;&gt; &quot;title&quot; &#39;café&#39;".into(),
        "Public synthetic rows".into(),
        "Records".into(),
    ];
    for i in 0..1024 {
        result.extend([
            format!("{i:04}"),
            format!("Row {i:04} &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;"),
            format!("Record {i:04}"),
            format!("Public &lt;&amp;&gt; &quot;value&quot; &#39;café&#39; {i:04}"),
            format!("/records/{i:04}?view=full&amp;public=1"),
            "Open record".into(),
        ]);
    }
    result.push("End of public report".into());
    result
}

// Exact adapter append order, replacing only render_to_string(view) with body.
#[cfg(all(feature = "component-probes", not(feature = "allocation-counts")))]
fn envelope(metadata: &DocumentMetadata, body: &str) -> String {
    let mut html = String::from("<!doctype html><html lang=\"");
    html.push_str(metadata.language().as_str());
    html.push_str("\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>");
    html.push_str(&yetlet_html::escape_text(metadata.title()));
    html.push_str("</title></head><body>");
    html.push_str(body);
    html.push_str("</body></html>");
    html
}

#[cfg(feature = "component-probes")]
pub fn component_timing(repetition: usize) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("component timing requires an ordinary build without allocation-counts".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let (metadata, view) = page(1024);
        let inputs = corpus(metadata.title(), &view);
        let body =
            std::fs::read_to_string("target/yt145-ssr-cost-vpx_z1d5/rendered/large-body.html")?;
        let oracle = expected(1024);
        assert_eq!(body, oracle.0);
        let expected = expected_corpus();
        assert_eq!(inputs.len(), 6150);
        for (input, expected) in inputs.iter().zip(&expected) {
            assert_eq!(yetlet_html::escape_text(input), *expected);
        }
        assert_eq!(envelope(&metadata, &body), oracle.1);
        let input_bytes: usize = inputs.iter().map(|s| s.len()).sum();
        let escaped_bytes: usize = expected.iter().map(String::len).sum();
        eprintln!(
            "corpus_calls={} input_bytes={input_bytes} escaped_bytes={escaped_bytes} body_append_bytes={} document_bytes={}",
            inputs.len(),
            body.len(),
            oracle.1.len()
        );
        let mut probes = ["escape-corpus", "envelope"];
        if repetition == 2 {
            probes.reverse();
        }
        let mut observations = Vec::with_capacity(25);
        println!("repetition,sequence,probe,population,index,elapsed_ns,result_len");
        let mut sequence = 0;
        for probe in probes {
            for index in 0..25 {
                let started = std::time::Instant::now();
                let (elapsed, result_len) = if probe == "escape-corpus" {
                    let mut total = 0;
                    for input in black_box(&inputs) {
                        let output = black_box(yetlet_html::escape_text(black_box(input)));
                        total += black_box(output.len());
                        drop(output);
                    }
                    let result = black_box(total);
                    (started.elapsed().as_nanos(), result)
                } else {
                    let output = black_box(envelope(black_box(&metadata), black_box(&body)));
                    let elapsed = started.elapsed().as_nanos();
                    let len = output.len();
                    drop(output);
                    (elapsed, len)
                };
                observations.push((index, elapsed, result_len));
            }
            for (index, elapsed, len) in observations.drain(..) {
                let population = if index < 5 { "warmup" } else { "measured" };
                let index = if index < 5 { index } else { index - 5 };
                println!("{repetition},{sequence},{probe},{population},{index},{elapsed},{len}");
                sequence += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serializer-attribution")]
    #[test]
    fn yt146_shadow_matches_all_fixed_oracles() {
        for (_, rows) in CASES {
            let (metadata, view) = page(rows);
            let oracle = expected(rows);
            assert_eq!(shadow::render_to_string(&view).unwrap(), oracle.0);
            assert_eq!(
                shadow::render_document_with_metadata(&metadata, &view).unwrap(),
                oracle.1
            );
            assert_eq!(
                shadow::render_to_string(&view),
                yetlet_html::render_to_string(&view)
            );
            assert_eq!(
                shadow::render_document_with_metadata(&metadata, &view),
                render_document_with_metadata(&metadata, &view)
            );
        }
    }

    #[cfg(any(feature = "serializer-attribution", feature = "escape-comparison"))]
    fn validation_views() -> Vec<View> {
        let mut views: Vec<View> = vec![
            Element::new("div onload=evil").into(),
            Element::new("script").into(),
            Element::new("div").attribute("bad name", "x").into(),
            Element::new("div")
                .attribute("id", "a")
                .attribute("id", "b")
                .into(),
            Element::new("input").child(View::text("x")).into(),
            Element::new("form")
                .child(Element::new("section").child(Element::new("form")))
                .into(),
            Element::new("textarea").child(Element::new("p")).into(),
            Element::new("option").child(Element::new("span")).into(),
            Element::new("select").child(Element::new("div")).into(),
            Element::new("p")
                .attribute("title", "\" onclick='evil()' &")
                .child(View::text("<img src=x onerror=1> & ' \""))
                .into(),
            Element::new("textarea")
                .child(View::text("\r"))
                .child(View::text("\n<draft>\rnext"))
                .into(),
            Element::new("textarea").child(View::text("normal")).into(),
        ];
        for tag in ["style", "iframe", "object", "embed"] {
            views.push(Element::new(tag).into());
        }
        for attr in ["onclick", "style", "srcdoc"] {
            views.push(Element::new("div").attribute(attr, "x").into());
        }
        for url in [
            "Ja\nVaScript:alert(1)",
            "VB\tscript:evil",
            " Da\u{7}Ta:text/html,<script>",
        ] {
            views.push(Element::new("a").attribute("href", url).into());
        }
        views
    }

    #[cfg(feature = "serializer-attribution")]
    #[test]
    fn yt146_shadow_preserves_validation_errors_and_textarea() {
        let metadata =
            DocumentMetadata::with_language("<&> \"'café", LanguageTag::new("en").unwrap());
        for view in validation_views() {
            assert_eq!(
                shadow::render_to_string(&view),
                yetlet_html::render_to_string(&view)
            );
            assert_eq!(
                shadow::render_document_with_metadata(&metadata, &view),
                render_document_with_metadata(&metadata, &view)
            );
        }
    }

    #[cfg(feature = "escape-comparison")]
    #[test]
    fn yt147_reference_current_match_all_fixed_literals() {
        for (_, rows) in CASES {
            let (metadata, view) = page(rows);
            let (body, document) = expected(rows);
            for variant in ["reference", "current"] {
                assert_eq!(
                    comparison_render(variant, "body", &metadata, &view).unwrap(),
                    body
                );
                assert_eq!(
                    comparison_render(variant, "document", &metadata, &view).unwrap(),
                    document
                );
            }
        }
    }

    #[cfg(feature = "escape-comparison")]
    #[test]
    fn yt147_reference_current_preserve_errors_and_control_content() {
        use yetlet_html::RenderError;
        let metadata =
            DocumentMetadata::with_language("<&> \"'café", LanguageTag::new("en").unwrap());
        let mut views = validation_views();
        views.extend([
            View::text(""),
            View::text("水🦀é&<>\"'"),
            Element::new("textarea")
                .child(View::text("\n<&>é\r\n"))
                .into(),
            Element::new("select")
                .child(View::text(" "))
                .child(Element::new("option").child(View::text("<&>é")))
                .into(),
        ]);
        for view in views {
            for phase in ["body", "document"] {
                assert_eq!(
                    comparison_render("reference", phase, &metadata, &view),
                    comparison_render("current", phase, &metadata, &view)
                );
            }
        }
        for (view, error) in [
            (
                Element::new("div")
                    .attribute("id", "a")
                    .attribute("id", "b")
                    .attribute("onclick", "x")
                    .into(),
                RenderError::DuplicateAttribute("id".into()),
            ),
            (
                Element::new("textarea")
                    .attribute("onclick", "x")
                    .child(Element::new("p"))
                    .into(),
                RenderError::InvalidChildren("textarea".into()),
            ),
            (
                Element::new("script").attribute("onclick", "x").into(),
                RenderError::InvalidTag("script".into()),
            ),
        ] {
            for variant in ["reference", "current"] {
                for phase in ["body", "document"] {
                    assert_eq!(
                        comparison_render(variant, phase, &metadata, &view),
                        Err(error.clone())
                    );
                }
            }
        }
    }

    #[cfg(all(feature = "component-probes", not(feature = "allocation-counts")))]
    #[test]
    fn yt146_component_outputs_match_literals() {
        let (metadata, view) = page(1024);
        let inputs = corpus(metadata.title(), &view);
        let escaped = expected_corpus();
        assert_eq!(inputs.len(), 6150);
        assert_eq!(escaped.len(), 6150);
        for (input, expected) in inputs.iter().zip(&escaped) {
            assert_eq!(yetlet_html::escape_text(input), *expected);
        }
        let (body, document) = expected(1024);
        assert_eq!(envelope(&metadata, &body), document);
    }
}

#[cfg(feature = "escape-comparison")]
fn comparison_render(
    variant: &str,
    phase: &str,
    metadata: &DocumentMetadata,
    view: &View,
) -> Result<String, yetlet_html::RenderError> {
    match (variant, phase) {
        ("reference", "body") => shadow::render_to_string(view),
        ("reference", "document") => shadow::render_document_with_metadata(metadata, view),
        ("current", "body") => yetlet_html::render_to_string(view),
        ("current", "document") => render_document_with_metadata(metadata, view),
        _ => unreachable!("comparison variant/phase validated before work"),
    }
}

#[cfg(feature = "escape-comparison")]
pub fn comparison_output(directory: &Path) -> Result<(), Box<dyn Error>> {
    for (name, rows) in CASES {
        let (metadata, view) = page(rows);
        let (body, document) = expected(rows);
        for variant in ["reference", "current"] {
            for (phase, oracle) in [("body", &body), ("document", &document)] {
                let output = comparison_render(variant, phase, &metadata, &view)?;
                assert_eq!(&output, oracle);
                std::fs::write(
                    directory.join(format!("{variant}-{name}-{phase}.html")),
                    output,
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(feature = "escape-comparison")]
pub fn comparison_timing(repetition: usize) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("comparison timing requires an ordinary build without allocation-counts".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let (metadata, view) = page(1024);
        let mut groups = [
            ("reference", "body"),
            ("current", "body"),
            ("reference", "document"),
            ("current", "document"),
        ];
        if repetition == 2 {
            groups.reverse();
        }
        let mut observations = Vec::with_capacity(25);
        let mut sequence = 0;
        println!("repetition,sequence,variant,phase,population,index,elapsed_ns,result_len");
        for (variant, phase) in groups {
            for index in 0..25 {
                let started = std::time::Instant::now();
                let output = black_box(comparison_render(
                    variant,
                    phase,
                    black_box(&metadata),
                    black_box(&view),
                )?);
                let elapsed = started.elapsed().as_nanos();
                let result_len = output.len();
                drop(output);
                observations.push((index, elapsed, result_len));
            }
            for (index, elapsed, len) in observations.drain(..) {
                let population = if index < 5 { "warmup" } else { "measured" };
                let index = if index < 5 { index } else { index - 5 };
                println!(
                    "{repetition},{sequence},{variant},{phase},{population},{index},{elapsed},{len}"
                );
                sequence += 1;
            }
        }
        Ok(())
    }
}

#[cfg(all(feature = "escape-comparison", feature = "allocation-counts"))]
pub fn comparison_counts(variant: &str, phase: &str) -> Result<(), Box<dyn Error>> {
    if !matches!(variant, "reference" | "current") || !matches!(phase, "body" | "document") {
        return Err("expected reference|current body|document".into());
    }
    let (metadata, view) = page(1024);
    let mut records = [(counts::snapshot(), counts::snapshot(), counts::snapshot()); 3];
    for record in &mut records {
        let start = counts::begin();
        let output = black_box(comparison_render(
            variant,
            phase,
            black_box(&metadata),
            black_box(&view),
        )?);
        let held = counts::snapshot();
        drop(output);
        let released = counts::snapshot();
        *record = (start, held, released);
    }
    println!(
        "variant,phase,operation,point,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (operation, (start, held, released)) in records.iter().enumerate() {
        for (point, s) in [("start", start), ("held", held), ("released", released)] {
            println!(
                "{variant},{phase},{operation},{point},{},{},{},{},{},{},{},{},{},{},{},{}",
                s.allocations,
                s.zeroed,
                s.reallocations,
                s.deallocations,
                s.allocated_bytes,
                s.realloc_old_bytes,
                s.realloc_new_bytes,
                s.deallocated_bytes,
                s.failed_allocations,
                s.failed_reallocations,
                s.live,
                s.peak
            );
        }
    }
    // Preserve raw observations before checking balance/failures.
    for (start, _, released) in &records {
        assert_eq!(released.live, start.live, "release balance");
        assert_eq!(released.failed_allocations, start.failed_allocations);
        assert_eq!(released.failed_reallocations, start.failed_reallocations);
    }
    Ok(())
}

#[cfg(feature = "url-probes")]
fn url_corpus(view: &View) -> Vec<&str> {
    fn visit<'a>(view: &'a View, values: &mut Vec<&'a str>) {
        if let View::Element(element) = view {
            for attribute in element.attributes() {
                if url_check::URL_ATTRIBUTES.contains(&attribute.name) {
                    values.push(&attribute.value);
                }
            }
            for child in element.child_views() {
                visit(child, values);
            }
        }
    }
    let mut values = Vec::with_capacity(1024);
    visit(view, &mut values);
    values
}

#[cfg(feature = "url-probes")]
fn url_operation(values: &[&str]) -> usize {
    let mut unsafe_count = 0;
    for value in black_box(values) {
        unsafe_count += usize::from(black_box(url_check::unsafe_scheme(black_box(value))));
    }
    black_box(unsafe_count)
}

#[cfg(feature = "url-probes")]
pub fn url_timing(repetition: usize) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("URL timing requires an ordinary build without allocation-counts".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let (_metadata, view) = page(1024);
        let values = url_corpus(&view);
        let input_bytes: usize = values.iter().map(|value| value.len()).sum();
        assert_eq!((values.len(), input_bytes), (1024, 32768));
        let mut observations = Vec::with_capacity(25);
        for index in 0..25 {
            let started = std::time::Instant::now();
            let unsafe_count = url_operation(&values);
            let elapsed_ns = started.elapsed().as_nanos();
            observations.push((index, elapsed_ns, unsafe_count));
        }
        println!("repetition,population,index,elapsed_ns,url_count,input_bytes,unsafe_count");
        for (index, elapsed_ns, unsafe_count) in &observations {
            let population = if *index < 5 { "warmup" } else { "measured" };
            let index = if *index < 5 { *index } else { *index - 5 };
            println!(
                "{repetition},{population},{index},{elapsed_ns},{},{input_bytes},{unsafe_count}",
                values.len()
            );
        }
        assert!(observations.iter().all(|record| record.2 == 0));
        Ok(())
    }
}

#[cfg(all(feature = "url-probes", feature = "allocation-counts"))]
pub fn url_counts() -> Result<(), Box<dyn Error>> {
    let (_metadata, view) = page(1024);
    let values = url_corpus(&view);
    let input_bytes: usize = values.iter().map(|value| value.len()).sum();
    assert_eq!((values.len(), input_bytes), (1024, 32768));
    let mut observations = [(counts::snapshot(), counts::snapshot(), 0); 3];
    for record in &mut observations {
        let start = counts::begin();
        let unsafe_count = url_operation(&values);
        let end = counts::snapshot();
        *record = (start, end, unsafe_count);
    }
    println!(
        "operation,point,url_count,input_bytes,unsafe_count,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (operation, (start, end, unsafe_count)) in observations.iter().enumerate() {
        for (point, s) in [("start", start), ("end", end)] {
            println!(
                "{operation},{point},{},{input_bytes},{unsafe_count},{},{},{},{},{},{},{},{},{},{},{},{}",
                values.len(),
                s.allocations,
                s.zeroed,
                s.reallocations,
                s.deallocations,
                s.allocated_bytes,
                s.realloc_old_bytes,
                s.realloc_new_bytes,
                s.deallocated_bytes,
                s.failed_allocations,
                s.failed_reallocations,
                s.live,
                s.peak
            );
        }
    }
    let mut first = None;
    for (start, end, unsafe_count) in observations {
        let relative = (
            end.allocations - start.allocations,
            end.zeroed - start.zeroed,
            end.reallocations - start.reallocations,
            end.deallocations - start.deallocations,
            end.allocated_bytes - start.allocated_bytes,
            end.realloc_old_bytes - start.realloc_old_bytes,
            end.realloc_new_bytes - start.realloc_new_bytes,
            end.deallocated_bytes - start.deallocated_bytes,
            end.failed_allocations - start.failed_allocations,
            end.failed_reallocations - start.failed_reallocations,
            end.live - start.live,
            end.peak - start.live,
        );
        assert_eq!(
            (relative.1, relative.8, relative.9, relative.10),
            (0, 0, 0, 0)
        );
        assert_eq!(unsafe_count, 0);
        if let Some(expected) = first {
            assert_eq!(relative, expected);
        } else {
            first = Some(relative);
        }
    }
    Ok(())
}

#[cfg(feature = "url-comparison")]
mod current_url_check {
    // Source-derived timing copy of the private product checker, not a product call.
    pub(super) fn unsafe_scheme(value: &str) -> bool {
        let mut normalized = value
            .chars()
            .filter(|character| !character.is_ascii_whitespace() && !character.is_ascii_control())
            .flat_map(char::to_lowercase);
        let expected = match normalized.next() {
            Some('j') => "avascript:",
            Some('v') => "bscript:",
            Some('d') => "ata:",
            _ => return false,
        };
        expected
            .chars()
            .all(|character| normalized.next() == Some(character))
    }
}

#[cfg(feature = "url-comparison")]
fn current_url_operation(values: &[&str]) -> usize {
    let mut unsafe_count = 0;
    for value in black_box(values) {
        unsafe_count += usize::from(black_box(current_url_check::unsafe_scheme(black_box(
            value,
        ))));
    }
    black_box(unsafe_count)
}

#[cfg(feature = "url-comparison")]
pub fn url_comparison_output(directory: &Path) -> Result<(), Box<dyn Error>> {
    for (name, rows) in CASES {
        let (body, document) = verify(rows)?;
        std::fs::write(directory.join(format!("{name}-body.html")), body)?;
        std::fs::write(directory.join(format!("{name}-document.html")), document)?;
    }
    Ok(())
}

#[cfg(feature = "url-comparison")]
pub fn url_comparison_timing(repetition: usize) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("URL comparison timing requires an ordinary build without allocation-counts".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let (_metadata, view) = page(1024);
        let values = url_corpus(&view);
        let input_bytes: usize = values.iter().map(|value| value.len()).sum();
        assert_eq!((values.len(), input_bytes), (1024, 32768));
        let mut observations = Vec::with_capacity(50);
        let mut variants: [(&str, fn(&[&str]) -> usize); 2] = [
            ("reference", url_operation),
            ("current", current_url_operation),
        ];
        if repetition == 2 {
            variants.reverse();
        }
        for (order, (variant, operation)) in variants.into_iter().enumerate() {
            for index in 0..25 {
                let started = std::time::Instant::now();
                let unsafe_count = operation(&values);
                let elapsed_ns = started.elapsed().as_nanos();
                observations.push((order, variant, index, elapsed_ns, unsafe_count));
            }
        }
        println!(
            "repetition,order,variant,population,index,elapsed_ns,url_count,input_bytes,unsafe_count"
        );
        for (order, variant, index, elapsed_ns, unsafe_count) in &observations {
            let population = if *index < 5 { "warmup" } else { "measured" };
            let index = if *index < 5 { *index } else { *index - 5 };
            println!(
                "{repetition},{order},{variant},{population},{index},{elapsed_ns},{},{input_bytes},{unsafe_count}",
                values.len()
            );
        }
        assert!(observations.iter().all(|record| record.4 == 0));
        Ok(())
    }
}

#[cfg(all(feature = "url-comparison", feature = "allocation-counts"))]
pub fn url_comparison_counts(variant: &str) -> Result<(), Box<dyn Error>> {
    let operation: fn(&[&str]) -> usize = match variant {
        "reference" => url_operation,
        "current" => current_url_operation,
        _ => return Err("expected reference|current".into()),
    };
    let (_metadata, view) = page(1024);
    let values = url_corpus(&view);
    let input_bytes: usize = values.iter().map(|value| value.len()).sum();
    assert_eq!((values.len(), input_bytes), (1024, 32768));
    let mut observations = [(counts::snapshot(), counts::snapshot(), 0); 3];
    for record in &mut observations {
        let start = counts::begin();
        let unsafe_count = operation(&values);
        let end = counts::snapshot();
        *record = (start, end, unsafe_count);
    }
    println!(
        "variant,operation,point,url_count,input_bytes,unsafe_count,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (operation, (start, end, unsafe_count)) in observations.iter().enumerate() {
        for (point, s) in [("start", start), ("end", end)] {
            println!(
                "{variant},{operation},{point},{},{input_bytes},{unsafe_count},{},{},{},{},{},{},{},{},{},{},{},{}",
                values.len(),
                s.allocations,
                s.zeroed,
                s.reallocations,
                s.deallocations,
                s.allocated_bytes,
                s.realloc_old_bytes,
                s.realloc_new_bytes,
                s.deallocated_bytes,
                s.failed_allocations,
                s.failed_reallocations,
                s.live,
                s.peak
            );
        }
    }
    let mut first = None;
    for (start, end, unsafe_count) in observations {
        let relative = (
            end.allocations - start.allocations,
            end.zeroed - start.zeroed,
            end.reallocations - start.reallocations,
            end.deallocations - start.deallocations,
            end.allocated_bytes - start.allocated_bytes,
            end.realloc_old_bytes - start.realloc_old_bytes,
            end.realloc_new_bytes - start.realloc_new_bytes,
            end.deallocated_bytes - start.deallocated_bytes,
            end.failed_allocations - start.failed_allocations,
            end.failed_reallocations - start.failed_reallocations,
            end.live - start.live,
            end.peak - start.live,
        );
        assert_eq!((relative.8, relative.9, relative.10), (0, 0, 0));
        assert_eq!(unsafe_count, 0);
        if let Some(expected) = first {
            assert_eq!(relative, expected);
        } else {
            first = Some(relative);
        }
    }
    Ok(())
}

#[cfg(all(test, feature = "url-probes"))]
mod url_tests {
    use super::*;

    #[test]
    fn yt148_borrowed_corpus_and_source_identity() {
        borrowed_corpus_and_legacy_source_identity();
    }

    fn borrowed_corpus_and_legacy_source_identity() {
        let (_metadata, view) = page(1024);
        let values = url_corpus(&view);
        assert_eq!(values.len(), 1024);
        assert_eq!(values.iter().map(|value| value.len()).sum::<usize>(), 32768);
        for (index, value) in values.iter().enumerate() {
            assert_eq!(*value, format!("/records/{index:04}?view=full&public=1"));
            assert!(!url_check::unsafe_scheme(value));
        }
        let selected: View = Element::new("div")
            .attribute("title", "not a URL")
            .attribute("src", "/first")
            .attribute("href", "/second")
            .child(View::text("not a URL"))
            .child(Element::new("form").attribute("action", "/third"))
            .into();
        assert_eq!(url_corpus(&selected), ["/first", "/second", "/third"]);
        let product = include_str!("../../../crates/yetlet-html/src/lib.rs");
        let sample = include_str!("attribution.rs");
        // Pinned legacy normalizer from product baseline f01d981c9e924cecc0713b3494772628c6542954.
        let legacy_fn = r#"
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
"#;
        let sample_fn = sample
            .split_once("fn unsafe_scheme(value: &str) -> bool {")
            .unwrap()
            .1
            .split_once("\n    }")
            .unwrap()
            .0;
        assert_eq!(
            legacy_fn.lines().map(str::trim).collect::<Vec<_>>(),
            sample_fn.lines().map(str::trim).collect::<Vec<_>>()
        );
        let product_list = product
            .split_once("const URL_ATTRIBUTES:")
            .unwrap()
            .1
            .split_once(';')
            .unwrap()
            .0;
        let sample_list = sample
            .split_once("const URL_ATTRIBUTES:")
            .unwrap()
            .1
            .split_once(';')
            .unwrap()
            .0;
        assert_eq!(
            product_list.split_whitespace().collect::<String>(),
            sample_list.split_whitespace().collect::<String>()
        );
    }

    #[test]
    fn yt148_literal_url_security_and_public_render_results() {
        literal_url_security_and_public_render_results();
    }

    fn literal_url_security_and_public_render_results() {
        use yetlet_html::{RenderError, render_to_string};
        let cases: &[(&str, bool, Result<&str, RenderError>)] = &[
            ("", false, Ok("<a href=\"\"></a>")),
            (
                "/records/0000?view=full&public=1",
                false,
                Ok("<a href=\"/records/0000?view=full&amp;public=1\"></a>"),
            ),
            ("#fragment", false, Ok("<a href=\"#fragment\"></a>")),
            (
                "HTTPS://example.test/",
                false,
                Ok("<a href=\"HTTPS://example.test/\"></a>"),
            ),
            (
                "/a?x=\"'&",
                false,
                Ok("<a href=\"/a?x=&quot;&#39;&amp;\"></a>"),
            ),
            ("/水🦀é", false, Ok("<a href=\"/水🦀é\"></a>")),
            ("İavascript:", false, Ok("<a href=\"İavascript:\"></a>")),
            (
                "JavaScript:x",
                true,
                Err(RenderError::UnsafeUrl("href".into())),
            ),
            (
                "VbScRiPt:x",
                true,
                Err(RenderError::UnsafeUrl("href".into())),
            ),
            ("DaTa:x", true, Err(RenderError::UnsafeUrl("href".into()))),
            (
                " ja va\0sc\u{7}ri\tpt\n:\r\u{7f}x",
                true,
                Err(RenderError::UnsafeUrl("href".into())),
            ),
            (
                "v b\0s\u{7}c\tr\ni\rp\u{7f}t:x",
                true,
                Err(RenderError::UnsafeUrl("href".into())),
            ),
            (
                " d\0a\u{7}t\ta\n\r\u{7f}:x",
                true,
                Err(RenderError::UnsafeUrl("href".into())),
            ),
            ("javascriptx:", false, Ok("<a href=\"javascriptx:\"></a>")),
            ("vbscriptx:", false, Ok("<a href=\"vbscriptx:\"></a>")),
            ("datax:", false, Ok("<a href=\"datax:\"></a>")),
            (
                "\u{a0}javascript:x",
                false,
                Ok("<a href=\"\u{a0}javascript:x\"></a>"),
            ),
            (
                "java\u{2003}script:x",
                false,
                Ok("<a href=\"java\u{2003}script:x\"></a>"),
            ),
        ];
        for (value, unsafe_expected, rendered) in cases {
            assert_eq!(
                url_check::unsafe_scheme(value),
                *unsafe_expected,
                "{value:?}"
            );
            let view = Element::new("a").attribute("href", *value).into();
            assert_eq!(
                render_to_string(&view),
                rendered.clone().map(str::to_owned),
                "{value:?}"
            );
        }
        let attributes: &[(&str, &str, Result<&str, RenderError>)] = &[
            ("src", "data:x", Err(RenderError::UnsafeUrl("src".into()))),
            (
                "action",
                "vbscript:x",
                Err(RenderError::UnsafeUrl("action".into())),
            ),
            (
                "formaction",
                "javascript:x",
                Err(RenderError::UnsafeUrl("formaction".into())),
            ),
            (
                "title",
                "javascript:x",
                Ok("<a title=\"javascript:x\"></a>"),
            ),
            (
                "xlink:href",
                "javascript:x",
                Err(RenderError::InvalidAttribute("xlink:href".into())),
            ),
        ];
        for (name, value, rendered) in attributes {
            let view = Element::new("a").attribute(name, *value).into();
            assert_eq!(render_to_string(&view), rendered.clone().map(str::to_owned));
        }
        for (view, expected) in [
            (
                Element::new("a")
                    .attribute("href", "/safe")
                    .attribute("href", "javascript:x")
                    .into(),
                RenderError::DuplicateAttribute("href".into()),
            ),
            (
                Element::new("a")
                    .attribute("bad:name", "javascript:x")
                    .attribute("href", "javascript:x")
                    .into(),
                RenderError::InvalidAttribute("bad:name".into()),
            ),
            (
                Element::new("a")
                    .attribute("onclick", "javascript:x")
                    .attribute("href", "javascript:x")
                    .into(),
                RenderError::ForbiddenAttribute("onclick".into()),
            ),
            (
                Element::new("script")
                    .attribute("href", "javascript:x")
                    .into(),
                RenderError::InvalidTag("script".into()),
            ),
            (
                Element::new("bad:tag")
                    .attribute("href", "javascript:x")
                    .into(),
                RenderError::InvalidTag("bad:tag".into()),
            ),
        ] {
            assert_eq!(render_to_string(&view), Err(expected));
        }
    }

    #[cfg(feature = "url-comparison")]
    #[test]
    fn yt149_source_identity_and_borrowed_population() {
        borrowed_corpus_and_legacy_source_identity();
        let product = include_str!("../../../crates/yetlet-html/src/lib.rs");
        let sample = include_str!("attribution.rs");
        let product_fn = product
            .split_once("fn unsafe_scheme(value: &str) -> bool {")
            .unwrap()
            .1
            .split_once("\n}")
            .unwrap()
            .0;
        let current_fn = sample
            .split_once("mod current_url_check {")
            .unwrap()
            .1
            .split_once("fn unsafe_scheme(value: &str) -> bool {")
            .unwrap()
            .1
            .split_once("\n    }")
            .unwrap()
            .0;
        assert_eq!(
            product_fn.lines().map(str::trim).collect::<Vec<_>>(),
            current_fn.lines().map(str::trim).collect::<Vec<_>>()
        );
    }

    #[cfg(feature = "url-comparison")]
    #[test]
    fn yt149_literal_public_security_and_error_results() {
        literal_url_security_and_public_render_results();
    }

    #[cfg(feature = "url-comparison")]
    #[test]
    fn yt149_finite_prefix_differential() {
        let cases = [
            ("", false),
            (" \0\u{7}\t\n\r\u{7f}", false),
            ("JaVaScRiPt:", true),
            ("VbScRiPt:", true),
            ("DaTa:", true),
            (" \0\u{7}\t\n\r\u{7f}javascript:", true),
            ("ja va\0sc\u{7}ri\tpt\n:\r\u{7f}x", true),
            ("v b\0s\u{7}c\tr\ni\rp\u{7f}t:x", true),
            (" d\0a\u{7}t\ta\n\r\u{7f}:x", true),
            ("j", false),
            ("javascript", false),
            ("vbscript", false),
            ("data", false),
            ("javascriptx:", false),
            ("vbscriptx:", false),
            ("datax:", false),
            ("/safe?javascript:x", false),
            ("\u{a0}javascript:", false),
            ("java\u{2003}script:", false),
            ("\u{80}data:", false),
            ("/水🦀é", false),
            ("İjavascript:", false),
            ("javascrİpt:", false),
            ("javascript:İ水🦀", true),
            ("vbscript:İ", true),
            ("data:水", true),
        ];
        for (value, expected) in cases {
            assert_eq!(
                url_check::unsafe_scheme(value),
                expected,
                "reference {value:?}"
            );
            assert_eq!(
                current_url_check::unsafe_scheme(value),
                expected,
                "current {value:?}"
            );
        }
        let tail = "İ水🦀x".repeat(4096);
        for (prefix, expected) in [
            ("/safe/", false),
            ("javascript:", true),
            ("vbscript:", true),
            ("data:", true),
        ] {
            let value = format!("{prefix}{tail}");
            assert_eq!(url_check::unsafe_scheme(&value), expected);
            assert_eq!(current_url_check::unsafe_scheme(&value), expected);
        }
    }
}
