#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::executor::{CommandExecutor, ExecutionResult};
    use crate::interpreter::Interpreter;
    use crate::tests::mock_executor::MockExecutor;

    fn bash_interpreter() -> Interpreter {
        Interpreter {
            command: "bash".to_string(),
            args: vec![],
            extension: ".sh".to_string(),
        }
    }

    #[test]
    fn test_mock_executor_default() {
        let executor = MockExecutor::new();
        assert_eq!(executor.call_count(), 0);
        assert!(executor.last_call().is_none());
    }

    #[test]
    fn test_mock_executor_default_response() {
        let executor = MockExecutor::new();
        let result = executor
            .execute("echo 'test'", &bash_interpreter(), 30)
            .unwrap();

        assert_eq!(result.stdout, "mock output");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.duration_ms, 10);
        assert_eq!(executor.call_count(), 1);
    }

    #[test]
    fn test_mock_executor_expect_call() {
        let mut executor = MockExecutor::new();
        executor.expect_call(
            "echo 'hello'",
            ExecutionResult {
                stdout: "hello".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 5,
            },
        );

        let result = executor
            .execute("echo 'hello'", &bash_interpreter(), 30)
            .unwrap();

        assert_eq!(result.stdout, "hello");
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.duration_ms, 5);
    }

    #[test]
    fn test_mock_executor_expect_timeout() {
        let mut executor = MockExecutor::new();
        executor.expect_timeout("slow_command");

        let result = executor
            .execute("slow_command", &bash_interpreter(), 10)
            .unwrap();

        assert_eq!(result.stdout, "");
        assert_eq!(result.stderr, "Timeout");
        assert_eq!(result.exit_code, 124);
        assert_eq!(result.duration_ms, 1000);
    }

    #[test]
    fn test_mock_executor_expect_error() {
        let mut executor = MockExecutor::new();
        executor.expect_error("failing_command", 1, "Command not found");

        let result = executor
            .execute("failing_command", &bash_interpreter(), 30)
            .unwrap();

        assert_eq!(result.stdout, "");
        assert_eq!(result.stderr, "Command not found");
        assert_eq!(result.exit_code, 1);
        assert_eq!(result.duration_ms, 5);
    }

    #[test]
    fn test_mock_executor_last_call() {
        let executor = MockExecutor::new();

        executor
            .execute("test_script", &bash_interpreter(), 60)
            .unwrap();

        let last_call = executor.last_call().unwrap();
        assert_eq!(last_call.0, "test_script");
        assert_eq!(last_call.1.extension, ".sh");
        assert_eq!(last_call.1.command, "bash");
        assert_eq!(last_call.2, 60);
    }

    #[test]
    fn test_mock_executor_call_count() {
        let executor = MockExecutor::new();
        assert_eq!(executor.call_count(), 0);

        executor.execute("cmd1", &bash_interpreter(), 30).unwrap();
        assert_eq!(executor.call_count(), 1);

        executor.execute("cmd2", &bash_interpreter(), 30).unwrap();
        assert_eq!(executor.call_count(), 2);

        executor.execute("cmd3", &bash_interpreter(), 30).unwrap();
        assert_eq!(executor.call_count(), 3);
    }

    #[test]
    fn test_mock_executor_multiple_expected_calls() {
        let mut executor = MockExecutor::new();

        executor.expect_call(
            "cmd1",
            ExecutionResult {
                stdout: "output1".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 10,
            },
        );

        executor.expect_call(
            "cmd2",
            ExecutionResult {
                stdout: "output2".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 20,
            },
        );

        let result1 = executor.execute("cmd1", &bash_interpreter(), 30).unwrap();
        assert_eq!(result1.stdout, "output1");
        assert_eq!(result1.duration_ms, 10);

        let result2 = executor.execute("cmd2", &bash_interpreter(), 30).unwrap();
        assert_eq!(result2.stdout, "output2");
        assert_eq!(result2.duration_ms, 20);

        // Unmapped command should return default
        let result3 = executor.execute("cmd3", &bash_interpreter(), 30).unwrap();
        assert_eq!(result3.stdout, "mock output");
    }

    #[test]
    fn test_mock_executor_chain_expectations() {
        let mut executor = MockExecutor::new();

        executor
            .expect_call(
                "cmd1",
                ExecutionResult {
                    stdout: "first".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: 5,
                },
            )
            .expect_timeout("cmd2")
            .expect_error("cmd3", 127, "not found");

        let result1 = executor.execute("cmd1", &bash_interpreter(), 30).unwrap();
        assert_eq!(result1.stdout, "first");

        let result2 = executor.execute("cmd2", &bash_interpreter(), 30).unwrap();
        assert_eq!(result2.exit_code, 124);

        let result3 = executor.execute("cmd3", &bash_interpreter(), 30).unwrap();
        assert_eq!(result3.exit_code, 127);
        assert_eq!(result3.stderr, "not found");
    }

    #[test]
    fn test_execution_result_clone() {
        let result = ExecutionResult {
            stdout: "test output".to_string(),
            stderr: "test error".to_string(),
            exit_code: 42,
            duration_ms: 100,
        };

        let cloned = result.clone();
        assert_eq!(cloned.stdout, result.stdout);
        assert_eq!(cloned.stderr, result.stderr);
        assert_eq!(cloned.exit_code, result.exit_code);
        assert_eq!(cloned.duration_ms, result.duration_ms);
    }

    #[test]
    fn test_execution_result_debug() {
        let result = ExecutionResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: 1,
            duration_ms: 50,
        };

        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("ExecutionResult"));
        assert!(debug_str.contains("stdout"));
        assert!(debug_str.contains("output"));
    }

    #[test]
    fn test_execution_result_partial_eq() {
        let result1 = ExecutionResult {
            stdout: "test".to_string(),
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 10,
        };

        let result2 = ExecutionResult {
            stdout: "test".to_string(),
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 10,
        };

        let result3 = ExecutionResult {
            stdout: "different".to_string(),
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 10,
        };

        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }
}
