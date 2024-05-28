use tree_sitter::{Query, QueryCursor};

use super::{input::Input, Db, Jar};

#[salsa::tracked(jar = Jar)]
pub struct AST {
    pub functions: Vec<Function>,
}

#[salsa::tracked(jar = Jar)]
pub struct Function {
    pub name: String,
}

#[salsa::tracked(jar = Jar)]
pub fn ast(db: &dyn Db, input: Input) -> AST {
    // Query function declarations (for proof-of-concept code completion)
    let query = Query::new(&tree_sitter_kotlin::language(), "(function_declaration (simple_identifier) @name)").unwrap(); // TODO: Use proper error handling
    let mut cursor = QueryCursor::new();
    let mut functions = Vec::new();
    for query_match in cursor.matches(&query, input.tree(db).root_node(), input.bytes(db) as &[u8]) {
        let name = query_match.captures[0].node.utf8_text(input.bytes(db)).unwrap().to_owned(); // TODO: Use proper error handling
        functions.push(Function::new(db, name));
    }
    AST::new(db, functions)
}
