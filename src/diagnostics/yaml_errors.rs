//! YAML syntax error diagnostics

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

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

    /// Add a YAML syntax error diagnostic
    pub fn add_yaml_error(&mut self, message: String, line: u32, column: u32) {
        self.diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line,
                    character: column,
                },
                end: Position {
                    line,
                    character: column,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
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
        self.diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line,
                    character: column,
                },
                end: Position {
                    line,
                    character: column,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            code: None,
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
                    character: column,
                },
            },
            severity: Some(DiagnosticSeverity::HINT),
            code: None,
            code_description: None,
            source: Some("yaml-tftpl-lsp".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        });
    }

    /// Convert into the final list of diagnostics
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
