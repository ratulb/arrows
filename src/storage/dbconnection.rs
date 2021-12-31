use crate::constants::{ARROWS_DB_PATH, DATABASE};
use rusqlite::Connection;
use std::path::PathBuf;

pub(crate) struct DBConnection {
    pub inner: Connection,
}
impl DBConnection {
    pub(crate) fn new() -> Self {
        let path = std::env::var(ARROWS_DB_PATH)
            .expect("Please set ARROWS_DB_PATH pointing to an existing directory!");
        let mut path = PathBuf::from(path);
        path.push(DATABASE);
        let result = Connection::open(path);
        if let Ok(inner) = result {
            inner.set_prepared_statement_cache_capacity(100); //TODO make it configurable
            Self { inner }
        } else {
            panic!("Failed to obtain db connection");
        }
    }
}
