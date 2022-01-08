use crate::catalog::{self};

use crate::constants::{BUFFER_MAX_SIZE, EVENTS_INSERT, EVENT_MAX_AGE};
use crate::routing::Router;

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
    earliest_event_instant: Option<Instant>,
}
impl EventBuffer {
    pub(crate) fn new() -> Self {
        Self {
            events: Vec::new(),
            earliest_event_instant: None,
        }
    }
    pub(crate) fn overflown(&self) -> bool {
        self.events.len() >= BUFFER_MAX_SIZE
    }
    pub fn has_matured(&self) -> bool {
        match self.earliest_event_instant {
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
            self.earliest_event_instant = Some(Instant::now());
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
            catalog::perist_buffered(events);
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
            let persisted_events = catalog::perist_buffered(self.buffer.flush());
            let persisted_msgs = catalog::load_messages(persisted_events);
            self.router.route(persisted_msgs);
        }
    }

    pub(crate) fn route_past_events(&mut self) {
        let msgs = catalog::past_events();
        println!("Handling past mags. Events = {:?}", msgs.len());
        self.router.route(msgs);
    }
}
