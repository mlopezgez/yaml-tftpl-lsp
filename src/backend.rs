//! LSP Backend implementation

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::Document;

/// The LSP backend that handles all language server requests
pub struct Backend {
    /// The LSP client for sending notifications
    client: Client,
    /// Map of document URIs to their state
    documents: Arc<RwLock<HashMap<Url, Document>>>,
}

impl Backend {
    /// Create a new backend instance
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate a document and publish diagnostics
    async fn validate_document(&self, uri: &Url, text: &str, version: Option<i32>) {
        let diagnostics = self.compute_diagnostics(text);

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, version)
            .await;
    }

    /// Compute diagnostics for the given text
    fn compute_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        use crate::diagnostics::DiagnosticCollector;
        use crate::parser::preprocess_expressions;

        let mut collector = DiagnosticCollector::new();

        // Preprocess expressions to replace ${} and $${} with placeholders
        let (preprocessed, expression_map) = preprocess_expressions(text);

        // Parse YAML and collect errors
        crate::parser::parse_yaml(&preprocessed, &expression_map, &mut collector);

        collector.into_diagnostics()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "yaml-tftpl-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("Server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        tracing::debug!("Document opened: {}", uri);

        // Store document
        {
            let mut docs = self.documents.write().await;
            docs.insert(uri.clone(), Document::new(text.clone(), version));
        }

        // Validate and publish diagnostics
        self.validate_document(&uri, &text, Some(version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get the full text from the changes (we use FULL sync)
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;

            tracing::debug!("Document changed: {}", uri);

            // Update document
            {
                let mut docs = self.documents.write().await;
                docs.insert(uri.clone(), Document::new(text.clone(), version));
            }

            // Validate and publish diagnostics
            self.validate_document(&uri, &text, Some(version)).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::debug!("Document saved: {}", params.text_document.uri);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        tracing::debug!("Document closed: {}", uri);

        // Remove document from our state
        {
            let mut docs = self.documents.write().await;
            docs.remove(&uri);
        }

        // Clear diagnostics for this document
        self.client.publish_diagnostics(uri, vec![], None).await;
    }
}
