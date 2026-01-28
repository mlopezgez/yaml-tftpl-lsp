//! YAML syntax error diagnostics
//!
//! This module provides diagnostic collection and conversion to LSP format,
//! with support for different severity levels and diagnostic codes.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

/// Diagnostic codes for categorizing errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    /// YAML syntax error (parsing failed)
    YamlSyntax,
    /// Invalid indentation
    InvalidIndentation,
    /// Unclosed string literal
    UnclosedString,
    /// Workflow structure error
    WorkflowStructure,
    /// Unknown workflow keyword
    UnknownKeyword,
}

impl DiagnosticCode {
    /// Get the string code for this diagnostic
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticCode::YamlSyntax => "yaml-syntax",
            DiagnosticCode::InvalidIndentation => "invalid-indentation",
            DiagnosticCode::UnclosedString => "unclosed-string",
            DiagnosticCode::WorkflowStructure => "workflow-structure",
            DiagnosticCode::UnknownKeyword => "unknown-keyword",
        }
    }

    /// Infer the diagnostic code from an error message
    pub fn from_message(message: &str) -> Self {
        let msg_lower = message.to_lowercase();
        if msg_lower.contains("indent") || msg_lower.contains("mapping values") {
            DiagnosticCode::InvalidIndentation
        } else if msg_lower.contains("unclosed")
            || msg_lower.contains("unterminated")
            || msg_lower.contains("quote")
        {
            DiagnosticCode::UnclosedString
        } else {
            DiagnosticCode::YamlSyntax
        }
    }
}

/// Collects diagnostics during parsing and validation
#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    /// Create a new empty collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a YAML syntax error diagnostic with automatic code inference
    pub fn add_yaml_error(&mut self, message: String, line: u32, column: u32) {
        let code = DiagnosticCode::from_message(&message);
        self.add_yaml_error_with_code(message, line, column, code);
    }

    /// Add a YAML syntax error diagnostic with explicit code
    pub fn add_yaml_error_with_code(
        &mut self,
        message: String,
        line: u32,
        column: u32,
        code: DiagnosticCode,
    ) {
        self.add_yaml_error_with_range(message, line, column, line, column + 1, code);
    }

    /// Add a YAML syntax error diagnostic with explicit range
    pub fn add_yaml_error_with_range(
        &mut self,
        message: String,
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
        code: DiagnosticCode,
    ) {
        self.diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_column,
                },
                end: Position {
                    line: end_line,
                    character: end_column,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(code.as_str().to_string())),
            code_description: None,
            source: Some("yaml-tftpl-lsp".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        });
    }

    /// Add a workflow structure warning
    #[allow(dead_code)]
    pub fn add_workflow_warning(&mut self, message: String, line: u32, column: u32) {
        self.add_workflow_warning_with_code(
            message,
            line,
            column,
            DiagnosticCode::WorkflowStructure,
        );
    }

    /// Add a workflow structure warning with explicit code
    #[allow(dead_code)]
    pub fn add_workflow_warning_with_code(
        &mut self,
        message: String,
        line: u32,
        column: u32,
        code: DiagnosticCode,
    ) {
        self.diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line,
                    character: column,
                },
                end: Position {
                    line,
                    character: column + 1,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String(code.as_str().to_string())),
            code_description: None,
            source: Some("yaml-tftpl-lsp".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        });
    }

    /// Add a hint diagnostic
    #[allow(dead_code)]
    pub fn add_hint(&mut self, message: String, line: u32, column: u32) {
        self.diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line,
                    character: column,
                },
                end: Position {
                    line,
                    character: column + 1,
                },
            },
            severity: Some(DiagnosticSeverity::HINT),
            code: Some(NumberOrString::String(
                DiagnosticCode::UnknownKeyword.as_str().to_string(),
            )),
            code_description: None,
            source: Some("yaml-tftpl-lsp".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        });
    }

    /// Get the number of diagnostics collected
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Check if there are no diagnostics
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Convert into the final list of diagnostics
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_code_from_message() {
        assert_eq!(
            DiagnosticCode::from_message("invalid indentation"),
            DiagnosticCode::InvalidIndentation
        );
        assert_eq!(
            DiagnosticCode::from_message("mapping values are not allowed"),
            DiagnosticCode::InvalidIndentation
        );
        assert_eq!(
            DiagnosticCode::from_message("unclosed string"),
            DiagnosticCode::UnclosedString
        );
        assert_eq!(
            DiagnosticCode::from_message("unterminated quote"),
            DiagnosticCode::UnclosedString
        );
        assert_eq!(
            DiagnosticCode::from_message("some other error"),
            DiagnosticCode::YamlSyntax
        );
    }

    #[test]
    fn test_add_yaml_error() {
        let mut collector = DiagnosticCollector::new();
        collector.add_yaml_error("test error".to_string(), 5, 10);

        let diagnostics = collector.into_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.line, 5);
        assert_eq!(diagnostics[0].range.start.character, 10);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostics[0].source.as_deref(), Some("yaml-tftpl-lsp"));
    }

    #[test]
    fn test_add_yaml_error_with_range() {
        let mut collector = DiagnosticCollector::new();
        collector.add_yaml_error_with_range(
            "test error".to_string(),
            5,
            10,
            5,
            20,
            DiagnosticCode::YamlSyntax,
        );

        let diagnostics = collector.into_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.line, 5);
        assert_eq!(diagnostics[0].range.start.character, 10);
        assert_eq!(diagnostics[0].range.end.line, 5);
        assert_eq!(diagnostics[0].range.end.character, 20);
    }

    #[test]
    fn test_diagnostic_code_string() {
        assert_eq!(DiagnosticCode::YamlSyntax.as_str(), "yaml-syntax");
        assert_eq!(
            DiagnosticCode::InvalidIndentation.as_str(),
            "invalid-indentation"
        );
        assert_eq!(DiagnosticCode::UnclosedString.as_str(), "unclosed-string");
    }

    #[test]
    fn test_workflow_warning() {
        let mut collector = DiagnosticCollector::new();
        collector.add_workflow_warning("missing steps".to_string(), 0, 0);

        let diagnostics = collector.into_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn test_hint() {
        let mut collector = DiagnosticCollector::new();
        collector.add_hint("unknown keyword".to_string(), 0, 0);

        let diagnostics = collector.into_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::HINT));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut collector = DiagnosticCollector::new();
        assert!(collector.is_empty());
        assert_eq!(collector.len(), 0);

        collector.add_yaml_error("error".to_string(), 0, 0);
        assert!(!collector.is_empty());
        assert_eq!(collector.len(), 1);
    }
}
