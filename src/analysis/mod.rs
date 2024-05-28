pub mod ast;
pub mod parse;

// Define jars and databases for Salsa as per https://salsa-rs.github.io/salsa/tutorial/jar.html

#[salsa::jar(db = Db)]
pub struct Jar(
    ast::AST,
    ast::Function,
    parse::Parse,
);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}

#[salsa::db(Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}
