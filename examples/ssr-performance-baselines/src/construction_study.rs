//! YT-151: fixed source-derived fixture attribution, using actual product APIs.
use super::*;

const ROWS: usize = 1024;
const NODES: usize = 11 + 7 * ROWS;
type Input = (DocumentMetadata, View);

pub(crate) mod events {
    #[derive(Clone, Copy)]
    #[repr(usize)]
    pub enum Bucket {
        Format,
        String,
        AttributeVec,
        ChildrenVec,
        Metadata,
        Node,
        Other,
    }
    pub const NAMES: [&str; 7] = [
        "format",
        "string",
        "attribute_vec",
        "children_vec",
        "metadata",
        "node",
        "other",
    ];

    #[cfg(not(feature = "allocation-counts"))]
    #[inline]
    pub fn tagged<T>(_: Bucket, work: impl FnOnce() -> T) -> T {
        work()
    }

    #[cfg(feature = "allocation-counts")]
    mod counted {
        use super::Bucket;
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

        // alloc, zeroed, realloc, A bytes, R old/new bytes, failed A/R.
        static EVENTS: [[AtomicUsize; 8]; 7] = [const { [const { AtomicUsize::new(0) }; 8] }; 7];
        static ACTIVE: AtomicUsize = AtomicUsize::new(Bucket::Other as usize);

        struct Restore(usize);
        impl Drop for Restore {
            fn drop(&mut self) {
                ACTIVE.store(self.0, Relaxed);
            }
        }

        #[inline(never)]
        pub fn tagged<T>(bucket: Bucket, work: impl FnOnce() -> T) -> T {
            let _restore = Restore(ACTIVE.swap(bucket as usize, Relaxed));
            work()
        }

        pub fn snapshot() -> [[usize; 8]; 7] {
            std::array::from_fn(|b| std::array::from_fn(|f| EVENTS[b][f].load(Relaxed)))
        }

        pub fn allocation(bytes: usize, zeroed: bool, failed: bool) {
            let row = &EVENTS[ACTIVE.load(Relaxed)];
            if failed {
                row[6].fetch_add(1, Relaxed);
            } else {
                row[0].fetch_add(1, Relaxed);
                row[1].fetch_add(usize::from(zeroed), Relaxed);
                row[3].fetch_add(bytes, Relaxed);
            }
        }

        pub fn reallocation(old: usize, new: usize, failed: bool) {
            let row = &EVENTS[ACTIVE.load(Relaxed)];
            if failed {
                row[7].fetch_add(1, Relaxed);
            } else {
                row[2].fetch_add(1, Relaxed);
                row[4].fetch_add(old, Relaxed);
                row[5].fetch_add(new, Relaxed);
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
    pub use counted::{allocation, reallocation, snapshot, tagged};
}

use events::{Bucket, tagged};

fn element(tag: &'static str) -> Element {
    tagged(Bucket::Node, || Element::new(tag))
}
fn into_view(element: Element) -> View {
    tagged(Bucket::Node, || element.into())
}
fn text(value: impl Into<String>) -> View {
    let value = tagged(Bucket::String, || value.into());
    tagged(Bucket::Node, || View::text(value))
}
fn attribute(parent: Element, name: &'static str, value: impl Into<String>) -> Element {
    let value = tagged(Bucket::String, || value.into());
    tagged(Bucket::AttributeVec, || parent.attribute(name, value))
}
fn child(parent: Element, value: impl Into<View>) -> Element {
    let value = tagged(Bucket::Node, || value.into());
    tagged(Bucket::ChildrenVec, || parent.child(value))
}
fn children(parent: Element, values: impl IntoIterator<Item = View>) -> Element {
    tagged(Bucket::ChildrenVec, || parent.children(values))
}

#[derive(Clone, Copy)]
#[repr(usize)]
enum Field {
    Id,
    Title,
    Heading,
    Text,
    Url,
}
fn formatted(field: Field, index: usize) -> String {
    tagged(Bucket::Format, || match field {
        Field::Id => format!("{index:04}"),
        Field::Title => format!("Row {index:04} <&> \"quoted\" 'café'"),
        Field::Heading => format!("Record {index:04}"),
        Field::Text => format!("Public <&> \"value\" 'café' {index:04}"),
        Field::Url => format!("/records/{index:04}?view=full&public=1"),
    })
}
fn row_with(mut value: impl FnMut(Field) -> String) -> View {
    let row = attribute(element("li"), "data-row", value(Field::Id));
    let row = attribute(row, "title", value(Field::Title));
    let row = child(row, child(element("h2"), text(value(Field::Heading))));
    let row = child(row, child(element("p"), text(value(Field::Text))));
    let row = child(
        row,
        child(
            attribute(element("a"), "href", value(Field::Url)),
            text("Open record"),
        ),
    );
    into_view(row)
}
fn attributed_row(index: usize) -> View {
    row_with(|field| formatted(field, index))
}
fn frame(rows: usize, row: impl FnMut(usize) -> View) -> Input {
    let view = attribute(element("main"), "id", "ssr-report");
    let view = child(
        view,
        child(
            child(
                element("header"),
                child(element("h1"), text("SSR report <&> \"title\" 'café'")),
            ),
            child(element("p"), text("Public synthetic rows")),
        ),
    );
    let view = child(
        view,
        child(
            attribute(element("section"), "aria-label", "Records"),
            children(element("ul"), (0..rows).map(row)),
        ),
    );
    let view = child(
        view,
        child(
            element("footer"),
            child(element("p"), text("End of public report")),
        ),
    );
    let view = into_view(view);
    let metadata = tagged(Bucket::Metadata, || {
        DocumentMetadata::with_language(TITLE, LanguageTag::new("en").unwrap())
    });
    (metadata, view)
}
fn attributed_page(rows: usize) -> Input {
    frame(rows, attributed_row)
}

struct RowPayload {
    values: [Option<String>; 5],
}
fn payload(index: usize) -> RowPayload {
    RowPayload {
        values: [
            Field::Id,
            Field::Title,
            Field::Heading,
            Field::Text,
            Field::Url,
        ]
        .map(|field| Some(formatted(field, index))),
    }
}
fn prepared(rows: usize) -> Vec<Option<RowPayload>> {
    (0..rows).map(|index| Some(payload(index))).collect()
}
fn assembled(payloads: &mut [Option<RowPayload>]) -> Input {
    frame(payloads.len(), |index| {
        let mut payload = payloads[index].take().expect("one owned payload per row");
        row_with(|field| payload.values[field as usize].take().unwrap())
    })
}

#[derive(Default)]
struct Inventory {
    elements: usize,
    texts: usize,
    attributes: usize,
    edges: usize,
    child_histogram: [usize; 5],
    max_children: usize,
    string_len: usize,
    string_capacity: usize,
}
fn inventory(view: &View, result: &mut Inventory) {
    match view {
        View::Text(value) => {
            result.texts += 1;
            result.string_len += value.len();
            result.string_capacity += value.capacity();
        }
        View::Element(element) => {
            result.elements += 1;
            result.attributes += element.attributes().len();
            for attribute in element.attributes() {
                result.string_len += attribute.value.len();
                result.string_capacity += attribute.value.capacity();
            }
            let count = element.child_views().len();
            result.edges += count;
            result.child_histogram[count.min(4)] += 1;
            result.max_children = result.max_children.max(count);
            for child in element.child_views() {
                inventory(child, result);
            }
        }
    }
}

pub fn output(directory: &Path) -> Result<(), Box<dyn Error>> {
    if !directory.is_dir() {
        return Err("output directory must exist".into());
    }
    for (name, rows) in CASES {
        let actual = attributed_page(rows);
        assert_eq!(actual, page(rows));
        let body = yetlet::html::render_to_string(&actual.1)?;
        let document = render_document_with_metadata(&actual.0, &actual.1)?;
        let oracle = expected(rows);
        assert_eq!(body, oracle.0);
        assert_eq!(document, oracle.1);
        std::fs::write(directory.join(format!("{name}-body.html")), body)?;
        std::fs::write(directory.join(format!("{name}-document.html")), document)?;
    }
    let (_, view) = attributed_page(ROWS);
    let mut info = Inventory::default();
    inventory(&view, &mut info);
    println!("kind,value");
    for (name, value) in [
        ("elements", info.elements),
        ("texts", info.texts),
        ("attributes", info.attributes),
        ("edges", info.edges),
        ("public_string_len_bytes", info.string_len),
        ("public_string_capacity_bytes", info.string_capacity),
        ("max_children", info.max_children),
        ("size_view", std::mem::size_of::<View>()),
        ("size_element", std::mem::size_of::<Element>()),
        (
            "size_attribute",
            std::mem::size_of::<yetlet::view::Attribute>(),
        ),
        ("size_metadata", std::mem::size_of::<DocumentMetadata>()),
    ] {
        println!("{name},{value}");
    }
    for (index, count) in info.child_histogram.into_iter().enumerate() {
        println!(
            "children_{}{},{count}",
            index,
            if index == 4 { "_plus" } else { "" }
        );
    }
    Ok(())
}

#[cfg(not(feature = "allocation-counts"))]
fn format_operation() -> usize {
    let mut bytes = 0;
    for index in 0..ROWS {
        let owned = black_box(payload(black_box(index)));
        bytes += owned
            .values
            .iter()
            .map(|x| x.as_ref().unwrap().len())
            .sum::<usize>();
        black_box(&owned);
        drop(owned);
    }
    black_box(bytes)
}

pub fn timing(repetition: usize) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("construction timing requires an ordinary build".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition1|2".into());
        }
        let probes = if repetition == 1 {
            ["format", "assembly"]
        } else {
            ["assembly", "format"]
        };
        println!("repetition,probe,order,population,index,elapsed_ns,result_len");
        for (order, probe) in probes.into_iter().enumerate() {
            let mut observations = Vec::with_capacity(25);
            for index in 0..25 {
                let (elapsed, len) = if probe == "format" {
                    let started = std::time::Instant::now();
                    let len = format_operation();
                    (started.elapsed().as_nanos(), len)
                } else {
                    let mut values = prepared(ROWS);
                    let started = std::time::Instant::now();
                    let owned = black_box(assembled(black_box(&mut values)));
                    let elapsed = started.elapsed().as_nanos();
                    drop(owned);
                    drop(values);
                    (elapsed, NODES)
                };
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
pub fn allocation_counts() -> Result<(), Box<dyn Error>> {
    let global = counts::snapshot();
    let buckets = events::snapshot();
    let mut records = [(global, global, global, buckets, buckets); 3];
    for record in &mut records {
        let start = counts::begin();
        let before = events::snapshot();
        let owned = black_box(attributed_page(black_box(ROWS)));
        let held = counts::snapshot();
        let after = events::snapshot();
        drop(owned);
        let released = counts::snapshot();
        *record = (start, held, released, before, after);
    }
    println!(
        "kind,operation,point,bucket,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (op, (start, held, released, before, after)) in records.iter().enumerate() {
        for (point, s) in [("start", start), ("held", held), ("released", released)] {
            println!(
                "global,{op},{point},all,{},{},{},{},{},{},{},{},{},{},{},{}",
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
        for (b, name) in events::NAMES.into_iter().enumerate() {
            let d: [usize; 8] = std::array::from_fn(|f| after[b][f] - before[b][f]);
            println!(
                "bucket,{op},creation,{name},{},{},{},,{},{},{},,{},{},,",
                d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]
            );
        }
    }
    let mut first = None;
    for (start, held, released, before, after) in records {
        assert_eq!(released.live, start.live);
        assert_eq!(released.failed_allocations, start.failed_allocations);
        assert_eq!(released.failed_reallocations, start.failed_reallocations);
        let delta: [[usize; 8]; 7] =
            std::array::from_fn(|b| std::array::from_fn(|f| after[b][f] - before[b][f]));
        let sum: [usize; 8] = std::array::from_fn(|f| delta.iter().map(|b| b[f]).sum());
        let g = relative(held, start);
        assert_eq!(sum, [g[0], g[1], g[2], g[4], g[5], g[6], g[8], g[9]]);
        let observed = (g, relative(released, start), delta);
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
    fn yt151_structural_and_prepared_assembly_match_actual_page() {
        for (_, rows) in CASES {
            let expected = page(rows);
            assert_eq!(attributed_page(rows), expected);
            let mut values = prepared(rows);
            assert_eq!(assembled(&mut values), expected);
            assert!(values.iter().all(Option::is_none));
            let literal = super::super::expected(rows);
            assert_eq!(
                yetlet::html::render_to_string(&expected.1).unwrap(),
                literal.0
            );
            assert_eq!(
                render_document_with_metadata(&expected.0, &expected.1).unwrap(),
                literal.1
            );
        }
    }
    #[test]
    #[cfg(feature = "allocation-counts")]
    fn yt151_nested_restore_and_actual_allocator_dispatch() {
        assert_eq!(events::active(), Bucket::Other as usize);
        tagged(Bucket::ChildrenVec, || {
            tagged(Bucket::Node, || {
                assert_eq!(events::active(), Bucket::Node as usize)
            });
            assert_eq!(events::active(), Bucket::ChildrenVec as usize);
            let panic = std::panic::catch_unwind(|| {
                tagged(Bucket::Metadata, || panic!("intentional restoration check"))
            });
            assert!(panic.is_err());
            assert_eq!(events::active(), Bucket::ChildrenVec as usize);
        });
        assert_eq!(events::active(), Bucket::Other as usize);
        let before = events::snapshot();
        let owned = tagged(Bucket::Format, || black_box(Box::new(black_box([7u8; 16]))));
        let after = events::snapshot();
        assert_eq!(after[0][0] - before[0][0], 1);
        assert_eq!(after[0][3] - before[0][3], 16);
        drop(owned);
        assert_eq!(events::active(), Bucket::Other as usize);
    }
}
