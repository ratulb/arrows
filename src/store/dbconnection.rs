use crate::common::config::Config;
use crate::constants::DATABASE;
use rusqlite::Connection;
use std::path::PathBuf;

pub(crate) struct DBConnection {
    pub inner: Connection,
}
impl DBConnection {
    pub(crate) fn new() -> Self {
        let path = Config::get_shared().db_path().to_string();
        println!("Using db path {}", path);
        let mut path = PathBuf::from(path);
        path.push(DATABASE);
        let result = Connection::open(path);
        match result {
            Ok(inner) => {
                inner.set_prepared_statement_cache_capacity(100); //TODO make it configurable
                Self { inner }
            }
            Err(err) => {
                panic!("Failed to obtain db connection: {}", err);
            }
        }
    }
}
