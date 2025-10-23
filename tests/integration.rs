// Integration tests for atento-core
// These tests use the public API to verify functionality with real system calls

#![allow(clippy::collapsible_if, clippy::useless_format, clippy::print_literal)]

use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// File system and I/O tests
#[test]
fn test_run_file_not_found() {
    let result = atento_core::run("nonexistent_file.yaml");
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Io { path, .. }) = result {
        assert_eq!(path, "nonexistent_file.yaml");
    } else {
        panic!("Expected Io error");
    }
}

#[test]
fn test_run_nonexistent_file_with_special_chars() {
    let result = atento_core::run("file_with_ñ_ümläuts.yaml");
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Io { path, .. }) = result {
        assert_eq!(path, "file_with_ñ_ümläuts.yaml");
    } else {
        panic!("Expected Io error");
    }
}

#[test]
fn test_run_empty_filename() {
    let result = atento_core::run("");
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Io { path, .. }) = result {
        assert_eq!(path, "");
    } else {
        panic!("Expected Io error");
    }
}

#[test]
fn test_run_directory_instead_of_file() {
    // Try to run a directory instead of a file
    let result = atento_core::run("/tmp");
    assert!(result.is_err());
    // On most systems, trying to read a directory as a file should result in an IO error
    assert!(matches!(result, Err(atento_core::AtentoError::Io { .. })));
}

// YAML parsing tests
#[test]
fn test_run_invalid_yaml_syntax() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "invalid: yaml: {{").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::YamlParse { .. }) = result {
        // Expected
    } else {
        panic!("Expected YamlParse error");
    }
}

#[test]
fn test_run_invalid_yaml_with_tabs() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "name: test\n\tsteps:").unwrap(); // Tabs are invalid in YAML
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(atento_core::AtentoError::YamlParse { .. })
    ));
}

#[test]
fn test_run_yaml_with_invalid_unicode() {
    let mut temp_file = NamedTempFile::new().unwrap();
    // Create invalid UTF-8 sequence
    temp_file.write_all(&[0xFF, 0xFE]).unwrap();
    temp_file.write_all(b"name: test").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    // Should fail on file reading due to invalid UTF-8
    assert!(matches!(result, Err(atento_core::AtentoError::Io { .. })));
}

#[test]
fn test_run_completely_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    // Don't write anything - file is completely empty
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    // An empty file should either parse as an empty YAML doc or fail with YAML parse error
    match result {
        Ok(()) => {
            // Some YAML parsers accept empty files as valid empty documents
        }
        Err(atento_core::AtentoError::YamlParse { .. }) => {
            // Other parsers may reject empty files
        }
        Err(e) => {
            panic!("Expected YamlParse error or success, got: {e:?}");
        }
    }
}

#[test]
fn test_run_yaml_syntax_error_missing_colon() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "name test").unwrap(); // Missing colon
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(atento_core::AtentoError::YamlParse { .. })
    ));
}

#[test]
fn test_run_yaml_with_duplicate_keys() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "name: first\nname: second").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    // This might succeed with the second value taking precedence,
    // or fail depending on YAML parser - both are valid behaviors
    // We're just testing that we handle it gracefully
    assert!(result.is_ok() || matches!(result, Err(atento_core::AtentoError::YamlParse { .. })));
}

// Workflow validation tests (use real file I/O + validation)
#[cfg(unix)]
#[test]
fn test_run_workflow_forward_reference_error() {
    let yaml = r"
name: forward_ref_workflow
steps:
  step1:
    type: script::bash
    script: |
      echo {{ inputs.future }}
    inputs:
      future:
        ref: steps.step2.outputs.value
  step2:
    type: script::bash
    script: |
      echo 'value: 42'
    outputs:
      value:
        pattern: 'value: (\d+)'
        type: int
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    match result {
        Err(atento_core::AtentoError::Validation(msg)) => {
            assert!(msg.contains("future step output"));
        }
        Err(e) => {
            panic!("Expected Validation error about forward reference, got: {e:?}");
        }
        Ok(()) => {
            panic!("Expected error but got success");
        }
    }
}

#[cfg(unix)]
#[test]
fn test_run_workflow_empty_output_pattern() {
    let yaml = r"
name: empty_pattern_workflow
steps:
  step1:
    type: script::bash
    script: echo test
    outputs:
      value:
        pattern: ''
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Validation(_)) = result {
        // Expected
    } else {
        panic!("Expected Validation error");
    }
}

#[cfg(unix)]
#[test]
fn test_run_workflow_invalid_regex_pattern() {
    let yaml = r"
name: invalid_regex_workflow
steps:
  step1:
    type: script::bash
    script: echo test
    outputs:
      value:
        pattern: '([invalid'
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Validation(msg)) = result {
        assert!(msg.contains("invalid regex pattern"));
    } else {
        panic!("Expected Validation error about regex");
    }
}

#[cfg(unix)]
#[test]
fn test_run_workflow_unused_input() {
    let yaml = r"
name: unused_input_workflow
steps:
  step1:
    type: script::bash
    script: echo hello
    inputs:
      unused:
        type: string
        value: never used
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Validation(msg)) = result {
        assert!(msg.contains("never used"));
    } else {
        panic!("Expected Validation error about unused input");
    }
}

#[cfg(unix)]
#[test]
fn test_run_workflow_undeclared_input() {
    let yaml = r"
name: undeclared_input_workflow
steps:
  step1:
    type: script::bash
    script: echo {{ inputs.undefined }}
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Validation(msg)) = result {
        assert!(msg.contains("not declared"));
    } else {
        panic!("Expected Validation error about undeclared input");
    }
}

#[cfg(unix)]
#[test]
fn test_run_workflow_with_validation_error() {
    let yaml = r"
name: invalid_workflow
steps:
  step1:
    type: script::bash
    script: echo {{ inputs.undefined }}
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Validation(_)) = result {
        // Expected
    } else {
        panic!("Expected Validation error");
    }
}

// Basic workflow execution tests (minimal setup)
#[test]
fn test_run_empty_workflow() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "name: empty").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_run_workflow_with_name() {
    let yaml = r"
name: named_workflow
steps:
  step1:
    type: script::bash
    script: echo test
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_run_workflow_without_name() {
    let yaml = r"
steps:
  step1:
    type: script::bash
    script: echo test
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_run_simple_workflow_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("simple.yaml");

    let workflow_content = r#"
name: "Simple Test Workflow"
steps:
  test_step:
    type: script::bash
    script: echo "Hello from integration test"
"#;

    fs::write(&workflow_path, workflow_content).unwrap();

    // This should run successfully using the public API
    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[test]
fn test_run_nonexistent_file() {
    let result = atento_core::run("nonexistent_file.yaml");
    assert!(result.is_err());

    if let Err(atento_core::AtentoError::Io { path, .. }) = result {
        assert_eq!(path, "nonexistent_file.yaml");
    } else {
        panic!("Expected Io error");
    }
}

#[test]
fn test_run_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("invalid.yaml");

    fs::write(&workflow_path, "invalid: yaml: content: [").unwrap();

    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_err());

    assert!(matches!(
        result,
        Err(atento_core::AtentoError::YamlParse { .. })
    ));
}

#[cfg(unix)]
#[test]
fn test_run_bash_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("bash_test.yaml");

    let workflow_content = r#"
name: "Bash Integration Test"
steps:
  bash_step:
    type: script::bash
    script: |
      echo "Testing bash execution"
      echo "Exit code: $?"
"#;

    fs::write(&workflow_path, workflow_content).unwrap();

    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_run_python_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("python_test.yaml");

    let workflow_content = r#"
name: "Python Integration Test"
steps:
  python_step:
    type: script::python
    script: |
      print("Testing python execution")
      print(f"2 + 2 = {2 + 2}")
"#;

    fs::write(&workflow_path, workflow_content).unwrap();

    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_batch_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("batch_test.yaml");

    let workflow_content = r#"
name: "Batch Integration Test"
steps:
  batch_step:
    type: script::batch
    script: |
      echo Testing batch execution
      echo Exit code: %ERRORLEVEL%
"#;

    fs::write(&workflow_path, workflow_content).unwrap();

    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_powershell_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("powershell_test.yaml");

    let workflow_content = r#"
name: "PowerShell Integration Test"
steps:
  powershell_step:
    type: script::powershell
    script: |
      Write-Host "Testing PowerShell execution"
      Write-Host "2 + 2 = $(2 + 2)"
"#;

    fs::write(&workflow_path, workflow_content).unwrap();

    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_python_workflow_windows() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("python_test.yaml");

    let workflow_content = r#"
name: "Python Integration Test (Windows)"
steps:
  python_step:
    type: script::python
    script: |
      print("Testing python execution on Windows")
      print(f"2 + 2 = {2 + 2}")
"#;

    fs::write(&workflow_path, workflow_content).unwrap();

    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_ok());
}

// Windows-specific versions of key tests using batch commands
#[cfg(windows)]
#[test]
fn test_run_workflow_with_name_windows() {
    let yaml = r"
name: named_workflow
steps:
  step1:
    type: script::batch
    script: echo test
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_workflow_without_name_windows() {
    let yaml = r"
steps:
  step1:
    type: script::batch
    script: echo test
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_simple_workflow_from_file_windows() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("simple.yaml");

    let workflow_content = r#"
name: "Simple Test Workflow"
steps:
  test_step:
    type: script::batch
    script: echo Hello from integration test
"#;

    fs::write(&workflow_path, workflow_content).unwrap();

    // This should run successfully using the public API
    let result = atento_core::run(workflow_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_workflow_undeclared_input_windows() {
    let yaml = r"
name: undeclared_input_workflow
steps:
  step1:
    type: script::batch
    script: echo {{ inputs.undefined }}
";
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{yaml}").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_err());
    if let Err(atento_core::AtentoError::Validation(msg)) = result {
        assert!(msg.contains("not declared"));
    } else {
        panic!("Expected Validation error about undeclared input");
    }
}

// Comprehensive workflow tests - QA smoke tests
#[cfg(unix)]
#[test]
fn test_workflow_smoke_tests_unix() {
    // The test runs from atento-core directory, so workflows are in tests/workflows/unix
    let workflow_dir = std::path::Path::new("tests/workflows/unix");

    // Skip if workflows directory doesn't exist (development environments)
    if !workflow_dir.exists() {
        println!("Skipping Unix workflow tests - directory not found");
        return;
    }

    let mut test_results = Vec::new();

    // Discover and run all .yaml files in the unix directory
    let entries = fs::read_dir(workflow_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let workflow_name = path.file_name().unwrap().to_str().unwrap();
            eprintln!("\x1b[36mRunning Unix workflow: {}\x1b[0m", workflow_name);

            // Parse the workflow and run it to obtain a WorkflowResult so we can inspect step stderr
            let contents = fs::read_to_string(&path).unwrap_or_default();
            let wf: atento_core::Workflow = match serde_yaml::from_str(&contents) {
                Ok(w) => w,
                Err(e) => {
                    test_results.push((
                        workflow_name.to_string(),
                        format!("FAILED: invalid YAML: {}", e),
                    ));
                    eprintln!(
                        "\x1b[31m✗ {} - FAILED: invalid YAML: {}\x1b[0m",
                        workflow_name, e
                    );
                    continue;
                }
            };

            // Pre-check that interpreters required by the workflow steps are actually runnable on this host.
            // This checks the exact program the runtime will invoke (for example 'python3' vs 'python').
            let mut missing_progs = Vec::new();
            for (_k, step) in &wf.steps {
                // Get the program that will be invoked for this interpreter
                let args = step.interpreter.args();
                if args.is_empty() {
                    continue;
                }
                let prog = args[0];

                // Build candidate commands to try: prefer the exact prog, but for common aliases try fallbacks
                let candidates: Vec<Vec<&str>> = if prog == "python3" {
                    vec![
                        vec!["python3", "-c", "import sys; sys.exit(0)"],
                        vec!["python", "-c", "import sys; sys.exit(0)"],
                    ]
                } else if prog == "python" {
                    vec![
                        vec!["python", "-c", "import sys; sys.exit(0)"],
                        vec!["python3", "-c", "import sys; sys.exit(0)"],
                    ]
                } else if prog == "pwsh" {
                    vec![
                        vec!["pwsh", "-c", "exit 0"],
                        vec!["powershell", "-Command", "exit 0"],
                    ]
                } else if prog == "powershell" {
                    vec![
                        vec!["powershell", "-Command", "exit 0"],
                        vec!["pwsh", "-c", "exit 0"],
                    ]
                } else if prog == "bash" {
                    vec![vec!["bash", "-c", "exit 0"]]
                } else {
                    vec![vec![prog, "--version"]]
                };

                let mut runnable = false;
                for cand in candidates.iter() {
                    let prog = cand[0];
                    let args = &cand[1..];
                    let attempted = std::process::Command::new(prog).args(args).output();
                    if let Ok(output) = attempted
                        && output.status.success()
                    {
                        runnable = true;
                        break;
                    }
                }

                if !runnable {
                    missing_progs.push(prog.to_string());
                }
            }

            if !missing_progs.is_empty() {
                let msg = format!(
                    "SKIPPED: Missing exact interpreter executables: {}",
                    missing_progs.join(", ")
                );
                test_results.push((workflow_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", workflow_name, msg);
                continue;
            }

            let result = wf.run();
            let json = serde_json::to_string_pretty(&result).unwrap_or_default();
            println!("{}", json);

            // If there are no errors the workflow passed
            if result.errors.is_empty() {
                test_results.push((workflow_name.to_string(), "PASSED".to_string()));
                eprintln!("\x1b[32m✓ {} - PASSED\x1b[0m", workflow_name);
                continue;
            }

            // Inspect step stderr to detect missing interpreters -> mark as SKIPPED
            let mut detected_missing = false;
            let missing_indicators = [
                "was not found",
                "not recognized",
                "no such file or directory",
                "command not found",
                "not found",
            ];

            if let Some(steps_map) = result.steps {
                for (_k, step_res) in steps_map.iter() {
                    let stderr = step_res.stderr.clone().unwrap_or_default().to_lowercase();
                    eprintln!(
                        "DEBUG: step exit_code={} stderr=[{}]",
                        step_res.exit_code, stderr
                    );
                    if step_res.exit_code == 9009
                        || missing_indicators.iter().any(|ind| stderr.contains(ind))
                    {
                        detected_missing = true;
                        break;
                    }
                }
            }

            if detected_missing {
                let msg = format!("SKIPPED: missing interpreter detected in step output");
                test_results.push((workflow_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", workflow_name, msg);
            } else {
                test_results.push((
                    workflow_name.to_string(),
                    format!("FAILED: {}", "Workflow completed with errors"),
                ));
                eprintln!(
                    "\x1b[31m✗ {} - FAILED: {}\x1b[0m",
                    workflow_name, "Workflow completed with errors"
                );
            }
        }
    }

    // Print summary
    eprintln!("\n\x1b[1m\x1b[36m=== UNIX WORKFLOW SMOKE TEST RESULTS ===\x1b[0m");

    let passed_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("PASSED"))
        .count();
    let failed_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("FAILED"))
        .count();
    let skipped_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("SKIPPED"))
        .count();

    for (workflow, result) in &test_results {
        if result.starts_with("PASSED") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", workflow, result);
        } else if result.starts_with("SKIPPED") {
            eprintln!("\x1b[33m{}: {}\x1b[0m", workflow, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", workflow, result);
        }
    }

    // Ensure we found and ran some workflows
    assert!(
        !test_results.is_empty(),
        "No workflow files found in unix directory"
    );

    // Report summary statistics
    eprintln!(
        "\n\x1b[1mSummary: {} PASSED, {} FAILED, {} SKIPPED (Total: {})\x1b[0m",
        passed_count,
        failed_count,
        skipped_count,
        test_results.len()
    );

    // Ensure no workflows failed
    if failed_count > 0 {
        panic!(
            "{} out of {} Unix workflows failed",
            failed_count,
            test_results.len()
        );
    }

    // Ensure we actually ran some workflows (not all skipped)
    if passed_count == 0 {
        panic!(
            "No Unix workflows could be executed - all {} were skipped. This likely indicates missing interpreters in CI environment.",
            test_results.len()
        );
    }

    eprintln!(
        "\x1b[1m\x1b[32m🎉 {} Unix workflow(s) passed successfully!\x1b[0m",
        passed_count
    );
}

// QA-friendly test that shows results in assertion messages
#[cfg(unix)]
#[test]
fn test_qa_workflow_summary_unix() {
    let workflow_dir = std::path::Path::new("tests/workflows/unix");

    if !workflow_dir.exists() {
        panic!("QA workflows directory not found: tests/workflows/unix");
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut workflow_names = Vec::new();

    let entries = fs::read_dir(workflow_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let workflow_name = path.file_name().unwrap().to_str().unwrap();
            workflow_names.push(workflow_name.to_string());

            match atento_core::run(path.to_str().unwrap()) {
                Ok(()) => passed += 1,
                Err(_) => failed += 1,
            }
        }
    }

    assert_eq!(
        failed,
        0,
        "QA Smoke Test Results: {} PASSED, {} FAILED workflows: [{}]",
        passed,
        failed,
        workflow_names.join(", ")
    );
}

// QA-friendly test that shows results in assertion messages - Windows
#[cfg(windows)]
#[test]
fn test_qa_workflow_summary_windows() {
    let workflow_dir = std::path::Path::new("tests/workflows/windows");

    if !workflow_dir.exists() {
        panic!("QA workflows directory not found: tests/workflows/windows");
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut workflow_names = Vec::new();

    let entries = fs::read_dir(workflow_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let workflow_name = path.file_name().unwrap().to_str().unwrap();
            workflow_names.push(workflow_name.to_string());

            match atento_core::run(path.to_str().unwrap()) {
                Ok(()) => passed += 1,
                Err(_) => failed += 1,
            }
        }
    }

    assert_eq!(
        failed,
        0,
        "QA Smoke Test Results: {} PASSED, {} FAILED workflows: [{}]",
        passed,
        failed,
        workflow_names.join(", ")
    );
}

#[cfg(windows)]
#[test]
fn test_workflow_smoke_tests_windows() {
    // The test runs from atento-core directory, so workflows are in tests/workflows/windows
    let workflow_dir = std::path::Path::new("tests/workflows/windows");

    // Skip if workflows directory doesn't exist (development environments)
    if !workflow_dir.exists() {
        println!("Skipping Windows workflow tests - directory not found");
        return;
    }

    let mut test_results = Vec::new();

    // Discover and run all .yaml files in the windows directory
    let entries = fs::read_dir(workflow_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let workflow_name = path.file_name().unwrap().to_str().unwrap();
            eprintln!("\x1b[36mRunning Windows workflow: {}\x1b[0m", workflow_name);

            // Parse the workflow and run it to inspect step outputs for missing interpreters
            let contents = fs::read_to_string(&path).unwrap_or_default();
            let wf: atento_core::Workflow = match serde_yaml::from_str(&contents) {
                Ok(w) => w,
                Err(e) => {
                    test_results.push((
                        workflow_name.to_string(),
                        format!("FAILED: invalid YAML: {}", e),
                    ));
                    eprintln!(
                        "\x1b[31m✗ {} - FAILED: invalid YAML: {}\x1b[0m",
                        workflow_name, e
                    );
                    continue;
                }
            };

            let result = wf.run();
            let json = serde_json::to_string_pretty(&result).unwrap_or_default();
            println!("{}", json);

            if result.errors.is_empty() {
                test_results.push((workflow_name.to_string(), "PASSED".to_string()));
                eprintln!("\x1b[32m✓ {} - PASSED\x1b[0m", workflow_name);
                continue;
            }

            // Inspect step stderr/duration to detect missing interpreters and mark SKIPPED
            let mut detected_missing = false;
            let missing_indicators = [
                "was not found",
                "not recognized",
                "no such file or directory",
                "command not found",
                "not found",
            ];

            if let Some(steps_map) = result.steps {
                for (_k, step_res) in steps_map.iter() {
                    let stderr = step_res.stderr.clone().unwrap_or_default().to_lowercase();
                    eprintln!(
                        "DEBUG: step exit_code={} stderr=[{}]",
                        step_res.exit_code, stderr
                    );
                    if step_res.exit_code == 9009
                        || missing_indicators.iter().any(|ind| stderr.contains(ind))
                    {
                        detected_missing = true;
                        break;
                    }
                }
            }

            if detected_missing {
                let msg = format!("SKIPPED: missing interpreter detected in step output");
                test_results.push((workflow_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", workflow_name, msg);
            } else {
                test_results.push((
                    workflow_name.to_string(),
                    format!("FAILED: {}", "Workflow completed with errors"),
                ));
                eprintln!(
                    "\x1b[31m✗ {} - FAILED: {}\x1b[0m",
                    workflow_name, "Workflow completed with errors"
                );
            }
        }
    }

    // Print summary
    eprintln!("\n\x1b[1m\x1b[35m=== WINDOWS WORKFLOW SMOKE TEST RESULTS ===\x1b[0m");

    let passed_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("PASSED"))
        .count();
    let failed_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("FAILED"))
        .count();
    let skipped_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("SKIPPED"))
        .count();

    for (workflow, result) in &test_results {
        if result.starts_with("PASSED") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", workflow, result);
        } else if result.starts_with("SKIPPED") {
            eprintln!("\x1b[33m{}: {}\x1b[0m", workflow, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", workflow, result);
        }
    }

    // Ensure we found and ran some workflows
    assert!(
        !test_results.is_empty(),
        "No workflow files found in windows directory"
    );

    // Report summary statistics
    eprintln!(
        "\n\x1b[1mSummary: {} PASSED, {} FAILED, {} SKIPPED (Total: {})\x1b[0m",
        passed_count,
        failed_count,
        skipped_count,
        test_results.len()
    );

    // Ensure no workflows failed
    if failed_count > 0 {
        panic!(
            "{} out of {} Windows workflows failed",
            failed_count,
            test_results.len()
        );
    }

    // Ensure we actually ran some workflows (not all skipped)
    if passed_count == 0 {
        panic!(
            "No Windows workflows could be executed - all {} were skipped. This likely indicates missing interpreters in CI environment.",
            test_results.len()
        );
    }

    eprintln!(
        "\x1b[1m\x1b[32m🎉 {} Windows workflow(s) passed successfully!\x1b[0m",
        passed_count
    );
}

// Cross-platform workflow smoke tests
#[test]
fn test_workflow_smoke_tests_cross_platform() {
    // The test runs from atento-core directory, so workflows are in tests/workflows/cross-platform
    let workflow_dir = std::path::Path::new("tests/workflows/cross-platform");

    // Skip if workflows directory doesn't exist (development environments)
    if !workflow_dir.exists() {
        println!("Skipping Cross-platform workflow tests - directory not found");
        return;
    }

    let mut test_results = Vec::new();

    // Discover and run all .yaml files in the cross-platform directory
    let entries = fs::read_dir(workflow_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let workflow_name = path.file_name().unwrap().to_str().unwrap();
            eprintln!(
                "\x1b[36mRunning Cross-platform workflow: {}\x1b[0m",
                workflow_name
            );
            // Read the workflow and detect required interpreters by simple text scan.
            // This is intentionally permissive and avoids YAML parsing edge-cases in tests.
            let content = fs::read_to_string(&path).unwrap_or_default();
            let content_lc = content.to_lowercase();
            let mut required_bins = std::collections::HashSet::new();
            if content_lc.contains("script::python") || content_lc.contains("type: script::python")
            {
                required_bins.insert("python");
            }
            if content_lc.contains("script::bash") || content_lc.contains("type: script::bash") {
                required_bins.insert("bash");
            }
            if content_lc.contains("script::powershell") || content_lc.contains("script::pwsh") {
                required_bins.insert("pwsh_or_powershell");
            }

            // Helper to try running a minimal command with the given interpreter to ensure it's usable.
            fn is_runnable(bin: &str) -> bool {
                use std::process::Command;

                let try_cmds: Vec<Vec<String>> = match bin {
                    "python" => vec![
                        vec![
                            "python".into(),
                            "-c".into(),
                            "import sys; sys.exit(0)".into(),
                        ],
                        vec![
                            "python3".into(),
                            "-c".into(),
                            "import sys; sys.exit(0)".into(),
                        ],
                    ],
                    "bash" => vec![vec!["bash".into(), "-c".into(), "exit 0".into()]],
                    "pwsh_or_powershell" => vec![
                        vec!["pwsh".into(), "-c".into(), "exit 0".into()],
                        vec!["powershell".into(), "-Command".into(), "exit 0".into()],
                    ],
                    other => vec![vec![other.to_string(), "--version".into()]],
                };

                for cmd in try_cmds {
                    if cmd.is_empty() {
                        continue;
                    }
                    let prog = &cmd[0];
                    let args = &cmd[1..];
                    let res = Command::new(prog).args(args).output();
                    if let Ok(output) = res {
                        // Consider runnable if the process executed and returned success
                        if output.status.success() {
                            return true;
                        }
                    }
                }
                false
            }

            // Check required bins; if missing, skip this workflow (mark SKIPPED)
            let mut missing = Vec::new();
            for bin in &required_bins {
                if *bin == "pwsh_or_powershell" {
                    if !(is_runnable("pwsh_or_powershell")) {
                        missing.push("pwsh/powershell");
                    }
                } else if !is_runnable(bin) {
                    missing.push(bin);
                }
            }

            eprintln!("DEBUG: required_bins={:?}", required_bins.clone());
            eprintln!("DEBUG: missing={:?}", missing.clone());

            if !missing.is_empty() {
                let msg = format!("SKIPPED: Missing interpreters: {}", missing.join(", "));
                test_results.push((workflow_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", workflow_name, msg);
                continue;
            }

            // Parse the workflow and run it to inspect step outputs for missing interpreters
            let contents = fs::read_to_string(&path).unwrap_or_default();
            let wf: atento_core::Workflow = match serde_yaml::from_str(&contents) {
                Ok(w) => w,
                Err(e) => {
                    test_results.push((
                        workflow_name.to_string(),
                        format!("FAILED: invalid YAML: {}", e),
                    ));
                    eprintln!(
                        "\x1b[31m✗ {} - FAILED: invalid YAML: {}\x1b[0m",
                        workflow_name, e
                    );
                    continue;
                }
            };

            // Pre-check exact interpreter executables required by steps (skip if missing)
            let mut missing_progs = Vec::new();
            for (_k, step) in &wf.steps {
                let args = step.interpreter.args();
                if args.is_empty() {
                    continue;
                }
                let prog = args[0];

                let candidates: Vec<Vec<&str>> = if prog == "python3" {
                    vec![
                        vec!["python3", "-c", "import sys; sys.exit(0)"],
                        vec!["python", "-c", "import sys; sys.exit(0)"],
                    ]
                } else if prog == "python" {
                    vec![
                        vec!["python", "-c", "import sys; sys.exit(0)"],
                        vec!["python3", "-c", "import sys; sys.exit(0)"],
                    ]
                } else if prog == "pwsh" {
                    vec![
                        vec!["pwsh", "-c", "exit 0"],
                        vec!["powershell", "-Command", "exit 0"],
                    ]
                } else if prog == "powershell" {
                    vec![
                        vec!["powershell", "-Command", "exit 0"],
                        vec!["pwsh", "-c", "exit 0"],
                    ]
                } else if prog == "bash" {
                    vec![vec!["bash", "-c", "exit 0"]]
                } else {
                    vec![vec![prog, "--version"]]
                };

                let mut runnable = false;
                for cand in candidates.iter() {
                    let prog = cand[0];
                    let args = &cand[1..];
                    if let Ok(output) = std::process::Command::new(prog).args(args).output()
                        && output.status.success()
                    {
                        runnable = true;
                        break;
                    }
                }

                if !runnable {
                    missing_progs.push(prog.to_string());
                }
            }

            if !missing_progs.is_empty() {
                let msg = format!(
                    "SKIPPED: Missing exact interpreter executables: {}",
                    missing_progs.join(", ")
                );
                test_results.push((workflow_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", workflow_name, msg);
                continue;
            }

            let result = wf.run();
            let json = serde_json::to_string_pretty(&result).unwrap_or_default();
            println!("{}", json);

            if result.errors.is_empty() {
                test_results.push((workflow_name.to_string(), "PASSED".to_string()));
                eprintln!("\x1b[32m✓ {} - PASSED\x1b[0m", workflow_name);
                continue;
            }

            // Inspect step stderr/duration to detect missing interpreters and mark SKIPPED
            let mut detected_missing = false;
            let missing_indicators = [
                "was not found",
                "not recognized",
                "no such file or directory",
                "command not found",
                "not found",
            ];

            if let Some(steps_map) = result.steps {
                for (_k, step_res) in steps_map.iter() {
                    let stderr = step_res.stderr.clone().unwrap_or_default().to_lowercase();
                    eprintln!(
                        "DEBUG: step exit_code={} stderr=[{}]",
                        step_res.exit_code, stderr
                    );
                    if step_res.exit_code == 9009
                        || missing_indicators.iter().any(|ind| stderr.contains(ind))
                    {
                        detected_missing = true;
                        break;
                    }
                }
            }

            if detected_missing {
                let msg = format!("SKIPPED: missing interpreter detected in step output");
                test_results.push((workflow_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", workflow_name, msg);
            } else {
                test_results.push((
                    workflow_name.to_string(),
                    format!("FAILED: {}", "Workflow completed with errors"),
                ));
                eprintln!(
                    "\x1b[31m✗ {} - FAILED: {}\x1b[0m",
                    workflow_name, "Workflow completed with errors"
                );
            }
        }
    }

    // Print summary
    eprintln!("\n\x1b[1m\x1b[33m=== CROSS-PLATFORM WORKFLOW SMOKE TEST RESULTS ===\x1b[0m");

    let passed_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("PASSED"))
        .count();
    let failed_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("FAILED"))
        .count();
    let skipped_count = test_results
        .iter()
        .filter(|(_, result)| result.starts_with("SKIPPED"))
        .count();

    for (workflow, result) in &test_results {
        if result.starts_with("PASSED") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", workflow, result);
        } else if result.starts_with("SKIPPED") {
            eprintln!("\x1b[33m{}: {}\x1b[0m", workflow, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", workflow, result);
        }
    }

    // Ensure we found and ran some workflows
    assert!(
        !test_results.is_empty(),
        "No workflow files found in cross-platform directory"
    );

    // Report summary statistics
    eprintln!(
        "\n\x1b[1mSummary: {} PASSED, {} FAILED, {} SKIPPED (Total: {})\x1b[0m",
        passed_count,
        failed_count,
        skipped_count,
        test_results.len()
    );

    // Ensure no workflows failed
    if failed_count > 0 {
        panic!(
            "{} out of {} Cross-platform workflows failed",
            failed_count,
            test_results.len()
        );
    }

    // Ensure we actually ran some workflows (not all skipped)
    if passed_count == 0 {
        panic!(
            "No cross-platform workflows could be executed - all {} were skipped. This likely indicates missing interpreters in CI environment.",
            test_results.len()
        );
    }

    eprintln!(
        "\x1b[1m\x1b[32m🎉 {} Cross-platform workflow(s) passed successfully!\x1b[0m",
        passed_count
    );
}

// Cross-platform workflow validation test
#[test]
fn test_workflow_file_validation() {
    // The test runs from atento-core directory, so workflows are in tests/workflows
    let base_dir = std::path::Path::new("tests/workflows");
    if !base_dir.exists() {
        println!("Skipping workflow validation - workflows directory not found");
        return;
    }

    let mut total_workflows = 0;
    let mut validation_results = Vec::new();

    // Check unix, windows, and cross-platform directories
    for platform in &["unix", "windows", "cross-platform"] {
        let platform_dir = base_dir.join(platform);
        if !platform_dir.exists() {
            continue;
        }

        let entries = fs::read_dir(&platform_dir).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            if path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
            {
                total_workflows += 1;
                let workflow_name = format!(
                    "{}/{}",
                    platform,
                    path.file_name().unwrap().to_str().unwrap()
                );

                // Read and basic validation - just ensure it's valid YAML
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        // Try to parse as YAML (basic validation)
                        match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                            Ok(_) => {
                                validation_results.push((workflow_name, "VALID YAML".to_string()));
                            }
                            Err(e) => {
                                validation_results
                                    .push((workflow_name, format!("INVALID YAML: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        validation_results.push((workflow_name, format!("READ ERROR: {}", e)));
                    }
                }
            }
        }
    }

    // Print validation results
    eprintln!("\n\x1b[1m\x1b[33m=== WORKFLOW FILE VALIDATION RESULTS ===\x1b[0m");
    for (workflow, result) in &validation_results {
        if result.starts_with("VALID") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", workflow, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", workflow, result);
        }
    }

    // Ensure we found some workflows
    if total_workflows == 0 {
        println!("No workflow files found - skipping validation test");
        return;
    }

    // Ensure all workflows have valid YAML
    let invalid_count = validation_results
        .iter()
        .filter(|(_, result)| !result.starts_with("VALID"))
        .count();

    assert_eq!(
        invalid_count, 0,
        "{} out of {} workflow files have invalid YAML",
        invalid_count, total_workflows
    );

    eprintln!(
        "\x1b[1m\x1b[32m✅ All {} workflow files have valid YAML syntax!\x1b[0m",
        total_workflows
    );
}
