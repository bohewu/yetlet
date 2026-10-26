//! Fixed native SSR observations, without HTTP or optional provider work.
#[cfg(any(
    feature = "serializer-attribution",
    feature = "component-probes",
    feature = "escape-comparison",
    feature = "url-probes"
))]
mod attribution;
#[cfg(feature = "buffer-study")]
mod buffer_study;
#[cfg(feature = "child-capacity")]
mod child_capacity;
#[cfg(feature = "construction-study")]
mod construction_study;
#[cfg(feature = "allocation-counts")]
mod counts;
#[cfg(feature = "document-capacity")]
mod document_capacity;
#[cfg(feature = "latency-study")]
mod latency_study;
#[cfg(feature = "page-comparison")]
mod page_comparison;

#[cfg(not(feature = "allocation-counts"))]
use std::time::Instant;
use std::{error::Error, hint::black_box, path::Path};
use yetlet_core::{Element, View};
use yetlet_server_axum::{DocumentMetadata, LanguageTag, render_document_with_metadata};

const CASES: [(&str, usize); 3] = [("small", 8), ("medium", 128), ("large", 1024)];
const PHASES: [&str; 4] = ["construct", "body", "document", "combined"];
const TITLE: &str = "YT145 public report <&> \"quoted\" 'café'";
const PREFIX: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>YT145 public report &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;</title></head><body>";
const BODY_START: &str = "<main id=\"ssr-report\"><header><h1>SSR report &lt;&amp;&gt; &quot;title&quot; &#39;café&#39;</h1><p>Public synthetic rows</p></header><section aria-label=\"Records\"><ul>";
const BODY_END: &str = "</ul></section><footer><p>End of public report</p></footer></main>";

fn row(index: usize) -> View {
    Element::new("li")
        .attribute("data-row", format!("{index:04}"))
        .attribute("title", format!("Row {index:04} <&> \"quoted\" 'café'"))
        .child(Element::new("h2").child(View::text(format!("Record {index:04}"))))
        .child(Element::new("p").child(View::text(format!(
            "Public <&> \"value\" 'café' {index:04}"
        ))))
        .child(
            Element::new("a")
                .attribute("href", format!("/records/{index:04}?view=full&public=1"))
                .child(View::text("Open record")),
        )
        .into()
}

fn page(rows: usize) -> (DocumentMetadata, View) {
    let view = Element::new("main")
        .attribute("id", "ssr-report")
        .child(
            Element::new("header")
                .child(Element::new("h1").child(View::text("SSR report <&> \"title\" 'café'")))
                .child(Element::new("p").child(View::text("Public synthetic rows"))),
        )
        .child(
            Element::new("section")
                .attribute("aria-label", "Records")
                .child(Element::new("ul").children((0..rows).map(row))),
        )
        .child(
            Element::new("footer")
                .child(Element::new("p").child(View::text("End of public report"))),
        )
        .into();
    (
        DocumentMetadata::with_language(TITLE, LanguageTag::new("en").unwrap()),
        view,
    )
}

// Independent already-escaped literal oracle, not a second general serializer.
fn expected(rows: usize) -> (String, String) {
    let mut body = BODY_START.to_owned();
    for i in 0..rows {
        body.push_str(&format!("<li data-row=\"{i:04}\" title=\"Row {i:04} &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;\"><h2>Record {i:04}</h2><p>Public &lt;&amp;&gt; &quot;value&quot; &#39;café&#39; {i:04}</p><a href=\"/records/{i:04}?view=full&amp;public=1\">Open record</a></li>"));
    }
    body.push_str(BODY_END);
    let document = format!("{PREFIX}{body}</body></html>");
    (body, document)
}

fn verify(rows: usize) -> Result<(String, String), Box<dyn Error>> {
    let (metadata, view) = page(rows);
    let actual_body = yetlet_html::render_to_string(&view)?;
    let actual_document = render_document_with_metadata(&metadata, &view)?;
    let (body, document) = expected(rows);
    assert_eq!(actual_body, body);
    assert_eq!(actual_document, document);
    Ok((actual_body, actual_document))
}

fn size(name: &str) -> Result<usize, &'static str> {
    CASES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, n)| *n)
        .ok_or("expected small|medium|large")
}

#[cfg(feature = "allocation-counts")]
fn proc_status(label: &str) -> Result<(), Box<dyn Error>> {
    let status = std::fs::read_to_string("/proc/self/status")?;
    for line in status
        .lines()
        .filter(|line| line.starts_with("VmRSS:") || line.starts_with("VmHWM:"))
    {
        eprintln!("{label} {line}");
    }
    Ok(())
}

#[cfg(not(feature = "allocation-counts"))]
fn timing(repetition: usize) -> Result<(), Box<dyn Error>> {
    if !(1..=2).contains(&repetition) {
        return Err("expected repetition 1|2".into());
    }
    // Allocate storage before all timed work. Rows contain only stack values.
    let mut observations = Vec::with_capacity(60);
    let mut sequence = 0;
    println!("repetition,sequence,size,phase,population,index,elapsed_ns,result_len");
    let mut cases = CASES;
    let mut phases = PHASES;
    if repetition == 2 {
        cases.reverse();
        phases.reverse();
    }
    for (name, rows) in cases {
        for phase in phases {
            drop(verify(rows)?);
            // Only borrowed renderer phases retain a prebuilt tree.
            let retained = if phase == "body" || phase == "document" {
                Some(page(rows))
            } else {
                None
            };
            for index in 0..60 {
                let started = Instant::now();
                let (elapsed, result_len) = match phase {
                    "construct" => {
                        let owned = black_box(page(black_box(rows)));
                        let elapsed = started.elapsed().as_nanos();
                        drop(owned);
                        (elapsed, 11 + 7 * rows)
                    }
                    "body" => {
                        let view = &retained.as_ref().unwrap().1;
                        let output = black_box(yetlet_html::render_to_string(black_box(view))?);
                        let elapsed = started.elapsed().as_nanos();
                        let len = output.len();
                        drop(output);
                        (elapsed, len)
                    }
                    "document" => {
                        let (metadata, view) = retained.as_ref().unwrap();
                        let output = black_box(render_document_with_metadata(
                            black_box(metadata),
                            black_box(view),
                        )?);
                        let elapsed = started.elapsed().as_nanos();
                        let len = output.len();
                        drop(output);
                        (elapsed, len)
                    }
                    "combined" => {
                        let (metadata, view) = page(black_box(rows));
                        let output = black_box(render_document_with_metadata(
                            black_box(&metadata),
                            black_box(&view),
                        )?);
                        let len = black_box(output.len());
                        drop(output);
                        drop(view);
                        drop(metadata);
                        (started.elapsed().as_nanos(), len)
                    }
                    _ => unreachable!(),
                };
                observations.push((name, phase, index, elapsed, result_len));
            }
            drop(retained);
            for (name, phase, index, elapsed, len) in observations.drain(..) {
                let population = if index < 10 { "warmup" } else { "measured" };
                let index = if index < 10 { index } else { index - 10 };
                println!(
                    "{repetition},{sequence},{name},{phase},{population},{index},{elapsed},{len}"
                );
                sequence += 1;
            }
        }
    }
    Ok(())
}

#[cfg(feature = "allocation-counts")]
fn allocation_counts(name: &str, phase: &str) -> Result<(), Box<dyn Error>> {
    let rows = size(name)?;
    if !PHASES.contains(&phase) {
        return Err("expected construct|body|document|combined".into());
    }
    proc_status("startup")?;
    let retained = if phase == "body" || phase == "document" {
        Some(page(rows))
    } else {
        None
    };
    // Stack-only snapshots, emitted only after all counted operations.
    let mut observations = [(counts::snapshot(), counts::snapshot(), counts::snapshot()); 3];
    for record in &mut observations {
        let start = counts::begin();
        let held;
        match phase {
            "construct" => {
                let owned = black_box(page(black_box(rows)));
                held = counts::snapshot();
                drop(owned);
            }
            "body" => {
                let output = black_box(yetlet_html::render_to_string(black_box(
                    &retained.as_ref().unwrap().1,
                ))?);
                held = counts::snapshot();
                drop(output);
            }
            "document" => {
                let (metadata, view) = retained.as_ref().unwrap();
                let output = black_box(render_document_with_metadata(
                    black_box(metadata),
                    black_box(view),
                )?);
                held = counts::snapshot();
                drop(output);
            }
            "combined" => {
                let (metadata, view) = page(black_box(rows));
                let output = black_box(render_document_with_metadata(
                    black_box(&metadata),
                    black_box(&view),
                )?);
                held = counts::snapshot();
                drop(output);
                drop(view);
                drop(metadata);
            }
            _ => unreachable!(),
        }
        let released = counts::snapshot();
        *record = (start, held, released);
        assert_eq!(
            released.live, start.live,
            "region-owned objects must be released"
        );
        assert_eq!(released.failed_allocations, start.failed_allocations);
        assert_eq!(released.failed_reallocations, start.failed_reallocations);
    }
    proc_status("post-work-input-held")?;
    println!(
        "size,phase,operation,point,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (operation, (start, held, released)) in observations.into_iter().enumerate() {
        for (point, s) in [("start", start), ("held", held), ("released", released)] {
            println!(
                "{name},{phase},{operation},{point},{},{},{},{},{},{},{},{},{},{},{},{}",
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
    drop(retained);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        #[cfg(feature = "latency-study")]
        ["latency-api", variant, repetition] => latency_study::api(variant, repetition.parse()?),
        #[cfg(feature = "latency-study")]
        ["latency-tail", repetition] => latency_study::tail(repetition.parse()?),
        #[cfg(feature = "document-capacity")]
        ["document-capacity-output", variant, directory] => document_capacity::output(variant, Path::new(directory)),
        #[cfg(feature = "document-capacity")]
        ["document-capacity-timing", variant, repetition] => document_capacity::timing(variant, repetition.parse()?),
        #[cfg(all(feature = "document-capacity", feature = "allocation-counts"))]
        ["document-capacity-counts", variant, group] => document_capacity::allocation_counts(variant, group),
        #[cfg(feature = "buffer-study")]
        ["buffer-output", directory] => buffer_study::output(Path::new(directory)),
        #[cfg(feature = "buffer-study")]
        ["buffer-timing", repetition] => buffer_study::timing(repetition.parse()?),
        #[cfg(all(feature = "buffer-study", feature = "allocation-counts"))]
        ["buffer-counts"] => buffer_study::allocation_counts(),
        #[cfg(feature = "child-capacity")]
        ["child-output", variant, directory] => child_capacity::output(variant, Path::new(directory)),
        #[cfg(feature = "child-capacity")]
        ["child-timing", variant, repetition] => child_capacity::timing(variant, repetition.parse()?),
        #[cfg(all(feature = "child-capacity", feature = "allocation-counts"))]
        ["child-counts", variant, group] => child_capacity::allocation_counts(variant, group),
        #[cfg(feature = "construction-study")]
        ["construction-output", directory] => construction_study::output(Path::new(directory)),
        #[cfg(feature = "construction-study")]
        ["construction-timing", repetition] => construction_study::timing(repetition.parse()?),
        #[cfg(all(feature = "construction-study", feature = "allocation-counts"))]
        ["construction-counts"] => construction_study::allocation_counts(),
        #[cfg(feature = "page-comparison")]
        ["page-output", variant, directory] => page_comparison::output(variant, Path::new(directory)),
        #[cfg(feature = "page-comparison")]
        ["page-timing", variant, repetition] => page_comparison::timing(variant, repetition.parse()?),
        #[cfg(all(feature = "page-comparison", feature = "allocation-counts"))]
        ["page-counts", variant, phase] => page_comparison::allocation_counts(variant, phase),
        #[cfg(feature = "url-comparison")]
        ["url-comparison-output", directory] => attribution::url_comparison_output(Path::new(directory)),
        #[cfg(feature = "url-comparison")]
        ["url-comparison-timing", repetition] => attribution::url_comparison_timing(repetition.parse()?),
        #[cfg(all(feature = "url-comparison", feature = "allocation-counts"))]
        ["url-comparison-counts", variant] => attribution::url_comparison_counts(variant),
        #[cfg(feature = "url-probes")]
        ["url-timing", repetition] => attribution::url_timing(repetition.parse()?),
        #[cfg(all(feature = "url-probes", feature = "allocation-counts"))]
        ["url-counts"] => attribution::url_counts(),
        #[cfg(feature = "escape-comparison")]
        ["comparison-output", directory] => attribution::comparison_output(Path::new(directory)),
        #[cfg(feature = "escape-comparison")]
        ["comparison-timing", repetition] => attribution::comparison_timing(repetition.parse()?),
        #[cfg(all(feature = "escape-comparison", feature = "allocation-counts"))]
        ["comparison-counts", variant, phase] => attribution::comparison_counts(variant, phase),
        #[cfg(feature = "serializer-attribution")]
        ["attribution-output", directory] => attribution::output(Path::new(directory)),
        #[cfg(feature = "serializer-attribution")]
        ["attribution-counts", phase] => attribution::allocation_counts(phase),
        #[cfg(feature = "component-probes")]
        ["component-timing", repetition] => attribution::component_timing(repetition.parse()?),
        ["output", name, directory] => {
            let (body, document) = verify(size(name)?)?;
            std::fs::write(Path::new(directory).join(format!("{name}-body.html")), body)?;
            std::fs::write(Path::new(directory).join(format!("{name}-document.html")), document)?;
            Ok(())
        }
        #[cfg(not(feature = "allocation-counts"))]
        ["timing", repetition] => timing(repetition.parse()?),
        #[cfg(feature = "allocation-counts")]
        ["counts", name, phase] => allocation_counts(name, phase),
        _ => Err("expected output SIZE EXISTING_DIRECTORY; uninstrumented: timing 1|2; counted: counts SIZE PHASE".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // elements, texts, attributes, owned payload bytes, max depth
    fn dimensions(view: &View, depth: usize) -> (usize, usize, usize, usize, usize) {
        match view {
            View::Text(text) => (0, 1, 0, text.len(), depth),
            View::Element(element) => {
                let mut result = (
                    1,
                    0,
                    element.attributes().len(),
                    element.attributes().iter().map(|a| a.value.len()).sum(),
                    depth,
                );
                for child in element.child_views() {
                    let d = dimensions(child, depth + 1);
                    result = (
                        result.0 + d.0,
                        result.1 + d.1,
                        result.2 + d.2,
                        result.3 + d.3,
                        result.4.max(d.4),
                    );
                }
                result
            }
        }
    }

    #[test]
    fn all_fixed_pages_match_independent_escaped_oracles_and_dimensions() {
        for ((_, rows), (payload, body_len, document_len)) in CASES.into_iter().zip([
            (1074, 2146, 2383),
            (15234, 30826, 31063),
            (120962, 244970, 245207),
        ]) {
            let (metadata, view) = page(rows);
            let d = dimensions(&view, 1);
            assert_eq!(
                (d.0, d.1, d.2, d.4),
                (8 + 4 * rows, 3 + 3 * rows, 2 + 3 * rows, 6)
            );
            assert_eq!(
                d.3 + metadata.title().len() + metadata.language().as_str().len(),
                payload
            );
            let (body, document) = verify(rows).unwrap();
            assert_eq!(body.len(), body_len);
            assert_eq!(document.len(), document_len);
            assert!(!document.contains("<script"));
        }
    }
}
