use tree_sitter::Tree;

use super::Jar;

#[salsa::input(jar = Jar)]
pub struct Input {
    #[return_ref]
    pub bytes: Vec<u8>,
    #[return_ref]
    pub tree: Tree,
}
