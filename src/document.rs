//! Document state management

/// Represents the state of a text document
#[derive(Debug, Clone)]
pub struct Document {
    /// The document text content
    pub text: String,
    /// The document version
    pub version: i32,
}

impl Document {
    /// Create a new document with the given text and version
    pub fn new(text: String, version: i32) -> Self {
        Self { text, version }
    }
}
