//! README examples runner
//!
//! This example runs the workflows featured in README.md to ensure they work correctly.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Running README.md example workflows...\n");

    // Test user greeting workflow
    println!("=== Running user_greeting.yaml ===");
    atento_core::run("tests/workflows/cross-platform/user_greeting.yaml")?;

    println!("\n=== Running data_pipeline.yaml ===");
    atento_core::run("tests/workflows/cross-platform/data_pipeline.yaml")?;

    println!("\n✅ All README examples completed successfully!");

    Ok(())
}
