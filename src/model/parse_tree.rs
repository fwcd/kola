// TODO: ref-cast

use std::{hash::Hash, ops::Deref};

use tree_sitter::Tree;

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct ParseTree(Tree);

impl From<Tree> for ParseTree {
    fn from(tree: Tree) -> Self {
        Self(tree)
    }
}

impl PartialEq for ParseTree {
    fn eq(&self, other: &Self) -> bool {
        // We ignore language for now, if we get an equivalent AST that is good
        // enough for us for now to consider the trees equal.
        self.0.root_node() == other.0.root_node()
    }
}

impl Eq for ParseTree {}

impl Hash for ParseTree {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.root_node().hash(state)
    }
}

impl Deref for ParseTree {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
