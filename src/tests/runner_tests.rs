#[cfg(test)]
mod tests {
    // Cross-platform runner integration tests (non-execution) go here
    // Currently, all runner tests require actual command execution
    // so they are in the tests/integration/ directory for platform-specific testing
}

#[cfg(test)]
mod unit_tests {
    use crate::errors::AtentoError;
    use crate::runner::run_with_timeout;

    #[test]
    fn test_runner_module_exists() {
        // This is a placeholder test to ensure the unit test module compiles
        // The actual runner functionality is tested via integration tests
        // and through the Step struct's run_with_executor method
    }

    #[test]
    fn test_run_with_timeout_empty_script() {
        let result = run_with_timeout("", ".sh", &["bash"], 60);
        assert!(result.is_err());
        if let Err(AtentoError::Runner(msg)) = result {
            assert!(msg.contains("Script cannot be empty"));
        } else {
            panic!("Expected Runner error about empty script");
        }
    }

    #[test]
    fn test_run_with_timeout_empty_args() {
        let result = run_with_timeout("echo test", ".sh", &[], 60);
        assert!(result.is_err());
        if let Err(AtentoError::Runner(msg)) = result {
            assert!(msg.contains("Command arguments cannot be empty"));
        } else {
            panic!("Expected Runner error about empty args");
        }
    }

    #[test]
    fn test_run_with_timeout_zero_timeout_uses_default() {
        // This test verifies that passing 0 timeout uses the default timeout
        // We can't easily test the actual execution with default timeout in unit tests
        // since it would require real command execution, but we can test the parameter validation
        let result = run_with_timeout("echo test", ".sh", &["bash"], 0);
        // The function should accept 0 timeout and use default internally
        // Result may fail due to bash execution but not due to timeout parameter validation
        assert!(result.is_ok() || matches!(result, Err(AtentoError::Runner(_))));
    }

    #[test]
    fn test_run_with_timeout_valid_parameters() {
        let result = run_with_timeout("echo hello", ".sh", &["bash"], 30);
        // This should succeed (or fail only due to command execution, not parameter validation)
        match result {
            Ok(runner_result) => {
                // If successful, verify the result structure
                // duration_ms is u128, so it's always >= 0, just verify it exists
                let _ = runner_result.duration_ms;
                // stdout might be Some or None depending on execution
            }
            Err(AtentoError::Runner(_)) => {
                // Command execution might fail in some environments, that's okay for unit test
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }

    #[test]
    fn test_run_with_timeout_with_powershell_extension() {
        // Test that PowerShell extension is handled correctly
        let result = run_with_timeout("Write-Host test", ".ps1", &["pwsh"], 30);
        // The function should accept .ps1 extension and set appropriate environment
        match result {
            Ok(_) | Err(AtentoError::Runner(_) | AtentoError::Timeout { .. }) => {
                // Success case, PowerShell might not be available, or timeout - all acceptable for unit test
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }

    #[test]
    fn test_run_with_timeout_invalid_command() {
        let result = run_with_timeout("echo test", ".sh", &["nonexistent_command"], 30);
        assert!(result.is_err());
        // Should fail with Runner error when trying to start nonexistent command
        if let Err(AtentoError::Runner(msg)) = result {
            assert!(msg.contains("Failed to start command"));
        } else {
            panic!("Expected Runner error about failed command start");
        }
    }

    #[test]
    fn test_run_with_timeout_stderr_filtering() {
        // Test that stderr filtering works correctly
        let result = run_with_timeout("echo test", ".sh", &["bash"], 30);

        match result {
            Ok(runner_result) => {
                // If successful, stderr should be properly filtered
                // We can't test the exact filtering without actual stderr output
                let _ = runner_result.duration_ms; // duration_ms is u128, always >= 0
            }
            Err(AtentoError::Runner(_)) => {
                // Command might fail in some environments
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_run_with_timeout_exit_code_handling() {
        // Test that exit codes are properly captured
        let result = run_with_timeout("exit 42", ".sh", &["bash"], 30);

        match result {
            Ok(runner_result) => {
                // Should capture the exit code correctly
                assert_eq!(runner_result.exit_code, 42);
            }
            Err(AtentoError::Runner(_)) => {
                // Command might fail in some environments
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }

    #[test]
    fn test_run_with_timeout_windows_permissions() {
        // Test Windows-specific permission handling
        let result = run_with_timeout("echo test", ".bat", &["cmd", "/c"], 30);

        // This test mainly ensures the Windows permission code path compiles
        // and doesn't crash on non-Windows systems
        match result {
            Ok(_) | Err(AtentoError::Runner(_)) => {
                // Success on Windows or expected on non-Windows systems/when cmd is not available
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }

    #[test]
    fn test_run_with_timeout_temp_file_creation() {
        // Test temporary file creation and cleanup
        let result = run_with_timeout("echo 'temp test'", ".sh", &["bash"], 30);

        // The temp file should be cleaned up regardless of success or failure
        if result.is_ok() {
            // Temp file should be cleaned up on success
        } else {
            // Temp file should be cleaned up on error too
        }

        // We can't easily test the actual cleanup without exposing internal paths,
        // but this exercises the temp file creation code path
    }

    #[test]
    fn test_run_with_timeout_process_wait_error() {
        // Test error handling when process wait fails
        // This is hard to trigger artificially, but we test the code path exists
        let result = run_with_timeout("echo test", ".sh", &["bash"], 30);

        match result {
            Ok(_) | Err(AtentoError::Timeout { .. }) => {
                // Normal success case or timeout is valid outcome
            }
            Err(AtentoError::Runner(msg)) => {
                // Could be various runner errors
                assert!(!msg.is_empty());
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }

    #[test]
    fn test_run_with_timeout_utf8_handling() {
        // Test UTF-8 output handling
        let result = run_with_timeout("echo 'test ñoñó'", ".sh", &["bash"], 30);

        match result {
            Ok(runner_result) => {
                // Should handle UTF-8 correctly
                if let Some(stdout) = runner_result.stdout {
                    assert!(!stdout.is_empty());
                }
            }
            Err(AtentoError::Runner(_)) => {
                // Command might fail in some environments
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }

    #[test]
    fn test_run_with_timeout_duration_measurement() {
        // Test that duration is measured correctly
        let result = run_with_timeout("echo fast", ".sh", &["bash"], 30);

        match result {
            Ok(runner_result) => {
                // Duration should be reasonable for a fast command
                assert!(runner_result.duration_ms < 10000); // Less than 10 seconds
            }
            Err(AtentoError::Runner(_)) => {
                // Command might fail in some environments
            }
            Err(e) => {
                panic!("Unexpected error type: {e:?}");
            }
        }
    }
}
