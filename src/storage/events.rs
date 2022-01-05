use crate::constants::{INBOUND_INSERT, INBOX, OUTBOUND_INSERT, OUTBOX};
use rusqlite::{hooks::Action, Result, ToSql, Transaction};
use serde::{ser::SerializeTupleStruct, Deserialize, Serialize, Serializer};

#[derive(Debug)]
pub(crate) enum Events {
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

    pub(crate) fn is_inbound(&self) -> bool {
        self.0 == INBOX
    }
}

impl From<(i64, bool)> for DBEvent {
    fn from(directed_event: (i64, bool)) -> Self {
        match directed_event {
            (rowid, true) => DBEvent(INBOX.to_string(), rowid),
            (rowid, false) => DBEvent(OUTBOX.to_string(), rowid),
        }
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
