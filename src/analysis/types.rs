use comemo::{memoize, Tracked};

use crate::{model::{ASTNode, TypeExpr}, workspace::Workspace};

#[memoize]
pub fn inferred_type<F, T>(node: ASTNode, workspace: Tracked<Workspace>) -> TypeExpr {
    todo!()
}
