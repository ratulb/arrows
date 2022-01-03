use crate::apis::Storage;
use crate::constants::{BUCKET_MAX_SIZE, EVENT_MAX_AGE, INBOUND_INSERT, INBOX, OUTBOUND_INSERT};
use rusqlite::{hooks::Action, Result, ToSql, Transaction};
use serde::{ser::SerializeTupleStruct, Deserialize, Serialize, Serializer};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) enum Signal {
    Stop,
    DbUpdate(DBEvent),
}

pub(crate) struct DBEvent(pub String, pub i64);

impl DBEvent {
    pub(crate) fn persist(&self, tx: &Transaction<'_>) -> Result<usize> {
        let insert_cmd = if self.is_inbound() {
            INBOUND_INSERT
        } else {
            OUTBOUND_INSERT
        };
        let DBEvent(_, row_id) = self;
        tx.execute(insert_cmd, &[&row_id as &dyn ToSql])
    }
    //CREATE TABLE inbox_8116041356566675367 (msg_id TEXT PRIMARY KEY, msg BLOB);
    /***pub(crate) fn as_select_text(&mut self) -> &str {
        if self.2 == String::new() {
            let mut select = String::from("SELECT msg FROM ");
            select += &self.0;
            select += " WHERE row_id ='";
            select += &self.1.to_string();
            select += "'";
            self.2 = select;
        }
        &self.2
    }***/
    pub(crate) fn is_inbound(&self) -> bool {
        &self.0 == INBOX
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
    pub(crate) fn new() -> Self {
        Self {
            events: Vec::new(),
            oldest_receipt_instant: None,
        }
    }

    pub(crate) fn overflown(&self) -> bool {
        self.events.len() >= BUCKET_MAX_SIZE
    }

    pub fn oldest_matured(&self) -> bool {
        match self.oldest_receipt_instant {
            None => false,
            Some(instant) => instant.elapsed() >= Duration::new(EVENT_MAX_AGE, 0),
        }
    }

    pub(crate) fn should_persist_events(&self) -> bool {
        self.overflown() || self.oldest_matured()
    }
    pub(crate) fn add_event(&mut self, event: DBEvent) {
        self.events.push(event);
        if self.events.len() == 1 {
            self.oldest_receipt_instant = Some(Instant::now());
        }
        if self.should_persist_events() {
            let events = std::mem::take(&mut self.events);
            Self::deliver_messages(events);
        }
    }

    pub(crate) fn deliver_messages(events: Vec<DBEvent>) {
        let mut storage = Storage::new();
        storage.setup();
        let (ins, outs) = storage
            .persist_dbevents(events.into_iter())
            .expect("Events persisted");
        println!("Ins = {:?} and outs = {:?}", ins, outs);
    }
}

impl Drop for EventBucket {
    fn drop(&mut self) {
        if self.events.len() > 0 {
            Self::deliver_messages(std::mem::take(&mut self.events));
        }
    }
}
