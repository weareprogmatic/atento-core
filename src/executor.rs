use crate::errors::Result;

/// Trait for abstracting command execution to enable mocking in tests
pub trait CommandExecutor {
    fn execute(
        &self,
        script: &str,
        extension: &str,
        args: &[String],
        timeout: u64,
    ) -> Result<ExecutionResult>;
}

/// Result of command execution
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

/// Real implementation for production use
pub struct SystemExecutor;

impl CommandExecutor for SystemExecutor {
    fn execute(
        &self,
        script: &str,
        extension: &str,
        args: &[String],
        timeout: u64,
    ) -> Result<ExecutionResult> {
        let args_str: Vec<&str> = args.iter().map(std::string::String::as_str).collect();
        let result = crate::runner::run_with_timeout(script, extension, &args_str, timeout)?;
        Ok(ExecutionResult {
            stdout: result.stdout.unwrap_or_default(),
            stderr: result.stderr.unwrap_or_default(),
            exit_code: result.exit_code,
            duration_ms: u64::try_from(result.duration_ms).unwrap_or(u64::MAX),
        })
    }
}
