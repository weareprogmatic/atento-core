# Atento Core

[![CI](https://github.com/weareprogmatic/atento-core/workflows/CI/badge.svg)](https://github.com/weareprogmatic/atento-core/actions)
[![Crates.io](https://img.shields.io/crates/v/atento-core.svg)](https://crates.io/crates/atento-core)
[![Documentation](https://docs.rs/atento-core/badge.svg)](https://docs.rs/atento-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE.MIT)

**Deterministic workflows. YAML in, JSON out. No surprises.**

Atento Core is the foundational engine for building and executing deterministic workflows. It provides a robust, type-safe workflow execution system with clear input/output handling, making automation predictable and reliable.

## Features

- 🎯 **Deterministic Execution** - Same inputs always produce the same outputs
- 📋 **Declarative Workflows** - Define workflows in simple YAML, always get a JSON output.
- 🔄 **Step Dependencies** - Clear execution order with parameter passing
- 🎭 **Type Safety** - Strong typing for workflow parameters and results
- ⏱️ **Timeout Control** - Configurable timeouts at workflow and per-step level
- 📝 **Comprehensive Logging** - Captures stdout, stderr, and errors inline for full observability
- 🧪 **Testable** - Built-in support for testing and validation
- 🔌 **Extensible** - Easy to integrate custom executors
- 🪶 **Lightweight** - Minimal dependencies, fast compilation, small binary footprint
- � **Pure Rust** - Memory safe and performant

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
atento-core = "0.0.1"
```

## Quick Start

```rust
use atento_core::{Workflow, Runner};

// Load a workflow from YAML
let workflow = Workflow::from_yaml(r#"
name: example
steps:
  - name: greet
    command: echo
    args: ["Hello, World!"]
"#)?;

// Create a runner and execute
let runner = Runner::new();
let results = runner.run(&workflow)?;
```

## Workflow Examples

### Simple Two-Step Workflow

This example shows how to pass data between steps using step results:

```yaml
name: user-greeting
description: Greet a user and log the interaction

inputs:
  - name: username
    type: string
    required: true
  - name: log_level
    type: string
    default: "info"

steps:
  - name: create_greeting
    command: echo
    args:
      - "Hello, {{username}}! Welcome to Atento."
    outputs:
      - name: message
        type: string

  - name: log_greeting
    command: ./log.sh
    parameters:
      level: "{{log_level}}"
      message: "{{steps.create_greeting.outputs.message}}"
      timestamp: "{{global.timestamp}}"
    outputs:
      - name: log_id
        type: string

outputs:
  - name: greeting
    value: "{{steps.create_greeting.outputs.message}}"
  - name: log_entry
    value: "{{steps.log_greeting.outputs.log_id}}"
```

### Multi-Step Data Pipeline

This example demonstrates passing results through multiple steps with global values:

```yaml
name: data-pipeline
description: Process data through multiple transformation steps

inputs:
  - name: input_file
    type: string
    required: true
  - name: output_format
    type: string
    default: "json"

steps:
  - name: validate
    command: ./validate.sh
    parameters:
      file: "{{input_file}}"
      schema: "{{global.schema_path}}"
    outputs:
      - name: valid
        type: boolean
      - name: record_count
        type: integer

  - name: transform
    command: ./transform.sh
    parameters:
      file: "{{input_file}}"
      format: "{{output_format}}"
      records: "{{steps.validate.outputs.record_count}}"
      run_id: "{{global.run_id}}"
    outputs:
      - name: output_file
        type: string
      - name: processed_count
        type: integer

outputs:
  - name: result_file
    value: "{{steps.transform.outputs.output_file}}"
  - name: summary
    value: "Processed {{steps.transform.outputs.processed_count}} of {{steps.validate.outputs.record_count}} records"
```

## Core Concepts

### Workflows
Workflows define a sequence of steps with inputs, outputs, and dependencies.

### Steps
Each step represents a single operation with:
- **Command**: The executable to run
- **Parameters**: Input data for the step
- **Dependencies**: Results from previous steps

### Executors
Executors handle step execution. Custom executors can be implemented for different operation types.

### Results
Each step produces typed results that can be referenced by subsequent steps.

## Development

### Prerequisites

- Rust 1.85.0 or later (for Edition 2024 support)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Testing

```bash
# Run all tests
make test

# Run specific test suite
cargo test --test integration

# Run with output
cargo test -- --nocapture

# QA smoke tests
make qa
```

### Code Quality

```bash
# Format code
make format

# Run linter
make clippy

# Security audit
cargo audit

# Check licenses
cargo deny check

# Full pre-commit checks
make pre-commit
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for detailed information about:

- Setting up your development environment
- Running tests and quality checks
- Code style guidelines
- Submitting pull requests

## Examples

Check out the [`examples/`](examples/) directory for more use cases:

```bash
# Run the simple workflow example
cargo run --example simple_workflow
```

## Benchmarks

Run performance benchmarks:

```bash
cargo run --release --bin workflow_parsing
```

## Documentation

- [API Documentation](https://docs.rs/atento-core)
- [Contributing Guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)
- [Governance](GOVERNANCE.md)

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE.APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE.MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Support

- 📧 Email: atento@weareprogmatic.com
- 🐛 [Issue Tracker](https://github.com/weareprogmatic/atento-core/issues)
- 💬 [Discussions](https://github.com/weareprogmatic/atento-core/discussions)

---

Made with ❤️ by [We Are Progmatic](https://weareprogmatic.com)

