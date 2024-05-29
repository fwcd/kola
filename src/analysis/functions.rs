use comemo::{memoize, Tracked};
use tower_lsp::lsp_types::Url;
use tree_sitter::{Query, QueryCursor};

use crate::workspace::Workspace;

#[memoize]
pub fn declared_function_names(uri: Url, workspace: Tracked<Workspace>) -> Vec<String> {
    // Query function declarations (for proof-of-concept code completion)
    let Some(bytes) = workspace.bytes(&uri) else { return Vec::new() };
    let Some(parse_tree) = workspace.parse_tree(&uri) else { return Vec::new() };
    let query = Query::new(&tree_sitter_kotlin::language(), "(function_declaration (simple_identifier) @name)").unwrap(); // TODO: Use proper error handling
    let mut cursor = QueryCursor::new();
    let mut functions = Vec::new();
    for query_match in cursor.matches(&query, parse_tree.root_node(), &bytes as &[u8]) {
        let name = query_match.captures[0].node.utf8_text(&bytes).unwrap().to_owned(); // TODO: Use proper error handling
        functions.push(name);
    }
    functions
}
