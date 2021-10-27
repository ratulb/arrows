use crate::actors::SysActors;
use arrows_common::Message;
use async_std::{fs::DirBuilder, path::PathBuf, task::block_on};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mailbox {
    outbox: VecDeque<Message>,
    inbox: VecDeque<Message>,
}

impl Mailbox {
    pub(crate) fn add_to_inbox(&mut self, msg: Message) {
        self.inbox.push_back(msg);
    }
    pub(crate) fn unread_count(&self) -> usize {
        self.inbox.len()
    }

    pub(crate) fn read_inbox(&mut self) -> Option<Message> {
        self.inbox.pop_front()
    }
    pub(crate) fn add_to_outbox(&mut self, msg: Message) {
        self.outbox.push_back(msg);
    }
    pub(crate) fn outgoing_count(&self) -> usize {
        self.outbox.len()
    }
    pub(crate) fn send_outgoing(&mut self) -> Option<Message> {
        self.outbox.pop_front()
    }
}

lazy_static! {
    pub(crate) static ref STORE: RwLock<BoxStore> =
        block_on(async { RwLock::new(BoxStore::init().await) });
}

#[derive(Debug)]
pub(crate) struct BoxStore {
    pub(crate) process_dir: PathBuf,
    pub(crate) outboxes: HashMap<u64, Mailbox>,
    pub(crate) inboxes: HashMap<u64, Mailbox>,
    pub(crate) sys_actors: SysActors,
}

impl BoxStore {
    pub async fn init() -> BoxStore {
        let sys_actors = SysActors::new();
        let directory = Self::process_dir().await;
        if !directory.exists().await || !directory.is_dir().await {
            println!("Process dir does not exists. Creating...");
            let builder = DirBuilder::new();
            builder.create(&directory.as_path()).await;
        }
        println!("System startup check 3 : Store got initialized");
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
