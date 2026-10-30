//! YT-155: unchanged actual APIs and a source-derived same-process tail proxy.
use super::*;
use yetlet::html::{RenderError, escape_text};

type Input = (DocumentMetadata, View);
type Prepared = (String, String);
const SIZES: [(&str, usize); 2] = [("small", 8), ("medium", 128)];
const CLOSE: &str = "</body></html>";
#[derive(Clone, Copy)]
enum Strategy {
    Legacy,
    Reserved,
}
impl Strategy {
    fn name(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Reserved => "reserved",
        }
    }
}
fn prepare(input: &Input) -> Result<Prepared, RenderError> {
    let mut html = String::from("<!doctype html><html lang=\"");
    html.push_str(input.0.language().as_str());
    html.push_str("\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>");
    html.push_str(&escape_text(input.0.title()));
    html.push_str("</title></head><body>");
    let body = yetlet::html::render_to_string(&input.1)?;
    Ok((html, body))
}
#[inline(never)]
fn legacy_tail(mut html: String, body: String) -> String {
    html.push_str(&body);
    drop(body);
    html.push_str(CLOSE);
    html
}
#[inline(never)]
fn reserved_tail(mut html: String, body: String) -> String {
    html.reserve_exact(body.len() + CLOSE.len());
    html.push_str(&body);
    drop(body);
    html.push_str(CLOSE);
    html
}

#[cfg(feature = "allocation-counts")]
pub fn api(_: &str, _: usize) -> Result<(), Box<dyn Error>> {
    Err("latency study requires an ordinary build".into())
}
#[cfg(feature = "allocation-counts")]
pub fn tail(_: usize) -> Result<(), Box<dyn Error>> {
    Err("latency study requires an ordinary build".into())
}
#[cfg(not(feature = "allocation-counts"))]
pub use normal::{api, tail};

#[cfg(not(feature = "allocation-counts"))]
mod normal {
    use super::*;
    use std::time::Instant;
    const WARM_NS: u128 = 50_000_000;
    const MAX_WARM_CALLS: usize = 100_000;
    const BLOCKS: usize = 10;
    const PER_BLOCK: usize = 32;
    #[derive(Clone, Copy)]
    struct Observation {
        ns: u128,
        len: usize,
        capacity: usize,
    }
    struct Warmup {
        calls: usize,
        ns: u128,
        completed: bool,
    }
    #[derive(Clone, Copy)]
    struct Context {
        tick_ns: u128,
        khz: Option<u64>,
    }
    fn context(epoch: &Instant) -> Context {
        let khz = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
            .ok()
            .and_then(|value| value.trim().parse().ok());
        Context {
            tick_ns: epoch.elapsed().as_nanos(),
            khz,
        }
    }
    #[inline(never)]
    fn observe(
        work: impl FnOnce() -> Result<String, RenderError>,
    ) -> Result<Observation, RenderError> {
        let started = Instant::now();
        let output = black_box(work()?);
        let ns = started.elapsed().as_nanos();
        let result = Observation {
            ns,
            len: output.len(),
            capacity: output.capacity(),
        };
        drop(output);
        Ok(result)
    }
    fn warmup(
        stride: usize,
        mut work: impl FnMut() -> Result<(), RenderError>,
    ) -> Result<Warmup, RenderError> {
        let started = Instant::now();
        let mut calls = 0;
        while calls < MAX_WARM_CALLS && started.elapsed().as_nanos() < WARM_NS {
            work()?;
            calls += stride;
        }
        let ns = started.elapsed().as_nanos();
        Ok(Warmup {
            calls,
            ns,
            completed: ns >= WARM_NS,
        })
    }
    fn sample(
        mode: &str,
        variant: &str,
        run: usize,
        group: &str,
        group_order: usize,
        phase: &str,
        block: usize,
        index: usize,
        o: Observation,
    ) {
        println!(
            "{{\"kind\":\"sample\",\"mode\":\"{mode}\",\"variant\":\"{variant}\",\"run\":{run},\"group\":\"{group}\",\"group_order\":{group_order},\"phase\":\"{phase}\",\"block\":{block},\"index\":{index},\"elapsed_ns\":{},\"result_len\":{},\"result_capacity\":{}}}",
            o.ns, o.len, o.capacity
        );
    }
    fn print_context(mode: &str, variant: &str, run: usize, group: &str, point: &str, c: Context) {
        let khz = c.khz.map_or_else(|| "null".to_owned(), |v| v.to_string());
        println!(
            "{{\"kind\":\"context\",\"mode\":\"{mode}\",\"variant\":\"{variant}\",\"run\":{run},\"group\":\"{group}\",\"point\":\"{point}\",\"tick_ns\":{},\"reported_cpu_khz\":{khz}}}",
            c.tick_ns
        );
    }
    fn print_warm(mode: &str, variant: &str, run: usize, group: &str, w: &Warmup) {
        println!(
            "{{\"kind\":\"warmup\",\"mode\":\"{mode}\",\"variant\":\"{variant}\",\"run\":{run},\"group\":\"{group}\",\"calls\":{},\"elapsed_ns\":{},\"minimum_ns\":{WARM_NS},\"max_calls\":{MAX_WARM_CALLS},\"completed\":{}}}",
            w.calls, w.ns, w.completed
        );
    }
    fn sizes(run: usize) -> Result<[(&'static str, usize); 2], Box<dyn Error>> {
        if !(1..=2).contains(&run) {
            return Err("expected run 1|2".into());
        }
        let mut sizes = SIZES;
        if run == 2 {
            sizes.reverse();
        }
        Ok(sizes)
    }
    fn actual(input: &Input) -> Result<String, RenderError> {
        render_document_with_metadata(black_box(&input.0), black_box(&input.1))
    }
    pub fn api(variant: &str, run: usize) -> Result<(), Box<dyn Error>> {
        if !matches!(variant, "reference" | "current") {
            return Err("expected source reference|current".into());
        }
        let sizes = sizes(run)?;
        let epoch = Instant::now();
        for (group_order, (group, rows)) in sizes.into_iter().enumerate() {
            let input = page(rows);
            let mut initial = Vec::with_capacity(8);
            let mut steady = Vec::with_capacity(BLOCKS * PER_BLOCK);
            let before = context(&epoch);
            for _ in 0..8 {
                initial.push(observe(|| actual(&input))?);
            }
            for (index, o) in initial.into_iter().enumerate() {
                sample(
                    "api",
                    variant,
                    run,
                    group,
                    group_order,
                    "initial",
                    0,
                    index,
                    o,
                );
            }
            let w = warmup(1, || {
                drop(black_box(actual(&input)?));
                Ok(())
            })?;
            let after_warm = context(&epoch);
            if !w.completed {
                print_warm("api", variant, run, group, &w);
                return Err("declared warmup bound not achieved".into());
            }
            for block in 0..BLOCKS {
                for index in 0..PER_BLOCK {
                    steady.push((block, index, observe(|| actual(&input))?));
                }
            }
            let after = context(&epoch);
            print_context("api", variant, run, group, "before_initial", before);
            print_context("api", variant, run, group, "after_warmup", after_warm);
            print_context("api", variant, run, group, "after_steady", after);
            print_warm("api", variant, run, group, &w);
            for (block, index, o) in steady {
                sample(
                    "api",
                    variant,
                    run,
                    group,
                    group_order,
                    "steady",
                    block,
                    index,
                    o,
                );
            }
        }
        Ok(())
    }
    fn tail_observation(strategy: Strategy, input: &Input) -> Result<Observation, RenderError> {
        let (html, body) = prepare(input)?;
        match strategy {
            Strategy::Legacy => observe(|| Ok(legacy_tail(black_box(html), black_box(body)))),
            Strategy::Reserved => observe(|| Ok(reserved_tail(black_box(html), black_box(body)))),
        }
    }
    pub fn tail(run: usize) -> Result<(), Box<dyn Error>> {
        let sizes = sizes(run)?;
        let pattern = if run == 1 {
            [
                Strategy::Legacy,
                Strategy::Reserved,
                Strategy::Reserved,
                Strategy::Legacy,
            ]
        } else {
            [
                Strategy::Reserved,
                Strategy::Legacy,
                Strategy::Legacy,
                Strategy::Reserved,
            ]
        };
        let epoch = Instant::now();
        for (group_order, (group, rows)) in sizes.into_iter().enumerate() {
            let input = page(rows);
            let mut observations = Vec::with_capacity(BLOCKS * PER_BLOCK * 2);
            let before = context(&epoch);
            let w = warmup(4, || {
                for strategy in pattern {
                    let (html, body) = prepare(&input)?;
                    let output = match strategy {
                        Strategy::Legacy => legacy_tail(black_box(html), black_box(body)),
                        Strategy::Reserved => reserved_tail(black_box(html), black_box(body)),
                    };
                    drop(black_box(output));
                }
                Ok(())
            })?;
            let after_warm = context(&epoch);
            if !w.completed {
                print_warm("tail", "paired", run, group, &w);
                return Err("declared warmup bound not achieved".into());
            }
            for block in 0..BLOCKS {
                for index in 0..PER_BLOCK * 2 {
                    let strategy = pattern[index % 4];
                    observations.push((
                        strategy,
                        block,
                        index,
                        tail_observation(strategy, &input)?,
                    ));
                }
            }
            let after = context(&epoch);
            print_context("tail", "paired", run, group, "before_warmup", before);
            print_context("tail", "paired", run, group, "after_warmup", after_warm);
            print_context("tail", "paired", run, group, "after_steady", after);
            print_warm("tail", "paired", run, group, &w);
            for (strategy, block, index, o) in observations {
                sample(
                    "tail",
                    strategy.name(),
                    run,
                    group,
                    group_order,
                    "steady",
                    block,
                    index,
                    o,
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yt155_tail_copies_and_preparation_match_actual_literals() {
        for (_, rows) in SIZES {
            let input = page(rows);
            let expected = expected(rows).1;
            let (html, body) = prepare(&input).unwrap();
            assert_eq!(legacy_tail(html, body), expected);
            let (html, body) = prepare(&input).unwrap();
            assert_eq!(reserved_tail(html, body), expected);
            assert_eq!(
                render_document_with_metadata(&input.0, &input.1).unwrap(),
                expected
            );
        }
        for (view, error) in [
            (
                Element::new("main")
                    .child(Element::new("a").attribute("href", "javascript:alert(1)"))
                    .into(),
                RenderError::UnsafeUrl("href".into()),
            ),
            (
                Element::new("input")
                    .child(View::text("invalid child"))
                    .into(),
                RenderError::VoidElementHasChildren("input".into()),
            ),
        ] {
            assert_eq!(
                prepare(&(DocumentMetadata::new("Untrusted <&>"), view)),
                Err(error)
            );
        }
    }
}
