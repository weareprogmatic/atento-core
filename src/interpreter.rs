use serde::{Deserialize, Serialize};

/// Interpreter configuration with command, arguments, and file extension
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Interpreter {
    /// The command to execute (e.g., "bash", "node", "/usr/bin/python3")
    pub command: String,
    /// Additional arguments to pass before the script file (not including the command)
    #[serde(default)]
    pub args: Vec<String>,
    /// File extension for the script (e.g., ".sh", ".js")
    pub extension: String,
}

/// Returns the default interpreter configurations as (key, Interpreter) pairs
#[must_use]
pub fn default_interpreters() -> Vec<(String, Interpreter)> {
    vec![
        (
            "bash".to_string(),
            Interpreter {
                command: "bash".to_string(),
                args: vec![],
                extension: ".sh".to_string(),
            },
        ),
        (
            "batch".to_string(),
            Interpreter {
                command: "cmd".to_string(),
                args: vec!["/c".to_string()],
                extension: ".bat".to_string(),
            },
        ),
        (
            "powershell".to_string(),
            Interpreter {
                command: "powershell".to_string(),
                args: vec![
                    "-NoLogo".to_string(),
                    "-NoProfile".to_string(),
                    "-NonInteractive".to_string(),
                    "-ExecutionPolicy".to_string(),
                    "Bypass".to_string(),
                    "-File".to_string(),
                ],
                extension: ".ps1".to_string(),
            },
        ),
        (
            "pwsh".to_string(),
            Interpreter {
                command: "pwsh".to_string(),
                args: vec![
                    "-NoLogo".to_string(),
                    "-NoProfile".to_string(),
                    "-NonInteractive".to_string(),
                    "-ExecutionPolicy".to_string(),
                    "Bypass".to_string(),
                    "-File".to_string(),
                ],
                extension: ".ps1".to_string(),
            },
        ),
        (
            "python".to_string(),
            Interpreter {
                command: "python3".to_string(),
                args: vec![],
                extension: ".py".to_string(),
            },
        ),
        (
            "python3".to_string(),
            Interpreter {
                command: "python3".to_string(),
                args: vec![],
                extension: ".py".to_string(),
            },
        ),
    ]
}

impl Interpreter {
    /// Returns the file extension associated with the interpreter
    #[must_use]
    pub fn extension(&self) -> &str {
        &self.extension
    }

    /// Returns the full command and arguments as a vector of strings
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.command.is_empty() && !self.extension.is_empty()
    }
}
