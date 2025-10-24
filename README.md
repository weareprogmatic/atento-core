# Atento Core

[![CI](https://github.com/weareprogmatic/atento-core/workflows/CI/badge.svg)](https://github.com/weareprogmatic/atento-core/actions)
[![Crates.io](https://img.shields.io/crates/v/atento-core.svg)](https://crates.io/crates/atento-core)
[![Documentation](https://docs.rs/atento-core/badge.svg)](https://docs.rs/atento-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE.MIT)

**A script chaining CLI. YAML in. JSON out. No surprises.**

Atento Core is the foundational engine for building and executing deterministic script chains. It provides a robust, type-safe chain execution system with clear input/output handling, making automation predictable and reliable.

## Features

- 🎯 **Deterministic Execution** - Same inputs always produce the same outputs
- 📋 **Declarative Chains** - Define script chains in simple YAML, always get JSON output
- 🔄 **Step Dependencies** - Clear execution order with parameter passing
- 🎭 **Type Safety** - Strong typing for chain parameters and results
- ⏱️ **Timeout Control** - Configurable timeouts at chain and per-step level
- 📝 **Embedded Logging** - Captures stdout, stderr, and errors inline in JSON results
- 🧪 **Testable** - Built-in support for testing and validation
- 🔌 **Extensible** - Easy to integrate custom executors and interpreters
- 🪶 **Lightweight** - Minimal dependencies, fast compilation, small binary footprint
- 🦀 **Pure Rust** - Memory safe and performant
- 🚫 **No Telemetry** - Never collects usage stats or requires licensing checks, fully offline

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
atento-core = "x.x.x"
```

## Quick Start

```rust
use atento_core;

// Run a chain from a YAML file
atento_core::run("chain.yaml")?;

// Or load and run programmatically
let yaml_content = std::fs::read_to_string("chain.yaml")?;
let chain: atento_core::Chain = serde_yaml::from_str(&yaml_content)?;
chain.validate()?;
let result = chain.run();

// Serialize results to JSON
let json_output = serde_json::to_string_pretty(&result)?;
println!("{}", json_output);
```

## Chain Examples

### Simple Two-Step Chain

This example shows how to pass data between steps using parameters and outputs:

> **See the full working example**: [`tests/chains/cross-platform/user_greeting.yaml`](tests/chains/cross-platform/user_greeting.yaml)

```yaml
name: "user-greeting"
description: "Greet a user and capture the message"

parameters:
  username:
    value: "World"
  greeting_count:
    type: int
    value: 42
  is_formal:
    type: bool
    value: true

steps:
  create_greeting:
    name: "Create Greeting"
    type: python
    script: |
      import sys
      formal = "{{ inputs.formal }}" == "true"
      count = int("{{ inputs.count }}")
      user = "{{ inputs.user }}"
      
      if formal:
          print(f"Good day, {user}! This is greeting number {count}.")
          print(f"GREETING=Good day, {user}!")
      else:
          print(f"Hey {user}! Greeting #{count}")
          print(f"GREETING=Hey {user}!")
    inputs:
      user:
        ref: parameters.username
      count:
        ref: parameters.greeting_count
      formal:
        ref: parameters.is_formal
    outputs:
      message:
        pattern: "GREETING=(.*)"

  confirm_greeting:
    name: "Confirm Greeting"
    type: python
    script: |
      msg = "{{ inputs.msg }}"
      print(f"Message created: {msg}")
      print("CONFIRMED=true")
    inputs:
      msg:
        ref: steps.create_greeting.outputs.message
    outputs:
      status:
        pattern: "CONFIRMED=(.*)"

results:
  greeting:
    ref: steps.create_greeting.outputs.message
  confirmed:
    ref: steps.confirm_greeting.outputs.status
```

### Multi-Step Data Pipeline Chain

This example demonstrates passing results through multiple steps with different data types:

> **See the full working example**: [`tests/chains/cross-platform/data_pipeline.yaml`](tests/chains/cross-platform/data_pipeline.yaml)

```yaml
name: "data-pipeline"
description: "Process data through multiple transformation steps"

parameters:
  input_file:
    value: "data.csv"
  output_format:
    value: "json"
  quality_threshold:
    type: float
    value: 0.95

steps:
  validate:
    name: "Validate Input"
    type: python
    script: |
      import os
      filename = "{{ inputs.file }}"
      threshold = float("{{ inputs.threshold }}")
      print(f"Validating {filename} with quality threshold {threshold}")
      
      # Simulate validation
      record_count = 100
      quality_score = 0.98
      is_valid = quality_score >= threshold
      
      print(f"VALID={str(is_valid).lower()}")
      print(f"RECORD_COUNT={record_count}")
      print(f"QUALITY_SCORE={quality_score}")
    inputs:
      file:
        ref: parameters.input_file
      threshold:
        ref: parameters.quality_threshold
    outputs:
      valid:
        pattern: "VALID=(.*)"
      record_count:
        pattern: "RECORD_COUNT=(\\d+)"
      quality:
        pattern: "QUALITY_SCORE=([0-9.]+)"

  transform:
    name: "Transform Data"
    type: python
    script: |
      import json
      
      input_file = "{{ inputs.file }}"
      output_format = "{{ inputs.format }}"
      record_count = int("{{ inputs.records }}")
      is_valid = "{{ inputs.is_valid }}" == "true"
      
      if not is_valid:
          print("ERROR: Cannot transform invalid data")
          exit(1)
      
      print(f"Transforming {record_count} records to {output_format}")
      
      output_file = f"output.{output_format}"
      processed = record_count
      
      print(f"OUTPUT_FILE={output_file}")
      print(f"PROCESSED_COUNT={processed}")
    inputs:
      file:
        ref: parameters.input_file
      format:
        ref: parameters.output_format
      records:
        ref: steps.validate.outputs.record_count
      is_valid:
        ref: steps.validate.outputs.valid
    outputs:
      output_file:
        pattern: "OUTPUT_FILE=(.*)"
      processed_count:
        pattern: "PROCESSED_COUNT=(\\d+)"

results:
  result_file:
    ref: steps.transform.outputs.output_file
  total_processed:
    ref: steps.transform.outputs.processed_count
  quality_score:
    ref: steps.validate.outputs.quality
```

## Core Concepts

### Chains
Chains define a sequence of steps with parameters, step execution, and results. Defined in YAML, they produce deterministic JSON output.

### Parameters
Global parameters with typed values (string, int, float, bool, datetime) that can be referenced by any step.

### Steps
Each step represents a script execution with:
- **Type**: The interpreter (bash, batch, powershell, pwsh, python)
- **Script**: The script content with `{{ inputs.name }}` placeholders
- **Inputs**: References to parameters or previous step outputs
- **Outputs**: Regex patterns to extract values from stdout

### Custom Interpreters
Override default interpreter behavior or add new interpreters by defining custom configurations:

```yaml
interpreters:
  bash:
    key: bash
    command: /bin/bash
    args:
      - "-e"  # Exit on error
      - "-x"  # Print commands
    extension: .sh
  
  python:
    key: python
    command: python3.11
    args:
      - "-u"  # Unbuffered output
    extension: .py
  
  node:  # New custom interpreter
    key: node
    command: node
    args:
      - "--no-warnings"
    extension: .js
```

Custom interpreters are looked up by key and override default settings. This allows you to:
- Use specific interpreter versions (e.g., `python3.11` instead of `python3`)
- Add custom flags (e.g., `-e` for bash to exit on error)
- Define entirely new interpreters (e.g., `node`, `ruby`, `php`)

See [`examples/custom_interpreter_chain.yaml`](examples/custom_interpreter_chain.yaml) for a complete example.

### Output Extraction
Outputs use regex patterns with capture groups to extract values from script stdout. Extracted values can be referenced by subsequent steps.

### Results
Chain-level results reference specific step outputs to be included in the final JSON output.

### Executors
Executors handle script execution with temporary files and timeout management. Custom executors can be implemented for testing.

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
# Run the simple chain example
cargo run --example simple_chain

# Run the README.md examples (validates the documented chains)
cargo run --example readme_examples
```

## Benchmarks

Run performance benchmarks:

```bash
cargo run --release --bin chain_parsing
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

