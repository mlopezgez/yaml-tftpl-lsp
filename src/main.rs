//! yaml-tftpl-lsp: LSP server for YAML Terraform template files with GCP Workflows syntax

use tower_lsp::{LspService, Server};
use tracing_subscriber::EnvFilter;

use yaml_tftpl_lsp::Backend;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting yaml-tftpl-lsp server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
