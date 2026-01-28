//! Integration tests for the yaml-tftpl-lsp server
//!
//! These tests verify the diagnostic pipeline works correctly end-to-end,
//! from document text to LSP diagnostics.

use std::fs;

/// Test helper to compute diagnostics for a given text
fn compute_diagnostics(text: &str) -> Vec<tower_lsp::lsp_types::Diagnostic> {
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

    let diagnostics = compute_diagnostics(text);
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

    let diagnostics = compute_diagnostics(text);
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

    let diagnostics = compute_diagnostics(text);
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

    let diagnostics = compute_diagnostics(text);
    assert!(diagnostics.is_empty(), "Empty document should be valid");
}

#[test]
fn test_comment_only_document() {
    let text = r#"
# This is a comment
# Another comment
"#;

    let diagnostics = compute_diagnostics(text);
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
