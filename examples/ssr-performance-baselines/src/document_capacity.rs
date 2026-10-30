//! YT-154: identical consumer compiled against two pinned real adapters.
use super::*;
use yetlet::html::RenderError;

type Input = (DocumentMetadata, View);
#[derive(Clone, Copy)]
struct Group {
    name: &'static str,
    rows: usize,
    combined: bool,
}
const GROUPS: [Group; 4] = [
    Group {
        name: "small",
        rows: 8,
        combined: false,
    },
    Group {
        name: "medium",
        rows: 128,
        combined: false,
    },
    Group {
        name: "large",
        rows: 1024,
        combined: false,
    },
    Group {
        name: "combined",
        rows: 1024,
        combined: true,
    },
];
fn label(value: &str) -> Result<(), &'static str> {
    if matches!(value, "reference" | "current") {
        Ok(())
    } else {
        Err("expected source label reference|current")
    }
}
fn document(input: &Input) -> Result<String, RenderError> {
    render_document_with_metadata(black_box(&input.0), black_box(&input.1))
}
fn combined() -> Result<String, RenderError> {
    let (metadata, view) = black_box(page(black_box(1024)));
    let output = render_document_with_metadata(black_box(&metadata), black_box(&view))?;
    drop(view);
    drop(metadata);
    Ok(output)
}
pub fn output(variant: &str, directory: &Path) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    if !directory.is_dir() {
        return Err("output directory must already exist".into());
    }
    for (name, rows) in CASES {
        let input = page(rows);
        let body = yetlet::html::render_to_string(&input.1)?;
        let document = document(&input)?;
        let oracle = expected(rows);
        assert_eq!(body, oracle.0);
        assert_eq!(document, oracle.1);
        std::fs::write(directory.join(format!("{name}-body.html")), body)?;
        std::fs::write(directory.join(format!("{name}-document.html")), document)?;
    }
    Ok(())
}
#[cfg(not(feature = "allocation-counts"))]
fn timed(
    work: impl FnOnce() -> Result<String, RenderError>,
) -> Result<(u128, usize, usize), RenderError> {
    let started = std::time::Instant::now();
    let output = black_box(work()?);
    let elapsed = started.elapsed().as_nanos();
    let result = (elapsed, output.len(), output.capacity());
    drop(output);
    Ok(result)
}
pub fn timing(variant: &str, repetition: usize) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("document capacity timing requires an ordinary build".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let mut groups = GROUPS;
        if repetition == 2 {
            groups.reverse();
        }
        println!(
            "variant,repetition,group,order,population,index,elapsed_ns,result_len,result_capacity"
        );
        for (order, group) in groups.into_iter().enumerate() {
            let retained = (!group.combined).then(|| page(group.rows));
            let mut observations = Vec::with_capacity(25);
            for index in 0..25 {
                let result = if group.combined {
                    timed(combined)?
                } else {
                    timed(|| document(retained.as_ref().unwrap()))?
                };
                observations.push((index, result));
            }
            for (index, (elapsed, len, capacity)) in observations {
                let population = if index < 5 { "warmup" } else { "measured" };
                let index = if index < 5 { index } else { index - 5 };
                println!(
                    "{variant},{repetition},{},{order},{population},{index},{elapsed},{len},{capacity}",
                    group.name
                );
            }
        }
        Ok(())
    }
}

#[cfg(feature = "allocation-counts")]
use crate::buffer_study::events;
#[cfg(feature = "allocation-counts")]
#[derive(Clone, Copy)]
struct Record {
    start: counts::Snapshot,
    held: counts::Snapshot,
    released: counts::Snapshot,
    len: usize,
    capacity: usize,
    trace: events::Trace,
}
#[cfg(feature = "allocation-counts")]
fn counted(
    trace_enabled: bool,
    work: impl FnOnce() -> Result<String, RenderError>,
) -> Result<Record, RenderError> {
    let start = counts::begin();
    if trace_enabled {
        events::begin();
    }
    let owned = black_box(work()?);
    let held = counts::snapshot();
    let len = owned.len();
    let capacity = owned.capacity();
    drop(owned);
    let released = counts::snapshot();
    let trace = events::finish();
    Ok(Record {
        start,
        held,
        released,
        len,
        capacity,
        trace,
    })
}
#[cfg(feature = "allocation-counts")]
fn relative(value: counts::Snapshot, start: counts::Snapshot) -> [usize; 12] {
    [
        value.allocations - start.allocations,
        value.zeroed - start.zeroed,
        value.reallocations - start.reallocations,
        value.deallocations - start.deallocations,
        value.allocated_bytes - start.allocated_bytes,
        value.realloc_old_bytes - start.realloc_old_bytes,
        value.realloc_new_bytes - start.realloc_new_bytes,
        value.deallocated_bytes - start.deallocated_bytes,
        value.failed_allocations - start.failed_allocations,
        value.failed_reallocations - start.failed_reallocations,
        value.live - start.live,
        value.peak - start.live,
    ]
}
#[cfg(feature = "allocation-counts")]
fn body_info(input: &Input) -> Result<(usize, usize), RenderError> {
    let body = yetlet::html::render_to_string(&input.1)?;
    Ok((body.len(), body.capacity()))
}
#[cfg(feature = "allocation-counts")]
pub fn allocation_counts(variant: &str, group: &str) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    let group = GROUPS
        .into_iter()
        .find(|g| g.name == group)
        .ok_or("expected small|medium|large|combined")?;
    let retained = (!group.combined).then(|| page(group.rows));
    let (body_len, body_capacity) = if let Some(input) = retained.as_ref() {
        body_info(input)?
    } else {
        body_info(&page(group.rows))?
    };
    let initial = counts::snapshot();
    let empty = Record {
        start: initial,
        held: initial,
        released: initial,
        len: 0,
        capacity: 0,
        trace: events::finish(),
    };
    let mut records = [empty; 3];
    for record in &mut records {
        *record = if group.combined {
            counted(false, combined)?
        } else {
            counted(true, || document(retained.as_ref().unwrap()))?
        };
    }
    println!(
        "{{\"kind\":\"fixture\",\"variant\":\"{variant}\",\"group\":\"{}\",\"body_len\":{body_len},\"body_capacity\":{body_capacity},\"trace_enabled\":{}}}",
        group.name, !group.combined
    );
    for (operation, r) in records.iter().enumerate() {
        for (point, s) in [
            ("start", r.start),
            ("held", r.held),
            ("released", r.released),
        ] {
            println!(
                "{{\"kind\":\"global\",\"variant\":\"{variant}\",\"group\":\"{}\",\"operation\":{operation},\"point\":\"{point}\",\"result_len\":{},\"result_capacity\":{},\"allocations\":{},\"zeroed\":{},\"reallocations\":{},\"deallocations\":{},\"allocated_bytes\":{},\"realloc_old_bytes\":{},\"realloc_new_bytes\":{},\"deallocated_bytes\":{},\"failed_allocations\":{},\"failed_reallocations\":{},\"live_bytes\":{},\"region_peak_bytes\":{}}}",
                group.name,
                r.len,
                r.capacity,
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
        println!(
            "{{\"kind\":\"trace\",\"variant\":\"{variant}\",\"group\":\"{}\",\"operation\":{operation},\"event_count\":{},\"overflow\":{}}}",
            group.name, r.trace.count, r.trace.overflow
        );
        for (index, row) in r
            .trace
            .data
            .iter()
            .take(r.trace.count.min(events::LIMIT))
            .enumerate()
        {
            let [kind, scope, old, new, live, peak] = *row;
            println!(
                "{{\"kind\":\"event\",\"variant\":\"{variant}\",\"group\":\"{}\",\"operation\":{operation},\"index\":{index},\"event\":\"{}\",\"scope\":\"{}\",\"old_bytes\":{old},\"new_bytes\":{new},\"live_bytes\":{live},\"region_peak_bytes\":{peak}}}",
                group.name,
                events::KINDS[kind],
                events::SCOPES[scope]
            );
        }
    }
    let mut first = None;
    for r in records {
        assert_eq!(r.released.live, r.start.live);
        assert_eq!(r.released.failed_allocations, r.start.failed_allocations);
        assert_eq!(
            r.released.failed_reallocations,
            r.start.failed_reallocations
        );
        assert_eq!(r.trace.overflow, 0);
        assert!(r.trace.count <= events::LIMIT);
        if group.combined {
            assert_eq!(r.trace.count, 0);
        }
        let mut trace = r.trace.data;
        for row in &mut trace[..r.trace.count] {
            row[4] -= r.start.live;
            row[5] -= r.start.live;
        }
        trace[r.trace.count..].fill([0; 6]);
        let observed = (
            relative(r.held, r.start),
            relative(r.released, r.start),
            r.len,
            r.capacity,
            r.trace.count,
            trace,
        );
        if let Some(expected) = first {
            assert_eq!(observed, expected);
        } else {
            first = Some(observed);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yt154_all_size_documents_metadata_and_security_match_literals() {
        for (_, rows) in CASES {
            let input = page(rows);
            let oracle = expected(rows);
            assert_eq!(yetlet::html::render_to_string(&input.1).unwrap(), oracle.0);
            assert_eq!(document(&input).unwrap(), oracle.1);
        }
        let metadata =
            DocumentMetadata::with_language("<>&\"'café", LanguageTag::new("en-US").unwrap());
        let view = Element::new("p").child(View::text("<&>")).into();
        let document = render_document_with_metadata(&metadata, &view).unwrap();
        assert!(document.starts_with("<!doctype html><html lang=\"en-US\">"));
        assert!(document.contains("<title>&lt;&gt;&amp;&quot;&#39;café</title>"));
        assert!(document.ends_with("<body><p>&lt;&amp;&gt;</p></body></html>"));
        let default = yetlet::server_axum::render_document("<&>", &View::text("<&>")).unwrap();
        assert!(default.starts_with("<!doctype html><html lang=\"zh-Hant\">"));
        assert!(default.contains("<title>&lt;&amp;&gt;</title>"));
        for (view, error) in [
            (
                Element::new("main")
                    .child(Element::new("a").attribute("href", "javascript:alert(1)"))
                    .into(),
                RenderError::UnsafeUrl("href".into()),
            ),
            (
                Element::new("main")
                    .child(Element::new("p").attribute("onclick", "bad()"))
                    .into(),
                RenderError::ForbiddenAttribute("onclick".into()),
            ),
            (
                Element::new("input")
                    .child(View::text("invalid child"))
                    .into(),
                RenderError::VoidElementHasChildren("input".into()),
            ),
        ] {
            assert_eq!(render_document_with_metadata(&metadata, &view), Err(error));
        }
    }
}
