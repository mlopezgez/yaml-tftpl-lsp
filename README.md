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

## License

MIT
