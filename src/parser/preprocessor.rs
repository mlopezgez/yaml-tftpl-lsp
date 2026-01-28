//! Expression placeholder substitution
//!
//! This module handles replacing Terraform ${...} and Workflows $${...}
//! expressions with valid YAML placeholders before parsing.

use lazy_static::lazy_static;
use regex::Regex;

use super::expressions::{Expression, ExpressionKind, ExpressionMap};

lazy_static! {
    // Terraform interpolation: ${...} (handles one level of nested braces)
    static ref TERRAFORM_EXPR: Regex =
        Regex::new(r#"\$\{(?:[^{}]|\{[^{}]*\})*\}"#).unwrap();

    // Workflows runtime expression: $${...} (handles one level of nested braces)
    static ref WORKFLOWS_EXPR: Regex =
        Regex::new(r#"\$\$\{(?:[^{}]|\{[^{}]*\})*\}"#).unwrap();
}

/// Preprocess a document by replacing expressions with placeholders
///
/// Returns the preprocessed text and a map of expressions for position adjustment.
pub fn preprocess_expressions(text: &str) -> (String, ExpressionMap) {
    let mut result = text.to_string();
    let mut expression_map = ExpressionMap::new();
    let mut counter = 0;

    // Process Workflows expressions first ($${}) since they're more specific
    // and we don't want the Terraform regex to partially match them
    let workflows_matches: Vec<_> = WORKFLOWS_EXPR.find_iter(text).collect();
    for mat in workflows_matches.iter().rev() {
        let placeholder = format!("__EXPR_{:03}__", counter);
        counter += 1;

        let (start_line, start_column) = offset_to_line_col(text, mat.start());
        let (end_line, end_column) = offset_to_line_col(text, mat.end());

        expression_map.add(Expression {
            original: mat.as_str().to_string(),
            placeholder: placeholder.clone(),
            start: mat.start(),
            end: mat.end(),
            start_line,
            start_column,
            end_line,
            end_column,
            kind: ExpressionKind::Workflows,
        });

        // Replace in the result string
        result.replace_range(mat.range(), &placeholder);
    }

    // Now process Terraform expressions (${})
    // We need to re-scan since positions have changed
    let result_snapshot = result.clone();
    let terraform_matches: Vec<_> = TERRAFORM_EXPR.find_iter(&result_snapshot).collect();
    for mat in terraform_matches.iter().rev() {
        // Skip if this is actually part of a $${} that wasn't fully replaced
        // (This shouldn't happen with our regex, but be defensive)
        if mat.start() > 0 && result_snapshot.as_bytes().get(mat.start() - 1) == Some(&b'$') {
            continue;
        }

        let placeholder = format!("__EXPR_{:03}__", counter);
        counter += 1;

        // For Terraform expressions, we calculate positions in the partially-processed text
        // This is a simplification; a full implementation would track position offsets
        let (start_line, start_column) = offset_to_line_col(&result_snapshot, mat.start());
        let (end_line, end_column) = offset_to_line_col(&result_snapshot, mat.end());

        expression_map.add(Expression {
            original: mat.as_str().to_string(),
            placeholder: placeholder.clone(),
            start: mat.start(),
            end: mat.end(),
            start_line,
            start_column,
            end_line,
            end_column,
            kind: ExpressionKind::Terraform,
        });

        result.replace_range(mat.range(), &placeholder);
    }

    (result, expression_map)
}

/// Convert a byte offset to (line, column) coordinates
fn offset_to_line_col(text: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;

    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_terraform_expression() {
        let input = "value: ${var.name}";
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert!(!result.contains("${"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(map.expressions[0].kind, ExpressionKind::Terraform);
    }

    #[test]
    fn test_preprocess_workflows_expression() {
        let input = "value: $${sys.now()}";
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert!(!result.contains("$${"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(map.expressions[0].kind, ExpressionKind::Workflows);
    }

    #[test]
    fn test_preprocess_nested_braces() {
        let input = "value: ${jsonencode({key: \"value\"})}";
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
    }

    #[test]
    fn test_preprocess_multiple_expressions() {
        let input = "a: ${var.a}\nb: $${sys.get_env(\"KEY\")}";
        let (result, map) = preprocess_expressions(input);

        assert!(!result.contains("${"));
        assert!(!result.contains("$${"));
        assert_eq!(map.expressions.len(), 2);
    }

    #[test]
    fn test_offset_to_line_col() {
        let text = "line1\nline2\nline3";
        assert_eq!(offset_to_line_col(text, 0), (0, 0));
        assert_eq!(offset_to_line_col(text, 5), (0, 5));
        assert_eq!(offset_to_line_col(text, 6), (1, 0));
        assert_eq!(offset_to_line_col(text, 10), (1, 4));
    }
}
