use crate::errors::{AtentoError, Result};
use crate::executor::CommandExecutor;
use crate::input::Input;
use crate::interpreter::Interpreter;
use crate::output::Output;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const INPUT_PLACEHOLDER_PATTERN: &str = r"\{\{\s*inputs\.(\w+)\s*\}\}";
const DEFAULT_STEP_TIMEOUT: u64 = 60;

// Helper function to provide the custom default for serde
fn default_step_timeout() -> u64 {
    DEFAULT_STEP_TIMEOUT
}

#[derive(Debug, Deserialize)]
pub struct Step {
    pub name: Option<String>,
    #[serde(default = "default_step_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub inputs: HashMap<String, Input>,
    #[serde(rename = "type")]
    pub interpreter: Interpreter,
    pub script: String,
    #[serde(default)]
    pub outputs: HashMap<String, Output>,
}

#[derive(Debug, Serialize)]
pub struct StepResult {
    pub name: Option<String>,
    pub duration_ms: u128,
    pub exit_code: i32,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AtentoError>,
}

impl Step {
    /// Creates a new Step with basic defaults for testing purposes
    #[cfg(test)]
    #[must_use]
    pub fn new(interpreter: Interpreter) -> Self {
        Step {
            name: None,
            timeout: default_step_timeout(),
            inputs: HashMap::new(),
            interpreter,
            script: String::new(),
            outputs: HashMap::new(),
        }
    }

    /// Validates the step configuration.
    ///
    /// # Errors
    /// Returns validation errors for unused inputs, undeclared inputs, or invalid output patterns.
    pub fn validate(&self, id: &str) -> Result<()> {
        let step_name = self.name.as_deref().unwrap_or(id);

        #[allow(clippy::expect_used)]
        let input_ref_regex = Regex::new(INPUT_PLACEHOLDER_PATTERN)
            .expect("Input placeholder regex pattern is valid");

        let mut used_inputs: HashSet<String> = HashSet::new();

        for cap in input_ref_regex.captures_iter(&self.script) {
            let ref_key = &cap[1];
            if !self.inputs.contains_key(ref_key) {
                return Err(AtentoError::Validation(format!(
                    "Step '{step_name}' script references input '{ref_key}' that is not declared"
                )));
            }
            used_inputs.insert(ref_key.to_string());
        }

        for input_name in self.inputs.keys() {
            if !used_inputs.contains(input_name) {
                return Err(AtentoError::Validation(format!(
                    "Step '{step_name}' has input '{input_name}' that is declared but never used in the script"
                )));
            }
        }

        for (out_name, out) in &self.outputs {
            if out.pattern.trim().is_empty() {
                return Err(AtentoError::Validation(format!(
                    "Output '{out_name}' in step '{step_name}' has empty capture pattern"
                )));
            }

            Regex::new(&out.pattern).map_err(|e| {
                AtentoError::Validation(format!(
                    "Output '{}' in step '{}' has invalid regex pattern '{}': {}",
                    out_name, step_name, out.pattern, e
                ))
            })?;
        }

        Ok(())
    }

    /// Calculates the effective timeout for this step.
    #[must_use]
    pub fn calculate_timeout(&self, time_left: u64) -> u64 {
        if self.timeout > 0 && time_left > 0 {
            // Case 1: If both are greater than 0, take the smallest (minimum).
            std::cmp::min(self.timeout, time_left)
        } else {
            // Otherwise (if at least one is 0), take the largest (maximum).
            // The maximum will be the single non-zero value, or 0 if both are 0.
            std::cmp::max(self.timeout, time_left)
        }
    }

    /// Builds the script with input substitution.
    #[must_use]
    pub fn build_script(&self, inputs: &HashMap<String, String>) -> String {
        if self.script.is_empty() {
            return String::new();
        }

        if inputs.is_empty() {
            return self.script.clone();
        }

        #[allow(clippy::expect_used)]
        let re = Regex::new(INPUT_PLACEHOLDER_PATTERN).expect("Valid regex pattern");

        re.replace_all(&self.script, |caps: &regex::Captures| {
            let key = &caps[1];
            inputs
                .get(key)
                .cloned()
                .unwrap_or_else(|| caps[0].to_string())
        })
        .to_string()
    }

    pub fn extract_outputs(&self, stdout: &mut String) -> Result<HashMap<String, String>> {
        if self.outputs.is_empty() {
            return Ok(HashMap::new());
        }

        let mut step_outputs = HashMap::new();

        for (out_name, out) in &self.outputs {
            let re = Regex::new(&out.pattern).map_err(|e| {
                AtentoError::Execution(format!("Invalid regex for output '{out_name}': {e}"))
            })?;

            let caps = re.captures(stdout).ok_or_else(|| {
                AtentoError::Execution(format!(
                    "Output '{}' pattern '{}' did not match stdout",
                    out_name, out.pattern
                ))
            })?;

            if caps.len() <= 1 {
                return Err(AtentoError::Execution(format!(
                    "Output '{}' regex '{}' did not capture a group",
                    out_name, out.pattern
                )));
            }

            step_outputs.insert(out_name.clone(), caps[1].to_string());
            *stdout = stdout.replace(&caps[0], "");
        }

        Ok(step_outputs)
    }

    /// Runs this step with the given executor and resolved inputs.
    ///
    /// # Errors
    /// Returns an error if script execution fails or output extraction fails.
    pub fn run<E: CommandExecutor>(
        &self,
        executor: &E,
        inputs: &HashMap<String, String>,
        time_left: u64,
    ) -> StepResult {
        let script = self.build_script(inputs);

        let timeout = self.calculate_timeout(time_left);

        let ext = self.interpreter.extension();
        let args: Vec<String> = self
            .interpreter
            .args()
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

        let start_time = std::time::Instant::now();
        match executor.execute(&script, ext, &args, timeout) {
            Ok(result) => {
                let duration_ms = start_time.elapsed().as_millis();

                let mut stdout = result.stdout;
                let step_outputs = match self.extract_outputs(&mut stdout) {
                    Ok(outputs) => outputs,
                    Err(e) => {
                        return StepResult {
                            name: self.name.clone(),
                            duration_ms,
                            exit_code: result.exit_code,
                            stdout: Some(stdout.trim().to_string()).filter(|s| !s.is_empty()),
                            stderr: Some(result.stderr).filter(|s| !s.is_empty()),
                            inputs: inputs.clone(),
                            outputs: HashMap::new(),
                            error: Some(e),
                        };
                    }
                };

                StepResult {
                    name: self.name.clone(),
                    duration_ms,
                    exit_code: result.exit_code,
                    stdout: Some(stdout.trim().to_string()).filter(|s| !s.is_empty()),
                    stderr: Some(result.stderr).filter(|s| !s.is_empty()),
                    inputs: inputs.clone(),
                    outputs: step_outputs,
                    error: None,
                }
            }
            Err(e) => {
                let duration_ms = start_time.elapsed().as_millis();
                StepResult {
                    name: self.name.clone(),
                    duration_ms,
                    exit_code: -1,
                    stdout: None,
                    stderr: None,
                    inputs: inputs.clone(),
                    outputs: HashMap::new(),
                    error: Some(e),
                }
            }
        }
    }
}
