//! yaml-tftpl-lsp: LSP server library for YAML Terraform template files with GCP Workflows syntax
//!
//! This library provides the core functionality for the yaml-tftpl-lsp server:
//! - Expression preprocessing for Terraform ${...} and Workflows $${...} syntax
//! - YAML parsing with error recovery
//! - Diagnostic collection and reporting
//!
//! # Example
//!
//! ```
//! use yaml_tftpl_lsp::diagnostics::DiagnosticCollector;
//! use yaml_tftpl_lsp::parser::{parse_yaml, preprocess_expressions};
//!
//! let text = "name: ${var.project}\nsteps:\n  - init: value";
//! let mut collector = DiagnosticCollector::new();
//! let (preprocessed, expression_map) = preprocess_expressions(text);
//! parse_yaml(&preprocessed, &expression_map, &mut collector);
//! let diagnostics = collector.into_diagnostics();
//! ```

pub mod diagnostics;
pub mod document;
pub mod parser;
pub mod schema;

mod backend;

pub use backend::Backend;
