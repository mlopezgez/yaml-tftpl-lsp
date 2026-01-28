//! Diagnostics module for error collection and reporting
//!
//! This module provides:
//! - `DiagnosticCollector`: Collects and converts errors to LSP diagnostics
//! - `DiagnosticCode`: Categorizes different types of diagnostics

mod workflow_validator;
mod yaml_errors;

pub use yaml_errors::{DiagnosticCode, DiagnosticCollector};
