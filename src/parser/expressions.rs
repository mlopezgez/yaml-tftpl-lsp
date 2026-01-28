//! Terraform ${} and Workflows $${} expression handling

/// Represents a single expression found in the document
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

impl Expression {
    /// Length of the original expression in bytes
    pub fn original_len(&self) -> usize {
        self.end - self.start
    }

    /// Length of the placeholder
    pub fn placeholder_len(&self) -> usize {
        self.placeholder.len()
    }

    /// The difference in length between original and placeholder
    /// Positive means original is longer, negative means placeholder is longer
    pub fn len_delta(&self) -> isize {
        self.original_len() as isize - self.placeholder_len() as isize
    }
}

/// The kind of expression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    /// Terraform interpolation: ${...}
    Terraform,
    /// GCP Workflows runtime expression: $${...}
    Workflows,
}

/// Represents a position offset caused by placeholder substitution
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PositionDelta {
    /// The line where this expression starts (in preprocessed text)
    preprocessed_line: u32,
    /// The column where the placeholder starts (in preprocessed text)
    preprocessed_column: u32,
    /// The column where the placeholder ends (in preprocessed text)
    preprocessed_end_column: u32,
    /// The original expression's start line
    original_line: u32,
    /// The original expression's start column
    original_column: u32,
    /// The original expression's end line
    original_end_line: u32,
    /// The original expression's end column
    original_end_column: u32,
    /// Whether this is a multi-line expression
    is_multiline: bool,
}

/// A map of all expressions found in a document
#[derive(Debug, Default)]
pub struct ExpressionMap {
    /// All expressions, in document order
    pub expressions: Vec<Expression>,
    /// Cached position deltas for efficient position adjustment
    position_deltas: Vec<PositionDelta>,
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
    #[allow(dead_code)]
    pub fn find_by_placeholder(&self, placeholder: &str) -> Option<&Expression> {
        self.expressions
            .iter()
            .find(|e| e.placeholder == placeholder)
    }

    /// Sort expressions by position and build position delta cache
    /// This should be called after all expressions have been added
    pub fn finalize(&mut self) {
        // Sort expressions by start position
        self.expressions.sort_by_key(|e| e.start);

        // Build position deltas for efficient position adjustment
        self.build_position_deltas();
    }

    /// Build position delta cache from expressions
    fn build_position_deltas(&mut self) {
        self.position_deltas.clear();

        // Track cumulative column offset for each line
        // For expressions on the same line, we need to account for previous substitutions
        let mut current_line = 0u32;
        let mut cumulative_column_offset: isize = 0;

        for expr in &self.expressions {
            // Reset cumulative offset when moving to a new line
            if expr.start_line != current_line {
                current_line = expr.start_line;
                cumulative_column_offset = 0;
            }

            // Calculate the preprocessed column (after previous substitutions on same line)
            let preprocessed_column =
                (expr.start_column as isize - cumulative_column_offset) as u32;
            let preprocessed_end_column = preprocessed_column + expr.placeholder_len() as u32;

            self.position_deltas.push(PositionDelta {
                preprocessed_line: expr.start_line,
                preprocessed_column,
                preprocessed_end_column,
                original_line: expr.start_line,
                original_column: expr.start_column,
                original_end_line: expr.end_line,
                original_end_column: expr.end_column,
                is_multiline: expr.start_line != expr.end_line,
            });

            // Update cumulative offset for next expression on same line
            cumulative_column_offset += expr.len_delta();
        }
    }

    /// Adjust a position from preprocessed coordinates back to original coordinates
    ///
    /// This handles the case where YAML parsing reports an error at a position
    /// that falls within or after a placeholder, mapping it back to the correct
    /// position in the original document.
    pub fn adjust_position(&self, line: u32, column: u32) -> (u32, u32) {
        let mut adjusted_column = column as i64;

        // Find all deltas that affect this position
        for delta in &self.position_deltas {
            // Only consider deltas on the same line (for single-line expressions)
            // or that might affect this line (for multi-line expressions)
            if delta.preprocessed_line == line {
                if column >= delta.preprocessed_column && column < delta.preprocessed_end_column {
                    // Position is within a placeholder - map to start of original expression
                    return (delta.original_line, delta.original_column);
                } else if column >= delta.preprocessed_end_column {
                    // Position is after this placeholder - adjust by the length difference
                    // Use signed arithmetic since placeholder can be longer than original
                    let placeholder_len =
                        (delta.preprocessed_end_column - delta.preprocessed_column) as i64;
                    let original_len = if delta.is_multiline {
                        // For multi-line expressions, only count first line portion
                        // This is a simplification; full implementation would track line breaks
                        (delta.original_end_column - delta.original_column) as i64
                    } else {
                        (delta.original_end_column - delta.original_column) as i64
                    };
                    adjusted_column += original_len - placeholder_len;
                }
            }
        }

        (line, adjusted_column.max(0) as u32)
    }

    /// Check if a position falls within any expression
    #[allow(dead_code)]
    pub fn is_within_expression(&self, line: u32, column: u32) -> bool {
        for delta in &self.position_deltas {
            if delta.preprocessed_line == line
                && column >= delta.preprocessed_column
                && column < delta.preprocessed_end_column
            {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_len_delta() {
        let expr = Expression {
            original: "${var.name}".to_string(),
            placeholder: "__EXPR_000__".to_string(),
            start: 7,
            end: 18, // ${var.name} is 11 chars
            start_line: 0,
            start_column: 7,
            end_line: 0,
            end_column: 18,
            kind: ExpressionKind::Terraform,
        };

        assert_eq!(expr.original_len(), 11);
        assert_eq!(expr.placeholder_len(), 12);
        assert_eq!(expr.len_delta(), -1); // placeholder is 1 char longer
    }

    #[test]
    fn test_adjust_position_no_expressions() {
        let map = ExpressionMap::new();
        assert_eq!(map.adjust_position(5, 10), (5, 10));
    }

    #[test]
    fn test_adjust_position_within_expression() {
        let mut map = ExpressionMap::new();
        map.add(Expression {
            original: "${var.name}".to_string(),
            placeholder: "__EXPR_000__".to_string(),
            start: 7,
            end: 18,
            start_line: 0,
            start_column: 7,
            end_line: 0,
            end_column: 18,
            kind: ExpressionKind::Terraform,
        });
        map.finalize();

        // Position within placeholder should map to start of original
        assert_eq!(map.adjust_position(0, 10), (0, 7));
    }

    #[test]
    fn test_adjust_position_after_expression() {
        let mut map = ExpressionMap::new();
        map.add(Expression {
            original: "${var.name}".to_string(),     // 11 chars
            placeholder: "__EXPR_000__".to_string(), // 12 chars
            start: 7,
            end: 18,
            start_line: 0,
            start_column: 7,
            end_line: 0,
            end_column: 18,
            kind: ExpressionKind::Terraform,
        });
        map.finalize();

        // Position after placeholder should be adjusted
        // placeholder ends at column 19 (7 + 12), original ends at column 18
        // Position 20 in preprocessed = position 19 in original
        assert_eq!(map.adjust_position(0, 20), (0, 19));
    }

    #[test]
    fn test_is_within_expression() {
        let mut map = ExpressionMap::new();
        map.add(Expression {
            original: "${var.name}".to_string(),
            placeholder: "__EXPR_000__".to_string(),
            start: 7,
            end: 18,
            start_line: 0,
            start_column: 7,
            end_line: 0,
            end_column: 18,
            kind: ExpressionKind::Terraform,
        });
        map.finalize();

        assert!(!map.is_within_expression(0, 5)); // Before
        assert!(map.is_within_expression(0, 7)); // Start
        assert!(map.is_within_expression(0, 15)); // Middle
        assert!(!map.is_within_expression(0, 19)); // After (placeholder ends at 19)
    }
}
