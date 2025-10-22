use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Interpreter {
    #[serde(rename = "script::bash")]
    Bash,
    #[serde(rename = "script::batch")]
    Batch,
    #[serde(rename = "script::powershell")]
    PowerShell,
    #[serde(rename = "script::pwsh")]
    Pwsh,
    #[serde(rename = "script::python")]
    Python,
}

impl Interpreter {
    pub fn extension(&self) -> &'static str {
        match self {
            Interpreter::Bash => ".sh",
            Interpreter::Batch => ".bat",
            Interpreter::PowerShell | Interpreter::Pwsh => ".ps1",
            Interpreter::Python => ".py",
        }
    }

    pub fn args(&self) -> Vec<&'static str> {
        match self {
            Interpreter::Bash => vec!["bash"],
            Interpreter::Batch => vec!["cmd", "/c"],
            Interpreter::PowerShell => vec![
                "powershell",
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ],
            Interpreter::Pwsh => vec![
                "pwsh",
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ],
            Interpreter::Python => vec!["python3"],
        }
    }
}

impl fmt::Display for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "script::{}",
            match self {
                Interpreter::Bash => "bash",
                Interpreter::Batch => "batch",
                Interpreter::PowerShell => "powershell",
                Interpreter::Pwsh => "pwsh",
                Interpreter::Python => "python",
            }
        )
    }
}
