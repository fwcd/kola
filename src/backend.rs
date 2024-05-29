use comemo::Track;
use tower_lsp::{jsonrpc::Result, lsp_types::{CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities, ServerInfo, TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind}, Client, LanguageServer};

use crate::{analysis::syntax::declared_function_names, workspace::Workspace};

pub struct Backend {
    client: Client,
    workspace: Workspace,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client: client.clone(),
            workspace: Workspace::new(client),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    // TODO: Provide semantic tokens from parse trees

    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Kola".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            capabilities: ServerCapabilities {
                completion_provider: Some(CompletionOptions::default()),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "Server initialized!").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.client.log_message(MessageType::INFO, format!("Opened {}", uri)).await;
        self.workspace.update(uri, TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text,
        }).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let edits = params.content_changes;
        self.client.log_message(MessageType::INFO, format!("Changed {}", uri)).await;
        for edit in edits {
            self.workspace.update(uri.clone(), edit).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client.log_message(MessageType::INFO, format!("Closed {}", params.text_document.uri)).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // TODO: Filter by prefix etc.
        let uri = params.text_document_position.text_document.uri;
        Ok(Some(CompletionResponse::Array(
            declared_function_names(uri, self.workspace.track()).iter().map(|name| CompletionItem {
                label: name.to_owned(),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            }).collect()
        )))
    }
}
