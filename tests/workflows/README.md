# QA Integration Test Suite

This directory contains comprehensive integration tests designed for Quality Assurance and smoke testing of the Atento workflow engine.

## Structure

```
atento-core/tests/workflows/
├── unix/                           # Unix-specific workflows (bash only)
│   ├── data_types_bash.yaml            # Test all data types with bash
│   └── input_output_chain.yaml         # Test complex input/output chaining
├── windows/                        # Windows-specific workflows (batch, powershell only)
│   ├── data_types_batch.yaml           # Test all data types with batch
│   └── data_types_powershell.yaml      # Test all data types with Windows PowerShell
└── cross-platform/                # Cross-platform workflows (python, pwsh)
    ├── data_types_python.yaml          # Test all data types with python
    ├── data_types_powershell_core.yaml # Test all data types with PowerShell Core
    └── python_advanced.yaml            # Test advanced Python operations
```

## Test Coverage

### Data Types Tested
- **String**: Text manipulation, length checks, content validation
- **Integer**: Arithmetic operations, comparisons, type conversion
- **Float**: Decimal calculations, precision testing, mathematical operations
- **Boolean**: Logic operations, true/false validation, conditional branching
- **DateTime**: ISO format parsing, date validation, timezone handling

### Interpreters Tested
- **Unix-specific**:
  - `bash`: Shell scripting, command execution, text processing
- **Windows-specific**:
  - `batch`: Windows command-line scripting, file operations
  - `powershell`: Windows PowerShell object manipulation, .NET integration, advanced scripting
- **Cross-platform**:
  - `python`: Complex data manipulation, JSON processing, mathematical operations
  - `pwsh`: PowerShell Core cross-platform scripting, .NET integration, works on Unix and Windows

### Features Tested
- **Parameter passing**: All data types as workflow parameters
- **Input/Output chaining**: Complex result references between steps
- **Type conversion**: Automatic conversion between string and typed values
- **Error handling**: Validation errors, execution failures
- **Output extraction**: Regex pattern matching, data extraction
- **Step dependencies**: Sequential execution with data flow

## Test Execution

### Automatic Discovery Tests
Integration tests automatically discover and execute all workflow files with **colorized output**:

```bash
# Run Unix workflows (automatically discovers all .yaml files)
cargo test test_workflow_smoke_tests_unix -- --nocapture

# Run Windows workflows (automatically discovers all .yaml files)  
cargo test test_workflow_smoke_tests_windows -- --nocapture

# Run Cross-platform workflows (automatically discovers all .yaml files)
cargo test test_workflow_smoke_tests_cross_platform -- --nocapture

# Run ALL smoke tests with colorized output
cargo test smoke -- --nocapture

# Validate all workflow files have correct YAML syntax
cargo test test_workflow_file_validation -- --nocapture

# Quick QA summary tests (shows results in assertion messages)
cargo test test_qa_workflow_summary_unix
cargo test test_qa_workflow_summary_windows
```

### Colorized Output Features
- 🟢 **Green**: Successful workflows and validation results
- 🔴 **Red**: Failed workflows and error messages  
- 🔵 **Blue**: Running workflow names and headers
- 🟡 **Yellow**: Validation test headers
- 🟣 **Purple**: Windows test headers
- 🎉 **Emojis**: Success celebrations and checkmarks

### Manual Workflow Execution
Individual workflows can be run manually for detailed inspection:

```bash
# Run specific Unix workflows
cargo run -- atento-core/tests/workflows/unix/data_types_bash.yaml
cargo run -- atento-core/tests/workflows/unix/input_output_chain.yaml

# Run specific Windows workflows
cargo run -- atento-core/tests/workflows/windows/data_types_batch.yaml
cargo run -- atento-core/tests/workflows/windows/data_types_powershell.yaml

# Run specific Cross-platform workflows (work on any OS)
cargo run -- atento-core/tests/workflows/cross-platform/data_types_python.yaml
cargo run -- atento-core/tests/workflows/cross-platform/data_types_powershell_core.yaml
cargo run -- atento-core/tests/workflows/cross-platform/python_advanced.yaml
```

## QA Workflow Design

Each workflow is designed with clear **OK/NOK** outputs:

1. **Self-Testing**: Workflows validate their own execution
2. **Clear Results**: Each step outputs OK or NOK status
3. **Final Assessment**: Final step aggregates all results
4. **Error Propagation**: Failures cause workflow termination with exit code 1

### Example Output Structure
```json
{
  "name": "Data Types Test - Bash",
  "duration_ms": 64,
  "steps": {
    "test_string_type": {
      "outputs": { "result": "OK" },
      "stdout": "Testing string: Hello World"
    },
    "final_result": {
      "outputs": { "final": "OK" },
      "stdout": "=== DATA TYPES TEST RESULTS ===\nString: OK\nInteger: OK\nFloat: OK\nBoolean: OK\nDateTime: OK"
    }
  }
}
```

## Adding New QA Tests

To add new QA workflows:

1. **Create workflow file** in appropriate platform directory
2. **Follow naming convention**: `feature_interpreter.yaml`
3. **Include self-validation**: Each step should validate its results
4. **Output OK/NOK status**: Clear success/failure indicators
5. **Add final assessment**: Aggregate all test results
6. **Test all data types**: Include comprehensive data type testing

### Template Structure
```yaml
name: "Test Name"
description: "Test description"

parameters:
  # Test parameters with all data types
  
steps:
  test_step:
    type: script::interpreter
    script: |
      # Test logic
      if [condition]; then
        echo "TEST_RESULT=OK"
      else
        echo "TEST_RESULT=NOK"
        exit 1
      fi
    outputs:
      result:
        pattern: "TEST_RESULT=(.*)"
        type: string

  final_assessment:
    # Aggregate all results and output FINAL_RESULT=OK/NOK
```

## Test Statistics

- **Total Workflows**: 7 (2 Unix + 2 Windows + 3 Cross-platform)
- **Data Types Covered**: 5 (string, int, float, bool, datetime)
- **Interpreters Covered**: 4 (bash, batch, powershell, python, pwsh)
- **Test Categories**: Data types, I/O chaining, validation, cross-platform compatibility
- **Execution Time**: ~1.9s for complete test suite
- **Test Automation**: Fully automated discovery and execution