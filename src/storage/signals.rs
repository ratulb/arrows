use crate::constants::{BUCKET_MAX_SIZE, EVENT_MAX_AGE, INBOUND_INSERT};
use rusqlite::{hooks::Action, Result, ToSql, Transaction};
use serde::{ser::SerializeTupleStruct, Deserialize, Serialize, Serializer};
use std::mem;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) enum Signal {
    Stop,
    DbUpdate(DBEvent),
}

pub(crate) struct DBEvent(pub String, pub i64);

impl DBEvent {
    pub(crate) fn persist(&self, tx: &Transaction<'_>) -> Result<usize> {
        let DBEvent(tbl, row_id) = self;
        let actor_id = match tbl.find('_') {
            None => return Ok(0),
            Some(idx) => &tbl[(idx + 1)..],
        };
        tx.execute(
            INBOUND_INSERT,
            &[&row_id as &dyn ToSql, &actor_id as &dyn ToSql],
        )
    }
}

impl std::fmt::Debug for DBEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DBEvent")
            .field("table", &self.0)
            .field("row_id", &self.1)
            .finish()
    }
}

impl Serialize for DBEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut event = serializer.serialize_tuple_struct("DBEvent", 2)?;
        event.serialize_field(&self.0)?;
        event.serialize_field(&self.1)?;
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

pub(crate) struct EventBucket {
    events: Vec<DBEvent>,
    oldest_receipt_instant: Option<Instant>,
}

impl EventBucket {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            oldest_receipt_instant: None,
        }
    }

    pub fn overflown(&self) -> bool {
        self.events.len() >= BUCKET_MAX_SIZE
    }

    pub fn oldest_matured(&self) -> bool {
        match self.oldest_receipt_instant {
            None => false,
            Some(instant) => instant.elapsed() >= Duration::new(EVENT_MAX_AGE, 0),
        }
    }

    pub fn should_invoke_actors(&self) -> bool {
        self.overflown() || self.oldest_matured()
    }
    pub fn add_event(&mut self, event: DBEvent) {
        if self.should_invoke_actors() {
            let events = mem::replace(&mut self.events, Vec::new());
            Self::deliver_actor_messages(events);
        }
        self.events.push(event);
        if self.events.len() == 1 {
            self.oldest_receipt_instant = Some(Instant::now());
        }
    }

    pub fn deliver_actor_messages(events: Vec<DBEvent>) {}
}
