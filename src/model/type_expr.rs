#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeExpr {
    Identifier(String),
    Function(Vec<TypeExpr>, Box<TypeExpr>),
}
