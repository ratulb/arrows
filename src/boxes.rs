use crate::actors::SysActors;
use async_std::{fs::DirBuilder, path::PathBuf, task::block_on};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MailBox {
    Outbox {
        read_offset: u64,
        write_offset: u64,
        open_for_read: bool,
        open_for_write: bool,
    },
    Inbox {
        read_offset: u64,
        write_offset: u64,
        open_for_read: bool,
        open_for_write: bool,
    },
}

lazy_static! {
    pub(crate) static ref STORE: RwLock<BoxStore> =
        block_on(async { RwLock::new(BoxStore::init().await) });
}

#[derive(Debug)]
pub(crate) struct BoxStore {
    pub(crate) process_dir: PathBuf,
    pub(crate) outboxes: HashMap<u64, MailBox>,
    pub(crate) inboxes: HashMap<u64, MailBox>,
    pub(crate) sys_actors: SysActors,
}

impl BoxStore {
    pub async fn init() -> Self {
        let sys_actors = SysActors::new();
        let directory = Self::process_dir().await;
        if !directory.exists().await || !directory.is_dir().await {
            println!("Process dir does not exists. Creating...");
            let builder = DirBuilder::new();
            builder.create(&directory.as_path()).await;
        }
        Self {
            sys_actors,
            process_dir: directory,
            outboxes: HashMap::new(),
            inboxes: HashMap::new(),
        }
    }
    pub async fn get_dir(&self) -> &PathBuf {
        &self.process_dir
    }

    pub async fn process_dir() -> PathBuf {
        let mut path_buf = std::env::current_dir().expect("Current dir call should not fail");
        path_buf.push("data");
        path_buf.into()
    }
}
