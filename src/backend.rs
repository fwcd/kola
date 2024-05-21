use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::{jsonrpc::Result, lsp_types::{DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url}, Client, LanguageServer};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_map: DashMap<Url, Rope>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
        }
    }

    async fn on_change(&self, uri: Url, text: &str) {
        self.document_map.insert(uri, Rope::from_str(text));
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Kola".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
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
        self.client.log_message(MessageType::INFO, format!("Opened {}", params.text_document.uri)).await;
        self.on_change(
            params.text_document.uri,
            &params.text_document.text,
        ).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // TODO: Incremental sync (requires us to specify the corresponding server capability)
        self.client.log_message(MessageType::INFO, format!("Changed {}", params.text_document.uri)).await;
        self.on_change(
            params.text_document.uri,
            &params.content_changes[0].text,
        ).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // TODO: Incremental sync (requires us to specify the corresponding server capability)
        self.client.log_message(MessageType::INFO, format!("Closed {}", params.text_document.uri)).await;
    }
}
