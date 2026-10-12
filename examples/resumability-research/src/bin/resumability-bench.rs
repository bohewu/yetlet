use std::{hint::black_box, time::Instant};
use yetlet_resumability_research::{Capture, Descriptor, Handler};

fn main() {
    const CYCLES: usize = 10_000;
    const WARMUPS: usize = 3;
    const SAMPLES: usize = 20;
    let inputs = [
        (Handler::Add, Capture::new(40, 2).unwrap(), 42),
        (Handler::Subtract, Capture::new(40, 2).unwrap(), 38),
        (
            Handler::Add,
            Capture::new(999_000, 1000).unwrap(),
            1_000_000,
        ),
        (Handler::Subtract, Capture::new(1000, 1000).unwrap(), 0),
    ];
    let descriptors: Vec<_> = inputs
        .iter()
        .map(|(handler, capture, _)| Descriptor::new(*handler, *capture))
        .collect();
    let encoded: Vec<_> = descriptors
        .iter()
        .map(|descriptor| descriptor.encode().unwrap())
        .collect();
    for (index, bytes) in encoded.iter().enumerate() {
        assert_eq!(
            Descriptor::decode(bytes).unwrap().dispatch().unwrap(),
            inputs[index].2
        );
        println!(
            "# wire_{index}_bytes={},json={}",
            bytes.len(),
            std::str::from_utf8(bytes).unwrap()
        );
    }
    let bytes_total: usize = encoded.iter().map(Vec::len).sum();
    let phases = [
        "direct",
        "encode",
        "decode",
        "dispatch",
        "roundtrip_dispatch",
    ];
    println!(
        "# warmups={WARMUPS},samples={SAMPLES},cycles={CYCLES},descriptors={},clock=Instant,build=release",
        inputs.len()
    );
    println!("sample,order,phase,operations,encoded_workload_bytes,elapsed_ns");
    for round in 0..(WARMUPS + SAMPLES) {
        for order in 0..phases.len() {
            let phase = phases[(round + order) % phases.len()];
            let start = Instant::now();
            for _ in 0..CYCLES {
                for index in 0..inputs.len() {
                    match phase {
                        "direct" => {
                            black_box(
                                black_box(inputs[index].0)
                                    .apply(black_box(inputs[index].1))
                                    .unwrap(),
                            );
                        }
                        "encode" => {
                            black_box(black_box(&descriptors[index]).encode().unwrap());
                        }
                        "decode" => {
                            black_box(Descriptor::decode(black_box(&encoded[index])).unwrap());
                        }
                        "dispatch" => {
                            black_box(black_box(&descriptors[index]).dispatch().unwrap());
                        }
                        "roundtrip_dispatch" => {
                            let bytes = black_box(&descriptors[index]).encode().unwrap();
                            let decoded = Descriptor::decode(black_box(&bytes)).unwrap();
                            black_box(decoded.dispatch().unwrap());
                        }
                        _ => unreachable!(),
                    }
                }
            }
            let elapsed = start.elapsed().as_nanos();
            if round >= WARMUPS {
                println!(
                    "{},{order},{phase},{},{bytes_total},{elapsed}",
                    round - WARMUPS,
                    CYCLES * inputs.len()
                );
            }
        }
    }
    println!("# cleanup=foreground_only,no_listeners_threads_watchers_or_browser_created");
}
