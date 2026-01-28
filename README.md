# yaml-tftpl-lsp

A Rust-based Language Server Protocol (LSP) implementation that provides syntax validation for `.yaml.tftpl` files - Terraform template files containing Google Cloud Workflows YAML syntax.

## The Challenge

These files have a dual nature:

1. **Google Cloud Workflows YAML syntax** - runtime syntax using `$${...}` expressions (e.g., `$${sys.now()}`, `$${variable_name}`)
2. **Terraform template interpolation** - build-time syntax using `${...}` expressions (e.g., `${project_id}`, `${jsonencode(config)}`)

Standard YAML LSPs flag these expressions as invalid, creating noise. This LSP preprocesses these expressions before YAML validation, eliminating false positives.

## Features

- YAML syntax validation with expression-aware preprocessing
- Handles Terraform `${...}` interpolations
- Handles GCP Workflows `$${...}` runtime expressions
- Supports nested braces in expressions

## Installation

### Building from source

```bash
cargo build --release
```

The binary will be available at `target/release/yaml-tftpl-lsp`.

## Editor Configuration

### Zed Editor

Add to your `.zed/settings.json`:

```json
{
  "file_types": {
    "YAML": ["*.yaml.tftpl", "*.yml.tftpl"]
  },
  "lsp": {
    "yaml-tftpl-lsp": {
      "binary": {
        "path": "/path/to/yaml-tftpl-lsp"
      }
    }
  },
  "languages": {
    "YAML": {
      "language_servers": ["yaml-tftpl-lsp"]
    }
  }
}
```

### OpenCode

Add to your `opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "lsp": {
    "yaml-tftpl-lsp": {
      "command": ["yaml-tftpl-lsp"],
      "extensions": [".yaml.tftpl", ".yml.tftpl"]
    }
  }
}
```

## Usage

The LSP server communicates over stdio. Run it directly or configure your editor to launch it for `.yaml.tftpl` files.

```bash
# Run with debug logging
RUST_LOG=debug yaml-tftpl-lsp
```

## Diagnostics

The LSP provides two layers of validation:

### YAML Syntax Errors (Error severity)
Standard YAML parse errors such as bad indentation, unclosed quotes, or missing colons. Terraform `${...}` and Workflows `$${...}` expressions are preprocessed into safe placeholders before parsing, so they won't trigger false positives.

### Workflow Structure Warnings (Warning/Hint severity)
Validates GCP Workflows conventions:
- **Warning**: Missing `main` block, missing `steps` in a workflow block, `steps` not being a list
- **Hint**: Unknown top-level keys, unknown step actions

## Troubleshooting

### Server doesn't start

Verify the binary runs and check for errors:

```bash
echo '{}' | RUST_LOG=debug yaml-tftpl-lsp
```

If you see no output, the server is waiting for LSP messages on stdin — this is normal. Check your editor's LSP log output for initialization errors.

### No diagnostics appear

1. Confirm the file extension matches your editor configuration (`.yaml.tftpl` or `.yml.tftpl`).
2. Check that your editor associates the file type with the LSP. For Zed, verify `file_types` maps to `YAML`.
3. Enable debug logging (`RUST_LOG=debug`) and check the server output in your editor's LSP logs.

### False positives on expressions

The preprocessor handles `${...}` (Terraform) and `$${...}` (Workflows) expressions, including nested braces like `${jsonencode({key: value})}`. If you see false errors on expressions:

- Ensure braces are balanced — unclosed `${...` won't be preprocessed and will be passed through as-is to the YAML parser.
- Multi-line expressions spanning multiple lines are supported.

### Unexpected workflow warnings

The server assumes `.yaml.tftpl` files contain GCP Workflows definitions. If your file is plain YAML (not a workflow), you may see warnings like "Workflow must have a 'main' block". These are informational and can be ignored for non-workflow files.

## License

MIT
