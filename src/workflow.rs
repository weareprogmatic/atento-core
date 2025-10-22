use crate::errors::{AtentoError, Result};
use crate::executor::CommandExecutor;
use crate::input::Input;
use crate::parameter::Parameter;
use crate::result_ref::ResultRef;
use crate::step::{Step, StepResult};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const DEFAULT_WORKFLOW_TIMEOUT: u64 = 300;

// Helper function to provide the custom default for serde
fn default_workflow_timeout() -> u64 {
    DEFAULT_WORKFLOW_TIMEOUT
}

#[derive(Debug, Deserialize)]
pub struct Workflow {
    pub name: Option<String>,
    #[serde(default = "default_workflow_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,
    #[serde(default)]
    pub steps: IndexMap<String, Step>,
    #[serde(default)]
    pub results: HashMap<String, ResultRef>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub duration_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<IndexMap<String, StepResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<AtentoError>,
    pub status: String,
}

impl Default for Workflow {
    fn default() -> Self {
        Self {
            name: None,
            timeout: default_workflow_timeout(),
            parameters: HashMap::new(),
            steps: IndexMap::new(),
            results: HashMap::new(),
        }
    }
}

impl Workflow {
    fn make_output_key(step_key: &str, output_key: &str) -> String {
        format!("steps.{step_key}.outputs.{output_key}")
    }

    /// Validates the workflow structure.
    ///
    /// # Errors
    /// Returns validation errors for unresolved references, forward references, or invalid patterns.
    pub fn validate(&self) -> Result<()> {
        let parameter_keys: HashSet<String> = self
            .parameters
            .keys()
            .map(|k| format!("parameters.{k}"))
            .collect();

        let mut step_output_keys = HashSet::new();

        for (step_key, step) in &self.steps {
            for (input_key, input) in &step.inputs {
                if let Input::Ref { ref_ } = input
                    && !parameter_keys.contains(ref_)
                    && !step_output_keys.contains(ref_)
                {
                    let forward_decl = self
                        .steps
                        .keys()
                        .skip_while(|k| *k != step_key)
                        .skip(1)
                        .any(|k| {
                            self.steps[k]
                                .outputs
                                .keys()
                                .any(|out_name| Self::make_output_key(k, out_name) == *ref_)
                        });

                    if forward_decl {
                        return Err(AtentoError::Validation(format!(
                            "Input '{input_key}' in step '{step_key}' references '{ref_}', which is a future step output"
                        )));
                    }

                    return Err(AtentoError::UnresolvedReference {
                        reference: ref_.clone(),
                        context: format!("step '{step_key}'"),
                    });
                }
            }

            step.validate(step_key)?;

            for (out_key, out) in &step.outputs {
                if out.pattern.is_empty() {
                    return Err(AtentoError::Validation(format!(
                        "Output '{out_key}' in step '{step_key}' has empty capture pattern"
                    )));
                }

                step_output_keys.insert(Self::make_output_key(step_key, out_key));
            }
        }

        for (result_key, result) in &self.results {
            if !step_output_keys.contains(&result.ref_) {
                return Err(AtentoError::UnresolvedReference {
                    reference: result.ref_.clone(),
                    context: format!("workflow result '{result_key}'"),
                });
            }
        }

        Ok(())
    }

    fn resolve_input(
        &self,
        input_name: &str,
        input: &Input,
        step_name: &str,
        resolved_outputs: &HashMap<String, String>,
    ) -> Result<String> {
        match input {
            Input::Inline { .. } => input.to_string_value().map_err(|e| {
                AtentoError::Execution(format!("Input '{input_name}' in step '{step_name}': {e}"))
            }),

            Input::Ref { ref_ } => {
                let param_key = ref_.strip_prefix("parameters.").unwrap_or(ref_);

                if let Some(param) = self.parameters.get(param_key) {
                    param.to_string_value().map_err(|e| {
                        AtentoError::Execution(format!(
                            "Parameter '{input_name}' in step '{step_name}': {e}"
                        ))
                    })
                } else if let Some(output) = resolved_outputs.get(ref_) {
                    Ok(output.clone())
                } else {
                    Err(AtentoError::UnresolvedReference {
                        reference: ref_.clone(),
                        context: format!("step '{step_name}'"),
                    })
                }
            }
        }
    }

    /// Executes the workflow with a custom executor (useful for testing).
    ///
    /// # Errors
    /// Returns an error if timeout is exceeded, a step fails, or output resolution fails.
    pub fn run_with_executor<E: CommandExecutor>(&self, executor: &E) -> WorkflowResult {
        let start_time = Instant::now();
        let mut resolved_outputs: HashMap<String, String> = HashMap::new();
        let mut step_results: IndexMap<String, StepResult> = IndexMap::new();
        let mut workflow_errors: Vec<AtentoError> = Vec::new();

        for (step_name, step) in &self.steps {
            let elapsed = start_time.elapsed().as_secs();
            let mut time_left: u64 = 0;

            if self.timeout > 0 {
                if elapsed >= self.timeout {
                    workflow_errors.push(AtentoError::Timeout {
                        context: format!("Workflow timed out before step '{step_name}'"),
                        timeout_secs: self.timeout,
                    });

                    break;
                }

                time_left = self.timeout.saturating_sub(elapsed);
            }

            let mut step_inputs = HashMap::new();
            let mut input_error = false;
            for (input_name, input) in &step.inputs {
                match self.resolve_input(input_name, input, step_name, &resolved_outputs) {
                    Ok(val) => {
                        step_inputs.insert(input_name.clone(), val);
                    }
                    Err(e) => {
                        workflow_errors.push(e);
                        input_error = true;
                        break;
                    }
                }
            }

            if input_error {
                break;
            }

            let step_result = step.run(executor, &step_inputs, time_left);

            for (k, v) in &step_result.outputs {
                resolved_outputs.insert(Self::make_output_key(step_name, k), v.clone());
            }

            // Check for step error before inserting
            if let Some(ref err) = step_result.error {
                workflow_errors.push(AtentoError::StepExecution {
                    step: step_name.clone(),
                    reason: err.to_string(),
                });
                step_results.insert(step_name.clone(), step_result);
                break;
            }

            step_results.insert(step_name.clone(), step_result);
        }

        // Collect workflow results
        let mut final_results = HashMap::new();
        for (result_name, result_ref) in &self.results {
            if let Some(val) = resolved_outputs.get(&result_ref.ref_) {
                final_results.insert(result_name.clone(), val.clone());
            } else {
                workflow_errors.push(AtentoError::UnresolvedReference {
                    reference: result_ref.ref_.clone(),
                    context: format!("Unresolved Reference '{result_name}'"),
                });
            }
        }

        // Build final workflow result
        let parameters = if self.parameters.is_empty() {
            None
        } else {
            match self
                .parameters
                .iter()
                .map(|(k, v)| v.to_string_value().map(|s| (k.clone(), s)))
                .collect::<Result<HashMap<_, _>>>()
            {
                Ok(params) => Some(params),
                Err(e) => {
                    workflow_errors.push(e);
                    None
                }
            }
        };

        let status = if workflow_errors.is_empty() {
            "ok".to_string()
        } else {
            "nok".to_string()
        };

        WorkflowResult {
            name: self.name.clone(),
            duration_ms: start_time.elapsed().as_millis(),
            parameters,
            steps: if step_results.is_empty() {
                None
            } else {
                Some(step_results)
            },
            results: if final_results.is_empty() {
                None
            } else {
                Some(final_results)
            },
            errors: workflow_errors,
            status,
        }
    }

    /// Executes the workflow using the system executor.
    ///
    /// # Errors
    /// Returns an error if timeout is exceeded, a step fails, or output resolution fails.
    #[must_use]
    pub fn run(&self) -> WorkflowResult {
        use crate::executor::SystemExecutor;
        let executor = SystemExecutor;
        self.run_with_executor(&executor)
    }
}
