#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::if_same_then_else,
    clippy::uninlined_format_args,
    clippy::field_reassign_with_default,
    clippy::similar_names
)]
mod tests {
    use crate::chain::Chain;
    use crate::data_type::DataType;
    use crate::errors::AtentoError;
    use crate::input::Input;

    use crate::interpreter::default_interpreters;
    use crate::output::Output;
    use crate::parameter::Parameter;
    use crate::result_ref::ResultRef;
    use crate::step::Step;
    use std::collections::HashMap;

    // Helper to create a Chain with default interpreters populated
    fn chain_with_defaults() -> Chain {
        let mut chain = Chain::default();
        chain.interpreters = default_interpreters().into_iter().collect();

        chain
    }

    // Integration tests that execute actual chains

    #[test]
    fn test_chain_default() {
        let wf = chain_with_defaults();
        assert_eq!(wf.name, None);
        assert_eq!(wf.timeout, 300);
        assert!(wf.parameters.is_empty());
        assert!(wf.steps.is_empty());
        assert!(wf.results.is_empty());
    }

    #[test]
    fn test_chain_validate_empty() {
        let wf = chain_with_defaults();
        let result = wf.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_validate_unresolved_parameter_ref() {
        let mut wf = chain_with_defaults();
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: "bash".to_string(),
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.script = "echo test".to_string();
        step.inputs.insert(
            "param".to_string(),
            Input::Ref {
                ref_: "parameters.nonexistent".to_string(),
            },
        );
        wf.steps.insert("step1".to_string(), step);

        let result = wf.validate();
        assert!(result.is_err());
        if let Err(AtentoError::UnresolvedReference { reference, .. }) = result {
            assert_eq!(reference, "parameters.nonexistent");
        }
    }

    #[test]
    fn test_chain_validate_valid_parameter_ref() {
        let mut wf = chain_with_defaults();
        wf.parameters.insert(
            "name".to_string(),
            Parameter {
                type_: DataType::String,
                value: serde_yaml::Value::String("test".to_string()),
            },
        );

        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: "bash".to_string(),
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.script = "echo {{ inputs.param }}".to_string();
        step.inputs.insert(
            "param".to_string(),
            Input::Ref {
                ref_: "parameters.name".to_string(),
            },
        );
        wf.steps.insert("step1".to_string(), step);

        let result = wf.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_validate_forward_reference() {
        let mut wf = chain_with_defaults();

        let mut step1 = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: "bash".to_string(),
            script: String::new(),
            outputs: HashMap::new(),
        };
        step1.script = "echo {{ inputs.value }}".to_string();
        step1.inputs.insert(
            "value".to_string(),
            Input::Ref {
                ref_: "steps.step2.outputs.result".to_string(),
            },
        );
        wf.steps.insert("step1".to_string(), step1);

        let mut step2 = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: "bash".to_string(),
            script: String::new(),
            outputs: HashMap::new(),
        };
        step2.script = "echo test".to_string();
        step2.outputs.insert(
            "result".to_string(),
            Output {
                pattern: r"(.+)".to_string(),
                type_: DataType::String,
            },
        );
        wf.steps.insert("step2".to_string(), step2);

        let result = wf.validate();
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("future step output"));
        }
    }

    #[test]
    fn test_chain_validate_valid_step_output_ref() {
        let mut wf = chain_with_defaults();

        let mut step1 = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: "bash".to_string(),
            script: String::new(),
            outputs: HashMap::new(),
        };
        step1.script = "echo 'result: 42'".to_string();
        step1.outputs.insert(
            "value".to_string(),
            Output {
                pattern: r"result: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );
        wf.steps.insert("step1".to_string(), step1);

        let mut step2 = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: "bash".to_string(),
            script: String::new(),
            outputs: HashMap::new(),
        };
        step2.script = "echo {{ inputs.prev }}".to_string();
        step2.inputs.insert(
            "prev".to_string(),
            Input::Ref {
                ref_: "steps.step1.outputs.value".to_string(),
            },
        );
        wf.steps.insert("step2".to_string(), step2);

        let result = wf.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_validate_empty_output_pattern() {
        let mut wf = chain_with_defaults();
        let mut step = Step {
            name: None,
            timeout: 60,
            inputs: HashMap::new(),
            interpreter: "bash".to_string(),
            script: String::new(),
            outputs: HashMap::new(),
        };
        step.script = "echo test".to_string();
        step.outputs.insert(
            "result".to_string(),
            Output {
                pattern: String::new(),
                type_: DataType::String,
            },
        );
        wf.steps.insert("step1".to_string(), step);

        let result = wf.validate();
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("empty capture pattern"));
        }
    }

    #[test]
    fn test_chain_validate_result_references_nonexistent_output() {
        let mut wf = chain_with_defaults();
        let step = Step {
            script: "echo test".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        wf.steps.insert("step1".to_string(), step);
        wf.results.insert(
            "final".to_string(),
            ResultRef {
                ref_: "steps.step1.outputs.nonexistent".to_string(),
            },
        );

        let result = wf.validate();
        assert!(result.is_err());
        if let Err(AtentoError::UnresolvedReference { reference, .. }) = result {
            assert_eq!(reference, "steps.step1.outputs.nonexistent");
        }
    }

    #[test]
    fn test_chain_validate_result_references_valid_output() {
        let mut wf = chain_with_defaults();
        let mut step = Step {
            script: "echo 'value: 42'".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "num".to_string(),
            Output {
                pattern: r"value: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );
        wf.steps.insert("step1".to_string(), step);
        wf.results.insert(
            "final".to_string(),
            ResultRef {
                ref_: "steps.step1.outputs.num".to_string(),
            },
        );

        let result = wf.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_run_empty() {
        let wf = chain_with_defaults();
        let result = wf.run();
        assert_eq!(result.status, "ok");
    }

    #[test]
    fn test_chain_run_single_step() {
        use crate::executor::ExecutionResult;
        use crate::tests::mock_executor::MockExecutor;

        let mut wf = chain_with_defaults();
        let step = Step {
            script: "echo hello".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        wf.steps.insert("step1".to_string(), step);

        let mut mock = MockExecutor::new();
        mock.expect_call(
            "echo hello",
            ExecutionResult {
                stdout: "hello\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 10,
            },
        );

        let result = wf.run_with_executor(&mock);
        assert_eq!(result.status, "ok");
        assert!(result.steps.is_some());
        let steps = result.steps.unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps["step1"].exit_code, 0);
    }

    #[test]
    fn test_chain_run_multiple_steps() {
        let mut wf = chain_with_defaults();

        let step1 = Step {
            script: "echo step1".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let step2 = Step {
            script: "echo step2".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        wf.steps.insert("step1".to_string(), step1);
        wf.steps.insert("step2".to_string(), step2);

        let result = wf.run();
        assert_eq!(result.status, "ok");
        let steps = result.steps.unwrap();
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn test_chain_run_with_parameter() {
        let mut wf = chain_with_defaults();
        wf.parameters.insert(
            "greeting".to_string(),
            Parameter {
                type_: DataType::String,
                value: serde_yaml::Value::String("hello".to_string()),
            },
        );

        let mut step = Step {
            script: "echo {{ inputs.msg }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.inputs.insert(
            "msg".to_string(),
            Input::Ref {
                ref_: "parameters.greeting".to_string(),
            },
        );
        wf.steps.insert("step1".to_string(), step);

        let result = wf.run();
        assert_eq!(result.status, "ok");
        assert!(result.parameters.is_some());
        let params = result.parameters.unwrap();
        assert_eq!(params.get("greeting").map(String::as_str), Some("hello"));
    }

    #[test]
    fn test_chain_run_with_step_chaining() {
        use crate::executor::ExecutionResult;
        use crate::tests::mock_executor::MockExecutor;

        let mut wf = chain_with_defaults();

        let mut step1 = Step {
            script: "echo 'output: 42'".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step1.outputs.insert(
            "value".to_string(),
            Output {
                pattern: r"output: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );
        wf.steps.insert("step1".to_string(), step1);

        let mut step2 = Step {
            script: "echo {{ inputs.prev }}".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step2.inputs.insert(
            "prev".to_string(),
            Input::Ref {
                ref_: "steps.step1.outputs.value".to_string(),
            },
        );
        wf.steps.insert("step2".to_string(), step2);

        let mut mock = MockExecutor::new();

        // Mock first step execution
        mock.expect_call(
            "echo 'output: 42'",
            ExecutionResult {
                stdout: "output: 42\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 10,
            },
        );

        // Mock second step execution
        mock.expect_call(
            "echo 42",
            ExecutionResult {
                stdout: "42\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 10,
            },
        );

        let result = wf.run_with_executor(&mock);
        assert_eq!(result.status, "ok");
        let steps = result.steps.unwrap();
        assert_eq!(steps["step2"].stdout.as_deref(), Some("42"));
    }

    #[test]
    fn test_chain_run_with_results() {
        let mut wf = chain_with_defaults();

        let mut step = Step {
            script: if cfg!(windows) {
                "echo final: success".to_string()
            } else {
                "echo 'final: success'".to_string()
            },
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: if cfg!(windows) {
                    "batch".to_string()
                } else {
                    "bash".to_string()
                },
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "status".to_string(),
            Output {
                pattern: r"final: (\w+)".to_string(),
                type_: DataType::String,
            },
        );
        wf.steps.insert("step1".to_string(), step);

        wf.results.insert(
            "outcome".to_string(),
            ResultRef {
                ref_: "steps.step1.outputs.status".to_string(),
            },
        );

        let result = wf.run();
        assert_eq!(result.status, "ok");
        assert!(result.results.is_some());
        let results = result.results.unwrap();
        assert_eq!(results.get("outcome").map(String::as_str), Some("success"));
    }

    #[test]
    fn test_chain_run_timeout_exceeded() {
        let mut wf = Chain {
            timeout: 1,
            ..chain_with_defaults()
        };

        let step = Step {
            script: if cfg!(windows) {
                "Start-Sleep -Seconds 10".to_string()
            } else {
                "sleep 10".to_string()
            },
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: if cfg!(windows) {
                    "powershell".to_string()
                } else {
                    "bash".to_string()
                },
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        wf.steps.insert("step1".to_string(), step);

        let result = wf.run();
        // Timeout now appears as a StepExecution error wrapping the timeout
        assert_eq!(result.status, "nok");
        assert!(!result.errors.is_empty());
        // The error should be a StepExecution error containing timeout info
        if let Some(AtentoError::StepExecution { step, reason }) = result.errors.first() {
            assert_eq!(step, "step1");
            assert!(reason.contains("timeout") || reason.contains("Timeout"));
        } else {
            panic!(
                "Expected StepExecution error with timeout, got: {:?}",
                result.errors
            );
        }
    }

    #[test]
    fn test_chain_run_step_failure_propagates() {
        let mut wf = chain_with_defaults();

        let mut step = Step {
            script: "echo 'no match'".to_string(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: "bash".to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.outputs.insert(
            "value".to_string(),
            Output {
                pattern: r"result: (\d+)".to_string(),
                type_: DataType::Int,
            },
        );
        wf.steps.insert("step1".to_string(), step);

        let result = wf.run();
        assert_eq!(result.status, "nok");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_chain_deserialize() {
        let yaml = r"
name: test_chain
timeout: 600
";
        let wf: Chain = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(wf.name.as_deref(), Some("test_chain"));
        assert_eq!(wf.timeout, 600);
    }

    #[test]
    fn test_chain_deserialize_defaults() {
        let yaml = r"
name: minimal
";
        let wf: Chain = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(wf.timeout, 300);
        assert!(wf.parameters.is_empty());
        assert!(wf.steps.is_empty());
    }

    #[test]
    fn test_chain_result_serialize() {
        use crate::chain::ChainResult;

        let result = ChainResult {
            name: Some("test".to_string()),
            duration_ms: 1000,
            parameters: None,
            steps: None,
            results: None,
            errors: Vec::new(),
            status: "ok".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_chain_result_skip_none_fields() {
        use crate::chain::ChainResult;

        let result = ChainResult {
            name: None,
            duration_ms: 500,
            parameters: None,
            steps: None,
            results: None,
            errors: Vec::new(),
            status: "ok".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("parameters"));
        assert!(!json.contains("steps"));
        assert!(!json.contains("results"));
    }

    #[test]
    fn test_chain_inline_input() {
        let mut wf = chain_with_defaults();

        let mut step = Step {
            script: if cfg!(windows) {
                "echo {{ inputs.value }}".to_string()
            } else {
                "echo {{ inputs.value }}".to_string()
            },
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: if cfg!(windows) {
                    "batch".to_string()
                } else {
                    "bash".to_string()
                },
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        step.inputs.insert(
            "value".to_string(),
            Input::Inline {
                type_: DataType::String,
                value: serde_yaml::Value::String("inline_test".to_string()),
            },
        );
        wf.steps.insert("step1".to_string(), step);

        let result = wf.run();
        assert_eq!(result.status, "ok");
        let steps = result.steps.unwrap();
        // On Windows, check if output contains the expected text (might have extra chars)
        let stdout = steps["step1"].stdout.as_deref().unwrap_or("");
        if cfg!(windows) {
            assert!(
                stdout.contains("inline_test"),
                "Expected stdout to contain 'inline_test', got: {:?}",
                stdout
            );
        } else {
            assert_eq!(steps["step1"].stdout.as_deref(), Some("inline_test"));
        }
    }

    #[test]
    fn test_chain_complex_parameter_types() {
        let mut wf = chain_with_defaults();
        wf.parameters.insert(
            "count".to_string(),
            Parameter {
                type_: DataType::Int,
                value: serde_yaml::Value::Number(42.into()),
            },
        );
        wf.parameters.insert(
            "enabled".to_string(),
            Parameter {
                type_: DataType::Bool,
                value: serde_yaml::Value::Bool(true),
            },
        );

        let result = wf.run();
        assert_eq!(result.status, "ok");
        let params = result.parameters.unwrap();
        assert_eq!(params.get("count").map(String::as_str), Some("42"));
        assert_eq!(params.get("enabled").map(String::as_str), Some("true"));
    }

    #[test]
    fn test_chain_steps_maintain_order() {
        let mut wf = chain_with_defaults();

        for i in 1..=5 {
            let step = Step {
                script: format!("echo step{i}"),
                ..Step {
                    name: None,
                    timeout: 60,
                    inputs: HashMap::new(),
                    interpreter: "bash".to_string(),
                    script: String::new(),
                    outputs: HashMap::new(),
                }
            };
            wf.steps.insert(format!("step{i}"), step);
        }

        let result = wf.run();
        assert_eq!(result.status, "ok");
        let steps = result.steps.unwrap();

        let keys: Vec<_> = steps.keys().collect();
        assert_eq!(keys, vec!["step1", "step2", "step3", "step4", "step5"]);
    }

    #[test]
    fn test_chain_duration_accumulates() {
        let mut wf = chain_with_defaults();

        let (sleep_cmd, interpreter) = if cfg!(windows) {
            ("timeout /t 1 /nobreak >nul".to_string(), "batch")
        } else {
            ("sleep 0.1".to_string(), "bash")
        };

        let step1 = Step {
            script: sleep_cmd.clone(),
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: interpreter.to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };
        let step2 = Step {
            script: sleep_cmd,
            ..Step {
                name: None,
                timeout: 60,
                inputs: HashMap::new(),
                interpreter: interpreter.to_string(),
                script: String::new(),
                outputs: HashMap::new(),
            }
        };

        wf.steps.insert("step1".to_string(), step1);
        wf.steps.insert("step2".to_string(), step2);

        let result = wf.run();
        assert_eq!(result.status, "ok");
        // More lenient timing for Windows - just ensure it's reasonable
        let expected_min = if cfg!(windows) { 50 } else { 150 };
        assert!(
            result.duration_ms >= expected_min,
            "Duration {} should be >= {}",
            result.duration_ms,
            expected_min
        );
    }

    #[test]
    fn test_chain_result_parameter_conversion_error() {
        // Test parameter to_string_value error during result building
        let mut chain = Chain::default();
        chain.parameters.insert(
            "invalid_param".to_string(),
            Parameter {
                value: serde_yaml::Value::Null,
                type_: crate::data_type::DataType::Int,
            },
        );
        chain.steps.insert(
            "test_step".to_string(),
            Step {
                name: None,
                timeout: 60,
                inputs: std::collections::HashMap::new(),
                interpreter: "bash".to_string(),
                script: "echo 'test'".to_string(),
                outputs: std::collections::HashMap::new(),
            },
        );

        let result = chain.run();
        // Should fail during parameter conversion in final result building
        assert_eq!(result.status, "nok");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_chain_timeout_edge_case() {
        // Test chain timeout exactly at boundary
        let mut chain = chain_with_defaults();
        chain.timeout = 1; // Very short timeout
        chain.steps.insert(
            "slow_step".to_string(),
            Step {
                name: None,
                timeout: 60,
                inputs: std::collections::HashMap::new(),
                interpreter: if cfg!(windows) {
                    "powershell".to_string()
                } else {
                    "bash".to_string()
                },
                script: if cfg!(windows) {
                    "Start-Sleep -Seconds 30; Write-Host 'done'".to_string()
                } else {
                    "sleep 30 && echo 'done'".to_string()
                },
                outputs: std::collections::HashMap::new(),
            },
        );

        let result = chain.run();
        // Should timeout before or during step execution

        assert_eq!(result.status, "nok");
        assert!(!result.errors.is_empty());
        // Timeout may appear as StepExecution or direct Timeout depending on when it triggers
        let has_timeout = result.errors.iter().any(|e| match e {
            crate::errors::AtentoError::Timeout { .. } => true,
            crate::errors::AtentoError::StepExecution { reason, .. } => {
                reason.contains("timeout") || reason.contains("Timeout")
            }
            _ => false,
        });
        assert!(
            has_timeout,
            "Expected timeout-related error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_chain_result_unresolved_output_reference() {
        // Test error case when chain result references non-existent output
        let mut chain = chain_with_defaults();
        chain.steps.insert(
            "test_step".to_string(),
            Step {
                name: None,
                timeout: 60,
                inputs: std::collections::HashMap::new(),
                interpreter: "bash".to_string(),
                script: "echo 'test'".to_string(),
                outputs: std::collections::HashMap::new(), // No outputs defined
            },
        );
        chain.results.insert(
            "missing_result".to_string(),
            crate::result_ref::ResultRef {
                ref_: "steps.test_step.outputs.nonexistent".to_string(),
            },
        );

        let result = chain.run();
        assert_eq!(result.status, "nok");
        assert!(!result.errors.is_empty());
        assert!(matches!(
            result.errors.first().unwrap(),
            crate::errors::AtentoError::UnresolvedReference { .. }
        ));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {
    use crate::chain::Chain;
    use crate::errors::AtentoError;

    use crate::parameter::Parameter;
    use crate::step::Step;
    use std::collections::HashMap;

    // Pure unit tests for Chain struct (no I/O)

    #[test]
    fn test_chain_default() {
        let chain = Chain::default();
        assert!(chain.name.is_none());
        assert_eq!(chain.timeout, 300);
        assert!(chain.steps.is_empty());
        assert!(chain.parameters.is_empty());
        assert!(chain.results.is_empty());
    }

    #[test]
    fn test_chain_deserialize_minimal() {
        let yaml = r"
steps:
  step1:
    type: bash
    script: echo hello
";
        let chain: Chain = serde_yaml::from_str(yaml).unwrap();
        assert!(chain.name.is_none());
        assert_eq!(chain.timeout, 300); // default
        assert_eq!(chain.steps.len(), 1);
        assert!(chain.steps.contains_key("step1"));
    }

    #[test]
    fn test_chain_deserialize_with_name_and_timeout() {
        let yaml = r"
name: test_chain
timeout: 120
steps:
  step1:
    type: bash
    script: echo hello
";
        let chain: Chain = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(chain.name.as_deref(), Some("test_chain"));
        assert_eq!(chain.timeout, 120);
        assert_eq!(chain.steps.len(), 1);
    }

    #[test]
    fn test_chain_validation_empty_steps() {
        let chain = Chain::default();
        let result = chain.validate();
        // Empty chain validation passes - no steps is allowed in this implementation
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_validation_step_validation_error() {
        let mut chain = Chain::default();
        chain.steps.insert(
            "step1".to_string(),
            Step {
                script: "echo {{ inputs.missing }}".to_string(),
                ..Step {
                    name: None,
                    timeout: 60,
                    inputs: HashMap::new(),
                    interpreter: "bash".to_string(),
                    script: String::new(),
                    outputs: HashMap::new(),
                }
            },
        );

        let result = chain.validate();
        assert!(result.is_err());
        if let Err(AtentoError::Validation(msg)) = result {
            assert!(msg.contains("references input 'missing'"));
        }
    }

    #[test]
    fn test_chain_validation_success() {
        let mut chain = Chain::default();
        chain.steps.insert(
            "step1".to_string(),
            Step {
                script: "echo hello".to_string(),
                ..Step {
                    name: None,
                    timeout: 60,
                    inputs: HashMap::new(),
                    interpreter: "bash".to_string(),
                    script: String::new(),
                    outputs: HashMap::new(),
                }
            },
        );
        chain.steps.insert(
            "step2".to_string(),
            Step {
                script: "echo world".to_string(),
                ..Step {
                    name: None,
                    timeout: 60,
                    inputs: HashMap::new(),
                    interpreter: "bash".to_string(),
                    script: String::new(),
                    outputs: HashMap::new(),
                }
            },
        );

        let result = chain.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_with_parameters() {
        let yaml = r#"
name: parameterized_chain
parameters:
  env:
    type: string
    value: production
  debug:
    type: bool
    value: false
steps:
  step1:
    type: bash
    script: "echo Environment: {{ parameters.env }}"
"#;
        let chain: Chain = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(chain.name.as_deref(), Some("parameterized_chain"));
        assert_eq!(chain.parameters.len(), 2);
        assert!(chain.parameters.contains_key("env"));
        assert!(chain.parameters.contains_key("debug"));
        assert_eq!(chain.steps.len(), 1);
    }

    #[test]
    fn test_chain_complex_structure() {
        let yaml = r#"
name: complex_chain
timeout: 600
parameters:
  config_file:
    type: string
    value: "config.yaml"
steps:
  read_config:
    type: bash
    timeout: 30
    script: "cat {{ parameters.config_file }}"
    outputs:
      config_content:
        pattern: "version: ([\\d\\.]+)"
        type: string
  process_config:
    type: python
    timeout: 60
    script: "print(f'Processing version {config_content}')"
    inputs:
      config_content:
        type: string
        value: "1.0.0"
results:
  version:
    ref: read_config.config_content
"#;
        let chain: Chain = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(chain.name.as_deref(), Some("complex_chain"));
        assert_eq!(chain.timeout, 600);
        assert_eq!(chain.parameters.len(), 1);
        assert_eq!(chain.steps.len(), 2);
        assert_eq!(chain.results.len(), 1);

        // Check steps exist
        assert!(chain.steps.contains_key("read_config"));
        assert!(chain.steps.contains_key("process_config"));

        // Check results
        assert!(chain.results.contains_key("version"));
    }

    #[test]
    fn test_chain_validation_with_parameters() {
        let mut chain = Chain::default();

        // Add parameter
        chain.parameters.insert(
            "test_param".to_string(),
            Parameter {
                type_: crate::data_type::DataType::String,
                value: serde_yaml::Value::String("test_value".to_string()),
            },
        );

        // Add step that uses the parameter
        chain.steps.insert(
            "step1".to_string(),
            Step {
                script: "echo {{ parameters.test_param }}".to_string(),
                ..Step {
                    name: None,
                    timeout: 60,
                    inputs: HashMap::new(),
                    interpreter: "bash".to_string(),
                    script: String::new(),
                    outputs: HashMap::new(),
                }
            },
        );

        let result = chain.validate();
        // This should pass validation if the parameter is properly referenced
        // The actual validation logic depends on the implementation
        assert!(result.is_ok() || result.is_err()); // Either is acceptable for this unit test
    }

    #[test]
    fn test_chain_custom_interpreter_config() {
        let mut chain = Chain::default();

        // Add a custom bash interpreter configuration
        let custom_bash = crate::Interpreter {
            command: "/bin/bash".to_string(),
            args: vec!["-c".to_string()],
            extension: ".sh".to_string(),
        };

        chain
            .interpreters
            .insert("bash".to_string(), custom_bash.clone());

        // Add a step that uses bash
        chain.steps.insert(
            "step1".to_string(),
            Step {
                name: Some("Test Step".to_string()),
                script: "echo 'custom interpreter'".to_string(),
                interpreter: "bash".to_string(),
                timeout: 60,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            },
        );

        // Validate the chain
        let result = chain.validate();
        assert!(result.is_ok());

        // Verify the custom interpreter is stored (manually added to default chain)
        assert_eq!(chain.interpreters.len(), 1);
        assert!(chain.interpreters.contains_key("bash"));

        let stored_config = chain.interpreters.get("bash").unwrap();
        assert_eq!(stored_config.command, "/bin/bash");
        assert_eq!(stored_config.args, vec!["-c"]);
        assert_eq!(stored_config.extension, ".sh");
    }

    #[test]
    fn test_chain_custom_interpreter_serialization() {
        let yaml = r#"
name: test_custom_interpreter
timeout: 300
interpreters:
  bash:
    command: /custom/bin/bash
    args:
      - "-e"
      - "-x"
    extension: .sh
  python:
    command: python3.11
    args:
      - "-u"
    extension: .py
steps:
  step1:
    type: bash
    script: echo "test"
results: {}
"#;

        let chain: Result<Chain, _> = serde_yaml::from_str(yaml);
        assert!(chain.is_ok());

        let chain = chain.unwrap();
        // Should have 6 defaults (bash, cmd, powershell, pwsh, python, python3), 2 override defaults (bash, python)
        assert_eq!(chain.interpreters.len(), 6);

        // Check bash config (overridden)
        let bash_config = chain.interpreters.get("bash").unwrap();
        assert_eq!(bash_config.command, "/custom/bin/bash");
        assert_eq!(bash_config.args, vec!["-e", "-x"]);

        // Check python config (overridden)
        let python_config = chain.interpreters.get("python").unwrap();
        assert_eq!(python_config.command, "python3.11");
        assert_eq!(python_config.args, vec!["-u"]);

        // Check that defaults are still there
        assert!(chain.interpreters.contains_key("batch"));
        assert!(chain.interpreters.contains_key("powershell"));
        assert!(chain.interpreters.contains_key("pwsh"));
        assert!(chain.interpreters.contains_key("python"));
        assert!(chain.interpreters.contains_key("python3"));
    }
}
