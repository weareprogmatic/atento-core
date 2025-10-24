#[cfg(test)]
mod tests {
    use crate::interpreter::{Interpreter, default_interpreters};

    #[test]
    fn test_interpreter_extension_method() {
        let interp = Interpreter {
            command: "bash".to_string(),
            args: vec![],
            extension: ".sh".to_string(),
        };
        assert_eq!(interp.extension(), ".sh");
    }

    #[test]
    fn test_interpreter_is_valid_true() {
        let interp = Interpreter {
            command: "bash".to_string(),
            args: vec![],
            extension: ".sh".to_string(),
        };
        assert!(interp.is_valid());
    }

    #[test]
    fn test_interpreter_is_valid_empty_command() {
        let interp = Interpreter {
            command: String::new(),
            args: vec![],
            extension: ".sh".to_string(),
        };
        assert!(!interp.is_valid());
    }

    #[test]
    fn test_interpreter_is_valid_empty_extension() {
        let interp = Interpreter {
            command: "bash".to_string(),
            args: vec![],
            extension: String::new(),
        };
        assert!(!interp.is_valid());
    }

    #[test]
    fn test_default_interpreters_returns_vec() {
        let interpreters = default_interpreters();
        assert_eq!(interpreters.len(), 6);

        // Verify keys
        let keys: Vec<&String> = interpreters.iter().map(|(k, _)| k).collect();
        assert!(keys.contains(&&"bash".to_string()));
        assert!(keys.contains(&&"batch".to_string()));
        assert!(keys.contains(&&"powershell".to_string()));
        assert!(keys.contains(&&"pwsh".to_string()));
        assert!(keys.contains(&&"python".to_string()));
        assert!(keys.contains(&&"python3".to_string()));
    }

    #[test]
    fn test_default_interpreters_bash_config() {
        let interpreters = default_interpreters();
        let bash = interpreters
            .iter()
            .find(|(k, _)| k == "bash")
            .map(|(_, v)| v);
        if let Some(bash) = bash {
            assert_eq!(bash.command, "bash");
            assert_eq!(bash.extension, ".sh");
            assert!(bash.args.is_empty());
        } else {
            panic!("bash interpreter should exist in defaults");
        }
    }

    #[test]
    fn test_default_interpreters_batch_config() {
        let interpreters = default_interpreters();
        let batch = interpreters
            .iter()
            .find(|(k, _)| k == "batch")
            .map(|(_, v)| v);
        if let Some(batch) = batch {
            assert_eq!(batch.command, "cmd");
            assert_eq!(batch.extension, ".bat");
            assert_eq!(batch.args, vec!["/c"]);
        } else {
            panic!("batch interpreter should exist in defaults");
        }
    }

    #[test]
    fn test_default_interpreters_all_valid() {
        let interpreters = default_interpreters();
        for (key, interp) in &interpreters {
            assert!(interp.is_valid(), "Interpreter '{key}' should be valid");
        }
    }
}
