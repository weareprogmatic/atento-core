//! Simple workflow example
//!
//! This example demonstrates running a basic workflow from a file.

use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a temporary workflow file
    let workflow_yaml = r#"
name: greeting-workflow
description: A simple greeting workflow

parameters:
  user_name:
    type: string
    value: "World"

steps:
  greet:
    name: "Greet User"
    type: script::bash
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
    type: script::bash
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
    let temp_file = "example_workflow.yaml";
    fs::write(temp_file, workflow_yaml)?;

    println!("Running workflow from {}", temp_file);
    
    // Run the workflow
    atento_core::run(temp_file)?;

    // Clean up
    fs::remove_file(temp_file)?;

    println!("\n✅ Workflow completed successfully!");

    Ok(())
}
