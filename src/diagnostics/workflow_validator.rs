//! GCP Workflows structure validation
//!
//! Validates the structure of Google Cloud Workflows YAML documents,
//! checking for required fields, valid step structures, and unknown keys.

use serde_yaml::Value;

use super::yaml_errors::{DiagnosticCode, DiagnosticCollector};

/// Validate a parsed YAML value as a GCP Workflow document.
///
/// This checks structural rules like:
/// - Top-level must contain workflow definitions (e.g. `main`)
/// - `main` must have `steps`
/// - `steps` must be a list
/// - Each step should have exactly one named key
/// - Subworkflows should have `params` or `steps`
/// - Unknown top-level keys produce hints
pub fn validate_workflow(value: &Value, text: &str, collector: &mut DiagnosticCollector) {
    let mapping = match value.as_mapping() {
        Some(m) => m,
        None => {
            // Not a mapping at top level - not a valid workflow
            collector.add_workflow_warning(
                "Workflow document must be a YAML mapping".to_string(),
                0,
                0,
            );
            return;
        }
    };

    let line_index = LineIndex::new(text);
    let mut has_main = false;

    for (key, val) in mapping {
        let key_str = match key.as_str() {
            Some(s) => s,
            None => continue,
        };

        let key_line = find_key_line(&line_index, key_str);

        if key_str == "main" {
            has_main = true;
            validate_workflow_block(val, key_str, &line_index, collector);
        } else if is_likely_subworkflow(val) {
            validate_workflow_block(val, key_str, &line_index, collector);
        } else {
            // Unknown top-level key - emit hint
            collector.add_hint(
                format!("Unknown workflow element: '{}'", key_str),
                key_line,
                0,
            );
        }
    }

    if !has_main && !mapping.is_empty() {
        collector.add_workflow_warning("Workflow must have a 'main' block".to_string(), 0, 0);
    }
}

/// Check if a value looks like a subworkflow definition (has params or steps)
fn is_likely_subworkflow(value: &Value) -> bool {
    if let Some(map) = value.as_mapping() {
        map.keys().any(|k| {
            k.as_str()
                .map_or(false, |s| s == "params" || s == "steps")
        })
    } else {
        false
    }
}

/// Validate a workflow or subworkflow block (must have `steps`)
fn validate_workflow_block(
    value: &Value,
    name: &str,
    line_index: &LineIndex,
    collector: &mut DiagnosticCollector,
) {
    let mapping = match value.as_mapping() {
        Some(m) => m,
        None => {
            let line = find_key_line(line_index, name);
            collector.add_workflow_warning(
                format!("'{}' block must be a mapping", name),
                line,
                0,
            );
            return;
        }
    };

    let has_steps = mapping
        .keys()
        .any(|k| k.as_str().map_or(false, |s| s == "steps"));

    if !has_steps {
        let line = find_key_line(line_index, name);
        collector.add_workflow_warning(
            format!("'{}' block must contain 'steps'", name),
            line,
            0,
        );
        return;
    }

    // Validate steps
    for (k, v) in mapping {
        if k.as_str() == Some("steps") {
            validate_steps(v, line_index, collector);
        }
    }

    // Check for unknown keys in workflow block
    let valid_keys = ["params", "steps"];
    for key in mapping.keys() {
        if let Some(s) = key.as_str() {
            if !valid_keys.contains(&s) {
                let line = find_key_line(line_index, s);
                collector.add_hint(
                    format!("Unknown key '{}' in workflow block '{}'", s, name),
                    line,
                    0,
                );
            }
        }
    }
}

/// Validate a `steps` list
fn validate_steps(value: &Value, line_index: &LineIndex, collector: &mut DiagnosticCollector) {
    let steps = match value.as_sequence() {
        Some(s) => s,
        None => {
            let line = find_key_line(line_index, "steps");
            collector.add_workflow_warning("'steps' must be a list".to_string(), line, 0);
            return;
        }
    };

    for step in steps {
        let mapping = match step.as_mapping() {
            Some(m) => m,
            None => continue,
        };

        if mapping.len() != 1 {
            // Try to find approximate line
            if let Some((first_key, _)) = mapping.iter().next() {
                if let Some(s) = first_key.as_str() {
                    let line = find_key_line(line_index, s);
                    collector.add_workflow_warning_with_code(
                        "Step should have exactly one named key".to_string(),
                        line,
                        0,
                        DiagnosticCode::WorkflowStructure,
                    );
                }
            }
        }

        // Validate step content
        for (_step_name, step_value) in mapping {
            validate_step_body(step_value, line_index, collector);
        }
    }
}

/// Validate the body of a single step
fn validate_step_body(
    value: &Value,
    line_index: &LineIndex,
    collector: &mut DiagnosticCollector,
) {
    let mapping = match value.as_mapping() {
        Some(m) => m,
        None => return, // scalar or sequence step body - not necessarily invalid
    };

    use crate::schema;

    for key in mapping.keys() {
        if let Some(s) = key.as_str() {
            if !schema::is_step_action(s) && !is_step_modifier(s) {
                let line = find_key_line(line_index, s);
                collector.add_hint(
                    format!("Unknown step action: '{}'", s),
                    line,
                    0,
                );
            }
        }
    }
}

/// Check if a key is a valid step modifier (not an action but valid in step context)
fn is_step_modifier(key: &str) -> bool {
    matches!(key, "args" | "result" | "condition" | "value" | "index" | "range" | "in"
        | "branches" | "shared" | "concurrency_limit" | "exception_policy"
        | "except" | "retry" | "as" | "steps" | "predicate" | "max_retries"
        | "backoff" | "initial_delay" | "max_delay" | "multiplier" | "params"
        | "next")
}

/// Simple line index for finding key positions in text
struct LineIndex {
    lines: Vec<String>,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        Self {
            lines: text.lines().map(|l| l.to_string()).collect(),
        }
    }

    /// Find the first line containing the given key pattern "key:"
    fn find_key(&self, key: &str) -> Option<u32> {
        let pattern = format!("{}:", key);
        // Also match "- key:" for list items
        let list_pattern = format!("- {}:", key);
        // And bare key as a list item name "- key" or "  key:"
        for (i, line) in self.lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with(&pattern)
                || trimmed.starts_with(&list_pattern)
                || trimmed == key
                || trimmed == format!("{}:", key)
            {
                return Some(i as u32);
            }
        }
        None
    }
}

/// Find the line where a key appears in the document
fn find_key_line(line_index: &LineIndex, key: &str) -> u32 {
    line_index.find_key(key).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::DiagnosticCollector;

    fn parse_and_validate(yaml: &str) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        let value: Value = serde_yaml::from_str(yaml).expect("test YAML should parse");
        let mut collector = DiagnosticCollector::new();
        validate_workflow(&value, yaml, &mut collector);
        collector.into_diagnostics()
    }

    #[test]
    fn test_valid_workflow() {
        let yaml = r#"
main:
  steps:
    - init:
        assign:
          - result: "hello"
    - done:
        return: result
"#;
        let diagnostics = parse_and_validate(yaml);
        assert!(diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", diagnostics);
    }

    #[test]
    fn test_missing_main() {
        let yaml = r#"
other:
  steps:
    - init:
        assign:
          - x: 1
"#;
        let diagnostics = parse_and_validate(yaml);
        // Should not warn about missing main since "other" looks like a subworkflow
        // Actually it has steps so it IS a subworkflow, but no main
        assert!(diagnostics.iter().any(|d| d.message.contains("'main'")));
    }

    #[test]
    fn test_main_missing_steps() {
        let yaml = r#"
main:
  params:
    - name
"#;
        let diagnostics = parse_and_validate(yaml);
        assert!(diagnostics.iter().any(|d| d.message.contains("'steps'")));
    }

    #[test]
    fn test_steps_not_a_list() {
        let yaml = r#"
main:
  steps:
    init:
      assign:
        - x: 1
"#;
        let diagnostics = parse_and_validate(yaml);
        assert!(diagnostics.iter().any(|d| d.message.contains("'steps' must be a list")));
    }

    #[test]
    fn test_unknown_top_level_key() {
        let yaml = r#"
main:
  steps:
    - init:
        assign:
          - x: 1
something_else: true
"#;
        let diagnostics = parse_and_validate(yaml);
        assert!(diagnostics.iter().any(|d| d.message.contains("Unknown workflow element")));
    }

    #[test]
    fn test_valid_subworkflow() {
        let yaml = r#"
main:
  steps:
    - callSub:
        call: helper
        args:
          name: "test"
helper:
  params:
    - name
  steps:
    - init:
        assign:
          - x: 1
"#;
        let diagnostics = parse_and_validate(yaml);
        assert!(diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", diagnostics);
    }

    #[test]
    fn test_workflow_with_expressions() {
        // After preprocessing, expressions become placeholders, so the YAML
        // structure should still validate correctly
        let yaml = r#"
main:
  steps:
    - init:
        assign:
          - project: __EXPR_000__
          - timestamp: __EXPR_001__
"#;
        let diagnostics = parse_and_validate(yaml);
        assert!(diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", diagnostics);
    }

    #[test]
    fn test_non_mapping_document() {
        let yaml = "- item1\n- item2";
        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let mut collector = DiagnosticCollector::new();
        validate_workflow(&value, yaml, &mut collector);
        let diagnostics = collector.into_diagnostics();
        assert!(diagnostics.iter().any(|d| d.message.contains("YAML mapping")));
    }
}
