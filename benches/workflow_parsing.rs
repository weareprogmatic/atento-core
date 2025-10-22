//! Basic workflow parsing benchmark

use atento_core::Workflow;
use std::hint::black_box;

fn main() {
    let yaml = r#"
name: benchmark-workflow
description: Simple workflow for benchmarking

steps:
  step1:
    name: "First Step"
    type: script::bash
    script: echo "Hello"

  step2:
    name: "Second Step"
    type: script::bash
    script: echo "World"
"#;

    // Simple benchmark - run 1000 times
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let workflow: Workflow =
            serde_yaml::from_str(black_box(yaml)).expect("Failed to parse workflow");
        black_box(workflow);
    }
    let duration = start.elapsed();

    println!("Parsed 1000 workflows in {:?}", duration);
    println!("Average: {:?} per workflow", duration / 1000);
}
