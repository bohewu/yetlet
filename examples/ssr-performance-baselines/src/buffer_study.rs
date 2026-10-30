//! YT-153: source-derived adapter observations using the actual HTML renderer.
use super::*;
use yetlet::html::{RenderError, escape_text};

const ROWS: usize = 1024;
const OPEN: &str = "<!doctype html><html lang=\"";
const HEAD: &str = "\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>";
const BODY_OPEN: &str = "</title></head><body>";
const CLOSE: &str = "</body></html>";
const POINT_NAMES: [&str; 9] = [
    "initial",
    "language",
    "head",
    "title_ready",
    "title_appended",
    "body_open",
    "body_ready",
    "body_appended",
    "closed",
];

pub(crate) mod events {
    #[derive(Clone, Copy)]
    #[repr(usize)]
    pub enum Scope {
        Body,
        Title,
        Envelope,
        Other,
    }
    #[cfg(not(feature = "allocation-counts"))]
    #[inline]
    pub fn tagged<T>(_: Scope, work: impl FnOnce() -> T) -> T {
        work()
    }

    #[cfg(feature = "allocation-counts")]
    mod counted {
        use super::Scope;
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
        pub const LIMIT: usize = 64;
        pub const SCOPES: [&str; 4] = ["body", "title", "envelope", "other"];
        pub const KINDS: [&str; 7] = [
            "unused",
            "alloc",
            "zeroed",
            "realloc",
            "dealloc",
            "failed_alloc",
            "failed_realloc",
        ];
        static ENABLED: AtomicBool = AtomicBool::new(false);
        static ACTIVE: AtomicUsize = AtomicUsize::new(Scope::Other as usize);
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        static OVERFLOW: AtomicUsize = AtomicUsize::new(0);
        // kind, call site, old/new requests, post-event global live/peak.
        static DATA: [[AtomicUsize; 6]; LIMIT] =
            [const { [const { AtomicUsize::new(0) }; 6] }; LIMIT];
        struct Restore(usize);
        impl Drop for Restore {
            fn drop(&mut self) {
                ACTIVE.store(self.0, Relaxed);
            }
        }
        #[inline(never)]
        pub fn tagged<T>(scope: Scope, work: impl FnOnce() -> T) -> T {
            let _restore = Restore(ACTIVE.swap(scope as usize, Relaxed));
            work()
        }
        pub fn record(kind: usize, old: usize, new: usize) {
            if !ENABLED.load(Relaxed) {
                return;
            }
            let index = COUNT.fetch_add(1, Relaxed);
            if index >= LIMIT {
                OVERFLOW.fetch_add(1, Relaxed);
                return;
            }
            let global = crate::counts::snapshot();
            let values = [
                kind,
                ACTIVE.load(Relaxed),
                old,
                new,
                global.live,
                global.peak,
            ];
            for (cell, value) in DATA[index].iter().zip(values) {
                cell.store(value, Relaxed);
            }
        }
        #[derive(Clone, Copy)]
        pub struct Trace {
            pub count: usize,
            pub overflow: usize,
            pub data: [[usize; 6]; LIMIT],
        }
        pub fn begin() {
            COUNT.store(0, Relaxed);
            OVERFLOW.store(0, Relaxed);
            ENABLED.store(true, Relaxed);
        }
        pub fn finish() -> Trace {
            ENABLED.store(false, Relaxed);
            Trace {
                count: COUNT.load(Relaxed),
                overflow: OVERFLOW.load(Relaxed),
                data: std::array::from_fn(|i| std::array::from_fn(|j| DATA[i][j].load(Relaxed))),
            }
        }
        #[cfg(test)]
        pub fn active() -> usize {
            ACTIVE.load(Relaxed)
        }
    }
    #[cfg(all(test, feature = "allocation-counts"))]
    pub use counted::active;
    #[cfg(feature = "allocation-counts")]
    pub use counted::{KINDS, LIMIT, SCOPES, Trace, begin, finish, record, tagged};
}
use events::{Scope, tagged};

#[derive(Clone, Copy, Debug, Default)]
struct Point {
    len: usize,
    capacity: usize,
    temporary_len: usize,
    temporary_capacity: usize,
    #[cfg(feature = "allocation-counts")]
    live: usize,
    #[cfg(feature = "allocation-counts")]
    peak: usize,
}
type Points = [Point; 9];
fn point(envelope: (usize, usize), temporary: (usize, usize)) -> Point {
    #[cfg(feature = "allocation-counts")]
    let global = counts::snapshot();
    Point {
        len: envelope.0,
        capacity: envelope.1,
        temporary_len: temporary.0,
        temporary_capacity: temporary.1,
        #[cfg(feature = "allocation-counts")]
        live: global.live,
        #[cfg(feature = "allocation-counts")]
        peak: global.peak,
    }
}
fn mark(points: &mut Points, index: usize, value: &String) {
    points[index] = point((value.len(), value.capacity()), (0, 0));
}
fn title(value: &str, envelope: (usize, usize), points: &mut Points) -> String {
    let owned = tagged(Scope::Title, || escape_text(value));
    points[3] = point(envelope, (owned.len(), owned.capacity()));
    owned
}
fn body(view: &View, envelope: (usize, usize), points: &mut Points) -> Result<String, RenderError> {
    let owned = tagged(Scope::Body, || yetlet::html::render_to_string(view))?;
    points[6] = point(envelope, (owned.len(), owned.capacity()));
    Ok(owned)
}
// Keep the original temporary append expressions and destruction boundaries.
fn observed_document(
    metadata: &DocumentMetadata,
    view: &View,
    points: &mut Points,
) -> Result<String, RenderError> {
    tagged(Scope::Envelope, || {
        let mut html = String::from(OPEN);
        mark(points, 0, &html);
        html.push_str(metadata.language().as_str());
        mark(points, 1, &html);
        html.push_str(HEAD);
        mark(points, 2, &html);
        let envelope = (html.len(), html.capacity());
        html.push_str(&title(metadata.title(), envelope, points));
        mark(points, 4, &html);
        html.push_str(BODY_OPEN);
        mark(points, 5, &html);
        let envelope = (html.len(), html.capacity());
        html.push_str(&body(view, envelope, points)?);
        mark(points, 7, &html);
        html.push_str(CLOSE);
        mark(points, 8, &html);
        Ok(html)
    })
}
fn prefix(metadata: &DocumentMetadata) -> String {
    let mut html = String::from(OPEN);
    html.push_str(metadata.language().as_str());
    html.push_str(HEAD);
    html.push_str(&escape_text(metadata.title()));
    html.push_str(BODY_OPEN);
    html
}
fn envelope(metadata: &DocumentMetadata, body: &str) -> String {
    let mut html = prefix(metadata);
    html.push_str(body);
    html.push_str(CLOSE);
    html
}
fn check_probes(metadata: &DocumentMetadata, body: &str, document: &str) {
    assert_eq!(envelope(metadata, body), document);
    let mut prepared = prefix(metadata);
    prepared.push_str(body);
    assert_eq!(prepared, document.strip_suffix(CLOSE).unwrap());
    prepared.push_str(CLOSE);
    assert_eq!(prepared, document);
}
pub fn output(directory: &Path) -> Result<(), Box<dyn Error>> {
    if !directory.is_dir() {
        return Err("output directory must already exist".into());
    }
    for (name, rows) in CASES {
        let (metadata, view) = page(rows);
        let mut points = [Point::default(); 9];
        let actual_body = yetlet::html::render_to_string(&view)?;
        let actual_document = observed_document(&metadata, &view, &mut points)?;
        let oracle = expected(rows);
        assert_eq!(actual_body, oracle.0);
        assert_eq!(actual_document, oracle.1);
        assert_eq!(
            actual_document,
            render_document_with_metadata(&metadata, &view)?
        );
        check_probes(&metadata, &actual_body, &actual_document);
        assert_eq!(points[6].temporary_len, actual_body.len());
        assert_eq!(points[8].capacity, actual_document.capacity());
        std::fs::write(directory.join(format!("{name}-body.html")), actual_body)?;
        std::fs::write(
            directory.join(format!("{name}-document.html")),
            actual_document,
        )?;
    }
    Ok(())
}

pub fn timing(repetition: usize) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("buffer timing requires an ordinary build".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let (metadata, view) = page(ROWS);
        let body = yetlet::html::render_to_string(&view)?;
        let mut probes = ["envelope", "body_append", "suffix_append"];
        if repetition == 2 {
            probes.reverse();
        }
        println!("repetition,probe,order,population,index,elapsed_ns,result_len");
        for (order, probe) in probes.into_iter().enumerate() {
            let mut observations = Vec::with_capacity(25);
            for index in 0..25 {
                let (owned, elapsed) = if probe == "envelope" {
                    let started = std::time::Instant::now();
                    let owned = black_box(envelope(black_box(&metadata), black_box(&body)));
                    let elapsed = started.elapsed().as_nanos();
                    (owned, elapsed)
                } else {
                    let mut prepared = prefix(&metadata);
                    if probe == "suffix_append" {
                        prepared.push_str(&body);
                    }
                    if probe == "body_append" {
                        let started = std::time::Instant::now();
                        prepared.push_str(black_box(&body));
                        let owned = black_box(prepared);
                        let elapsed = started.elapsed().as_nanos();
                        (owned, elapsed)
                    } else {
                        let started = std::time::Instant::now();
                        prepared.push_str(black_box(CLOSE));
                        let owned = black_box(prepared);
                        let elapsed = started.elapsed().as_nanos();
                        (owned, elapsed)
                    }
                };
                let len = owned.len();
                drop(owned);
                observations.push((index, elapsed, len));
            }
            for (index, elapsed, len) in observations {
                let population = if index < 5 { "warmup" } else { "measured" };
                let index = if index < 5 { index } else { index - 5 };
                println!("{repetition},{probe},{order},{population},{index},{elapsed},{len}");
            }
        }
        Ok(())
    }
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
fn print_global(operation: usize, point: &str, s: counts::Snapshot) {
    println!(
        "{{\"kind\":\"global\",\"operation\":{operation},\"point\":\"{point}\",\"allocations\":{},\"zeroed\":{},\"reallocations\":{},\"deallocations\":{},\"allocated_bytes\":{},\"realloc_old_bytes\":{},\"realloc_new_bytes\":{},\"deallocated_bytes\":{},\"failed_allocations\":{},\"failed_reallocations\":{},\"live_bytes\":{},\"region_peak_bytes\":{}}}",
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
#[cfg(feature = "allocation-counts")]
pub fn allocation_counts() -> Result<(), Box<dyn Error>> {
    let (metadata, view) = page(ROWS);
    let initial = counts::snapshot();
    let trace = events::finish();
    let mut records = [(initial, initial, initial, [Point::default(); 9], trace); 3];
    for record in &mut records {
        let start = counts::begin();
        let mut points = [Point::default(); 9];
        events::begin();
        let owned = black_box(observed_document(
            black_box(&metadata),
            black_box(&view),
            &mut points,
        )?);
        let held = counts::snapshot();
        drop(owned);
        let released = counts::snapshot();
        let trace = events::finish();
        *record = (start, held, released, points, trace);
    }
    for (operation, (start, held, released, points, trace)) in records.iter().enumerate() {
        print_global(operation, "start", *start);
        print_global(operation, "held", *held);
        print_global(operation, "released", *released);
        for (name, p) in POINT_NAMES.into_iter().zip(points) {
            println!(
                "{{\"kind\":\"boundary\",\"operation\":{operation},\"point\":\"{name}\",\"envelope_len\":{},\"envelope_capacity\":{},\"temporary_len\":{},\"temporary_capacity\":{},\"live_bytes\":{},\"region_peak_bytes\":{}}}",
                p.len, p.capacity, p.temporary_len, p.temporary_capacity, p.live, p.peak
            );
        }
        println!(
            "{{\"kind\":\"trace\",\"operation\":{operation},\"event_count\":{},\"overflow\":{}}}",
            trace.count, trace.overflow
        );
        for (index, event) in trace
            .data
            .iter()
            .take(trace.count.min(events::LIMIT))
            .enumerate()
        {
            let [kind, scope, old, new, live, peak] = *event;
            println!(
                "{{\"kind\":\"event\",\"operation\":{operation},\"index\":{index},\"event\":\"{}\",\"scope\":\"{}\",\"old_bytes\":{old},\"new_bytes\":{new},\"live_bytes\":{live},\"region_peak_bytes\":{peak}}}",
                events::KINDS[kind],
                events::SCOPES[scope]
            );
        }
    }
    let mut first = None;
    for (start, held, released, points, trace) in records {
        assert_eq!(trace.overflow, 0);
        assert!(trace.count <= events::LIMIT);
        assert_eq!(released.live, start.live);
        assert_eq!(released.failed_allocations, start.failed_allocations);
        assert_eq!(released.failed_reallocations, start.failed_reallocations);
        let p = points.map(|p| {
            [
                p.len,
                p.capacity,
                p.temporary_len,
                p.temporary_capacity,
                p.live - start.live,
                p.peak - start.live,
            ]
        });
        let mut events = trace.data;
        for row in &mut events[..trace.count] {
            row[4] -= start.live;
            row[5] -= start.live;
        }
        // Ignore unused slots; the trace is bounded but not a ring or owner table.
        events[trace.count..].fill([0; 6]);
        let observed = (
            relative(held, start),
            relative(released, start),
            p,
            trace.count,
            events,
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
    fn yt153_composition_probes_literals_and_errors_match_actual() {
        for (_, rows) in CASES {
            let (metadata, view) = page(rows);
            let mut points = [Point::default(); 9];
            let document = observed_document(&metadata, &view, &mut points).unwrap();
            let actual_body = yetlet::html::render_to_string(&view).unwrap();
            let oracle = expected(rows);
            assert_eq!(
                document,
                render_document_with_metadata(&metadata, &view).unwrap()
            );
            assert_eq!(document, oracle.1);
            assert_eq!(actual_body, oracle.0);
            check_probes(&metadata, &actual_body, &document);
            assert_eq!(points[6].temporary_len, actual_body.len());
            assert_eq!(points[8].len, document.len());
            assert_eq!(points[8].capacity, document.capacity());
        }
        let metadata = DocumentMetadata::new("Untrusted <&> title");
        let invalid = [
            Element::new("main")
                .child(Element::new("a").attribute("href", "javascript:alert(1)"))
                .into(),
            Element::new("main")
                .child(Element::new("p").attribute("onclick", "bad()"))
                .into(),
            Element::new("input")
                .child(View::text("invalid child"))
                .into(),
        ];
        for view in invalid {
            let actual = render_document_with_metadata(&metadata, &view);
            assert!(actual.is_err());
            assert_eq!(
                observed_document(&metadata, &view, &mut [Point::default(); 9]),
                actual
            );
        }
    }
    #[test]
    #[cfg(feature = "allocation-counts")]
    fn yt153_scope_restore_and_actual_allocator_event_dispatch() {
        assert_eq!(events::active(), Scope::Other as usize);
        tagged(Scope::Envelope, || {
            tagged(Scope::Body, || {
                assert_eq!(events::active(), Scope::Body as usize)
            });
            let panic = std::panic::catch_unwind(|| {
                tagged(Scope::Title, || panic!("intentional tag unwind"))
            });
            assert!(panic.is_err());
            assert_eq!(events::active(), Scope::Envelope as usize);
        });
        assert_eq!(events::active(), Scope::Other as usize);
        events::begin();
        let owned = tagged(Scope::Body, || black_box(Box::new(black_box([7u8; 16]))));
        tagged(Scope::Envelope, || drop(owned));
        let trace = events::finish();
        assert_eq!(trace.count, 2);
        assert_eq!(trace.overflow, 0);
        assert_eq!(&trace.data[0][..4], &[1, Scope::Body as usize, 0, 16]);
        assert_eq!(&trace.data[1][..4], &[4, Scope::Envelope as usize, 16, 0]);
        assert_eq!(trace.data[0][4] - trace.data[1][4], 16);
    }
}
