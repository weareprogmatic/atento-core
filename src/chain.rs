use crate::errors::{AtentoError, Result};
use crate::executor::CommandExecutor;
use crate::input::Input;
use crate::interpreter::{Interpreter, default_interpreters};
use crate::parameter::Parameter;
use crate::result_ref::ResultRef;
use crate::step::{Step, StepResult};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const DEFAULT_CHAIN_TIMEOUT: u64 = 300;

// Helper function to provide the custom default for serde
fn default_chain_timeout() -> u64 {
    DEFAULT_CHAIN_TIMEOUT
}

#[derive(Debug, Deserialize)]
#[serde(from = "ChainHelper")]
pub struct Chain {
    pub name: Option<String>,
    pub timeout: u64,
    pub interpreters: HashMap<String, Interpreter>,
    pub parameters: HashMap<String, Parameter>,
    pub steps: IndexMap<String, Step>,
    pub results: HashMap<String, ResultRef>,
}

// Helper struct for deserialization
#[derive(Deserialize)]
struct ChainHelper {
    name: Option<String>,
    #[serde(default = "default_chain_timeout")]
    timeout: u64,
    #[serde(default)]
    interpreters: HashMap<String, Interpreter>,
    #[serde(default)]
    parameters: HashMap<String, Parameter>,
    #[serde(default)]
    steps: IndexMap<String, Step>,
    #[serde(default)]
    results: HashMap<String, ResultRef>,
}

impl From<ChainHelper> for Chain {
    fn from(helper: ChainHelper) -> Self {
        // Start with default interpreters
        let mut interpreters: HashMap<String, Interpreter> =
            default_interpreters().into_iter().collect();

        // Override with user-provided interpreters
        interpreters.extend(helper.interpreters);

        Chain {
            name: helper.name,
            timeout: helper.timeout,
            interpreters,
            parameters: helper.parameters,
            steps: helper.steps,
            results: helper.results,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChainResult {
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

impl Default for Chain {
    fn default() -> Self {
        Self {
            name: None,
            timeout: default_chain_timeout(),
            parameters: HashMap::new(),
            interpreters: HashMap::new(),
            steps: IndexMap::new(),
            results: HashMap::new(),
        }
    }
}

impl Chain {
    fn make_output_key(step_key: &str, output_key: &str) -> String {
        format!("steps.{step_key}.outputs.{output_key}")
    }

    /// Validates the chain structure.
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
                    context: format!("chain result '{result_key}'"),
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

    fn check_timeout(&self, start_time: &Instant, step_name: &str) -> Result<u64> {
        if self.timeout == 0 {
            return Ok(0);
        }

        let elapsed = start_time.elapsed().as_secs();
        if elapsed >= self.timeout {
            return Err(AtentoError::Timeout {
                context: format!("Chain timed out before step '{step_name}'"),
                timeout_secs: self.timeout,
            });
        }

        Ok(self.timeout.saturating_sub(elapsed))
    }

    fn resolve_step_inputs(
        &self,
        step: &Step,
        step_name: &str,
        resolved_outputs: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        step.inputs
            .iter()
            .map(|(input_name, input)| {
                self.resolve_input(input_name, input, step_name, resolved_outputs)
                    .map(|val| (input_name.clone(), val))
            })
            .collect()
    }

    fn lookup_interpreter(&self, step: &Step, step_name: &str) -> Result<&Interpreter> {
        self.interpreters.get(&step.interpreter).ok_or_else(|| {
            AtentoError::Validation(format!(
                "Unknown interpreter '{}' in step '{}'",
                step.interpreter, step_name
            ))
        })
    }

    fn process_step_result(
        step_name: &str,
        step_result: &StepResult,
        resolved_outputs: &mut HashMap<String, String>,
    ) -> Option<AtentoError> {
        // Store step outputs
        for (k, v) in &step_result.outputs {
            resolved_outputs.insert(Self::make_output_key(step_name, k), v.clone());
        }

        // Check for step error
        step_result
            .error
            .as_ref()
            .map(|err| AtentoError::StepExecution {
                step: step_name.to_string(),
                reason: err.to_string(),
            })
    }

    fn collect_chain_results(
        &self,
        resolved_outputs: &HashMap<String, String>,
    ) -> (HashMap<String, String>, Vec<AtentoError>) {
        let mut final_results = HashMap::new();
        let mut errors = Vec::new();

        for (result_name, result_ref) in &self.results {
            if let Some(val) = resolved_outputs.get(&result_ref.ref_) {
                final_results.insert(result_name.clone(), val.clone());
            } else {
                errors.push(AtentoError::UnresolvedReference {
                    reference: result_ref.ref_.clone(),
                    context: format!("Unresolved Reference '{result_name}'"),
                });
            }
        }

        (final_results, errors)
    }

    fn serialize_parameters(&self) -> (Option<HashMap<String, String>>, Vec<AtentoError>) {
        if self.parameters.is_empty() {
            return (None, Vec::new());
        }

        match self
            .parameters
            .iter()
            .map(|(k, v)| v.to_string_value().map(|s| (k.clone(), s)))
            .collect::<Result<HashMap<_, _>>>()
        {
            Ok(params) => (Some(params), Vec::new()),
            Err(e) => (None, vec![e]),
        }
    }

    /// Executes the chain with a custom executor (useful for testing).
    ///
    /// # Errors
    /// Returns an error if timeout is exceeded, a step fails, or output resolution fails.
    pub fn run_with_executor<E: CommandExecutor>(&self, executor: &E) -> ChainResult {
        let start_time = Instant::now();
        let mut resolved_outputs = HashMap::new();
        let mut step_results = IndexMap::new();
        let mut chain_errors = Vec::new();

        for (step_name, step) in &self.steps {
            // Check timeout
            let time_left = match self.check_timeout(&start_time, step_name) {
                Ok(time) => time,
                Err(e) => {
                    chain_errors.push(e);
                    break;
                }
            };

            // Resolve step inputs
            let step_inputs = match self.resolve_step_inputs(step, step_name, &resolved_outputs) {
                Ok(inputs) => inputs,
                Err(e) => {
                    chain_errors.push(e);
                    break;
                }
            };

            // Lookup interpreter
            let interpreter = match self.lookup_interpreter(step, step_name) {
                Ok(interp) => interp,
                Err(e) => {
                    chain_errors.push(e);
                    break;
                }
            };

            // Run step
            let step_result = step.run(executor, &step_inputs, time_left, interpreter);

            // Process result and check for errors
            if let Some(err) =
                Self::process_step_result(step_name, &step_result, &mut resolved_outputs)
            {
                chain_errors.push(err);
                step_results.insert(step_name.clone(), step_result);
                break;
            }

            step_results.insert(step_name.clone(), step_result);
        }

        // Collect chain results and parameters
        let (final_results, mut result_errors) = self.collect_chain_results(&resolved_outputs);
        chain_errors.append(&mut result_errors);

        let (parameters, mut param_errors) = self.serialize_parameters();
        chain_errors.append(&mut param_errors);

        let status = if chain_errors.is_empty() { "ok" } else { "nok" }.to_string();

        ChainResult {
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
            errors: chain_errors,
            status,
        }
    }

    /// Executes the chain using the system executor.
    ///
    /// # Errors
    /// Returns an error if timeout is exceeded, a step fails, or output resolution fails.
    #[must_use]
    pub fn run(&self) -> ChainResult {
        use crate::executor::SystemExecutor;
        let executor = SystemExecutor;
        self.run_with_executor(&executor)
    }
}
