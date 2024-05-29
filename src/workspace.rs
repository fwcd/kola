use comemo::track;
use dashmap::DashMap;
use ropey::Rope;
use tokio::sync::Mutex;
use tower_lsp::{lsp_types::{MessageType, TextDocumentContentChangeEvent, Url}, Client};
use tree_sitter::{InputEdit, Parser, Point, Tree};

use crate::{model::{Document, ParseTree}, utils::{format_tree, FromLsp, RopeExt}};

pub struct Workspace {
    client: Client,
    document_map: DashMap<Url, Rope>,
    parse_map: DashMap<Url, Tree>,
    parser: Mutex<Parser>,
}

impl Workspace {
    pub fn new(client: Client) -> Self {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_kotlin::language()).expect("Could not load Kotlin grammar");
        Self {
            client,
            document_map: DashMap::new(),
            parse_map: DashMap::new(),
            parser: Mutex::new(parser),
        }
    }

    pub async fn update(&self, uri: Url, edit: TextDocumentContentChangeEvent) {
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
}

#[track]
impl Workspace {
    pub fn document(&self, uri: &Url) -> Option<Document> {
        self.document_map.get(uri).map(|rope| Document::from(rope.clone()))
    }

    pub fn bytes(&self, uri: &Url) -> Option<Vec<u8>> {
        self.document_map.get(uri).map(|rope| rope.bytes().collect())
    }

    pub fn parse_tree(&self, uri: &Url) -> Option<ParseTree> {
        self.parse_map.get(uri).map(|tree| ParseTree::from(tree.clone()))
    }
}
