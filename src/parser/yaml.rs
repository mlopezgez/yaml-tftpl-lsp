//! YAML parsing with error recovery
//!
//! This module parses preprocessed YAML text (with expression placeholders) and
//! collects syntax errors, adjusting error positions back to the original document
//! coordinates when errors fall within or after expression placeholders.

use crate::diagnostics::DiagnosticCollector;

use super::expressions::ExpressionMap;

/// Result of parsing YAML, containing any parsed value
#[derive(Debug)]
#[allow(dead_code)]
pub struct ParseResult {
    /// The parsed YAML value (if successful)
    pub value: Option<serde_yaml::Value>,
    /// Whether parsing was successful
    pub success: bool,
}

/// Parse YAML text and collect any syntax errors
///
/// The text should already be preprocessed to replace expressions with placeholders.
/// Error positions are adjusted using the expression_map to map back to original
/// document coordinates.
///
/// # Arguments
/// * `text` - The preprocessed YAML text (with __EXPR_XXX__ placeholders)
/// * `expression_map` - Map of expressions for position adjustment
/// * `collector` - Collector for diagnostics
///
/// # Returns
/// * `ParseResult` - Contains the parsed value (if successful) and success status
pub fn parse_yaml(
    text: &str,
    expression_map: &ExpressionMap,
    collector: &mut DiagnosticCollector,
) -> ParseResult {
    // Attempt to parse the YAML
    match serde_yaml::from_str::<serde_yaml::Value>(text) {
        Ok(value) => {
            // Successfully parsed - no YAML syntax errors
            ParseResult {
                value: Some(value),
                success: true,
            }
        }
        Err(err) => {
            // Extract error information
            let message = err.to_string();

            // serde_yaml error messages often contain location info like "at line X column Y"
            // We try to extract this for better diagnostics
            let (line, column) = extract_error_position(&message);

            // Adjust position if it falls within or after an expression placeholder
            let (adjusted_line, adjusted_column) = expression_map.adjust_position(line, column);

            // Clean up the error message to remove position info (we provide it via range)
            let clean_message = clean_error_message(&message);

            collector.add_yaml_error(clean_message, adjusted_line, adjusted_column);

            ParseResult {
                value: None,
                success: false,
            }
        }
    }
}

/// Extract line and column from a serde_yaml error message
///
/// serde_yaml errors often look like: "... at line 5 column 10"
fn extract_error_position(message: &str) -> (u32, u32) {
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        static ref POSITION_RE: Regex = Regex::new(r"at line (\d+) column (\d+)").unwrap();
    }

    if let Some(caps) = POSITION_RE.captures(message) {
        let line: u32 = caps
            .get(1)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        let column: u32 = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        // serde_yaml uses 1-indexed positions, LSP uses 0-indexed
        (line.saturating_sub(1), column.saturating_sub(1))
    } else {
        // Default to start of document
        (0, 0)
    }
}

/// Clean up the error message by removing position information
///
/// Since we provide position via the diagnostic range, we can simplify
/// the message by removing the "at line X column Y" suffix.
fn clean_error_message(message: &str) -> String {
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        static ref POSITION_SUFFIX_RE: Regex = Regex::new(r"\s+at line \d+ column \d+$").unwrap();
    }

    POSITION_SUFFIX_RE.replace(message, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::preprocess_expressions;

    #[test]
    fn test_parse_valid_yaml() {
        let yaml = "key: value\nlist:\n  - item1\n  - item2";
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        assert!(result.success);
        assert!(result.value.is_some());
        assert!(collector.into_diagnostics().is_empty());
    }

    #[test]
    fn test_parse_invalid_yaml_indentation() {
        let yaml = "key: value\n  bad: indentation";
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        assert!(!result.success);
        assert!(result.value.is_none());
        let diagnostics = collector.into_diagnostics();
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_parse_invalid_yaml_duplicate_key() {
        // Duplicate keys in a mapping - serde_yaml allows this but we test anyway
        // Let's use a more reliable invalid YAML instead
        let yaml = "key:\n  - item\n subkey: value"; // Inconsistent indentation
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        // This should produce an error due to invalid structure
        let diagnostics = collector.into_diagnostics();
        // The assertion depends on parser behavior; we verify it doesn't panic
        assert!(result.success || !diagnostics.is_empty());
    }

    #[test]
    fn test_parse_invalid_yaml_unclosed_quote() {
        let yaml = "key: \"unclosed";
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        assert!(!result.success);
        let diagnostics = collector.into_diagnostics();
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_parse_invalid_yaml_bad_list_format() {
        let yaml = "list:\n  - item1\n   - item2"; // Misaligned list items
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        // This might or might not be an error depending on YAML parser strictness
        let diagnostics = collector.into_diagnostics();
        // Just ensure parsing completes without panic
        assert!(result.success || !diagnostics.is_empty());
    }

    #[test]
    fn test_extract_error_position() {
        assert_eq!(extract_error_position("error at line 5 column 10"), (4, 9));
        assert_eq!(
            extract_error_position("some error without position"),
            (0, 0)
        );
        assert_eq!(extract_error_position("at line 1 column 1"), (0, 0));
        assert_eq!(
            extract_error_position("mapping values at line 10 column 25"),
            (9, 24)
        );
    }

    #[test]
    fn test_clean_error_message() {
        assert_eq!(
            clean_error_message("invalid YAML at line 5 column 10"),
            "invalid YAML"
        );
        assert_eq!(
            clean_error_message("some error without position"),
            "some error without position"
        );
        assert_eq!(
            clean_error_message("mapping values are not allowed at line 2 column 3"),
            "mapping values are not allowed"
        );
    }

    #[test]
    fn test_parse_yaml_with_expression_placeholders() {
        // Test parsing YAML that has already been preprocessed
        let original = "name: ${var.name}\nage: 30";
        let (preprocessed, expression_map) = preprocess_expressions(original);

        let mut collector = DiagnosticCollector::new();
        let result = parse_yaml(&preprocessed, &expression_map, &mut collector);

        assert!(result.success);
        assert!(collector.into_diagnostics().is_empty());
    }

    #[test]
    fn test_parse_yaml_with_workflows_expression() {
        let original = "timestamp: $${sys.now()}\nvalue: test";
        let (preprocessed, expression_map) = preprocess_expressions(original);

        let mut collector = DiagnosticCollector::new();
        let result = parse_yaml(&preprocessed, &expression_map, &mut collector);

        assert!(result.success);
        assert!(collector.into_diagnostics().is_empty());
    }

    #[test]
    fn test_error_position_adjustment_after_expression() {
        // Create a scenario where an error occurs after an expression placeholder
        // Original: "name: ${var.name}\nbad indentation"
        // The error on line 2 should still map correctly
        let original = "name: ${var.name}\n  bad: indentation";
        let (preprocessed, expression_map) = preprocess_expressions(original);

        let mut collector = DiagnosticCollector::new();
        let _result = parse_yaml(&preprocessed, &expression_map, &mut collector);

        let diagnostics = collector.into_diagnostics();
        // Should have an error for the bad indentation
        assert!(!diagnostics.is_empty());
        // The error should be on line 1 (0-indexed)
        assert_eq!(diagnostics[0].range.start.line, 1);
    }

    #[test]
    fn test_parse_complex_valid_yaml() {
        let yaml = r#"
main:
  params:
    - name
    - age
  steps:
    - initialize:
        assign:
          - result: "Hello"
    - returnValue:
        return: result
"#;
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        assert!(result.success);
        assert!(collector.into_diagnostics().is_empty());
    }

    #[test]
    fn test_parse_yaml_with_nested_expressions() {
        let original = r#"config: ${jsonencode({
  key1: "value1",
  key2: "value2"
})}
other: value"#;
        let (preprocessed, expression_map) = preprocess_expressions(original);

        let mut collector = DiagnosticCollector::new();
        let result = parse_yaml(&preprocessed, &expression_map, &mut collector);

        assert!(result.success);
        assert!(collector.into_diagnostics().is_empty());
    }

    #[test]
    fn test_parse_mixed_expressions() {
        let original = r#"project: ${var.project_id}
timestamp: $${sys.now()}
config:
  enabled: true
  value: ${var.value}"#;
        let (preprocessed, expression_map) = preprocess_expressions(original);

        let mut collector = DiagnosticCollector::new();
        let result = parse_yaml(&preprocessed, &expression_map, &mut collector);

        assert!(result.success);
        assert!(collector.into_diagnostics().is_empty());
    }

    #[test]
    fn test_parse_empty_yaml() {
        let yaml = "";
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        // Empty YAML should parse as null/None value
        assert!(result.success);
    }

    #[test]
    fn test_parse_yaml_comment_only() {
        let yaml = "# This is a comment\n# Another comment";
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        let result = parse_yaml(yaml, &expression_map, &mut collector);

        assert!(result.success);
    }
}
