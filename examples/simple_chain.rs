//! Simple chain example
//!
//! This example demonstrates running a basic chain from a file.

use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a temporary chain file
    let chain_yaml = r#"
name: greeting-chain
description: A simple greeting chain

parameters:
  user_name:
    type: string
    value: "World"

steps:
  greet:
    name: "Greet User"
    type: bash
    script: |
      echo "Hello, {{ inputs.user }}!"
      echo "GREETING=Hello, {{ inputs.user }}!"
    inputs:
      user:
        ref: parameters.user_name
    outputs:
      message:
        pattern: "GREETING=(.*)"

  farewell:
    name: "Say Goodbye"
    type: bash
    script: |
      echo "Goodbye, {{ inputs.user }}!"
      echo "Previous greeting was: {{ inputs.greeting }}"
    inputs:
      user:
        ref: parameters.user_name
      greeting:
        ref: steps.greet.outputs.message

results:
  final_greeting:
    ref: steps.greet.outputs.message
"#;

    // Write to temporary file
    let temp_file = "example_chain.yaml";
    fs::write(temp_file, chain_yaml)?;

    println!("Running chain from {}", temp_file);

    // Run the chain
    atento_core::run(temp_file)?;

    // Clean up
    fs::remove_file(temp_file)?;

    println!("\n✅ Chain completed successfully!");

    Ok(())
}
