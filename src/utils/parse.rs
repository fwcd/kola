use tree_sitter::Tree;

/// Creates a debug representation of the given parse tree.
pub fn format_tree(tree: &Tree) -> String {
    let mut cursor = tree.root_node().walk();
    let mut lines = Vec::new();
    'outer: loop {
        lines.push(format!("{}{:?}", " ".repeat(cursor.depth() as usize), cursor.node()));
        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        while cursor.goto_parent() {
            if cursor.goto_next_sibling() {
                continue 'outer;
            }
        }
        break;
    }
    lines.join("\n")
}
