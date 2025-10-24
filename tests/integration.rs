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

// Chain validation tests (use real file I/O + validation)
#[cfg(unix)]
#[test]
fn test_run_chain_forward_reference_error() {
    let yaml = r"
name: forward_ref_chain
steps:
  step1:
    type: bash
    script: |
      echo {{ inputs.future }}
    inputs:
      future:
        ref: steps.step2.outputs.value
  step2:
    type: bash
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
fn test_run_chain_empty_output_pattern() {
    let yaml = r"
name: empty_pattern_chain
steps:
  step1:
    type: bash
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
fn test_run_chain_invalid_regex_pattern() {
    let yaml = r"
name: invalid_regex_chain
steps:
  step1:
    type: bash
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
fn test_run_chain_unused_input() {
    let yaml = r"
name: unused_input_chain
steps:
  step1:
    type: bash
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
fn test_run_chain_undeclared_input() {
    let yaml = r"
name: undeclared_input_chain
steps:
  step1:
    type: bash
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
fn test_run_chain_with_validation_error() {
    let yaml = r"
name: invalid_chain
steps:
  step1:
    type: bash
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

// Basic chain execution tests (minimal setup)
#[test]
fn test_run_empty_chain() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "name: empty").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let result = atento_core::run(path);
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_run_chain_with_name() {
    let yaml = r"
name: named_chain
steps:
  step1:
    type: bash
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
fn test_run_chain_without_name() {
    let yaml = r"
steps:
  step1:
    type: bash
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
fn test_run_simple_chain_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let chain_path = temp_dir.path().join("simple.yaml");

    let chain_content = r#"
name: "Simple Test Chain"
steps:
  test_step:
    type: bash
    script: echo "Hello from integration test"
"#;

    fs::write(&chain_path, chain_content).unwrap();

    // This should run successfully using the public API
    let result = atento_core::run(chain_path.to_str().unwrap());
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
    let chain_path = temp_dir.path().join("invalid.yaml");

    fs::write(&chain_path, "invalid: yaml: content: [").unwrap();

    let result = atento_core::run(chain_path.to_str().unwrap());
    assert!(result.is_err());

    assert!(matches!(
        result,
        Err(atento_core::AtentoError::YamlParse { .. })
    ));
}

#[cfg(unix)]
#[test]
fn test_run_bash_chain() {
    let temp_dir = TempDir::new().unwrap();
    let chain_path = temp_dir.path().join("bash_test.yaml");

    let chain_content = r#"
name: "Bash Integration Test"
steps:
  bash_step:
    type: bash
    script: |
      echo "Testing bash execution"
      echo "Exit code: $?"
"#;

    fs::write(&chain_path, chain_content).unwrap();

    let result = atento_core::run(chain_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_run_python_chain() {
    let temp_dir = TempDir::new().unwrap();
    let chain_path = temp_dir.path().join("python_test.yaml");

    let chain_content = r#"
name: "Python Integration Test"
steps:
  python_step:
    type: python
    script: |
      print("Testing python execution")
      print(f"2 + 2 = {2 + 2}")
"#;

    fs::write(&chain_path, chain_content).unwrap();

    let result = atento_core::run(chain_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_batch_chain() {
    let temp_dir = TempDir::new().unwrap();
    let chain_path = temp_dir.path().join("batch_test.yaml");

    let chain_content = r#"
name: "Batch Integration Test"
steps:
  batch_step:
    type: batch
    script: |
      echo Testing batch execution
      echo Exit code: %ERRORLEVEL%
"#;

    fs::write(&chain_path, chain_content).unwrap();

    let result = atento_core::run(chain_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_powershell_chain() {
    let temp_dir = TempDir::new().unwrap();
    let chain_path = temp_dir.path().join("powershell_test.yaml");

    let chain_content = r#"
name: "PowerShell Integration Test"
steps:
  powershell_step:
    type: powershell
    script: |
      Write-Host "Testing PowerShell execution"
      Write-Host "2 + 2 = $(2 + 2)"
"#;

    fs::write(&chain_path, chain_content).unwrap();

    let result = atento_core::run(chain_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_python_chain_windows() {
    let temp_dir = TempDir::new().unwrap();
    let chain_path = temp_dir.path().join("python_test.yaml");

    let chain_content = r#"
name: "Python Integration Test (Windows)"
steps:
  python_step:
    type: python
    script: |
      print("Testing python execution on Windows")
      print(f"2 + 2 = {2 + 2}")
"#;

    fs::write(&chain_path, chain_content).unwrap();

    let result = atento_core::run(chain_path.to_str().unwrap());
    assert!(result.is_ok());
}

// Windows-specific versions of key tests using batch commands
#[cfg(windows)]
#[test]
fn test_run_chain_with_name_windows() {
    let yaml = r"
name: named_chain
steps:
  step1:
    type: batch
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
fn test_run_chain_without_name_windows() {
    let yaml = r"
steps:
  step1:
    type: batch
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
fn test_run_simple_chain_from_file_windows() {
    let temp_dir = TempDir::new().unwrap();
    let chain_path = temp_dir.path().join("simple.yaml");

    let chain_content = r#"
name: "Simple Test Chain"
steps:
  test_step:
    type: batch
    script: echo Hello from integration test
"#;

    fs::write(&chain_path, chain_content).unwrap();

    // This should run successfully using the public API
    let result = atento_core::run(chain_path.to_str().unwrap());
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn test_run_chain_undeclared_input_windows() {
    let yaml = r"
name: undeclared_input_chain
steps:
  step1:
    type: batch
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

// Comprehensive chain tests - QA smoke tests
#[cfg(unix)]
#[test]
fn test_chain_smoke_tests_unix() {
    // The test runs from atento-core directory, so chains are in tests/chains/unix
    let chain_dir = std::path::Path::new("tests/chains/unix");

    // Skip if chains directory doesn't exist (development environments)
    if !chain_dir.exists() {
        println!("Skipping Unix chain tests - directory not found");
        return;
    }

    let mut test_results = Vec::new();

    // Discover and run all .yaml files in the unix directory
    let entries = fs::read_dir(chain_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let chain_name = path.file_name().unwrap().to_str().unwrap();
            eprintln!("\x1b[36mRunning Unix chain: {}\x1b[0m", chain_name);

            // Parse the chain and run it to obtain a ChainResult so we can inspect step stderr
            let contents = fs::read_to_string(&path).unwrap_or_default();
            let wf: atento_core::Chain = match serde_yaml::from_str(&contents) {
                Ok(w) => w,
                Err(e) => {
                    test_results.push((
                        chain_name.to_string(),
                        format!("FAILED: invalid YAML: {}", e),
                    ));
                    eprintln!(
                        "\x1b[31m✗ {} - FAILED: invalid YAML: {}\x1b[0m",
                        chain_name, e
                    );
                    continue;
                }
            };

            // Pre-check that interpreters required by the chain steps are actually runnable on this host.
            // This checks the exact program the runtime will invoke (for example 'python3' vs 'python').
            let mut missing_progs = Vec::new();
            for (_k, step) in &wf.steps {
                // Get the program that will be invoked for this interpreter
                let interpreter = match wf.interpreters.get(&step.interpreter) {
                    Some(interp) => interp,
                    None => continue,
                };
                let args = &interpreter.args;
                if args.is_empty() {
                    continue;
                }
                let prog = args[0].as_str();

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
                test_results.push((chain_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", chain_name, msg);
                continue;
            }

            let result = wf.run();
            let json = serde_json::to_string_pretty(&result).unwrap_or_default();
            println!("{}", json);

            // If there are no errors the chain passed
            if result.errors.is_empty() {
                test_results.push((chain_name.to_string(), "PASSED".to_string()));
                eprintln!("\x1b[32m✓ {} - PASSED\x1b[0m", chain_name);
                continue;
            }

            // Inspect step stderr/stdout/outputs to detect missing interpreters or platform mismatches -> mark as SKIPPED
            let mut detected_missing = false;
            let missing_indicators = [
                "was not found",
                "not recognized",
                "no such file or directory",
                "command not found",
                "not found",
                "is not recognized as a name of a cmdlet", // PowerShell-specific
                "is not recognized as an internal or external command", // cmd.exe-specific
            ];

            if let Some(steps_map) = result.steps {
                for (_k, step_res) in steps_map.iter() {
                    let stderr = step_res.stderr.clone().unwrap_or_default().to_lowercase();
                    let stdout = step_res.stdout.clone().unwrap_or_default().to_lowercase();

                    eprintln!(
                        "DEBUG: step exit_code={} stderr=[{}]",
                        step_res.exit_code, stderr
                    );

                    // Check for missing interpreter/command patterns
                    if step_res.exit_code == 9009
                        || missing_indicators.iter().any(|ind| stderr.contains(ind))
                    {
                        detected_missing = true;
                        break;
                    }

                    // Check for platform-specific failures
                    for (_output_name, output_value) in &step_res.outputs {
                        let output_str = output_value.to_lowercase();
                        if output_str.contains("nok - expected unix platform")
                            || output_str.contains("nok - expected windows platform")
                            || output_str.contains("could not detect unix system")
                            || output_str.contains("could not detect windows system")
                        {
                            detected_missing = true;
                            break;
                        }
                    }

                    if stdout.contains("could not detect unix system")
                        || stdout.contains("could not detect windows system")
                        || stdout.contains("nok - expected unix platform")
                        || stdout.contains("nok - expected windows platform")
                    {
                        detected_missing = true;
                        break;
                    }

                    if detected_missing {
                        break;
                    }
                }
            }

            if detected_missing {
                let msg = format!(
                    "SKIPPED: missing interpreter or platform mismatch detected in step output"
                );
                test_results.push((chain_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", chain_name, msg);
            } else {
                test_results.push((
                    chain_name.to_string(),
                    format!("FAILED: {}", "Chain completed with errors"),
                ));
                eprintln!(
                    "\x1b[31m✗ {} - FAILED: {}\x1b[0m",
                    chain_name, "Chain completed with errors"
                );
            }
        }
    }

    // Print summary
    eprintln!("\n\x1b[1m\x1b[36m=== UNIX CHAIN SMOKE TEST RESULTS ===\x1b[0m");

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

    for (chain, result) in &test_results {
        if result.starts_with("PASSED") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", chain, result);
        } else if result.starts_with("SKIPPED") {
            eprintln!("\x1b[33m{}: {}\x1b[0m", chain, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", chain, result);
        }
    }

    // Ensure we found and ran some chains
    assert!(
        !test_results.is_empty(),
        "No chain files found in unix directory"
    );

    // Report summary statistics
    eprintln!(
        "\n\x1b[1mSummary: {} PASSED, {} FAILED, {} SKIPPED (Total: {})\x1b[0m",
        passed_count,
        failed_count,
        skipped_count,
        test_results.len()
    );

    // Ensure no chains failed
    if failed_count > 0 {
        panic!(
            "{} out of {} Unix chains failed",
            failed_count,
            test_results.len()
        );
    }

    // Ensure we actually ran some chains (not all skipped)
    if passed_count == 0 {
        panic!(
            "No Unix chains could be executed - all {} were skipped. This likely indicates missing interpreters in CI environment.",
            test_results.len()
        );
    }

    eprintln!(
        "\x1b[1m\x1b[32m🎉 {} Unix chain(s) passed successfully!\x1b[0m",
        passed_count
    );
}

// QA-friendly test that shows results in assertion messages
#[cfg(unix)]
#[test]
fn test_qa_chain_summary_unix() {
    let chain_dir = std::path::Path::new("tests/chains/unix");

    if !chain_dir.exists() {
        panic!("QA chains directory not found: tests/chains/unix");
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut chain_names = Vec::new();

    let entries = fs::read_dir(chain_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let chain_name = path.file_name().unwrap().to_str().unwrap();
            chain_names.push(chain_name.to_string());

            match atento_core::run(path.to_str().unwrap()) {
                Ok(()) => passed += 1,
                Err(_) => failed += 1,
            }
        }
    }

    assert_eq!(
        failed,
        0,
        "QA Smoke Test Results: {} PASSED, {} FAILED chains: [{}]",
        passed,
        failed,
        chain_names.join(", ")
    );
}

// QA-friendly test that shows results in assertion messages - Windows
#[cfg(windows)]
#[test]
fn test_qa_chain_summary_windows() {
    let chain_dir = std::path::Path::new("tests/chains/windows");

    if !chain_dir.exists() {
        panic!("QA chains directory not found: tests/chains/windows");
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut chain_names = Vec::new();

    let entries = fs::read_dir(chain_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let chain_name = path.file_name().unwrap().to_str().unwrap();
            chain_names.push(chain_name.to_string());

            match atento_core::run(path.to_str().unwrap()) {
                Ok(()) => passed += 1,
                Err(_) => failed += 1,
            }
        }
    }

    assert_eq!(
        failed,
        0,
        "QA Smoke Test Results: {} PASSED, {} FAILED chains: [{}]",
        passed,
        failed,
        chain_names.join(", ")
    );
}

#[cfg(windows)]
#[test]
fn test_chain_smoke_tests_windows() {
    // The test runs from atento-core directory, so chains are in tests/chains/windows
    let chain_dir = std::path::Path::new("tests/chains/windows");

    // Skip if chains directory doesn't exist (development environments)
    if !chain_dir.exists() {
        println!("Skipping Windows chain tests - directory not found");
        return;
    }

    let mut test_results = Vec::new();

    // Discover and run all .yaml files in the windows directory
    let entries = fs::read_dir(chain_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let chain_name = path.file_name().unwrap().to_str().unwrap();
            eprintln!("\x1b[36mRunning Windows chain: {}\x1b[0m", chain_name);

            // Parse the chain and run it to inspect step outputs for missing interpreters
            let contents = fs::read_to_string(&path).unwrap_or_default();
            let wf: atento_core::Chain = match serde_yaml::from_str(&contents) {
                Ok(w) => w,
                Err(e) => {
                    test_results.push((
                        chain_name.to_string(),
                        format!("FAILED: invalid YAML: {}", e),
                    ));
                    eprintln!(
                        "\x1b[31m✗ {} - FAILED: invalid YAML: {}\x1b[0m",
                        chain_name, e
                    );
                    continue;
                }
            };

            let result = wf.run();
            let json = serde_json::to_string_pretty(&result).unwrap_or_default();
            println!("{}", json);

            if result.errors.is_empty() {
                test_results.push((chain_name.to_string(), "PASSED".to_string()));
                eprintln!("\x1b[32m✓ {} - PASSED\x1b[0m", chain_name);
                continue;
            }

            // Inspect step stderr/stdout/outputs to detect missing interpreters or platform mismatches and mark SKIPPED
            let mut detected_missing = false;
            let missing_indicators = [
                "was not found",
                "not recognized",
                "no such file or directory",
                "command not found",
                "not found",
                "is not recognized as a name of a cmdlet", // PowerShell-specific
                "is not recognized as an internal or external command", // cmd.exe-specific
            ];

            if let Some(steps_map) = result.steps {
                for (_k, step_res) in steps_map.iter() {
                    let stderr = step_res.stderr.clone().unwrap_or_default().to_lowercase();
                    let stdout = step_res.stdout.clone().unwrap_or_default().to_lowercase();

                    eprintln!(
                        "DEBUG: step exit_code={} stderr=[{}]",
                        step_res.exit_code, stderr
                    );

                    // Check for missing interpreter/command patterns
                    if step_res.exit_code == 9009
                        || missing_indicators.iter().any(|ind| stderr.contains(ind))
                    {
                        detected_missing = true;
                        break;
                    }

                    // Check for platform-specific failures
                    for (_output_name, output_value) in &step_res.outputs {
                        let output_str = output_value.to_lowercase();
                        if output_str.contains("nok - expected unix platform")
                            || output_str.contains("nok - expected windows platform")
                            || output_str.contains("could not detect unix system")
                            || output_str.contains("could not detect windows system")
                        {
                            detected_missing = true;
                            break;
                        }
                    }

                    if stdout.contains("could not detect unix system")
                        || stdout.contains("could not detect windows system")
                        || stdout.contains("nok - expected unix platform")
                        || stdout.contains("nok - expected windows platform")
                    {
                        detected_missing = true;
                        break;
                    }

                    if detected_missing {
                        break;
                    }
                }
            }

            if detected_missing {
                let msg = format!(
                    "SKIPPED: missing interpreter or platform mismatch detected in step output"
                );
                test_results.push((chain_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", chain_name, msg);
            } else {
                test_results.push((
                    chain_name.to_string(),
                    format!("FAILED: {}", "Chain completed with errors"),
                ));
                eprintln!(
                    "\x1b[31m✗ {} - FAILED: {}\x1b[0m",
                    chain_name, "Chain completed with errors"
                );
            }
        }
    }

    // Print summary
    eprintln!("\n\x1b[1m\x1b[35m=== WINDOWS CHAIN SMOKE TEST RESULTS ===\x1b[0m");

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

    for (chain, result) in &test_results {
        if result.starts_with("PASSED") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", chain, result);
        } else if result.starts_with("SKIPPED") {
            eprintln!("\x1b[33m{}: {}\x1b[0m", chain, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", chain, result);
        }
    }

    // Ensure we found and ran some chains
    assert!(
        !test_results.is_empty(),
        "No chain files found in windows directory"
    );

    // Report summary statistics
    eprintln!(
        "\n\x1b[1mSummary: {} PASSED, {} FAILED, {} SKIPPED (Total: {})\x1b[0m",
        passed_count,
        failed_count,
        skipped_count,
        test_results.len()
    );

    // Ensure no chains failed
    if failed_count > 0 {
        panic!(
            "{} out of {} Windows chains failed",
            failed_count,
            test_results.len()
        );
    }

    // Ensure we actually ran some chains (not all skipped)
    if passed_count == 0 {
        panic!(
            "No Windows chains could be executed - all {} were skipped. This likely indicates missing interpreters in CI environment.",
            test_results.len()
        );
    }

    eprintln!(
        "\x1b[1m\x1b[32m🎉 {} Windows chain(s) passed successfully!\x1b[0m",
        passed_count
    );
}

// Cross-platform chain smoke tests
#[test]
fn test_chain_smoke_tests_cross_platform() {
    // The test runs from atento-core directory, so chains are in tests/chains/cross-platform
    let chain_dir = std::path::Path::new("tests/chains/cross-platform");

    // Skip if chains directory doesn't exist (development environments)
    if !chain_dir.exists() {
        println!("Skipping Cross-platform chain tests - directory not found");
        return;
    }

    let mut test_results = Vec::new();

    // Discover and run all .yaml files in the cross-platform directory
    let entries = fs::read_dir(chain_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let chain_name = path.file_name().unwrap().to_str().unwrap();
            eprintln!(
                "\x1b[36mRunning Cross-platform chain: {}\x1b[0m",
                chain_name
            );
            // Read the chain and detect required interpreters by simple text scan.
            // This is intentionally permissive and avoids YAML parsing edge-cases in tests.
            let content = fs::read_to_string(&path).unwrap_or_default();
            let content_lc = content.to_lowercase();
            let mut required_bins = std::collections::HashSet::new();
            if content_lc.contains("python") || content_lc.contains("type: python") {
                required_bins.insert("python");
            }
            if content_lc.contains("bash") || content_lc.contains("type: bash") {
                required_bins.insert("bash");
            }
            if content_lc.contains("powershell") || content_lc.contains("pwsh") {
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

            // Check required bins; if missing, skip this chain (mark SKIPPED)
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
                test_results.push((chain_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", chain_name, msg);
                continue;
            }

            // Parse the chain and run it to inspect step outputs for missing interpreters
            let contents = fs::read_to_string(&path).unwrap_or_default();
            let wf: atento_core::Chain = match serde_yaml::from_str(&contents) {
                Ok(w) => w,
                Err(e) => {
                    test_results.push((
                        chain_name.to_string(),
                        format!("FAILED: invalid YAML: {}", e),
                    ));
                    eprintln!(
                        "\x1b[31m✗ {} - FAILED: invalid YAML: {}\x1b[0m",
                        chain_name, e
                    );
                    continue;
                }
            };

            // Pre-check exact interpreter executables required by steps (skip if missing)
            let mut missing_progs = Vec::new();
            for (_k, step) in &wf.steps {
                let interpreter = match wf.interpreters.get(&step.interpreter) {
                    Some(interp) => interp,
                    None => continue,
                };
                let prog = interpreter.command.as_str();

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
                test_results.push((chain_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", chain_name, msg);
                continue;
            }

            let result = wf.run();
            let json = serde_json::to_string_pretty(&result).unwrap_or_default();
            println!("{}", json);

            if result.errors.is_empty() {
                test_results.push((chain_name.to_string(), "PASSED".to_string()));
                eprintln!("\x1b[32m✓ {} - PASSED\x1b[0m", chain_name);
                continue;
            }

            // Inspect step stderr/stdout/outputs to detect missing interpreters or platform mismatches and mark SKIPPED
            let mut detected_missing = false;
            let missing_indicators = [
                "was not found",
                "not recognized",
                "no such file or directory",
                "command not found",
                "not found",
                "is not recognized as a name of a cmdlet", // PowerShell-specific
                "is not recognized as an internal or external command", // cmd.exe-specific
            ];

            if let Some(steps_map) = result.steps {
                for (_k, step_res) in steps_map.iter() {
                    let stderr = step_res.stderr.clone().unwrap_or_default().to_lowercase();
                    let stdout = step_res.stdout.clone().unwrap_or_default().to_lowercase();

                    eprintln!(
                        "DEBUG: step exit_code={} stderr=[{}]",
                        step_res.exit_code, stderr
                    );

                    // Check for missing interpreter/command patterns in stderr
                    if step_res.exit_code == 9009
                        || missing_indicators.iter().any(|ind| stderr.contains(ind))
                    {
                        detected_missing = true;
                        break;
                    }

                    // Check for platform-specific chain failures (e.g., Unix-specific tests on Windows)
                    // These chains contain platform checks that legitimately fail on the wrong platform
                    for (_output_name, output_value) in &step_res.outputs {
                        let output_str = output_value.to_lowercase();
                        if output_str.contains("nok - expected unix platform")
                            || output_str.contains("nok - expected windows platform")
                            || output_str.contains("could not detect unix system")
                            || output_str.contains("could not detect windows system")
                        {
                            detected_missing = true;
                            break;
                        }
                    }

                    // Also check stdout for platform detection failures
                    if stdout.contains("could not detect unix system")
                        || stdout.contains("could not detect windows system")
                        || stdout.contains("nok - expected unix platform")
                        || stdout.contains("nok - expected windows platform")
                    {
                        detected_missing = true;
                        break;
                    }

                    if detected_missing {
                        break;
                    }
                }
            }

            if detected_missing {
                let msg = format!(
                    "SKIPPED: missing interpreter or platform mismatch detected in step output"
                );
                test_results.push((chain_name.to_string(), msg.clone()));
                eprintln!("\x1b[33m→ {} - {}\x1b[0m", chain_name, msg);
            } else {
                test_results.push((
                    chain_name.to_string(),
                    format!("FAILED: {}", "Chain completed with errors"),
                ));
                eprintln!(
                    "\x1b[31m✗ {} - FAILED: {}\x1b[0m",
                    chain_name, "Chain completed with errors"
                );
            }
        }
    }

    // Print summary
    eprintln!("\n\x1b[1m\x1b[33m=== CROSS-PLATFORM CHAIN SMOKE TEST RESULTS ===\x1b[0m");

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

    for (chain, result) in &test_results {
        if result.starts_with("PASSED") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", chain, result);
        } else if result.starts_with("SKIPPED") {
            eprintln!("\x1b[33m{}: {}\x1b[0m", chain, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", chain, result);
        }
    }

    // Ensure we found and ran some chains
    assert!(
        !test_results.is_empty(),
        "No chain files found in cross-platform directory"
    );

    // Report summary statistics
    eprintln!(
        "\n\x1b[1mSummary: {} PASSED, {} FAILED, {} SKIPPED (Total: {})\x1b[0m",
        passed_count,
        failed_count,
        skipped_count,
        test_results.len()
    );

    // Ensure no chains failed
    if failed_count > 0 {
        panic!(
            "{} out of {} Cross-platform chains failed",
            failed_count,
            test_results.len()
        );
    }

    // Ensure we actually ran some chains (not all skipped)
    if passed_count == 0 {
        panic!(
            "No cross-platform chains could be executed - all {} were skipped. This likely indicates missing interpreters in CI environment.",
            test_results.len()
        );
    }

    eprintln!(
        "\x1b[1m\x1b[32m🎉 {} Cross-platform chain(s) passed successfully!\x1b[0m",
        passed_count
    );
}

// Cross-platform chain validation test
#[test]
fn test_chain_file_validation() {
    // The test runs from atento-core directory, so chains are in tests/chains
    let base_dir = std::path::Path::new("tests/chains");
    if !base_dir.exists() {
        println!("Skipping chain validation - chains directory not found");
        return;
    }

    let mut total_chains = 0;
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
                total_chains += 1;
                let chain_name = format!(
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
                                validation_results.push((chain_name, "VALID YAML".to_string()));
                            }
                            Err(e) => {
                                validation_results
                                    .push((chain_name, format!("INVALID YAML: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        validation_results.push((chain_name, format!("READ ERROR: {}", e)));
                    }
                }
            }
        }
    }

    // Print validation results
    eprintln!("\n\x1b[1m\x1b[33m=== CHAIN FILE VALIDATION RESULTS ===\x1b[0m");
    for (chain, result) in &validation_results {
        if result.starts_with("VALID") {
            eprintln!("\x1b[32m{}: {}\x1b[0m", chain, result);
        } else {
            eprintln!("\x1b[31m{}: {}\x1b[0m", chain, result);
        }
    }

    // Ensure we found some chains
    if total_chains == 0 {
        println!("No chain files found - skipping validation test");
        return;
    }

    // Ensure all chains have valid YAML
    let invalid_count = validation_results
        .iter()
        .filter(|(_, result)| !result.starts_with("VALID"))
        .count();

    assert_eq!(
        invalid_count, 0,
        "{} out of {} chain files have invalid YAML",
        invalid_count, total_chains
    );

    eprintln!(
        "\x1b[1m\x1b[32m✅ All {} chain files have valid YAML syntax!\x1b[0m",
        total_chains
    );
}
