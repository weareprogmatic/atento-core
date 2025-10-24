//! README examples runner
//!
//! This example runs the chains featured in README.md to ensure they work correctly.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Running README.md example chains...\n");

    // Test user greeting chain
    println!("=== Running user_greeting.yaml ===");
    atento_core::run("tests/chains/cross-platform/user_greeting.yaml")?;

    println!("\n=== Running data_pipeline.yaml ===");
    atento_core::run("tests/chains/cross-platform/data_pipeline.yaml")?;

    println!("\n✅ All README examples completed successfully!");

    Ok(())
}
