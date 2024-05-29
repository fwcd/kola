use comemo::memoize;
use tree_sitter::{Query, QueryCursor};

use crate::model::{Document, ParseTree};

#[memoize]
pub fn declared_function_names(document: Document, parse_tree: ParseTree) -> Vec<String> {
    // Query function declarations (for proof-of-concept code completion)
    let bytes: Vec<_> = document.bytes().collect();
    let query = Query::new(&tree_sitter_kotlin::language(), "(function_declaration (simple_identifier) @name)").unwrap(); // TODO: Use proper error handling
    let mut cursor = QueryCursor::new();
    let mut functions = Vec::new();
    for query_match in cursor.matches(&query, parse_tree.root_node(), &bytes as &[u8]) {
        let name = query_match.captures[0].node.utf8_text(&bytes).unwrap().to_owned(); // TODO: Use proper error handling
        functions.push(name);
    }
    functions
}
