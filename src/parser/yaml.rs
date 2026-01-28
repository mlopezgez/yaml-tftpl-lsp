//! YAML parsing with error recovery

use crate::diagnostics::DiagnosticCollector;

use super::expressions::ExpressionMap;

/// Parse YAML text and collect any syntax errors
///
/// The text should already be preprocessed to replace expressions with placeholders.
pub fn parse_yaml(
    text: &str,
    _expression_map: &ExpressionMap,
    collector: &mut DiagnosticCollector,
) {
    // Attempt to parse the YAML
    match serde_yaml::from_str::<serde_yaml::Value>(text) {
        Ok(_value) => {
            // Successfully parsed - no YAML syntax errors
            // In later phases, we'll validate the workflow structure here
        }
        Err(err) => {
            // Extract error information
            let message = err.to_string();

            // serde_yaml error messages often contain location info like "at line X column Y"
            // We try to extract this for better diagnostics
            let (line, column) = extract_error_position(&message);

            collector.add_yaml_error(message, line, column);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_yaml() {
        let yaml = "key: value\nlist:\n  - item1\n  - item2";
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        parse_yaml(yaml, &expression_map, &mut collector);

        assert!(collector.into_diagnostics().is_empty());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "key: value\n  bad: indentation\n wrong";
        let expression_map = ExpressionMap::new();
        let mut collector = DiagnosticCollector::new();

        parse_yaml(yaml, &expression_map, &mut collector);

        let diagnostics = collector.into_diagnostics();
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_extract_error_position() {
        assert_eq!(extract_error_position("error at line 5 column 10"), (4, 9));
        assert_eq!(
            extract_error_position("some error without position"),
            (0, 0)
        );
    }
}
