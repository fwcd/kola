use tower_lsp::lsp_types::Position;
use tree_sitter::Point;

pub trait FromLsp<T> {
    fn from_lsp(value: T) -> Self;
}

impl FromLsp<Position> for Point {
    fn from_lsp(value: Position) -> Self {
        Self {
            row: value.line as usize,
            column: value.character as usize,
        }
    }
}
