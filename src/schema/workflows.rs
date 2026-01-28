//! GCP Workflows keywords and structure definitions

/// Reserved keywords in Google Cloud Workflows syntax
#[allow(dead_code)]
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
