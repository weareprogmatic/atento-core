#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::result_ref::ResultRef;

    #[test]
    fn test_result_ref_creation() {
        let result_ref = ResultRef {
            ref_: "steps.build.outputs.artifact".to_string(),
        };
        assert_eq!(result_ref.ref_, "steps.build.outputs.artifact");
    }

    #[test]
    fn test_result_ref_clone() {
        let result_ref = ResultRef {
            ref_: "steps.test.outputs.status".to_string(),
        };
        let cloned = result_ref.clone();
        assert_eq!(result_ref.ref_, cloned.ref_);
    }

    #[test]
    fn test_result_ref_debug() {
        let result_ref = ResultRef {
            ref_: "steps.deploy.outputs.url".to_string(),
        };
        let debug = format!("{result_ref:?}");
        assert!(debug.contains("ResultRef"));
        assert!(debug.contains("steps.deploy.outputs.url"));
    }

    #[test]
    fn test_result_ref_deserialize() {
        let yaml = r"
ref: steps.foo.outputs.bar
";
        let result_ref: ResultRef = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(result_ref.ref_, "steps.foo.outputs.bar");
    }

    #[test]
    fn test_result_ref_deserialize_json() {
        let json = r#"{"ref": "steps.build.outputs.version"}"#;
        let result_ref: ResultRef = serde_json::from_str(json).unwrap();
        assert_eq!(result_ref.ref_, "steps.build.outputs.version");
    }

    #[test]
    fn test_result_ref_empty_string() {
        let result_ref = ResultRef {
            ref_: String::new(),
        };
        assert_eq!(result_ref.ref_, "");
    }

    #[test]
    fn test_result_ref_with_dots() {
        let result_ref = ResultRef {
            ref_: "steps.step1.outputs.data.field".to_string(),
        };
        assert!(result_ref.ref_.contains('.'));
    }

    #[test]
    fn test_result_ref_with_underscores() {
        let result_ref = ResultRef {
            ref_: "steps.my_step.outputs.my_output".to_string(),
        };
        assert!(result_ref.ref_.contains('_'));
    }

    #[test]
    fn test_result_ref_with_numbers() {
        let result_ref = ResultRef {
            ref_: "steps.step123.outputs.output456".to_string(),
        };
        assert!(result_ref.ref_.contains("123"));
    }

    #[test]
    fn test_result_ref_long_path() {
        let result_ref = ResultRef {
            ref_: "steps.very.long.path.to.some.output.value".to_string(),
        };
        assert!(result_ref.ref_.len() > 20);
    }
}
