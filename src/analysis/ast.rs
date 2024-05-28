use super::{parse::Parse, Db, Jar};

#[salsa::tracked(jar = Jar)]
pub struct AST {
    pub functions: Vec<Function>,
}

#[salsa::tracked(jar = Jar)]
pub struct Function {
    pub name: String,
}

pub fn ast(db: &dyn Db, parse: Parse) -> AST {
    todo!()
}
