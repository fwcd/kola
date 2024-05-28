use dashmap::DashMap;
use ropey::Rope;
use tokio::sync::Mutex;
use tower_lsp::{jsonrpc::Result, lsp_types::{CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities, ServerInfo, TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, Url}, Client, LanguageServer};
use tree_sitter::{InputEdit, Parser, Point, Tree};

use crate::{analysis::{ast::ast, input::Input, Database}, utils::{format_tree, FromLsp, RopeExt}};

pub struct Backend {
    client: Client,
    document_map: DashMap<Url, Rope>,
    parse_map: DashMap<Url, Tree>,
    db: Mutex<Database>,
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
            db: Mutex::new(Database::new()),
            parser: Mutex::new(parser),
        }
    }

    // TODO: Use salsa to manage incremental parsing/recompilation etc.

    async fn update(&self, uri: Url, edit: TextDocumentContentChangeEvent) {
        if let Some(range) = edit.range {
            // Existing document changed

            // Update rope
            let mut rope = self.document_map.get_mut(&uri).expect("Could not find document");

            let start_char = rope.position_to_char(range.start);
            let old_end_char = rope.position_to_char(range.end);
            let start_byte = rope.char_to_byte(start_char);
            let old_end_byte = rope.char_to_byte(old_end_char);

            rope.try_remove(start_char..old_end_char).unwrap_or_else(|e| panic!("Could not remove chars {}-{}: {}", start_char, old_end_char, e));
            rope.try_insert(start_char, &edit.text).unwrap_or_else(|e| panic!("Could not insert text '{}': {}", edit.text, e));

            let new_end_char = start_char + edit.text.chars().count();
            let new_end_byte = rope.char_to_byte(new_end_char);
            let new_end = rope.char_to_position(new_end_char);

            // Repair tree
            let mut old_tree = self.parse_map.get_mut(&uri);
            if let Some(old_tree) = old_tree.as_mut() {
                old_tree.edit(&InputEdit {
                    start_byte,
                    old_end_byte,
                    new_end_byte,
                    start_position: Point::from_lsp(range.start),
                    old_end_position: Point::from_lsp(range.end),
                    new_end_position: Point::from_lsp(new_end),
                });
            }
        } else {
            // New document opened
            let rope = Rope::from_str(&edit.text);
            self.document_map.insert(uri.clone(), rope);
        }

        let bytes = self.document_map.get(&uri).unwrap().bytes().collect::<Vec<u8>>();

        if let Some(tree) = {
            let mut parser = self.parser.lock().await;
            let old_tree = self.parse_map.get(&uri);
            parser.parse(&bytes, old_tree.as_deref())
        } {
            self.client.log_message(MessageType::INFO, format!("Parsed {}\n{}", uri, format_tree(&tree))).await;
            self.parse_map.insert(uri, tree);
        } else {
            self.client.log_message(MessageType::WARNING, format!("Could not parse {}", uri)).await;
            self.parse_map.remove(&uri);
        }
    }

    async fn flush_db_logs(&self) {
        let logs: Vec<_> = self.db.lock().await.logs().lock().unwrap().drain(..).collect();
        for log in logs {
            self.client.log_message(MessageType::INFO, format!("Db: {}", log)).await;
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
        self.update(uri, TextDocumentContentChangeEvent {
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
            self.update(uri.clone(), edit).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client.log_message(MessageType::INFO, format!("Closed {}", params.text_document.uri)).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // TODO: Filter by prefix etc.
        let uri = params.text_document_position.text_document.uri;
        let Some(rope) = self.document_map.get(&uri) else {
            return Ok(None);
        };
        let Some(tree) = self.parse_map.get(&uri) else {
            return Ok(None);
        };
        let bytes = rope.bytes().collect::<Vec<_>>();
        let completions = {
            let db = self.db.lock().await;
            let input = Input::new(&*db, bytes, tree.clone());
            let ast = ast(&*db, input);
            ast.functions(&*db).iter().map(|f|
                CompletionItem {
                    label: f.name(&*db).to_owned(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    ..Default::default()
                }
            ).collect()
        };
        // TODO: Find a more elegant solution, perhaps we could write a db lock guard wrapper that 
        // automatically flushes the logs when dropped?
        self.flush_db_logs().await;
        Ok(Some(CompletionResponse::Array(completions)))
    }
}
