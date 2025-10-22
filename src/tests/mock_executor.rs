use crate::errors::Result;
use crate::executor::{CommandExecutor, ExecutionResult};
use std::cell::RefCell;
use std::collections::HashMap;

type CallRecord = (String, String, Vec<String>, u64);

/// Mock implementation for unit tests
pub struct MockExecutor {
    responses: HashMap<String, ExecutionResult>,
    default_response: ExecutionResult,
    call_count: RefCell<usize>,
    last_call: RefCell<Option<CallRecord>>,
}

impl MockExecutor {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            default_response: ExecutionResult {
                stdout: "mock output".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 10,
            },
            call_count: RefCell::new(0),
            last_call: RefCell::new(None),
        }
    }

    pub fn expect_call(&mut self, script: &str, response: ExecutionResult) -> &mut Self {
        self.responses.insert(script.to_string(), response);
        self
    }

    pub fn expect_timeout(&mut self, script: &str) -> &mut Self {
        self.responses.insert(
            script.to_string(),
            ExecutionResult {
                stdout: String::new(),
                stderr: "Timeout".to_string(),
                exit_code: 124,
                duration_ms: 1000,
            },
        );
        self
    }

    pub fn expect_error(&mut self, script: &str, exit_code: i32, stderr: &str) -> &mut Self {
        self.responses.insert(
            script.to_string(),
            ExecutionResult {
                stdout: String::new(),
                stderr: stderr.to_string(),
                exit_code,
                duration_ms: 5,
            },
        );
        self
    }

    pub fn call_count(&self) -> usize {
        *self.call_count.borrow()
    }

    pub fn last_call(&self) -> Option<(String, String, Vec<String>, u64)> {
        self.last_call.borrow().clone()
    }
}

impl CommandExecutor for MockExecutor {
    fn execute(
        &self,
        script: &str,
        extension: &str,
        args: &[String],
        timeout: u64,
    ) -> Result<ExecutionResult> {
        *self.call_count.borrow_mut() += 1;
        *self.last_call.borrow_mut() = Some((
            script.to_string(),
            extension.to_string(),
            args.to_vec(),
            timeout,
        ));

        Ok(self
            .responses
            .get(script)
            .cloned()
            .unwrap_or_else(|| self.default_response.clone()))
    }
}

impl Default for MockExecutor {
    fn default() -> Self {
        Self::new()
    }
}
