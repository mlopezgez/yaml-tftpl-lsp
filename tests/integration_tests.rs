//! Integration tests for the yaml-tftpl-lsp server
//!
//! These tests verify the diagnostic pipeline works correctly end-to-end,
//! from document text to LSP diagnostics.

use std::fs;
use tower_lsp::lsp_types::Diagnostic;

/// Compute diagnostics running the full pipeline (YAML parsing + workflow validation).
/// Matches what the LSP backend does.
fn compute_diagnostics(text: &str) -> Vec<Diagnostic> {
    use yaml_tftpl_lsp::diagnostics::{validate_workflow, DiagnosticCollector};
    use yaml_tftpl_lsp::parser::{parse_yaml, preprocess_expressions};

    let mut collector = DiagnosticCollector::new();
    let (preprocessed, expression_map) = preprocess_expressions(text);
    let result = parse_yaml(&preprocessed, &expression_map, &mut collector);

    if let Some(ref value) = result.value {
        validate_workflow(value, &preprocessed, &mut collector);
    }

    collector.into_diagnostics()
}

/// Compute only YAML parse diagnostics (no workflow structure validation).
/// Use this for tests that check YAML syntax handling with non-workflow documents.
fn compute_yaml_diagnostics(text: &str) -> Vec<Diagnostic> {
    use yaml_tftpl_lsp::diagnostics::DiagnosticCollector;
    use yaml_tftpl_lsp::parser::{parse_yaml, preprocess_expressions};

    let mut collector = DiagnosticCollector::new();
    let (preprocessed, expression_map) = preprocess_expressions(text);
    parse_yaml(&preprocessed, &expression_map, &mut collector);
    collector.into_diagnostics()
}

#[test]
fn test_valid_workflow_no_diagnostics() {
    let text = fs::read_to_string("tests/fixtures/valid/workflow.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics for valid workflow, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_simple_yaml_with_terraform_expressions() {
    let text = r#"
name: ${var.project_name}
config:
  enabled: true
  value: ${var.config_value}
"#;

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics for valid YAML with Terraform expressions"
    );
}

#[test]
fn test_simple_yaml_with_workflows_expressions() {
    let text = r#"
main:
  steps:
    - log:
        call: sys.log
        args:
          text: $${sys.now()}
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics for valid YAML with Workflows expressions"
    );
}

#[test]
fn test_mixed_expressions() {
    let text = r#"
project: ${var.project_id}
main:
  steps:
    - init:
        assign:
          - now: $${sys.now()}
          - config: ${jsonencode(var.config)}
"#;

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics for mixed expressions"
    );
}

#[test]
fn test_nested_terraform_expressions() {
    let text = r#"
config: ${jsonencode({
  key1: "value1",
  nested: {
    key2: "value2"
  }
})}
"#;

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics for nested Terraform expressions"
    );
}

#[test]
fn test_invalid_yaml_produces_diagnostic() {
    let text = r#"
key: value
  bad: indentation
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic for invalid YAML"
    );
}

#[test]
fn test_unclosed_quote_produces_diagnostic() {
    let text = r#"
key: "unclosed string
another: value
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        !diagnostics.is_empty(),
        "Expected diagnostic for unclosed quote"
    );
}

#[test]
fn test_empty_document() {
    let text = "";

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(diagnostics.is_empty(), "Empty document should be valid");
}

#[test]
fn test_comment_only_document() {
    let text = r#"
# This is a comment
# Another comment
"#;

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Comment-only document should be valid"
    );
}

#[test]
fn test_complex_workflow_template() {
    let text = r#"
main:
  params:
    - project_id
    - region
  steps:
    - initialize:
        assign:
          - project: ${var.project_id}
          - now: $${sys.now()}
    - callApi:
        try:
          call: http.post
          args:
            url: https://api.example.com/${var.endpoint}
            body:
              timestamp: $${now}
              project: $${project}
          result: response
        except:
          as: e
          steps:
            - logError:
                call: sys.log
                args:
                  text: $${e.message}
    - returnResult:
        return: $${response}
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Complex workflow template should be valid, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_diagnostic_has_correct_source() {
    let text = "key:\n  bad: indent";

    let diagnostics = compute_diagnostics(text);
    if !diagnostics.is_empty() {
        assert_eq!(
            diagnostics[0].source.as_deref(),
            Some("yaml-tftpl-lsp"),
            "Diagnostic source should be yaml-tftpl-lsp"
        );
    }
}

#[test]
fn test_diagnostic_severity_is_error_for_yaml_syntax() {
    let text = "key: \"unclosed";

    let diagnostics = compute_diagnostics(text);
    assert!(!diagnostics.is_empty());
    assert_eq!(
        diagnostics[0].severity,
        Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
        "YAML syntax errors should have ERROR severity"
    );
}

// ============================================================================
// Phase 5: Extended fixture tests
// ============================================================================

#[test]
fn test_nested_expressions_fixture() {
    let text = fs::read_to_string("tests/fixtures/edge_cases/nested_expressions.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    assert!(
        diagnostics.is_empty(),
        "Nested expressions should parse correctly, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_multiline_expression_fixture() {
    let text = fs::read_to_string("tests/fixtures/edge_cases/multiline_expression.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    assert!(
        diagnostics.is_empty(),
        "Multiline expressions should parse correctly, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_mixed_expressions_fixture() {
    let text = fs::read_to_string("tests/fixtures/edge_cases/mixed_expressions.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    assert!(
        diagnostics.is_empty(),
        "Mixed Terraform and Workflows expressions should parse correctly, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_unclosed_quote_fixture() {
    let text = fs::read_to_string("tests/fixtures/invalid/unclosed_quote.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    assert!(
        !diagnostics.is_empty(),
        "Unclosed quote should produce diagnostics"
    );
    // Verify the diagnostic has the correct source
    assert_eq!(diagnostics[0].source.as_deref(), Some("yaml-tftpl-lsp"));
}

#[test]
fn test_unclosed_brace_still_parses_yaml() {
    // When an expression has an unclosed brace, it won't be replaced
    // by a placeholder, so it stays in the YAML. This may or may not
    // cause a YAML parse error depending on context.
    let text = fs::read_to_string("tests/fixtures/invalid/unclosed_brace.yaml.tftpl")
        .expect("Failed to read fixture");

    // Just verify it doesn't panic - the behavior depends on how
    // the unclosed expression affects YAML parsing
    let _diagnostics = compute_diagnostics(&text);
}

#[test]
fn test_bad_indentation_fixture() {
    let text = fs::read_to_string("tests/fixtures/invalid/bad_indentation.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    assert!(
        !diagnostics.is_empty(),
        "Bad indentation should produce diagnostics"
    );
}

#[test]
fn test_missing_colon_fixture() {
    let text = fs::read_to_string("tests/fixtures/invalid/missing_colon.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    assert!(
        !diagnostics.is_empty(),
        "Missing colon should produce diagnostics"
    );
}

#[test]
fn test_diagnostic_has_code() {
    let text = "key: \"unclosed";

    let diagnostics = compute_diagnostics(text);
    assert!(!diagnostics.is_empty());

    // Verify diagnostic has a code
    assert!(
        diagnostics[0].code.is_some(),
        "Diagnostic should have a code"
    );
}

#[test]
fn test_diagnostic_range_is_valid() {
    let text = "key:\n  bad: indent";

    let diagnostics = compute_diagnostics(text);
    if !diagnostics.is_empty() {
        let range = &diagnostics[0].range;
        // Ensure end is at or after start
        assert!(
            range.end.line >= range.start.line,
            "Diagnostic end line should be >= start line"
        );
        if range.end.line == range.start.line {
            assert!(
                range.end.character >= range.start.character,
                "On same line, end character should be >= start character"
            );
        }
    }
}

#[test]
fn test_expression_in_yaml_string() {
    // Test that expressions inside YAML strings are handled
    let text = r#"
message: "Hello ${var.name}!"
timestamp: $${sys.now()}
"#;

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Expressions in strings should be valid"
    );
}

#[test]
fn test_multiple_expressions_same_line() {
    let text = "args: [${var.a}, ${var.b}, ${var.c}]";

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Multiple expressions on same line should be valid"
    );
}

#[test]
fn test_expression_at_yaml_key_position() {
    // While unusual, expressions can appear in key positions
    let text = "${var.key}: value";

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Expression as YAML key should be valid"
    );
}

#[test]
fn test_deeply_nested_yaml_with_expressions() {
    let text = r#"
level1:
  level2:
    level3:
      level4:
        value: ${var.deep_value}
        timestamp: $${sys.now()}
        config:
          setting1: true
          setting2: ${var.setting}
"#;

    let diagnostics = compute_yaml_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Deeply nested YAML with expressions should be valid"
    );
}

// ============================================================================
// Phase 7: Workflow structure validation integration tests
// ============================================================================

#[test]
fn test_workflow_missing_main_produces_warning() {
    let text = r#"
helper:
  steps:
    - init:
        assign:
          - x: 1
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics.iter().any(|d| d.message.contains("'main'")),
        "Missing 'main' block should produce a warning, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_workflow_main_missing_steps_produces_warning() {
    let text = r#"
main:
  params:
    - project_id
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics.iter().any(|d| d.message.contains("'steps'")),
        "Main block without 'steps' should produce a warning, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_workflow_steps_not_a_list() {
    let text = r#"
main:
  steps:
    init:
      assign:
        - x: 1
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("'steps' must be a list")),
        "Non-list 'steps' should produce a warning, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_workflow_unknown_top_level_key_produces_hint() {
    let text = r#"
main:
  steps:
    - init:
        assign:
          - x: 1
unknown_key: true
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("Unknown workflow element")),
        "Unknown top-level key should produce a hint, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_workflow_valid_subworkflow_no_extra_warnings() {
    let text = r#"
main:
  steps:
    - callHelper:
        call: helper
        args:
          name: "test"
helper:
  params:
    - name
  steps:
    - greet:
        assign:
          - msg: "hello"
"#;

    let diagnostics = compute_diagnostics(text);
    // Should have no errors or warnings (hints about unknown step modifiers are acceptable)
    let errors_and_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity != Some(tower_lsp::lsp_types::DiagnosticSeverity::HINT))
        .collect();
    assert!(
        errors_and_warnings.is_empty(),
        "Valid workflow with subworkflow should produce no errors/warnings, got: {:?}",
        errors_and_warnings
    );
}

#[test]
fn test_workflow_with_expressions_validates_structure() {
    let text = r#"
main:
  steps:
    - init:
        assign:
          - project: ${var.project_id}
          - timestamp: $${sys.now()}
    - callApi:
        call: http.post
        args:
          url: https://api.example.com/${var.endpoint}
        result: response
    - done:
        return: $${response}
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics.is_empty(),
        "Valid workflow with expressions should pass validation, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_non_mapping_document_produces_warning() {
    let text = r#"
- item1
- item2
- item3
"#;

    let diagnostics = compute_diagnostics(text);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("YAML mapping")),
        "Non-mapping document should produce a warning, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_workflow_fixture_with_full_validation() {
    let text = fs::read_to_string("tests/fixtures/valid/workflow.yaml.tftpl")
        .expect("Failed to read fixture");

    let diagnostics = compute_diagnostics(&text);

    // Filter to only errors and warnings (hints about unknown keys are acceptable)
    let errors_and_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity != Some(tower_lsp::lsp_types::DiagnosticSeverity::HINT))
        .collect();

    assert!(
        errors_and_warnings.is_empty(),
        "Valid workflow fixture should have no errors/warnings with full validation, got: {:?}",
        errors_and_warnings
    );
}
