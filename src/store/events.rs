use crate::constants::EVENTS_INSERT;
use crate::constants::{BUFFER_MAX_SIZE, EVENT_MAX_AGE};
use crate::registry::Context;

use crate::routing::Router;
use crate::DetailedMsg;
use rusqlite::{hooks::Action, Result, ToSql, Transaction};
use serde::{ser::SerializeTupleStruct, Deserialize, Serialize, Serializer};
use std::mem;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) enum Events {
    Stop,
    DbUpdate(DBEvent),
}

pub(crate) struct DBEvent(pub i64);

impl DBEvent {
    pub(crate) fn persist(&self, tx: &Transaction<'_>) -> Result<usize> {
        let DBEvent(row_id) = self;
        tx.execute(EVENTS_INSERT, &[&row_id as &dyn ToSql])
    }
}

impl std::fmt::Debug for DBEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DBEvent").field("row_id", &self.0).finish()
    }
}

impl Serialize for DBEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut event = serializer.serialize_tuple_struct("DBEvent", 1)?;
        event.serialize_field(&self.0)?;
        event.end()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum DBAction {
    Insert,
    Delete,
    Update,
    Unknown,
}

impl From<Action> for DBAction {
    fn from(action: Action) -> Self {
        match action {
            Action::SQLITE_DELETE => DBAction::Delete,
            Action::SQLITE_INSERT => DBAction::Insert,
            Action::SQLITE_UPDATE => DBAction::Update,
            _ => DBAction::Unknown,
        }
    }
}

pub(crate) struct EventBuffer {
    events: Vec<DBEvent>,
    first_event_receipt_at: Option<Instant>,
}
impl EventBuffer {
    pub(crate) fn new() -> Self {
        Self {
            events: Vec::new(),
            first_event_receipt_at: None,
        }
    }
    pub(crate) fn overflown(&self) -> bool {
        self.events.len() >= BUFFER_MAX_SIZE
    }
    pub fn has_matured(&self) -> bool {
        match self.first_event_receipt_at {
            None => false,
            Some(instant) => instant.elapsed() >= Duration::new(EVENT_MAX_AGE, 0),
        }
    }
    pub(crate) fn should_flush(&self) -> bool {
        self.overflown() || self.has_matured()
    }
    pub(crate) fn add(&mut self, event: DBEvent) {
        println!("Bufferingg event = {:?}", event);
        self.events.push(event);
        if self.events.len() == 1 {
            self.first_event_receipt_at = Some(Instant::now());
        }
    }
    pub(crate) fn flush(&mut self) -> Vec<DBEvent> {
        mem::take(&mut self.events)
    }
}
impl Drop for EventBuffer {
    fn drop(&mut self) {
        let events = self.flush();
        if !events.is_empty() {
            EventTracker::perist_buffered(events);
        }
    }
}

pub(crate) struct EventTracker {
    buffer: EventBuffer,
    router: Router,
}
impl EventTracker {
    pub(crate) fn new() -> Self {
        Self {
            buffer: EventBuffer::new(),
            router: Router::new(num_cpus::get()),
        }
    }
    pub(crate) fn track(&mut self, event: DBEvent) {
        self.buffer.add(event);
        if self.buffer.should_flush() {
            let persisted_events = Self::perist_buffered(self.buffer.flush());
            let persisted_msgs = Self::load_messages(persisted_events);
            self.router.route(persisted_msgs);
        }
    }

    //Persists the events to db
    pub(crate) fn perist_buffered(events: Vec<DBEvent>) -> Vec<i64> {
        let persisted_events = Context::instance()
            .store
            .persist_events(events.into_iter())
            .expect("Events persisted");
        println!("Clearing buffer. Persisted events = {:?}", persisted_events);
        persisted_events
    }
    pub(crate) fn load_messages(rowids: Vec<i64>) -> Vec<DetailedMsg> {
        Context::instance()
            .store
            .from_messages(rowids)
            .expect("Messages")
    }
    pub(crate) fn hand_off_past_events(&mut self) {
        let events = Context::instance()
            .store
            .read_events()
            .expect("Past events");
        let msgs = Self::load_messages(events);
        println!("Handling past mags. Events = {:?}", msgs.len());
        self.router.route(msgs);
    }
}
