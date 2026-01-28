//! Expression placeholder substitution
//!
//! This module handles replacing Terraform ${...} and Workflows $${...}
//! expressions with valid YAML placeholders before parsing.
//!
//! Uses a custom brace-matching algorithm to properly handle deeply nested
//! expressions like ${jsonencode({a: {b: {c: "value"}}})}

use super::expressions::{Expression, ExpressionKind, ExpressionMap};

/// Represents a match found by the expression scanner
#[derive(Debug, Clone)]
struct ExpressionMatch {
    start: usize,
    end: usize,
    text: String,
    kind: ExpressionKind,
}

/// Scan text for Terraform ${...} and Workflows $${...} expressions
/// using proper brace matching to handle arbitrary nesting depth.
fn scan_expressions(text: &str) -> Vec<ExpressionMatch> {
    let mut matches = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Check for $${...} (Workflows) first - more specific pattern
        if i + 2 < len && bytes[i] == b'$' && bytes[i + 1] == b'$' && bytes[i + 2] == b'{' {
            if let Some(end) = find_matching_brace(text, i + 2) {
                matches.push(ExpressionMatch {
                    start: i,
                    end,
                    text: text[i..end].to_string(),
                    kind: ExpressionKind::Workflows,
                });
                i = end;
                continue;
            }
        }
        // Check for ${...} (Terraform) - but not if preceded by another $
        else if i + 1 < len && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            // Make sure this isn't part of a $${
            if i > 0 && bytes[i - 1] == b'$' {
                i += 1;
                continue;
            }
            if let Some(end) = find_matching_brace(text, i + 1) {
                matches.push(ExpressionMatch {
                    start: i,
                    end,
                    text: text[i..end].to_string(),
                    kind: ExpressionKind::Terraform,
                });
                i = end;
                continue;
            }
        }
        i += 1;
    }

    matches
}

/// Find the matching closing brace for an opening brace at position `open_pos`.
/// Returns the end position (exclusive).
/// Handles nested braces, string literals (with escaped quotes), and multi-line content.
fn find_matching_brace(text: &str, open_pos: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(open_pos) != Some(&b'{') {
        return None;
    }

    let mut depth = 0;
    let mut i = open_pos;
    let len = bytes.len();

    while i < len {
        let ch = bytes[i];

        match ch {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            // Handle double-quoted strings - skip their contents
            b'"' => {
                i += 1;
                while i < len {
                    match bytes[i] {
                        b'\\' => i += 2, // Skip escaped character
                        b'"' => break,
                        _ => i += 1,
                    }
                    if i >= len {
                        break;
                    }
                }
            }
            // Handle single-quoted strings - skip their contents
            b'\'' => {
                i += 1;
                while i < len {
                    match bytes[i] {
                        b'\\' => i += 2, // Skip escaped character
                        b'\'' => break,
                        _ => i += 1,
                    }
                    if i >= len {
                        break;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Unclosed brace
    None
}

/// Preprocess a document by replacing expressions with placeholders
///
/// Returns the preprocessed text and a map of expressions for position adjustment.
pub fn preprocess_expressions(text: &str) -> (String, ExpressionMap) {
    let mut expression_map = ExpressionMap::new();

    // Scan for all expressions using our brace-matching algorithm
    let matches = scan_expressions(text);

    if matches.is_empty() {
        return (text.to_string(), expression_map);
    }

    // Build the result string by replacing matches from end to start
    // (to preserve offsets for earlier matches)
    let mut result = text.to_string();

    // Process matches in reverse order to preserve positions
    for (counter, mat) in matches.iter().rev().enumerate() {
        let placeholder = format!("__EXPR_{:03}__", counter);

        let (start_line, start_column) = offset_to_line_col(text, mat.start);
        let (end_line, end_column) = offset_to_line_col(text, mat.end);

        expression_map.add(Expression {
            original: mat.text.clone(),
            placeholder: placeholder.clone(),
            start: mat.start,
            end: mat.end,
            start_line,
            start_column,
            end_line,
            end_column,
            kind: mat.kind,
        });

        result.replace_range(mat.start..mat.end, &placeholder);
    }

    // Finalize the expression map to build position deltas
    expression_map.finalize();

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

    // === Edge case tests for Phase 2 ===

    #[test]
    fn test_deeply_nested_braces() {
        // Test deeply nested braces like ${jsonencode({a: {b: {c: "value"}}})}
        let input = r#"config: ${jsonencode({a: {b: {c: "value"}}})}"#;
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert!(!result.contains("${"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(
            map.expressions[0].original,
            r#"${jsonencode({a: {b: {c: "value"}}})}"#
        );
    }

    #[test]
    fn test_escaped_quotes_in_expression() {
        // Test expressions with escaped quotes inside strings
        let input = r#"value: ${format("Hello \"world\"")}"#;
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(
            map.expressions[0].original,
            r#"${format("Hello \"world\"")}"#
        );
    }

    #[test]
    fn test_multiline_expression() {
        // Test multi-line expression
        let input = r#"config: ${jsonencode({
  key1: "value1",
  key2: "value2"
})}"#;
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
        assert!(map.expressions[0].original.contains("key1"));
        assert!(map.expressions[0].original.contains("key2"));
        // The expression should span multiple lines
        assert!(map.expressions[0].start_line < map.expressions[0].end_line);
    }

    #[test]
    fn test_expression_inside_yaml_string() {
        // Test expressions as part of YAML quoted strings
        let input = r#"message: "Hello ${var.name}!""#;
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(map.expressions[0].original, "${var.name}");
    }

    #[test]
    fn test_workflows_with_function_call() {
        // Test Workflows expressions with function calls
        let input = r#"time: $${sys.now()}"#;
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(map.expressions[0].kind, ExpressionKind::Workflows);
        assert_eq!(map.expressions[0].original, "$${sys.now()}");
    }

    #[test]
    fn test_workflows_with_nested_expression() {
        // Test Workflows expressions with nested content
        let input = r#"result: $${map.get(data, "key")}"#;
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(map.expressions[0].kind, ExpressionKind::Workflows);
    }

    #[test]
    fn test_mixed_terraform_and_workflows() {
        // Test document with both Terraform and Workflows expressions
        let input = r#"project: ${var.project_id}
timestamp: $${sys.now()}
config: ${jsonencode(var.config)}
value: $${data.get("key")}"#;
        let (result, map) = preprocess_expressions(input);

        assert!(!result.contains("${"));
        assert!(!result.contains("$${"));
        assert_eq!(map.expressions.len(), 4);

        // Check we got both types
        let terraform_count = map
            .expressions
            .iter()
            .filter(|e| e.kind == ExpressionKind::Terraform)
            .count();
        let workflows_count = map
            .expressions
            .iter()
            .filter(|e| e.kind == ExpressionKind::Workflows)
            .count();
        assert_eq!(terraform_count, 2);
        assert_eq!(workflows_count, 2);
    }

    #[test]
    fn test_unclosed_brace_not_matched() {
        // Unclosed braces should not be matched as expressions
        let input = "value: ${var.name";
        let (_result, map) = preprocess_expressions(input);

        // Should not find any complete expressions
        assert_eq!(map.expressions.len(), 0);
    }

    #[test]
    fn test_no_expressions() {
        // Plain YAML without expressions
        let input = "key: value\nlist:\n  - item1\n  - item2";
        let (result, map) = preprocess_expressions(input);

        assert_eq!(result, input);
        assert_eq!(map.expressions.len(), 0);
    }

    #[test]
    fn test_expression_at_start_of_line() {
        let input = "${var.value}: key";
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
        assert_eq!(map.expressions[0].start_column, 0);
    }

    #[test]
    fn test_multiple_expressions_same_line() {
        let input = "args: [${var.a}, ${var.b}, ${var.c}]";
        let (result, map) = preprocess_expressions(input);

        assert!(!result.contains("${"));
        assert_eq!(map.expressions.len(), 3);

        // All on line 0
        for expr in &map.expressions {
            assert_eq!(expr.start_line, 0);
        }
    }

    #[test]
    fn test_single_quoted_strings_in_expression() {
        let input = r#"value: ${format('Hello %s', var.name)}"#;
        let (result, map) = preprocess_expressions(input);

        assert!(result.contains("__EXPR_"));
        assert_eq!(map.expressions.len(), 1);
    }

    #[test]
    fn test_expression_positions() {
        let input = "name: ${var.name}\nage: ${var.age}";
        let (_, map) = preprocess_expressions(input);

        assert_eq!(map.expressions.len(), 2);

        // First expression on line 0
        let first = &map.expressions[0];
        assert_eq!(first.start_line, 0);
        assert_eq!(first.start_column, 6); // after "name: "

        // Second expression on line 1
        let second = &map.expressions[1];
        assert_eq!(second.start_line, 1);
        assert_eq!(second.start_column, 5); // after "age: "
    }

    #[test]
    fn test_brace_matching_function() {
        // Test the find_matching_brace function directly
        assert_eq!(find_matching_brace("{}", 0), Some(2));
        assert_eq!(find_matching_brace("{a}", 0), Some(3));
        assert_eq!(find_matching_brace("{a{b}c}", 0), Some(7));
        assert_eq!(find_matching_brace("{", 0), None); // unclosed
        assert_eq!(find_matching_brace("abc", 0), None); // not a brace
    }

    #[test]
    fn test_brace_matching_with_strings() {
        // Braces inside strings should be ignored
        assert_eq!(find_matching_brace(r#"{"a": "}"}"#, 0), Some(10));
        assert_eq!(find_matching_brace(r#"{"\""}"#, 0), Some(6));
    }
}
