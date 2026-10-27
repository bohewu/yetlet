//! YT-152: identical fixture calls against pinned actual Core versions.
use super::*;
use yetlet_html::RenderError;

const ROWS: usize = 1024;
const TEXT: &str = "Safe <&> \"café\"";
const ESCAPED: &str = "Safe &lt;&amp;&gt; &quot;café&quot;";
type Input = (DocumentMetadata, View);

#[derive(Clone, Copy)]
enum Group {
    Single,
    Multi,
    Page,
    Combined,
}
impl Group {
    fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multi => "multi",
            Self::Page => "page",
            Self::Combined => "combined",
        }
    }
    #[cfg(feature = "allocation-counts")]
    fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "single" => Ok(Self::Single),
            "multi" => Ok(Self::Multi),
            "page" => Ok(Self::Page),
            "combined" => Ok(Self::Combined),
            _ => Err("expected single|multi|page|combined"),
        }
    }
}
enum Owned {
    Input(Input, usize),
    Document(String),
}
impl Owned {
    fn result_len(&self) -> usize {
        match self {
            Self::Input(input, nodes) => {
                black_box(input);
                *nodes
            }
            Self::Document(value) => value.len(),
        }
    }
}
fn label(value: &str) -> Result<(), &'static str> {
    if matches!(value, "reference" | "current") {
        Ok(())
    } else {
        Err("expected source label reference|current")
    }
}
fn shape(children: usize) -> Input {
    let view = Element::new("main")
        .children((0..ROWS).map(|_| {
            let mut parent = Element::new("section");
            for _ in 0..children {
                parent = parent.child(View::text(TEXT));
            }
            parent.into()
        }))
        .into();
    (
        DocumentMetadata::with_language(TITLE, LanguageTag::new("en").unwrap()),
        view,
    )
}
fn literal_shape(children: usize) -> (String, String) {
    let parent = format!("<section>{}</section>", ESCAPED.repeat(children));
    let body = format!("<main>{}</main>", parent.repeat(ROWS));
    let document = format!("{PREFIX}{body}</body></html>");
    (body, document)
}
fn single() -> Result<Owned, RenderError> {
    Ok(Owned::Input(black_box(shape(black_box(1))), 1 + ROWS * 2))
}
fn multi() -> Result<Owned, RenderError> {
    Ok(Owned::Input(black_box(shape(black_box(4))), 1 + ROWS * 5))
}
fn full() -> Result<Owned, RenderError> {
    Ok(Owned::Input(
        black_box(page(black_box(ROWS))),
        11 + 7 * ROWS,
    ))
}
fn combined() -> Result<Owned, RenderError> {
    let (metadata, view) = black_box(page(black_box(ROWS)));
    let document = render_document_with_metadata(black_box(&metadata), black_box(&view))?;
    drop(view);
    drop(metadata);
    Ok(Owned::Document(document))
}
pub fn output(variant: &str, directory: &Path) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    if !directory.is_dir() {
        return Err("output directory must already exist".into());
    }
    for (name, input, oracle) in [
        ("single", shape(1), literal_shape(1)),
        ("multi", shape(4), literal_shape(4)),
    ]
    .into_iter()
    .chain(CASES.map(|(name, rows)| (name, page(rows), expected(rows))))
    {
        let body = yetlet_html::render_to_string(&input.1)?;
        let document = render_document_with_metadata(&input.0, &input.1)?;
        assert_eq!(body, oracle.0);
        assert_eq!(document, oracle.1);
        std::fs::write(directory.join(format!("{name}-body.html")), body)?;
        std::fs::write(directory.join(format!("{name}-document.html")), document)?;
    }
    Ok(())
}

#[cfg(not(feature = "allocation-counts"))]
fn timed(work: impl FnOnce() -> Result<Owned, RenderError>) -> Result<(u128, usize), RenderError> {
    let started = std::time::Instant::now();
    let owned = black_box(work()?);
    let elapsed = started.elapsed().as_nanos();
    let len = owned.result_len();
    drop(owned);
    Ok((elapsed, len))
}
pub fn timing(variant: &str, repetition: usize) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("child timing requires an ordinary build".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let mut groups = [Group::Single, Group::Multi, Group::Page, Group::Combined];
        if repetition == 2 {
            groups.reverse();
        }
        println!("variant,repetition,group,order,population,index,elapsed_ns,result_len");
        for (order, group) in groups.into_iter().enumerate() {
            let mut observations = Vec::with_capacity(25);
            for index in 0..25 {
                let (elapsed, len) = match group {
                    Group::Single => timed(single)?,
                    Group::Multi => timed(multi)?,
                    Group::Page => timed(full)?,
                    Group::Combined => timed(combined)?,
                };
                observations.push((index, elapsed, len));
            }
            for (index, elapsed, len) in observations {
                let population = if index < 5 { "warmup" } else { "measured" };
                let index = if index < 5 { index } else { index - 5 };
                println!(
                    "{variant},{repetition},{},{order},{population},{index},{elapsed},{len}",
                    group.name()
                );
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
fn counted(
    work: impl FnOnce() -> Result<Owned, RenderError>,
) -> Result<(counts::Snapshot, counts::Snapshot, counts::Snapshot, usize), RenderError> {
    let start = counts::begin();
    let owned = black_box(work()?);
    let held = counts::snapshot();
    let len = owned.result_len();
    drop(owned);
    let released = counts::snapshot();
    Ok((start, held, released, len))
}
#[cfg(feature = "allocation-counts")]
pub fn allocation_counts(variant: &str, group: &str) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    let group = Group::parse(group)?;
    let s = counts::snapshot();
    let mut records = [(s, s, s, 0); 3];
    for record in &mut records {
        *record = match group {
            Group::Single => counted(single)?,
            Group::Multi => counted(multi)?,
            Group::Page => counted(full)?,
            Group::Combined => counted(combined)?,
        };
    }
    println!(
        "variant,group,operation,point,result_len,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (operation, (start, held, released, len)) in records.iter().enumerate() {
        for (point, s) in [("start", start), ("held", held), ("released", released)] {
            println!(
                "{variant},{},{operation},{point},{len},{},{},{},{},{},{},{},{},{},{},{},{}",
                group.name(),
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
    for (start, held, released, _) in records {
        assert_eq!(released.live, start.live);
        assert_eq!(released.failed_allocations, start.failed_allocations);
        assert_eq!(released.failed_reallocations, start.failed_reallocations);
        let observed = (relative(held, start), relative(released, start));
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
    fn yt152_shapes_literals_and_nested_security_outcomes() {
        for (input, oracle) in [(shape(1), literal_shape(1)), (shape(4), literal_shape(4))]
            .into_iter()
            .chain(CASES.map(|(_, rows)| (page(rows), expected(rows))))
        {
            assert_eq!(yetlet_html::render_to_string(&input.1).unwrap(), oracle.0);
            assert_eq!(
                render_document_with_metadata(&input.0, &input.1).unwrap(),
                oracle.1
            );
            assert_eq!(input.0.title(), TITLE);
        }
        for (attribute, value, expected) in [
            (
                "href",
                "javascript:alert(1)",
                RenderError::UnsafeUrl("href".into()),
            ),
            (
                "onclick",
                "alert(1)",
                RenderError::ForbiddenAttribute("onclick".into()),
            ),
        ] {
            let view = Element::new("main")
                .child(
                    Element::new("a")
                        .attribute(attribute, value)
                        .child(View::text(TEXT)),
                )
                .into();
            assert_eq!(yetlet_html::render_to_string(&view), Err(expected));
        }
        let view = Element::new("input").child(View::text(TEXT)).into();
        assert!(matches!(
            yetlet_html::render_to_string(&view),
            Err(RenderError::VoidElementHasChildren(_))
        ));
    }
}
