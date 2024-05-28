use tree_sitter::Tree;

use super::Jar;

#[salsa::input(jar = Jar)]
pub struct Parse {
    #[return_ref]
    pub tree: Tree,
}
