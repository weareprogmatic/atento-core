//! Basic chain parsing benchmark

use atento_core::Chain;
use std::hint::black_box;

fn main() {
    let yaml = r#"
name: benchmark-chain
description: Simple chain for benchmarking

steps:
  step1:
    name: "First Step"
    type: bash
    script: echo "Hello"

  step2:
    name: "Second Step"
    type: bash
    script: echo "World"
"#;

    // Simple benchmark - run 1000 times
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let chain: Chain = serde_yaml::from_str(black_box(yaml)).expect("Failed to parse chain");
        black_box(chain);
    }
    let duration = start.elapsed();

    println!("Parsed 1000 chains in {:?}", duration);
    println!("Average: {:?} per chain", duration / 1000);
}
