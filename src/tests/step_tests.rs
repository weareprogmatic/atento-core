#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use crate::data_type::DataType;
    use crate::errors::AtentoError;
    use crate::input::Input;
    use crate::interpreter::Interpreter;
    use crate::output::Output;
    use crate::step::Step;
    use std::collections::HashMap;

    // Step-specific tests

    #[test]
    fn test_step_deserialize() {
        let yaml = r#"
name: build
timeout: 120
type: script::python
script: print("test")
"#;
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.name.as_deref(), Some("build"));
        assert_eq!(step.timeout, 120);
        assert!(matches!(step.interpreter, Interpreter::Python));
        assert_eq!(step.script, "print(\"test\")");
    }

    #[test]
    fn test_step_deserialize_defaults() {
        let yaml = r"
type: script::bash
script: |
    echo test
";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.name, None);
        assert_eq!(step.timeout, 60);
        assert!(matches!(step.interpreter, Interpreter::Bash));
        assert!(step.inputs.is_empty());
        assert!(step.outputs.is_empty());
    }

    #[test]
    fn test_step_result_serialize() {
        use crate::step::StepResult;

        let mut result = StepResult {
            name: Some("test".to_string()),
            duration_ms: 100,
            exit_code: 0,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            stdout: Some("output".to_string()),
            stderr: None,
            error: None,
        };
        result
            .outputs
            .insert("key".to_string(), "value".to_string());

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("100"));
        assert!(json.contains("output"));
    }

    #[test]
    fn test_step_result_skip_empty_maps() {
        use crate::step::StepResult;

        let result = StepResult {
            name: None,
            duration_ms: 50,
            exit_code: 0,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            stdout: None,
            stderr: None,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        // Empty maps and None options should be skipped
        assert!(!json.contains("inputs"));
        assert!(!json.contains("outputs"));
        assert!(!json.contains("stdout"));
        assert!(!json.contains("stderr"));
    }

    #[test]
    fn test_step_validate_empty_script() {
        let step = Step {
            interpreter: Interpreter::Bash,
            script: String::new(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_step_validate_undeclared_input() {
        let step = Step {
            interpreter: Interpreter::Bash,
            script: "echo {{ inputs.foo }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("references input 'foo'"));
            assert!(msg.contains("not declared"));
        }
    }

    #[test]
    fn test_step_validate_unused_input() {
        let mut step = Step {
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.script = "echo hello".to_string();
        step.inputs.insert(
            "unused".to_string(),
            Input::Inline {
                type_: DataType::String,
                value: serde_yaml::Value::String("value".to_string()),
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("never used in the script"));
        }
    }

    #[test]
    fn test_step_validate_valid_input() {
        let mut step = Step {
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.script = "echo {{ inputs.name }}".to_string();
        step.inputs.insert(
            "name".to_string(),
            Input::Inline {
                type_: DataType::String,
                value: serde_yaml::Value::String("test".to_string()),
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_step_validate_empty_output_pattern() {
        let mut step = Step {
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: String::new(),
                type_: DataType::String,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("empty capture pattern"));
        }
    }

    #[test]
    fn test_step_validate_whitespace_output_pattern() {
        let mut step = Step {
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: "   ".to_string(),
                type_: DataType::String,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("empty capture pattern"));
        }
    }

    #[test]
    fn test_step_validate_invalid_regex_pattern() {
        let mut step = Step {
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: "[invalid".to_string(),
                type_: DataType::String,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("invalid regex pattern"));
        }
    }

    #[test]
    fn test_step_validate_valid_regex_pattern() {
        let mut step = Step {
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: r"(\d+)".to_string(),
                type_: DataType::String,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_step_validate_with_step_name() {
        let mut step = Step {
            name: Some("my_step".to_string()),
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.script = "echo hello".to_string();
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_step_validate_without_step_name() {
        let step = Step {
            interpreter: Interpreter::Bash,
            script: "echo hello".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_step_deserialize_with_interpreter() {
        let yaml = r#"
type: script::python
script: print("hello")
"#;
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(step.interpreter, Interpreter::Python));
        assert_eq!(step.script, "print(\"hello\")");
    }

    #[test]
    fn test_step_default_interpreter_is_bash() {
        let step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        assert!(matches!(step.interpreter, Interpreter::Bash));
    }

    #[test]
    fn test_step_requires_type_field() {
        let yaml = r"
script: echo hello
";
        let result: Result<Step, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod unit_tests {
    use crate::data_type::DataType;
    use crate::errors::AtentoError;
    use crate::executor::ExecutionResult;
    use crate::input::Input;
    use crate::interpreter::Interpreter;
    use crate::output::Output;
    use crate::step::Step;
    use crate::tests::mock_executor::MockExecutor;
    use std::collections::HashMap;

    // Test the Step struct field operations (no I/O) - Pure unit tests

    #[test]
    fn test_step_default() {
        let step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        assert!(step.name.is_none());
        assert_eq!(step.timeout, 60);
        assert!(step.inputs.is_empty());
        assert!(step.outputs.is_empty());
        assert_eq!(step.script, "");
        assert!(matches!(step.interpreter, Interpreter::Bash));
    }

    #[test]
    fn test_step_deserialize_from_yaml() {
        let yaml = r#"
name: test_step
timeout: 120
type: script::python
script: print("hello")
"#;
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.name.as_deref(), Some("test_step"));
        assert_eq!(step.timeout, 120);
        assert!(matches!(step.interpreter, Interpreter::Python));
        assert_eq!(step.script, "print(\"hello\")");
    }

    #[test]
    fn test_step_deserialize_with_defaults() {
        let yaml = r"
type: script::bash
script: echo hello
";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert!(step.name.is_none());
        assert_eq!(step.timeout, 60); // default
        assert!(step.inputs.is_empty());
        assert!(step.outputs.is_empty());
    }

    // Test timeout calculation logic (pure unit tests)

    #[test]
    fn test_calculate_timeout_both_positive() {
        let step = Step {
            timeout: 30,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        assert_eq!(step.calculate_timeout(60), 30); // min(30, 60)
    }

    #[test]
    fn test_calculate_timeout_step_zero() {
        let step = Step {
            timeout: 0,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        assert_eq!(step.calculate_timeout(60), 60); // max(0, 60)
    }

    #[test]
    fn test_calculate_timeout_time_left_zero() {
        let step = Step {
            timeout: 30,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        assert_eq!(step.calculate_timeout(0), 30); // max(30, 0)
    }

    #[test]
    fn test_calculate_timeout_both_zero() {
        let step = Step {
            timeout: 0,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        assert_eq!(step.calculate_timeout(0), 0); // max(0, 0)
    }

    #[test]
    fn test_calculate_timeout_equal_values() {
        let step = Step {
            timeout: 45,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        assert_eq!(step.calculate_timeout(45), 45); // min(45, 45)
    }

    // Test script building logic (template substitution)

    #[test]
    fn test_build_script_no_placeholders() {
        let step = Step {
            script: "echo hello world".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let inputs = HashMap::new();
        let result = step.build_script(&inputs);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_build_script_empty_script() {
        let step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        let inputs = HashMap::new();
        let result = step.build_script(&inputs);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_script_single_placeholder() {
        let step = Step {
            script: "echo {{ inputs.message }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let mut inputs = HashMap::new();
        inputs.insert("message".to_string(), "hello world".to_string());
        let result = step.build_script(&inputs);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_build_script_multiple_placeholders() {
        let step = Step {
            script: "echo {{ inputs.greeting }} {{ inputs.name }}!".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let mut inputs = HashMap::new();
        inputs.insert("greeting".to_string(), "Hello".to_string());
        inputs.insert("name".to_string(), "World".to_string());
        let result = step.build_script(&inputs);
        assert_eq!(result, "echo Hello World!");
    }

    #[test]
    fn test_build_script_same_placeholder_multiple_times() {
        let step = Step {
            script: "echo {{ inputs.word }} and {{ inputs.word }} again".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let mut inputs = HashMap::new();
        inputs.insert("word".to_string(), "test".to_string());
        let result = step.build_script(&inputs);
        assert_eq!(result, "echo test and test again");
    }

    #[test]
    fn test_build_script_placeholder_with_whitespace() {
        let step = Step {
            script: "echo {{  inputs.message  }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let mut inputs = HashMap::new();
        inputs.insert("message".to_string(), "spaced".to_string());
        let result = step.build_script(&inputs);
        assert_eq!(result, "echo spaced");
    }

    #[test]
    fn test_build_script_missing_input_keeps_placeholder() {
        let step = Step {
            script: "echo {{ inputs.missing }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let inputs = HashMap::new();
        let result = step.build_script(&inputs);
        assert_eq!(result, "echo {{ inputs.missing }}");
    }

    #[test]
    fn test_build_script_complex_mixed_placeholders() {
        let step = Step {
            script: "cp {{ inputs.source }} {{ inputs.dest }}/{{ inputs.filename }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let mut inputs = HashMap::new();
        inputs.insert("source".to_string(), "/tmp/file.txt".to_string());
        inputs.insert("dest".to_string(), "/home/user".to_string());
        inputs.insert("filename".to_string(), "newfile.txt".to_string());
        let result = step.build_script(&inputs);
        assert_eq!(result, "cp /tmp/file.txt /home/user/newfile.txt");
    }

    // Test validation logic (pure unit tests)

    #[test]
    fn test_validate_empty_script_passes() {
        let step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_script_without_inputs_passes() {
        let step = Step {
            script: "echo hello".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_undeclared_input_fails() {
        let step = Step {
            script: "echo {{ inputs.missing }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("references input 'missing'"));
            assert!(msg.contains("not declared"));
        }
    }

    #[test]
    fn test_validate_unused_input_fails() {
        let mut step = Step {
            script: "echo hello".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.inputs.insert(
            "unused".to_string(),
            Input::Inline {
                type_: DataType::String,
                value: serde_yaml::Value::String("value".to_string()),
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("never used in the script"));
        }
    }

    #[test]
    fn test_validate_declared_and_used_input_passes() {
        let mut step = Step {
            script: "echo {{ inputs.message }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.inputs.insert(
            "message".to_string(),
            Input::Inline {
                type_: DataType::String,
                value: serde_yaml::Value::String("test".to_string()),
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_output_pattern_fails() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: String::new(),
                type_: DataType::String,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("empty capture pattern"));
        }
    }

    #[test]
    fn test_validate_whitespace_output_pattern_fails() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: "   ".to_string(),
                type_: DataType::String,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("empty capture pattern"));
        }
    }

    #[test]
    fn test_validate_invalid_regex_pattern_fails() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: "[invalid".to_string(),
                type_: DataType::String,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("invalid regex pattern"));
        }
    }

    #[test]
    fn test_validate_valid_regex_pattern_passes() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: r"Result: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );
        let result = step.validate("test_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_uses_step_name_in_error() {
        let step = Step {
            name: Some("my_custom_step".to_string()),
            script: "echo {{ inputs.missing }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("my_custom_step"));
            assert!(!msg.contains("test_id"));
        }
    }

    #[test]
    fn test_validate_uses_id_when_no_name() {
        let step = Step {
            script: "echo {{ inputs.missing }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let result = step.validate("test_id");
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("test_id"));
        }
    }

    // Test output extraction logic (pure unit tests)

    #[test]
    fn test_extract_outputs_no_outputs_defined() {
        let step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        let mut stdout = "some output".to_string();
        let result = step.extract_outputs(&mut stdout).unwrap();
        assert!(result.is_empty());
        assert_eq!(stdout, "some output"); // unchanged
    }

    #[test]
    fn test_extract_outputs_successful_match() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: r"Result: (\w+)".to_string(),
                type_: DataType::String,
            },
        );

        let mut stdout = "Processing...\nResult: success\nDone.".to_string();
        let result = step.extract_outputs(&mut stdout).unwrap();

        assert_eq!(result.get("result").unwrap(), "success");
        assert_eq!(stdout, "Processing...\n\nDone."); // matched portion removed
    }

    #[test]
    fn test_extract_outputs_no_match_fails() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: r"Result: (\w+)".to_string(),
                type_: DataType::String,
            },
        );

        let mut stdout = "No match here".to_string();
        let result = step.extract_outputs(&mut stdout);

        assert!(result.is_err());
        if let Err(AtentoError::Execution(msg)) = result {
            assert!(msg.contains("did not match stdout"));
        }
    }

    #[test]
    fn test_extract_outputs_no_capture_group_fails() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: r"Result: \w+".to_string(), // No capture group
                type_: DataType::String,
            },
        );

        let mut stdout = "Result: success".to_string();
        let result = step.extract_outputs(&mut stdout);

        assert!(result.is_err());
        if let Err(AtentoError::Execution(msg)) = result {
            assert!(msg.contains("did not capture a group"));
        }
    }

    #[test]
    fn test_extract_outputs_multiple_outputs() {
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: Interpreter::Bash,
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.outputs.insert(
            "name".to_string(),
            Output {
                pattern: r"Name: (\w+)".to_string(),
                type_: DataType::String,
            },
        );
        step.outputs.insert(
            "age".to_string(),
            Output {
                pattern: r"Age: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );

        let mut stdout = "Name: John\nAge: 25\nOther info".to_string();
        let result = step.extract_outputs(&mut stdout).unwrap();

        assert_eq!(result.get("name").unwrap(), "John");
        assert_eq!(result.get("age").unwrap(), "25");
        assert_eq!(stdout, "\n\nOther info"); // Both matches removed
    }

    // Test complete step execution with mock executor

    #[test]
    fn test_run_with_mock_executor_simple() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "echo hello",
            ExecutionResult {
                stdout: "hello\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 5,
            },
        );

        let step = Step {
            script: "echo hello".to_string(),
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let inputs = HashMap::new();
        let result = step.run(&mock, &inputs, 60);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.as_deref(), Some("hello"));
        assert!(result.stderr.is_none());
        assert_eq!(mock.call_count(), 1);
    }

    #[test]
    fn test_run_with_mock_executor_input_substitution() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "echo world",
            ExecutionResult {
                stdout: "world\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 8,
            },
        );

        let step = Step {
            script: "echo {{ inputs.message }}".to_string(),
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let mut inputs = HashMap::new();
        inputs.insert("message".to_string(), "world".to_string());
        let result = step.run(&mock, &inputs, 60);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.as_deref(), Some("world"));

        // Verify the mock was called with the substituted script
        let (script, ext, args, timeout) = mock.last_call().unwrap();
        assert_eq!(script, "echo world");
        assert_eq!(ext, ".sh");
        assert_eq!(args, vec!["bash"]);
        assert_eq!(timeout, 60);
    }

    #[test]
    fn test_run_with_mock_executor_timeout_handling() {
        let mut mock = MockExecutor::new();
        mock.expect_timeout("sleep 10");

        let step = Step {
            script: "sleep 10".to_string(),
            timeout: 5,
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let inputs = HashMap::new();
        let result = step.run(&mock, &inputs, 60);

        // The mock should return the timeout error based on our expectation
        assert_eq!(result.exit_code, 124); // Timeout exit code
        assert_eq!(result.stderr.as_deref(), Some("Timeout"));
    }

    #[test]
    fn test_run_with_mock_executor_output_extraction() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "echo 'Result: 42'",
            ExecutionResult {
                stdout: "Result: 42\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 3,
            },
        );

        let mut step = Step {
            script: "echo 'Result: 42'".to_string(),
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "value".to_string(),
            Output {
                pattern: r"Result: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );

        let inputs = HashMap::new();
        let result = step.run(&mock, &inputs, 60);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.outputs.get("value").unwrap(), "42");
        // The matched portion should be removed from stdout, empty stdout becomes None
        assert_eq!(result.stdout.as_deref(), None);
    }

    #[test]
    fn test_run_with_mock_executor_error_handling() {
        let mut mock = MockExecutor::new();
        mock.expect_error("exit 1", 1, "command failed");

        let step = Step {
            script: "exit 1".to_string(),
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let inputs = HashMap::new();
        let result = step.run(&mock, &inputs, 60);

        assert_eq!(result.exit_code, 1);
        assert_eq!(result.stderr.as_deref(), Some("command failed"));
    }

    #[test]
    fn test_run_with_mock_executor_uses_correct_interpreter() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "print('hello')",
            ExecutionResult {
                stdout: "hello\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 15,
            },
        );

        let step = Step {
            script: "print('hello')".to_string(),
            interpreter: Interpreter::Python,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let inputs = HashMap::new();
        let result = step.run(&mock, &inputs, 60);

        assert_eq!(result.exit_code, 0);

        // Verify correct interpreter was used
        let (_, ext, args, _) = mock.last_call().unwrap();
        assert_eq!(ext, ".py");
        assert_eq!(args, vec!["python3"]);
    }

    #[test]
    fn test_run_with_mock_executor_complex_scenario() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "echo 'Name: Alice' && echo 'Age: 30'",
            ExecutionResult {
                stdout: "Name: Alice\nAge: 30\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 12,
            },
        );

        let mut step = Step {
            script: "echo 'Name: {{ inputs.name }}' && echo 'Age: {{ inputs.age }}'".to_string(),
            interpreter: Interpreter::Bash,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "person_name".to_string(),
            Output {
                pattern: r"Name: (\w+)".to_string(),
                type_: DataType::String,
            },
        );
        step.outputs.insert(
            "person_age".to_string(),
            Output {
                pattern: r"Age: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );

        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), "Alice".to_string());
        inputs.insert("age".to_string(), "30".to_string());

        let result = step.run(&mock, &inputs, 60);

        assert_eq!(result.exit_code, 0);
        assert_eq!(
            result
                .outputs
                .get("person_name")
                .expect("person_name should be in outputs"),
            "Alice"
        );
        assert_eq!(
            result
                .outputs
                .get("person_age")
                .expect("person_age should be in outputs"),
            "30"
        );
        assert_eq!(
            result.inputs.get("name").expect("name should be in inputs"),
            "Alice"
        );
        assert_eq!(
            result.inputs.get("age").expect("age should be in inputs"),
            "30"
        );
        // Both extracted patterns should be removed from stdout, empty stdout becomes None
        assert_eq!(result.stdout.as_deref(), None);
    }

    #[test]
    fn test_step_run_system_executor() {
        // Test the run() method that uses SystemExecutor internally
        let step = Step {
            name: Some("system_test".to_string()),
            interpreter: Interpreter::Bash,
            script: "echo 'test output'".to_string(),
            timeout: 30,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let inputs = HashMap::new();
        let executor = crate::executor::SystemExecutor;
        let result = step.run(&executor, &inputs, 60);

        // Should succeed - step.run() now returns StepResult directly
        assert_eq!(result.name, Some("system_test".to_string()));
        // Duration should be non-negative
    }

    #[test]
    fn test_step_stdout_stderr_filtering() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "echo test",
            ExecutionResult {
                stdout: "  test  ".to_string(),
                stderr: "  error  ".to_string(),
                exit_code: 0,
                duration_ms: 10,
            },
        );

        let step = Step {
            name: Some("filter_test".to_string()),
            interpreter: Interpreter::Bash,
            script: "echo test".to_string(),
            timeout: 30,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let result = step.run(&mock, &HashMap::new(), 60);

        // Should trim whitespace from stdout and stderr
        assert_eq!(result.stdout, Some("test".to_string()));
        assert_eq!(result.stderr, Some("  error  ".to_string())); // Note: stderr is NOT trimmed by the Step
    }

    #[test]
    fn test_step_empty_stdout_stderr_filtering() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "echo",
            ExecutionResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 5,
            },
        );

        let step = Step {
            name: Some("empty_test".to_string()),
            interpreter: Interpreter::Bash,
            script: "echo".to_string(),
            timeout: 30,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let result = step.run(&mock, &HashMap::new(), 60);

        // Empty strings should be filtered to None
        assert_eq!(result.stdout, None);
        assert_eq!(result.stderr, None);
    }

    #[test]
    fn test_step_args_conversion() {
        let mut mock = MockExecutor::new();
        mock.expect_call(
            "print('test')",
            ExecutionResult {
                stdout: "test".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 8,
            },
        );

        let step = Step {
            name: Some("args_test".to_string()),
            interpreter: Interpreter::Python,
            script: "print('test')".to_string(),
            timeout: 30,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: Interpreter::Bash,
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        let _result = step.run(&mock, &HashMap::new(), 60);

        // Verify that Python interpreter args were properly converted
        let (_, ext, args, _) = mock.last_call().unwrap();
        assert_eq!(ext, ".py");
        assert_eq!(args, vec!["python3"]); // Note: MockExecutor may not include all args
    }
}
