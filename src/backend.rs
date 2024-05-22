use dashmap::DashMap;
use ropey::Rope;
use tokio::sync::Mutex;
use tower_lsp::{jsonrpc::Result, lsp_types::{CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url}, Client, LanguageServer};
use tree_sitter::{Parser, Query, QueryCursor, Tree};

use crate::utils::format_tree;

pub struct Backend {
    client: Client,
    document_map: DashMap<Url, Rope>,
    parse_map: DashMap<Url, Tree>,
    functions_map: DashMap<Url, Vec<String>>,
    parser: Mutex<Parser>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_kotlin::language()).expect("Could not load Kotlin grammar");
        Self {
            client,
            document_map: DashMap::new(),
            parse_map: DashMap::new(),
            functions_map: DashMap::new(),
            parser: Mutex::new(parser),
        }
    }

    // TODO: Use salsa to manage incremental parsing/recompilation etc.

    async fn on_change(&self, uri: Url, text: &str) {
        self.document_map.insert(uri.clone(), Rope::from_str(text));
        // TODO: How do we handle old_tree properly for incremental parsing?
        if let Some(tree) = self.parser.lock().await.parse(text, None) {
            self.client.log_message(MessageType::INFO, format!("Parsed\n{}", format_tree(&tree))).await;

            // Query function declarations (for proof-of-concept code completion)
            let query = Query::new(&tree_sitter_kotlin::language(), "(function_declaration (simple_identifier) @name)").unwrap(); // TODO: Use proper error handling
            let mut cursor = QueryCursor::new();
            let mut functions = Vec::new();
            for query_match in cursor.matches(&query, tree.root_node(), text.as_bytes()) {
                let name = query_match.captures[0].node.utf8_text(text.as_bytes()).unwrap().to_owned(); // TODO: Use proper error handling
                functions.push(name);
            }
            self.functions_map.insert(uri.clone(), functions);

            self.parse_map.insert(uri, tree);
        } else {
            self.parse_map.remove(&uri);
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // TODO: Filter by prefix etc.
        let uri = params.text_document_position.text_document.uri;
        Ok(self.functions_map.get(&uri).map(|functions|
            CompletionResponse::Array(
                functions.iter().map(|name| CompletionItem {
                    label: name.to_owned(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    ..Default::default()
                }).collect()
            )
        ))
    }
}
