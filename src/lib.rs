#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

//! # Atento Core - Chain Execution Engine
//!
//! Atento Core is a powerful Rust library for defining and executing sequential chained scripts
//! with multi-interpreter support, robust error handling, and advanced variable passing capabilities.
//!
//! ## Key Features
//!
//! - **Multi-Interpreter Support**: Execute scripts in Bash, Batch, `PowerShell`, Pwsh, and Python
//! - **Sequential Execution**: Guaranteed step order with dependency management
//! - **Variable Passing**: Global parameters and step-to-step output chaining
//! - **Type Safety**: Strongly typed parameters (string, int, float, bool, datetime)
//! - **Cross-Platform**: Works reliably on Linux, macOS, and Windows
//! - **Secure Execution**: Temporary file isolation and proper permission handling
//! - **Embedded Logging**: Captures stdout, stderr, and errors inline in JSON results
//! - **No Telemetry**: Never collects usage stats or requires licensing checks
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use atento_core;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Run a chain from a YAML file
//!     atento_core::run("chain.yaml")?;
//!     Ok(())
//! }
//! ```
//!
//! ## Chain Structure
//!
//! Chains are defined in YAML format with the following structure:
//!
//! ```yaml
//! name: "Example Chain"
//! timeout: 300  # Global timeout in seconds
//!
//! parameters:
//!   project_name:
//!     type: string
//!     value: "my-project"
//!   build_number:
//!     type: int
//!     value: 42
//!
//! steps:
//!   setup:
//!     name: "Setup Environment"
//!     type: bash  # Interpreter: bash, batch, powershell, pwsh, python
//!     timeout: 60
//!     script: |
//!       echo "Setting up {{ inputs.project }}"
//!       echo "BUILD_DIR=/tmp/build-{{ inputs.build_num }}"
//!     inputs:
//!       project:
//!         ref: parameters.project_name
//!       build_num:
//!         ref: parameters.build_number
//!     outputs:
//!       build_directory:
//!         pattern: "BUILD_DIR=(.*)"
//!
//!   build:
//!     name: "Build Project"
//!     type: python
//!     script: |
//!       import os
//!       build_dir = "{{ inputs.build_dir }}"
//!       print(f"Building in {build_dir}")
//!       print("BUILD_SUCCESS=true")
//!     inputs:
//!       build_dir:
//!         ref: steps.setup.outputs.build_directory
//!     outputs:
//!       status:
//!         pattern: "BUILD_SUCCESS=(.*)"
//!
//! results:
//!   build_status:
//!     ref: steps.build.outputs.status
//!   workspace:
//!     ref: steps.setup.outputs.build_directory
//! ```
//!
//! ## Supported Interpreters
//!
//! | Type | Description | Platform |
//! |------|-------------|----------|
//! | `bash` | Bash shell scripts | Unix/Linux/macOS |
//! | `batch` | Windows batch files | Windows |
//! | `powershell` | `PowerShell` (Windows) | Windows |
//! | `pwsh` | `PowerShell` Core | Cross-platform |
//! | `python` | Python scripts | Cross-platform |
//! | `python3` | Python3 scripts | Cross-platform |
//!
//! ## Variable Substitution
//!
//! Use `{{ inputs.variable_name }}` syntax in scripts to substitute input values:
//!
//! ```yaml
//! script: |
//!   echo "Processing {{ inputs.filename }} in {{ inputs.directory }}"
//!   cp "{{ inputs.source }}" "{{ inputs.destination }}"
//! ```
//!
//! ## Output Extraction
//!
//! Capture values from command output using regex patterns with capture groups:
//!
//! ```yaml
//! outputs:
//!   version:
//!     pattern: "Version: ([0-9]+\.[0-9]+\.[0-9]+)"
//!   status:
//!     pattern: "Status: (SUCCESS|FAILED)"
//! ```
//!
//! ## Error Handling
//!
//! The library provides comprehensive error handling for:
//! - File I/O operations
//! - YAML parsing errors
//! - Chain validation failures
//! - Script execution timeouts
//! - Type conversion errors
//! - Unresolved variable references
//!
//! ## Example Usage
//!
//! ```no_run
//! # use atento_core::{Chain, AtentoError};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load and validate a chain
//! let yaml_content = std::fs::read_to_string("chain.yaml")?;
//! let chain: Chain = serde_yaml::from_str(&yaml_content)?;
//!
//! // Validate the chain structure
//! chain.validate()?;
//!
//! // Execute the chain
//! let result = chain.run();
//!
//! // Serialize results to JSON
//! let json_output = serde_json::to_string_pretty(&result)?;
//! println!("{}", json_output);
//! # Ok(())
//! # }
//! ```

use std::path::Path;

mod chain;
mod data_type;
mod errors;
mod executor;
mod input;
mod interpreter;
mod output;
mod parameter;
mod result_ref;
mod runner;
mod step;

#[cfg(test)]
mod tests;

// Re-export main types for library users
pub use chain::{Chain, ChainResult};
pub use data_type::DataType;
pub use errors::{AtentoError, Result};
pub use interpreter::{Interpreter, default_interpreters};
pub use step::{Step, StepResult};

/// Runs a chain from a YAML file.
///
/// # Arguments
/// * `filename` - Path to the chain YAML file
///
/// # Errors
/// Returns an error if:
/// - The file cannot be read
/// - The YAML cannot be parsed
/// - The chain validation fails
/// - The chain execution fails
/// - The results cannot be serialized to JSON
pub fn run(filename: &str) -> Result<()> {
    let path = Path::new(filename);

    let contents = std::fs::read_to_string(path).map_err(|e| AtentoError::Io {
        path: filename.to_string(),
        source: e,
    })?;

    let chain: Chain = serde_yaml::from_str(&contents).map_err(|e| AtentoError::YamlParse {
        context: filename.to_string(),
        source: e,
    })?;

    chain.validate()?; // Already returns Result<(), AtentoError>

    let result = chain.run(); // Returns ChainResult

    let json = serde_json::to_string_pretty(&result)?; // From trait converts to AtentoError

    println!("{json}");

    if result.errors.is_empty() {
        Ok(())
    } else {
        Err(AtentoError::Execution(
            "Chain completed with errors".to_string(),
        ))
    }
}
