use ropey::Rope;
use tower_lsp::lsp_types::Position;

pub trait RopeExt {
    fn position_to_byte(&self, position: Position) -> usize;

    fn position_to_char(&self, position: Position) -> usize;

    fn byte_to_position(&self, byte_idx: usize) -> Position;

    fn char_to_position(&self, char_idx: usize) -> Position;
}

impl RopeExt for Rope {
    fn position_to_byte(&self, position: Position) -> usize {
        self.char_to_byte(self.position_to_char(position))
    }

    fn position_to_char(&self, position: Position) -> usize {
        let line_char = self.line_to_char(position.line as usize);
        line_char + position.character as usize
    }

    fn byte_to_position(&self, byte_idx: usize) -> Position {
        self.char_to_position(self.byte_to_char(byte_idx))
    }

    fn char_to_position(&self, char_idx: usize) -> Position {
        let line = self.char_to_line(char_idx);
        let line_char = self.line_to_char(line);
        Position {
            line: line as u32,
            character: (char_idx - line_char) as u32,
        }
    }
}
