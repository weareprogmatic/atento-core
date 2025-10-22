#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn test_interpreter_extension() {
        assert_eq!(Interpreter::Bash.extension(), ".sh");
        assert_eq!(Interpreter::Batch.extension(), ".bat");
        assert_eq!(Interpreter::PowerShell.extension(), ".ps1");
        assert_eq!(Interpreter::Pwsh.extension(), ".ps1");
        assert_eq!(Interpreter::Python.extension(), ".py");
    }

    #[test]
    fn test_interpreter_args() {
        assert_eq!(Interpreter::Bash.args(), vec!["bash"]);
        assert_eq!(Interpreter::Batch.args(), vec!["cmd", "/c"]);
        assert_eq!(
            Interpreter::PowerShell.args(),
            vec![
                "powershell",
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File"
            ]
        );
        assert_eq!(
            Interpreter::Pwsh.args(),
            vec![
                "pwsh",
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File"
            ]
        );
        assert_eq!(Interpreter::Python.args(), vec!["python3"]);
    }

    #[test]
    fn test_interpreter_display() {
        assert_eq!(format!("{}", Interpreter::Bash), "script::bash");
        assert_eq!(format!("{}", Interpreter::Batch), "script::batch");
        assert_eq!(format!("{}", Interpreter::PowerShell), "script::powershell");
        assert_eq!(format!("{}", Interpreter::Pwsh), "script::pwsh");
        assert_eq!(format!("{}", Interpreter::Python), "script::python");
    }

    #[test]
    fn test_interpreter_debug() {
        let bash = Interpreter::Bash;
        let debug_str = format!("{bash:?}");
        assert!(debug_str.contains("Bash"));
    }

    #[test]
    fn test_interpreter_clone() {
        let original = Interpreter::Python;
        let cloned = original.clone();
        assert!(matches!(cloned, Interpreter::Python));
        assert_eq!(original.extension(), cloned.extension());
        assert_eq!(original.args(), cloned.args());
    }

    #[test]
    fn test_interpreter_bash_properties() {
        let bash_interpreter = Interpreter::Bash;
        assert_eq!(bash_interpreter.extension(), ".sh");
        assert_eq!(bash_interpreter.args(), vec!["bash"]);
    }

    #[test]
    fn test_interpreter_serialization() {
        let bash_json = serde_json::to_string(&Interpreter::Bash).unwrap();
        assert_eq!(bash_json, "\"script::bash\"");

        let batch_json = serde_json::to_string(&Interpreter::Batch).unwrap();
        assert_eq!(batch_json, "\"script::batch\"");

        let powershell_json = serde_json::to_string(&Interpreter::PowerShell).unwrap();
        assert_eq!(powershell_json, "\"script::powershell\"");

        let pwsh_json = serde_json::to_string(&Interpreter::Pwsh).unwrap();
        assert_eq!(pwsh_json, "\"script::pwsh\"");

        let python_json = serde_json::to_string(&Interpreter::Python).unwrap();
        assert_eq!(python_json, "\"script::python\"");
    }

    #[test]
    fn test_interpreter_deserialization() {
        let bash: Interpreter = serde_json::from_str("\"script::bash\"").unwrap();
        assert!(matches!(bash, Interpreter::Bash));

        let batch: Interpreter = serde_json::from_str("\"script::batch\"").unwrap();
        assert!(matches!(batch, Interpreter::Batch));

        let powershell: Interpreter = serde_json::from_str("\"script::powershell\"").unwrap();
        assert!(matches!(powershell, Interpreter::PowerShell));

        let pwsh: Interpreter = serde_json::from_str("\"script::pwsh\"").unwrap();
        assert!(matches!(pwsh, Interpreter::Pwsh));

        let python: Interpreter = serde_json::from_str("\"script::python\"").unwrap();
        assert!(matches!(python, Interpreter::Python));
    }

    #[test]
    fn test_interpreter_deserialization_error() {
        let result: Result<Interpreter, _> = serde_json::from_str("\"script::invalid\"");
        assert!(result.is_err());

        let result: Result<Interpreter, _> = serde_json::from_str("\"invalid_format\"");
        assert!(result.is_err());

        let result: Result<Interpreter, _> = serde_json::from_str("123");
        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_yaml_serialization() {
        let bash_yaml = serde_yaml::to_string(&Interpreter::Bash).unwrap();
        assert_eq!(bash_yaml.trim(), "script::bash");

        let python_yaml = serde_yaml::to_string(&Interpreter::Python).unwrap();
        assert_eq!(python_yaml.trim(), "script::python");
    }

    #[test]
    fn test_interpreter_yaml_deserialization() {
        let bash: Interpreter = serde_yaml::from_str("script::bash").unwrap();
        assert!(matches!(bash, Interpreter::Bash));

        let python: Interpreter = serde_yaml::from_str("script::python").unwrap();
        assert!(matches!(python, Interpreter::Python));
    }

    #[test]
    fn test_interpreter_yaml_deserialization_error() {
        let result: Result<Interpreter, _> = serde_yaml::from_str("script::invalid");
        assert!(result.is_err());

        let result: Result<Interpreter, _> = serde_yaml::from_str("invalid_format");
        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_extension_consistency() {
        // Test that PowerShell and Pwsh have the same extension
        assert_eq!(
            Interpreter::PowerShell.extension(),
            Interpreter::Pwsh.extension()
        );

        // Test that all extensions start with a dot
        for interpreter in [
            Interpreter::Bash,
            Interpreter::Batch,
            Interpreter::PowerShell,
            Interpreter::Pwsh,
            Interpreter::Python,
        ] {
            assert!(interpreter.extension().starts_with('.'));
            assert!(!interpreter.extension().is_empty());
        }
    }

    #[test]
    fn test_interpreter_args_consistency() {
        // Test that all interpreters have non-empty args
        for interpreter in [
            Interpreter::Bash,
            Interpreter::Batch,
            Interpreter::PowerShell,
            Interpreter::Pwsh,
            Interpreter::Python,
        ] {
            let args = interpreter.args();
            assert!(!args.is_empty());
            assert!(!args[0].is_empty()); // First arg should be the command
        }
    }

    #[test]
    fn test_interpreter_display_consistency() {
        // Test that all display strings start with "script::"
        for interpreter in [
            Interpreter::Bash,
            Interpreter::Batch,
            Interpreter::PowerShell,
            Interpreter::Pwsh,
            Interpreter::Python,
        ] {
            let display = format!("{interpreter}");
            assert!(display.starts_with("script::"));
            assert!(display.len() > 8); // More than just "script::"
        }
    }

    #[test]
    fn test_interpreter_roundtrip_json() {
        for interpreter in [
            Interpreter::Bash,
            Interpreter::Batch,
            Interpreter::PowerShell,
            Interpreter::Pwsh,
            Interpreter::Python,
        ] {
            let json = serde_json::to_string(&interpreter).unwrap();
            let deserialized: Interpreter = serde_json::from_str(&json).unwrap();

            // Check that they're the same variant
            assert_eq!(
                std::mem::discriminant(&interpreter),
                std::mem::discriminant(&deserialized)
            );

            // Check that properties are the same
            assert_eq!(interpreter.extension(), deserialized.extension());
            assert_eq!(interpreter.args(), deserialized.args());
            assert_eq!(format!("{interpreter}"), format!("{}", deserialized));
        }
    }

    #[test]
    fn test_interpreter_roundtrip_yaml() {
        for interpreter in [
            Interpreter::Bash,
            Interpreter::Batch,
            Interpreter::PowerShell,
            Interpreter::Pwsh,
            Interpreter::Python,
        ] {
            let yaml = serde_yaml::to_string(&interpreter).unwrap();
            let deserialized: Interpreter = serde_yaml::from_str(&yaml).unwrap();

            // Check that they're the same variant
            assert_eq!(
                std::mem::discriminant(&interpreter),
                std::mem::discriminant(&deserialized)
            );
        }
    }

    #[test]
    fn test_interpreter_edge_cases() {
        // Test that extensions are lowercase
        for interpreter in [
            Interpreter::Bash,
            Interpreter::Batch,
            Interpreter::PowerShell,
            Interpreter::Pwsh,
            Interpreter::Python,
        ] {
            let ext = interpreter.extension();
            assert_eq!(ext, ext.to_lowercase());
        }

        // Test that display format is lowercase after "script::"
        for interpreter in [
            Interpreter::Bash,
            Interpreter::Batch,
            Interpreter::PowerShell,
            Interpreter::Pwsh,
            Interpreter::Python,
        ] {
            let display = format!("{interpreter}");
            let suffix = &display[8..]; // After "script::"
            assert_eq!(suffix, suffix.to_lowercase());
        }
    }
}
