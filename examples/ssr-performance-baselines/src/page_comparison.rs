//! YT-150: identical sample code compiled against two pinned product sources.
use super::*;
use yetlet::html::RenderError;

const ROWS: usize = 1024;
const NODES: usize = 11 + 7 * ROWS;

type Input = (DocumentMetadata, View);

#[derive(Clone, Copy)]
enum Phase {
    Construct,
    Document,
    Combined,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Self::Construct => "construct",
            Self::Document => "document",
            Self::Combined => "combined",
        }
    }

    #[cfg(feature = "allocation-counts")]
    fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "construct" => Ok(Self::Construct),
            "document" => Ok(Self::Document),
            "combined" => Ok(Self::Combined),
            _ => Err("expected construct|document|combined"),
        }
    }
}

enum Owned {
    Construct(Input),
    Document(String),
}

impl Owned {
    fn result_len(&self) -> usize {
        match self {
            Self::Construct(input) => {
                black_box(input);
                NODES
            }
            Self::Document(value) => value.len(),
        }
    }
}

fn construct() -> Result<Owned, RenderError> {
    Ok(Owned::Construct(black_box(page(black_box(ROWS)))))
}

fn document(input: &Input) -> Result<Owned, RenderError> {
    render_document_with_metadata(black_box(&input.0), black_box(&input.1)).map(Owned::Document)
}

fn combined() -> Result<Owned, RenderError> {
    let (metadata, view) = black_box(page(black_box(ROWS)));
    let output = render_document_with_metadata(black_box(&metadata), black_box(&view))?;
    // Model a handler returning the document after its owned inputs are released.
    drop(view);
    drop(metadata);
    Ok(Owned::Document(output))
}

fn label(value: &str) -> Result<(), &'static str> {
    if matches!(value, "reference" | "current") {
        Ok(())
    } else {
        Err("expected source label reference|current")
    }
}

pub fn output(variant: &str, directory: &Path) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    if !directory.is_dir() {
        return Err("output directory must already exist".into());
    }
    for (name, rows) in CASES {
        let (body, document) = verify(rows)?;
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
    let result_len = owned.result_len();
    drop(owned);
    Ok((elapsed, result_len))
}

pub fn timing(variant: &str, repetition: usize) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    #[cfg(feature = "allocation-counts")]
    {
        let _ = repetition;
        Err("page timing requires an ordinary build".into())
    }
    #[cfg(not(feature = "allocation-counts"))]
    {
        if !(1..=2).contains(&repetition) {
            return Err("expected repetition 1|2".into());
        }
        let mut phases = [Phase::Construct, Phase::Document, Phase::Combined];
        if repetition == 2 {
            phases.reverse();
        }
        println!("variant,repetition,phase,order,population,index,elapsed_ns,result_len");
        for (order, phase) in phases.into_iter().enumerate() {
            let retained = matches!(phase, Phase::Document).then(|| page(ROWS));
            let mut observations = Vec::with_capacity(25);
            for index in 0..25 {
                // Dispatch is outside each timed region.
                let (elapsed, len) = match phase {
                    Phase::Construct => timed(construct)?,
                    Phase::Document => timed(|| document(retained.as_ref().unwrap()))?,
                    Phase::Combined => timed(combined)?,
                };
                observations.push((index, elapsed, len));
            }
            for (index, elapsed, len) in observations {
                let population = if index < 5 { "warmup" } else { "measured" };
                let index = if index < 5 { index } else { index - 5 };
                println!(
                    "{variant},{repetition},{},{order},{population},{index},{elapsed},{len}",
                    phase.name()
                );
            }
        }
        Ok(())
    }
}

#[cfg(feature = "allocation-counts")]
type Record = (counts::Snapshot, counts::Snapshot, counts::Snapshot, usize);

#[cfg(feature = "allocation-counts")]
fn counted(work: impl FnOnce() -> Result<Owned, RenderError>) -> Result<Record, RenderError> {
    let start = counts::begin();
    let owned = black_box(work()?);
    let held = counts::snapshot();
    let len = owned.result_len();
    drop(owned);
    let released = counts::snapshot();
    Ok((start, held, released, len))
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
pub fn allocation_counts(variant: &str, phase: &str) -> Result<(), Box<dyn Error>> {
    label(variant)?;
    let phase = Phase::parse(phase)?;
    let retained = matches!(phase, Phase::Document).then(|| page(ROWS));
    let initial = counts::snapshot();
    let mut records = [(initial, initial, initial, 0); 3];
    for record in &mut records {
        *record = match phase {
            Phase::Construct => counted(construct)?,
            Phase::Document => counted(|| document(retained.as_ref().unwrap()))?,
            Phase::Combined => counted(combined)?,
        };
    }
    println!(
        "variant,phase,operation,point,result_len,allocations,zeroed,reallocations,deallocations,allocated_bytes,realloc_old_bytes,realloc_new_bytes,deallocated_bytes,failed_allocations,failed_reallocations,live_bytes,region_peak_bytes"
    );
    for (operation, (start, held, released, len)) in records.iter().enumerate() {
        for (point, s) in [("start", start), ("held", held), ("released", released)] {
            println!(
                "{variant},{},{operation},{point},{len},{},{},{},{},{},{},{},{},{},{},{},{}",
                phase.name(),
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
    fn yt150_owned_and_borrowed_phase_results_match_literal_document() {
        let (_, expected_document) = expected(ROWS);
        let owned = construct().unwrap();
        assert_eq!(owned.result_len(), NODES);
        let Owned::Construct(input) = owned else {
            panic!("construct must retain owned inputs")
        };
        assert_eq!(input.0.title(), TITLE);
        let rendered = document(&input).unwrap();
        assert_eq!(rendered.result_len(), 245207);
        let Owned::Document(rendered) = rendered else {
            panic!("document must retain its output")
        };
        assert_eq!(rendered, expected_document);
        drop(rendered);
        assert_eq!(input.0.title(), TITLE);
        let Owned::Document(rendered) = combined().unwrap() else {
            panic!("combined must return only its document")
        };
        assert_eq!(rendered, expected_document);
    }
}
