//! YT-158 sample-only synchronous history intervention, never a product feature.
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
mod native {
    use crate::{Page, expected, page};
    use axum::{
        body::{HttpBody, to_bytes},
        http::header,
        response::IntoResponse,
    };
    use serde::Serialize;
    use std::{
        error::Error,
        ffi::{c_int, c_long},
        fs::File,
        hint::black_box,
        io::{self, Read},
        time::Instant,
    };

    const BLOCKS: usize = 8;
    const N: usize = 256;
    const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";

    // These private layouts match Linux x86_64 GNU time.h / malloc.h. The
    // diagnostic is cfg-excluded elsewhere; no allocator configuration is changed.
    #[repr(C)]
    #[derive(Default)]
    struct Timespec {
        sec: c_long,
        nsec: c_long,
    }
    #[repr(C)]
    #[derive(Clone, Copy, Serialize)]
    struct Heap {
        arena: usize,
        ordblks: usize,
        smblks: usize,
        hblks: usize,
        hblkhd: usize,
        usmblks: usize,
        fsmblks: usize,
        uordblks: usize,
        fordblks: usize,
        keepcost: usize,
    }
    unsafe extern "C" {
        fn clock_gettime(clock: c_int, time: *mut Timespec) -> c_int;
        fn clock_getres(clock: c_int, time: *mut Timespec) -> c_int;
        fn sched_getcpu() -> c_int;
        fn mallinfo2() -> Heap;
    }
    fn clock(resolution: bool) -> io::Result<u64> {
        let mut time = Timespec::default();
        // SAFETY: Writable correctly aligned x86_64 GNU timespec; clock ID 3
        // is Linux CLOCK_THREAD_CPUTIME_ID. The C call retains no pointer.
        let result = unsafe {
            if resolution {
                clock_getres(3, &mut time)
            } else {
                clock_gettime(3, &mut time)
            }
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        if time.sec < 0 || !(0..1_000_000_000).contains(&time.nsec) {
            return Err(io::Error::other("invalid thread clock"));
        }
        Ok(time.sec as u64 * 1_000_000_000 + time.nsec as u64)
    }
    #[derive(Clone, Copy, Serialize)]
    struct Time {
        wall_ns: u128,
        cpu_ns: u64,
        gap_ns: i128,
    }
    fn observe<T>(work: impl FnOnce() -> T) -> io::Result<(Time, T)> {
        let wall = Instant::now();
        let cpu = clock(false)?;
        let value = black_box(work());
        let cpu_ns = clock(false)?
            .checked_sub(cpu)
            .ok_or_else(|| io::Error::other("thread clock reversed"))?;
        let wall_ns = wall.elapsed().as_nanos();
        Ok((
            Time {
                wall_ns,
                cpu_ns,
                gap_ns: wall_ns as i128 - cpu_ns as i128,
            },
            value,
        ))
    }
    fn construct() -> Page {
        let (metadata, view) = page(black_box(128));
        black_box(Page::with_metadata(metadata, view))
    }
    fn predecessor(rendered: bool) {
        let page = construct();
        if rendered {
            drop(black_box(page.into_response()));
        } else {
            drop(page);
        }
    }
    fn proc_text<'a>(path: &str, buf: &'a mut [u8]) -> Result<&'a str, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut n = 0;
        loop {
            if n == buf.len() {
                return Err("proc context exceeded fixed buffer".into());
            }
            let read = file.read(&mut buf[n..])?;
            if read == 0 {
                break;
            }
            n += read;
        }
        Ok(std::str::from_utf8(&buf[..n])?)
    }
    #[derive(Clone, Copy, Serialize)]
    struct Context {
        tick_ns: u128,
        cpu: c_int,
        allowed_cpu: u32,
        // VmRSS, VmHWM (KiB), Threads, voluntary and involuntary switches.
        thread_status: [u64; 5],
        // Kernel runtime ns, runqueue wait ns, scheduling timeslices.
        schedstat: [u64; 3],
        heap: Heap,
    }
    fn context(epoch: Instant) -> Result<Context, Box<dyn Error>> {
        // SAFETY: Both read-only C calls take no pointers; the process has one
        // application thread and never invokes mallopt or allocator replacement.
        let (cpu, heap) = unsafe { (sched_getcpu(), mallinfo2()) };
        if cpu < 0 {
            return Err(io::Error::last_os_error().into());
        }
        let mut buffer = [0; 8192];
        let status = proc_text("/proc/thread-self/status", &mut buffer)?;
        let keys = [
            "VmRSS:",
            "VmHWM:",
            "Threads:",
            "voluntary_ctxt_switches:",
            "nonvoluntary_ctxt_switches:",
        ];
        let mut values = [0; 5];
        for (i, key) in keys.into_iter().enumerate() {
            values[i] = status
                .lines()
                .find_map(|l| l.strip_prefix(key))
                .ok_or("missing thread status")?
                .split_whitespace()
                .next()
                .ok_or("empty thread status")?
                .parse()?;
        }
        let allowed_cpu: u32 = status
            .lines()
            .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
            .ok_or("missing allowed CPU")?
            .trim()
            .parse()?;
        if cpu != 12 || allowed_cpu != 12 || values[2] != 1 {
            return Err("expected one application thread pinned to CPU12".into());
        }
        let stat = proc_text("/proc/thread-self/schedstat", &mut buffer)?;
        let mut fields = stat.split_whitespace();
        let mut schedstat = [0; 3];
        for value in &mut schedstat {
            *value = fields.next().ok_or("missing schedstat field")?.parse()?;
        }
        if fields.next().is_some() {
            return Err("unexpected schedstat layout".into());
        }
        Ok(Context {
            tick_ns: epoch.elapsed().as_nanos(),
            cpu,
            allowed_cpu,
            thread_status: values,
            schedstat,
            heap,
        })
    }
    #[derive(Serialize)]
    struct Sample {
        block: usize,
        order: usize,
        rendered: bool,
        index: usize,
        time: Time,
    }
    #[derive(Serialize)]
    struct Group {
        block: usize,
        order: usize,
        rendered: bool,
        warmup_cycles: usize,
        warmup_ns: u128,
        before: Context,
        after: Context,
    }
    async fn verify() -> Result<(), Box<dyn Error>> {
        let response = construct().into_response();
        let expected = expected(128).1;
        assert_eq!(expected.len(), 31063);
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let h = response.headers();
        assert_eq!(h[header::CONTENT_TYPE], "text/html; charset=utf-8");
        assert!(!h.contains_key(header::CONTENT_LENGTH));
        assert_eq!(response.body().size_hint().exact(), Some(31063));
        assert_eq!(h[header::CONTENT_SECURITY_POLICY], CSP);
        for name in [
            "set-cookie",
            "content-encoding",
            "server",
            "server-timing",
            "x-yt157-id",
        ] {
            assert!(!h.contains_key(name));
        }
        assert_eq!(
            to_bytes(response.into_body(), 32768).await?.as_ref(),
            expected.as_bytes()
        );
        Ok(())
    }
    pub(super) fn run() -> Result<(), Box<dyn Error>> {
        let mut args = std::env::args().skip(1);
        let variant = args.next().ok_or("expected reference|current and run1|2")?;
        let run: usize = args.next().ok_or("expected run1|2")?.parse()?;
        if !matches!(variant.as_str(), "reference" | "current")
            || !(1..=2).contains(&run)
            || args.next().is_some()
        {
            return Err("expected reference|current and run1|2".into());
        }
        let resolution = clock(true)?;
        // Correctness happens before measurement; this runtime is dropped before
        // warmups. No HTTP server, async task or runtime exists in measured work.
        tokio::runtime::Builder::new_current_thread()
            .build()?
            .block_on(verify())?;
        let epoch = Instant::now();
        let mut samples = Vec::with_capacity(BLOCKS * 2 * N);
        let mut groups = Vec::with_capacity(BLOCKS * 2);
        let mut empty = Vec::with_capacity(128);
        for _ in 0..128 {
            empty.push(observe(|| black_box(()))?.0);
        }
        for block in 0..BLOCKS {
            let mut contexts = [false, true];
            if (block + run - 1) % 2 == 1 {
                contexts.reverse();
            }
            for (order, rendered) in contexts.into_iter().enumerate() {
                let started = Instant::now();
                for _ in 0..N {
                    predecessor(rendered);
                    drop(construct());
                }
                let warmup_ns = started.elapsed().as_nanos();
                let before = context(epoch)?;
                for index in 0..N {
                    predecessor(rendered);
                    let (time, owned) = observe(construct)?;
                    drop(owned);
                    samples.push(Sample {
                        block,
                        order,
                        rendered,
                        index,
                        time,
                    });
                }
                groups.push(Group {
                    block,
                    order,
                    rendered,
                    warmup_cycles: N,
                    warmup_ns,
                    before,
                    after: context(epoch)?,
                });
            }
        }
        println!(
            "{}",
            serde_json::json!({"kind":"identity", "variant":variant,
            "run":run, "pid":std::process::id(), "clock_resolution_ns":resolution,
            "measured":samples.len(), "warmup_cycles":BLOCKS*2*N,
            "empty_brackets":empty.len(), "groups":groups.len(), "verified_bytes":31063})
        );
        for (index, time) in empty.iter().enumerate() {
            println!(
                "{}",
                serde_json::json!({"kind":"empty", "index":index, "time":time})
            );
        }
        for group in groups {
            println!("{}", serde_json::json!({"kind":"group", "data":group}));
        }
        for sample in samples {
            println!("{}", serde_json::json!({"kind":"sample", "data":sample}));
        }
        println!(
            "{}",
            serde_json::json!({"kind":"complete", "elapsed_ns":epoch.elapsed().as_nanos()})
        );
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[tokio::test]
        async fn yt158_actual_medium_response_and_both_predecessors() {
            verify().await.unwrap();
            assert!(clock(true).unwrap() > 0);
            for rendered in [false, true] {
                predecessor(rendered);
                let (time, owned) = observe(construct).unwrap();
                assert!(time.cpu_ns > 0 && time.wall_ns > 0);
                assert_eq!(time.gap_ns, time.wall_ns as i128 - time.cpu_ns as i128);
                drop(owned);
            }
        }
    }
}

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
    {
        native::run()
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu")))]
    {
        Err("ssr-context-study requires Linux x86_64 GNU".into())
    }
}
