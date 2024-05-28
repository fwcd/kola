pub mod ast;
pub mod input;

// Define jars and databases for Salsa as per https://salsa-rs.github.io/salsa/tutorial/jar.html

#[salsa::jar(db = Db)]
pub struct Jar(
    ast::AST,
    ast::ast,
    ast::Function,
    input::Input,
);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}

#[salsa::db(Jar)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Database {
            storage: self.storage.snapshot(),
        })
    }
}
