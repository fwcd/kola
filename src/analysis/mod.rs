use std::sync::{Arc, Mutex};

use salsa::DebugWithDb;

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
pub struct Database {
    storage: salsa::Storage<Self>,
    logs: Arc<Mutex<Vec<String>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            storage: Default::default(),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn logs(&self) -> &Arc<Mutex<Vec<String>>> {
        &self.logs
    }
}

impl salsa::Database for Database {
    fn salsa_event(&self, event: salsa::Event) {
        if let salsa::EventKind::WillExecute { .. } = event.kind {
            let mut logs = self.logs.lock().unwrap();
            logs.push(format!("Event: {:?}", event.debug(self)));
        }
    }
}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Database {
            storage: self.storage.snapshot(),
            logs: self.logs.clone(),
        })
    }
}
