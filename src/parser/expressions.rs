//! Terraform ${} and Workflows $${} expression handling

/// Represents a single expression found in the document
#[derive(Debug, Clone)]
pub struct Expression {
    /// The original text of the expression (e.g., "${var.name}")
    pub original: String,
    /// The placeholder that replaced it (e.g., "__EXPR_001__")
    pub placeholder: String,
    /// Start byte offset in the original document
    pub start: usize,
    /// End byte offset in the original document
    pub end: usize,
    /// Start line (0-indexed)
    pub start_line: u32,
    /// Start column (0-indexed)
    pub start_column: u32,
    /// End line (0-indexed)
    pub end_line: u32,
    /// End column (0-indexed)
    pub end_column: u32,
    /// Whether this is a Terraform (${}) or Workflows ($${}) expression
    pub kind: ExpressionKind,
}

/// The kind of expression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    /// Terraform interpolation: ${...}
    Terraform,
    /// GCP Workflows runtime expression: $${...}
    Workflows,
}

/// A map of all expressions found in a document
#[derive(Debug, Default)]
pub struct ExpressionMap {
    /// All expressions, indexed by their placeholder
    pub expressions: Vec<Expression>,
}

impl ExpressionMap {
    /// Create a new empty expression map
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an expression to the map
    pub fn add(&mut self, expr: Expression) {
        self.expressions.push(expr);
    }

    /// Find an expression by its placeholder
    pub fn find_by_placeholder(&self, placeholder: &str) -> Option<&Expression> {
        self.expressions
            .iter()
            .find(|e| e.placeholder == placeholder)
    }

    /// Adjust a position from preprocessed coordinates back to original coordinates
    ///
    /// If the position falls within a placeholder, it's mapped to the start of the
    /// original expression.
    pub fn adjust_position(&self, line: u32, column: u32) -> (u32, u32) {
        // For now, return the position as-is
        // Full implementation will map placeholder positions back to original
        (line, column)
    }
}
