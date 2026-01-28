yaml-tftpl-lsp: Implementation Plan
Overview
A Rust-based Language Server Protocol (LSP) implementation that provides syntax validation for .yaml.tftpl files - Terraform template files containing Google Cloud Workflows YAML syntax.
The Challenge
These files have a dual nature:
1. Google Cloud Workflows YAML syntax - runtime syntax using $${...} expressions (e.g., $${sys.now()}, $${variable_name})
2. Terraform template interpolation - build-time syntax using ${...} expressions (e.g., ${project_id}, ${jsonencode(config)})
Standard YAML LSPs flag these expressions as invalid, creating noise. There is no official JSON schema for Google Cloud Workflows syntax.
---
Architecture
yaml-tftpl-lsp/
├── Cargo.toml                      # Rust dependencies
├── Cargo.lock
├── README.md                       # Usage documentation
├── LICENSE
├── src/
│   ├── main.rs                     # LSP server entry point & stdio transport
│   ├── backend.rs                  # LanguageServer trait implementation
│   ├── document.rs                 # Document state management
│   ├── parser/
│   │   ├── mod.rs                  # Parser module exports
│   │   ├── preprocessor.rs         # Expression placeholder substitution
│   │   ├── yaml.rs                 # YAML parsing with error recovery
│   │   └── expressions.rs          # Terraform ${} and Workflows $${} handling
│   ├── diagnostics/
│   │   ├── mod.rs                  # Diagnostics module exports
│   │   ├── yaml_errors.rs          # YAML syntax error diagnostics
│   │   └── workflow_validator.rs   # GCP Workflows structure validation
│   └── schema/
│       ├── mod.rs                  # Schema module exports
│       └── workflows.rs            # GCP Workflows keywords & structure definitions
├── tests/
│   ├── fixtures/                   # Test .yaml.tftpl files
│   └── integration_tests.rs        # LSP integration tests
└── .github/
    └── workflows/
        ├── ci.yml                  # Build & test on PR
        └── release.yml             # Multi-platform binary releases
---
Core Components
1. Expression Preprocessor (src/parser/preprocessor.rs)
Purpose: Temporarily replace interpolation expressions with valid YAML placeholders before parsing.
Algorithm:
1. Scan for Terraform expressions: ${...} (handles nested braces)
2. Scan for Workflows expressions: $${...} (handles nested braces)
3. Replace each with a unique placeholder: __EXPR_001__, __EXPR_002__, etc.
4. Store mapping: placeholder -> (original_text, start_position, end_position)
5. Return preprocessed text and expression map
Regex Patterns:
// Terraform interpolation: ${...}
static TERRAFORM_EXPR: &str = r#"\$\{(?:[^{}]|\{[^{}]*\})*\}"#;
// Workflows runtime expression: $${...}
static WORKFLOWS_EXPR: &str = r#"\$\$\{(?:[^{}]|\{[^{}]*\})*\}"#;
Edge Cases:
- Nested braces: ${jsonencode({key: value})}
- Escaped quotes inside expressions
- Multi-line expressions
- Expressions inside YAML strings vs. as values
2. YAML Parser (src/parser/yaml.rs)
Purpose: Parse preprocessed YAML and collect syntax errors with positions.
Dependencies: 
- serde_yaml (0.9+) or serde_yml for parsing
- Consider yaml-rust2 for better error location reporting
Error Mapping:
struct YamlError {
    message: String,
    line: u32,      // 0-indexed
    column: u32,    // 0-indexed
    end_line: Option<u32>,
    end_column: Option<u32>,
}
Position Adjustment:
- After parsing, adjust error positions if they fall within placeholder ranges
- Map placeholder positions back to original expression positions
3. GCP Workflows Schema (src/schema/workflows.rs)
Purpose: Define the structure and keywords of Google Cloud Workflows syntax.
Reserved Keywords (from GCP documentation):
pub const WORKFLOW_KEYWORDS: &[&str] = &[
    // Step types
    "assign", "call", "switch", "for", "parallel", "try", "raise", "return", "next",
    
    // Call step
    "args", "result",
    
    // Switch step
    "condition",
    
    // For loop
    "value", "index", "range", "in",
    
    // Parallel step
    "branches", "shared", "concurrency_limit", "exception_policy",
    
    // Try/except
    "except", "retry", "as",
    
    // Retry policy
    "predicate", "max_retries", "backoff", "initial_delay", "max_delay", "multiplier",
    
    // Workflow structure
    "main", "params", "steps",
];
Structure Rules:
pub struct WorkflowValidationRules {
    // Top-level must have 'main' or be a subworkflow definition
    // 'main' must have 'steps' (list)
    // Each step is a map with exactly one key (step name)
    // Step value contains action keywords
    // Subworkflows: name -> { params: [...], steps: [...] }
}
4. Workflow Validator (src/diagnostics/workflow_validator.rs)
Purpose: Validate GCP Workflows structure beyond basic YAML syntax.
Validation Rules (MVP):
| Rule | Severity | Message |
|------|----------|---------|
| main block required for main workflow | Error | "Workflow must have a 'main' block" |
| steps required in main | Error | "'main' block must contain 'steps'" |
| Steps must be a list | Error | "'steps' must be a list" |
| Each step must have exactly one key | Warning | "Step should have exactly one named key" |
| Subworkflow must have params or steps | Warning | "Subworkflow should define 'params' or 'steps'" |
| Unknown top-level key | Hint | "Unknown workflow element: '{key}'" |
5. LSP Backend (src/backend.rs)
Purpose: Implement the LanguageServer trait from tower-lsp.
Capabilities:
ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Kind(
        TextDocumentSyncKind::FULL,  // Receive full document on change
    )),
    // Future enhancements:
    // hover_provider: Some(HoverProviderCapability::Simple(true)),
    // completion_provider: Some(CompletionOptions { ... }),
    ..Default::default()
}
Event Handlers:
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult>;
    async fn initialized(&self, params: InitializedParams);
    async fn shutdown(&self) -> Result<()>;
    
    async fn did_open(&self, params: DidOpenTextDocumentParams);
    async fn did_change(&self, params: DidChangeTextDocumentParams);
    async fn did_save(&self, params: DidSaveTextDocumentParams);
    async fn did_close(&self, params: DidCloseTextDocumentParams);
}
Diagnostic Flow:
did_open / did_change
    ↓
preprocess_expressions(text)
    ↓
parse_yaml(preprocessed_text)
    ↓
adjust_error_positions(errors, expression_map)
    ↓
validate_workflow_structure(parsed_yaml)
    ↓
publish_diagnostics(uri, diagnostics, version)
---
Dependencies (Cargo.toml)
[package]
name = "yaml-tftpl-lsp"
version = "0.1.0"
edition = "2021"
description = "LSP for YAML Terraform template files with GCP Workflows syntax"
license = "MIT"
repository = "https://github.com/<org>/yaml-tftpl-lsp"
[dependencies]
# LSP implementation
tower-lsp = "0.20"
# Alternative: tower-lsp-server from community
# Async runtime
tokio = { version = "1", features = ["full"] }
# YAML parsing
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
# Alternative: yaml-rust2 for better error positions
# Expression parsing
regex = "1"
lazy_static = "1.4"
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
[dev-dependencies]
tempfile = "3"
assert_matches = "1"
[profile.release]
lto = true
codegen-units = 1
strip = true
---
File Type Detection
The LSP should activate for these file patterns:
- *.yaml.tftpl
- *.yml.tftpl
- *.tftpl (with YAML content detection)
Content Detection (for generic .tftpl files):
fn is_yaml_content(text: &str) -> bool {
    // Check for YAML indicators:
    // - Lines starting with "- " (list items)
    // - Lines containing ": " (key-value pairs)
    // - Common workflow keywords
    let yaml_patterns = [
        r"^\s*-\s+",           // List item
        r"^\s*\w+:\s*",        // Key-value
        r"^\s*main:\s*$",      // Workflow main
        r"^\s*steps:\s*$",     // Steps block
    ];
    // ...
}
---
Editor Configurations
Zed Editor
Create .zed/settings.json in the project or ~/.config/zed/settings.json globally:
{
  file_types: {
    YAML: [*.yaml.tftpl, *.yml.tftpl]
  },
  lsp: {
    yaml-tftpl-lsp: {
      binary: {
        path: /usr/local/bin/yaml-tftpl-lsp,
        arguments: []
      }
    }
  },
  languages: {
    YAML: {
      language_servers: [yaml-tftpl-lsp, ...],
      format_on_save: off
    }
  }
}
Alternative: Create a Zed extension that bundles the LSP binary.
OpenCode
Create or update opencode.json in the project root:
{
  $schema: https://opencode.ai/config.json,
  lsp: {
    yaml-tftpl-lsp: {
      command: [yaml-tftpl-lsp],
      extensions: [.yaml.tftpl, .yml.tftpl]
    }
  }
}
Or with explicit path:
{
  $schema: https://opencode.ai/config.json,
  lsp: {
    yaml-tftpl-lsp: {
      command: [/path/to/yaml-tftpl-lsp],
      extensions: [.yaml.tftpl, .yml.tftpl],
      env: {
        RUST_LOG: info
      }
    }
  }
}
VS Code (Future)
Create a VS Code extension or configure via settings:
{
  files.associations: {
    *.yaml.tftpl: yaml,
    *.yml.tftpl: yaml
  }
}
---
Implementation Phases
Phase 1: Project Setup
- [X] Initialize Cargo project
- [X] Add dependencies
- [X] Create module structure
- [X] Add README with project description
Phase 2: Expression Preprocessor
- [X] Implement regex patterns for ${...} and $${...}
- [X] Handle nested braces correctly
- [X] Create placeholder substitution system
- [X] Build expression position mapping
- [X] Unit tests with various expression patterns
Phase 3: YAML Parser Integration
- [ ] Integrate serde_yaml or yaml-rust2
- [ ] Extract parse errors with positions
- [ ] Adjust error positions for placeholder substitution
- [ ] Unit tests with malformed YAML
Phase 4: LSP Server Foundation
- [ ] Implement LanguageServer trait
- [ ] Set up stdio transport
- [ ] Handle document sync events
- [ ] Implement publish_diagnostics
- [ ] Add logging with tracing
Phase 5: Diagnostic Publishing
- [ ] Convert YAML errors to LSP diagnostics
- [ ] Map error positions to line/column
- [ ] Test with real .yaml.tftpl files from the project
Phase 6: Workflow Structure Validation (Optional MVP Enhancement)
- [ ] Define workflow schema rules
- [ ] Implement structural validator
- [ ] Add warnings for unknown keys
- [ ] Add hints for best practices
Phase 7: Testing & Documentation
- [ ] Integration tests with LSP client
- [ ] Test with fixture files from terraform-modules-trigger
- [ ] Document installation and configuration
- [ ] Add troubleshooting guide
Phase 8: CI/CD & Release
- [ ] GitHub Actions for CI (build, test, lint)
- [ ] Multi-platform release builds (Linux, macOS, Windows)
- [ ] Create GitHub releases with binaries
- [ ] Add installation instructions
---
Test Fixtures
Use files from the current repository as test fixtures:
tests/fixtures/
├── valid/
│   ├── workflow.yaml.tftpl          # Copy from templates/workflow.yaml.tftpl
│   ├── dataform.yaml.tftpl          # Copy from templates/steps/dataform.yaml.tftpl
│   └── error_handling.yaml.tftpl    # Copy from templates/common/error_handling.yaml.tftpl
├── invalid/
│   ├── missing_colon.yaml.tftpl     # YAML syntax error
│   ├── bad_indentation.yaml.tftpl   # Indentation error
│   └── unclosed_brace.yaml.tftpl    # Expression with unclosed brace
└── edge_cases/
    ├── nested_expressions.yaml.tftpl
    ├── multiline_expression.yaml.tftpl
    └── mixed_expressions.yaml.tftpl
---
Future Enhancements (Post-MVP)
| Feature | Priority | Description |
|---------|----------|-------------|
| Autocompletion | High | Suggest workflow keywords, step types |
| Hover documentation | High | Show docs for call, switch, retry, etc. |
| Go to definition | Medium | Navigate to subworkflow definitions |
| Terraform variable validation | Medium | Validate ${var} references exist |
| Code actions | Low | Quick fixes for common issues |
| Formatting | Low | Auto-format workflow files |
| Snippets | Low | Insert common patterns (try/except, parallel) |
---
References
Google Cloud Workflows
- Syntax Overview (https://cloud.google.com/workflows/docs/reference/syntax)
- Standard Library (https://cloud.google.com/workflows/docs/reference/stdlib/overview)
- Syntax Cheat Sheet (https://cloud.google.com/workflows/docs/reference/syntax/syntax-cheat-sheet)
Terraform Templates
- templatefile Function (https://developer.hashicorp.com/terraform/language/functions/templatefile)
- Template syntax: ${...} for interpolation, %{...} for directives
LSP Resources
- LSP Specification (https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- tower-lsp crate (https://docs.rs/tower-lsp/latest/tower_lsp/)
