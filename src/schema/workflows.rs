//! GCP Workflows keywords and structure definitions
//!
//! This module defines the schema rules and validation structures for
//! Google Cloud Workflows syntax.

use std::collections::HashSet;

/// Reserved keywords in Google Cloud Workflows syntax
pub const WORKFLOW_KEYWORDS: &[&str] = &[
    // Step types
    "assign",
    "call",
    "switch",
    "for",
    "parallel",
    "try",
    "raise",
    "return",
    "next",
    // Call step
    "args",
    "result",
    // Switch step
    "condition",
    // For loop
    "value",
    "index",
    "range",
    "in",
    // Parallel step
    "branches",
    "shared",
    "concurrency_limit",
    "exception_policy",
    // Try/except
    "except",
    "retry",
    "as",
    // Retry policy
    "predicate",
    "max_retries",
    "backoff",
    "initial_delay",
    "max_delay",
    "multiplier",
    // Workflow structure
    "main",
    "params",
    "steps",
];

/// Step action keywords - valid inside a step definition
pub const STEP_ACTION_KEYWORDS: &[&str] = &[
    "assign", "call", "switch", "for", "parallel", "try", "raise", "return", "next",
];

/// Keywords valid inside a `call` step
pub const CALL_STEP_KEYWORDS: &[&str] = &["call", "args", "result"];

/// Keywords valid inside a `switch` step
pub const SWITCH_STEP_KEYWORDS: &[&str] = &["switch"];

/// Keywords valid inside switch conditions
pub const SWITCH_CONDITION_KEYWORDS: &[&str] = &["condition", "next", "return", "raise", "steps"];

/// Keywords valid inside a `for` loop
pub const FOR_STEP_KEYWORDS: &[&str] = &["for", "value", "index", "range", "in", "steps"];

/// Keywords valid inside a `parallel` step
pub const PARALLEL_STEP_KEYWORDS: &[&str] = &[
    "parallel",
    "branches",
    "shared",
    "concurrency_limit",
    "exception_policy",
    "steps",
];

/// Keywords valid inside a `try` block
pub const TRY_STEP_KEYWORDS: &[&str] = &["try", "except", "retry", "steps", "as"];

/// Keywords valid for retry policy
pub const RETRY_KEYWORDS: &[&str] = &[
    "predicate",
    "max_retries",
    "backoff",
    "initial_delay",
    "max_delay",
    "multiplier",
];

/// Keywords valid inside a subworkflow definition
pub const SUBWORKFLOW_KEYWORDS: &[&str] = &["params", "steps"];

/// Standard library connectors that can be called
#[allow(dead_code)]
pub const STDLIB_CONNECTORS: &[&str] = &[
    "http.get",
    "http.post",
    "http.request",
    "sys.get_env",
    "sys.now",
    "sys.sleep",
    "sys.log",
];

/// Check if a key is a known workflow keyword
pub fn is_workflow_keyword(key: &str) -> bool {
    WORKFLOW_KEYWORDS.contains(&key)
}

/// Check if a key is a valid step action
pub fn is_step_action(key: &str) -> bool {
    STEP_ACTION_KEYWORDS.contains(&key)
}

/// Get set of valid step action keywords
pub fn step_action_set() -> HashSet<&'static str> {
    STEP_ACTION_KEYWORDS.iter().copied().collect()
}

/// Get set of all workflow keywords
pub fn workflow_keyword_set() -> HashSet<&'static str> {
    WORKFLOW_KEYWORDS.iter().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_workflow_keyword() {
        assert!(is_workflow_keyword("main"));
        assert!(is_workflow_keyword("steps"));
        assert!(is_workflow_keyword("assign"));
        assert!(is_workflow_keyword("call"));
        assert!(!is_workflow_keyword("unknown_keyword"));
        assert!(!is_workflow_keyword("custom_step_name"));
    }

    #[test]
    fn test_is_step_action() {
        assert!(is_step_action("assign"));
        assert!(is_step_action("call"));
        assert!(is_step_action("switch"));
        assert!(is_step_action("return"));
        assert!(!is_step_action("main"));
        assert!(!is_step_action("params"));
    }

    #[test]
    fn test_step_action_set() {
        let set = step_action_set();
        assert!(set.contains("assign"));
        assert!(set.contains("call"));
        assert!(!set.contains("main"));
    }

    #[test]
    fn test_workflow_keyword_set() {
        let set = workflow_keyword_set();
        assert!(set.contains("main"));
        assert!(set.contains("steps"));
        assert!(set.contains("assign"));
    }
}
