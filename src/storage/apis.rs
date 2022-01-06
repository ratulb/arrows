use crate::common::utils::{from_byte_array, option_of_bytes};
use crate::constants::*;
use crate::dbconnection::DBConnection;
use crate::events::DBEvent;
use crate::pubsub::Publisher;
use crate::{Mail, Mail::*, Msg};
use fallible_streaming_iterator::FallibleStreamingIterator;
use rusqlite::{named_params, params, types::Value, Error::InvalidQuery, Result, ToSql};
use std::collections::{HashMap, VecDeque};
use std::io::{Error, ErrorKind};

use std::thread::JoinHandle;

unsafe impl Send for Store {}
unsafe impl Sync for Store {}

impl Drop for Store {
    fn drop(&mut self) {
        self.publisher.loopbreak();
        self.publisher
            .subscriber
            .take()
            .map(|mut subscriber| subscriber.join_handle.take().map(JoinHandle::join));
    }
}

pub(crate) struct Store {
    buffer: Vec<Msg>,
    conn: DBConnection,
    message_insert_stmt: Option<String>,
    inbox_select_stmts: HashMap<String, String>,
    actor_create_stmts: HashMap<String, String>,
    publisher: Publisher,
    subscriber_handle: Option<JoinHandle<()>>,
}
impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storeint")
            .field("message_insert_stmt", &self.message_insert_stmt)
            .field("inbox_select_stmts", &self.inbox_select_stmts)
            .field("actor_create_stmts", &self.actor_create_stmts)
            .finish()
    }
}
impl Store {
    pub(crate) fn new() -> Self {
        Self {
            buffer: Vec::new(),
            conn: DBConnection::new(),
            message_insert_stmt: None,
            inbox_select_stmts: HashMap::new(),
            actor_create_stmts: HashMap::new(),
            publisher: Publisher::new(),
            subscriber_handle: None,
        }
    }
    fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.len() >= BUFFER_MAX_SIZE {
            //TODO make it configurable
            self.persist(Blank)
        } else {
            Ok(())
        }
    }

    pub(crate) fn persist(&mut self, mail: Mail) -> Result<()> {
        match mail {
            Blank if self.buffer.is_empty() => Ok(()),
            Blank => self.persist_buffer(),
            Trade(msg) => {
                self.buffer.push(msg);
                self.flush_buffer()
            }
            Bulk(msgs) => {
                self.buffer.extend(msgs);
                self.flush_buffer()
            }
            _ => Ok(()),
        }
    }

    fn persist_buffer(&mut self) -> Result<()> {
        self.conn.inner.execute_batch(BEGIN_TRANSACTION)?;
        let stmt = Self::message_insert_stmt(&mut self.message_insert_stmt);
        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        match stmt {
            Some(ref mut s) => {
                for msg in self.buffer.drain(..) {
                    let actor_id = msg.get_to_id().to_string();
                    let bytes = msg.as_bytes();
                    let _status = s.execute(named_params! {":actor_id": &actor_id as &dyn ToSql, ":msg_id": &msg.id_as_string() as &dyn ToSql,":actor_id": &actor_id as &dyn ToSql, ":msg": &bytes as &dyn ToSql })?;
                }
            }
            None => panic!(),
        }
        self.conn.inner.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }

    pub(crate) fn setup(&mut self) -> Result<()> {
        self.conn.inner.execute(MESSAGES, [])?;
        self.conn.inner.execute(ACTORS, [])?;
        self.conn.inner.execute(EVENTS, [])?;
        self.publisher.start(&mut self.conn);
        println!("Set up arrows schema");
        Ok(())
    }

    pub(crate) fn all_actors(&mut self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .inner
            .prepare_cached("SELECT actor_id FROM actors")
            .ok();
        //TODO check capacity
        let mut actors = Vec::with_capacity(FETCH_LIMIT);
        match stmt {
            Some(ref mut s) => {
                let rows = s.query_map([], |row| row.get(0))?;
                for row in rows {
                    let value: String = row?;
                    actors.push(value);
                }
            }
            None => panic!("Error draining inbox - CachedStatement not found"),
        }
        Ok(actors)
    }

    pub(crate) fn purge_inbox_of(&mut self, actor_id: &String) -> Result<()> {
        let stmt = format!(
            "SELECT count(1) FROM sqlite_master WHERE type='table' AND name='inbox_{}'",
            actor_id
        );
        let mut stmt = self.conn.inner.prepare(&stmt)?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let value: usize = row.get(0)?;
            if value == 1 {
                let stmt = format!("DELETE FROM inbox_{}", actor_id);
                match self.conn.inner.execute(&stmt, []) {
                    Ok(deleted) => println!("Rows deleted: {:?}", deleted),
                    Err(err) => println!("Error occured: {:?}", err),
                }
            } else {
                println!("Table does not exist");
            }
        }
        Ok(())
    }

    pub(crate) fn delete_from_inbox(
        &mut self,
        actor_id: &String,
        msg_ids: Vec<&str>,
    ) -> std::io::Result<()> {
        let stmt = format!("DELETE FROM inbox_{} WHERE msg_id = ?", actor_id);
        self.conn
            .inner
            .execute_batch(BEGIN_TRANSACTION)
            .map_err(sql_to_io);
        let mut stmt = self.conn.inner.prepare_cached(&stmt).map_err(sql_to_io)?;
        for msg_id in msg_ids {
            stmt.execute(params![msg_id]).map_err(sql_to_io);
        }
        self.conn
            .inner
            .execute_batch(COMMIT_TRANSACTION)
            .map_err(sql_to_io);
        Ok(())
    }

    pub(crate) fn persist_builder(&mut self, identity: &String, build_def: &String) -> Result<()> {
        let mut stmt = self.conn.inner.prepare_cached(BUILD_DEF_INSERT).ok();
        match stmt {
            Some(ref mut s) => s.execute(
                named_params! { ":actor_id": identity as &dyn ToSql, ":build_def": build_def as &dyn ToSql },
            )?,
            None => panic!(),
        };
        Ok(())
    }
    pub(crate) fn remove_actor_permanent(&mut self, identity: &String) -> Result<()> {
        let mut stmt = self.conn.inner.prepare_cached(DELETE_ACTOR)?;
        stmt.execute(params![identity]).and_then(
            |c| {
                if c == 1 {
                    Ok(())
                } else {
                    Err(InvalidQuery)
                }
            },
        )
    }
    pub(crate) fn actor_is_present(&mut self, actor_id: &String) -> Result<()> {
        let mut stmt = self.conn.inner.prepare_cached(ACTOR_ROWID)?;
        let status = stmt
            .query(rusqlite::params![actor_id])?
            .count()
            .and_then(|c| if c == 1 { Ok(()) } else { Err(InvalidQuery) });
        status
    }
    pub(crate) fn retrieve_build_def(&mut self, actor_id: &String) -> Result<Option<String>> {
        let mut stmt = self.conn.inner.prepare_cached(BUILD_DEF)?;
        let mut rows = stmt.query(rusqlite::params![actor_id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(row.get(0)?));
        }
        Ok(None)
    }

    #[inline(always)]
    fn message_insert_stmt<'a>(stmt: &'a mut Option<String>) -> &'a str {
        match stmt {
            Some(ref s) => s,
            None => {
                *stmt = Some(INSERT_INTO_MESSAGES.to_string());
                INSERT_INTO_MESSAGES
            }
        }
    }

    pub(crate) fn from_messages(&mut self, rowids: Vec<i64>) -> Result<Vec<(Msg, bool, i64)>> {
        let rowids = rowids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let stmt = format!(
            "SELECT msg, inbound, msg_seq FROM messages WHERE rowid IN ({})",
            rowids
        );
        let mut stmt = self.conn.inner.prepare(&stmt)?;
        let mut rows = stmt.query([])?;
        let mut msgs = Vec::new();
        while let Some(row) = rows.next()? {
            let value: Value = row.get(0)?;
            let inbound: i64 = row.get(1)?;
            let msg_seq: i64 = row.get(2)?;
            msgs.push((value_to_msg(value), inbound == 1, msg_seq));
        }
        Ok(msgs)
    }

    pub(crate) fn into_inbox(&mut self, msg: Msg) -> Result<()> {
        let stmt = Self::message_insert_stmt(&mut self.message_insert_stmt);
        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        let msg_id = msg.id_as_string();
        let actor_id = msg.get_to_id().to_string();
        let bytes = option_of_bytes(&msg);
        match stmt {
            Some(ref mut s) => s.execute(
                named_params! {":actor_id": &actor_id as &dyn ToSql, ":msg_id": &msg_id as &dyn ToSql, ":actor_id": &actor_id as &dyn ToSql, ":msg": &bytes as &dyn ToSql },
            )?,
            None => panic!(),
        };
        Ok(())
    }
    pub(crate) fn read_inbox(&mut self, actor_id: &String) -> Result<VecDeque<Msg>> {
        let stmt = self
            .inbox_select_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                format!(
                    "SELECT msg FROM inbox_{} ORDER BY rowid ASC LIMIT {}",
                    actor_id, FETCH_LIMIT
                )
            });

        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        let mut messages = VecDeque::with_capacity(FETCH_LIMIT);
        match stmt {
            Some(ref mut s) => {
                //let rows = s.query_and_then([], |row| row.get::<_, Msg>(0))?;
                let rows = s.query_map([], |row| row.get(0))?;
                for row in rows {
                    let value: Value = row?;
                    messages.push_front(value_to_msg(value));
                }
            }
            None => {
                panic!("Error draining inbox - CachedStatement not found")
            }
        }
        Ok(messages)
    }

    pub(crate) fn rowids_of(&mut self, actor_id: &str) -> Result<Vec<i64>> {
        let stmt = format!(
            "SELECT rowid FROM messages WHERE actor_id = '{}' ORDER BY rowid ASC",
            actor_id
        );
        let mut stmt = self.conn.inner.prepare_cached(&stmt)?;
        let mut rows = stmt.query([])?;
        let mut rowids = Vec::new();
        while let Some(row) = rows.next()? {
            rowids.push(row.get(0)?);
        }
        Ok(rowids)
    }
    pub(crate) fn read_inbox_from(&mut self, actor_id: &str, start_at: i64) -> Result<Vec<Msg>> {
        let stmt =format!("SELECT msg FROM inbox WHERE actor_id = '{}' and rowid >= {} ORDER BY rowid ASC LIMIT {}", actor_id, start_at, FETCH_LIMIT);
        let mut stmt = self.conn.inner.prepare_cached(&stmt).ok();
        let mut messages = Vec::with_capacity(FETCH_LIMIT);
        match stmt {
            Some(ref mut s) => {
                let rows = s.query_map([], |row| row.get(0))?;
                for row in rows {
                    let value: Value = row?;
                    messages.push(value_to_msg(value));
                }
            }
            None => panic!("Error reading inbox!"),
        }
        Ok(messages)
    }

    pub(crate) fn read_past_events(&mut self) -> Result<Vec<i64>> {
        let mut stmt = self.conn.inner.prepare_cached(EVENTS_SELECT)?;
        let mut rows = stmt.query([])?;
        let mut events = Vec::new();
        while let Some(row) = rows.next()? {
            events.push(row.get(0)?);
        }
        Ok(events)
    }

    pub(crate) fn persist_events(
        &mut self,
        events: impl Iterator<Item = DBEvent>,
    ) -> Result<Vec<i64>> {
        let tx = self.conn.inner.transaction()?;
        let mut persisted_events = Vec::new();
        for event in events {
            event.persist(&tx)?;
            persisted_events.push(event.0);
        }
        tx.commit()?;
        Ok(persisted_events)
    }

    pub(crate) fn into_inbox_batch(&mut self, msgs: impl Iterator<Item = Msg>) -> Result<()> {
        self.conn.inner.execute_batch(BEGIN_TRANSACTION)?;
        let stmt = Self::message_insert_stmt(&mut self.message_insert_stmt);
        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        match stmt {
            Some(ref mut s) => {
                for msg in msgs {
                    let bytes = msg.as_bytes();
                    let actor_id = msg.get_to_id().to_string();
                    let _status = s.execute(named_params! { ":actor_id": &actor_id as &dyn ToSql, ":msg_id": &msg.id_as_string() as &dyn ToSql, ":actor_id": &actor_id as &dyn ToSql, ":msg": &bytes as &dyn ToSql })?;
                }
            }
            None => panic!(),
        };
        self.conn.inner.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }
}
pub(crate) fn sql_to_io(err: rusqlite::Error) -> std::io::Error {
    eprintln!("rusqlite::Error has occured: {:?}", err);
    Error::new(ErrorKind::Other, "rusqlite error")
}

pub(crate) fn value_to_msg(v: Value) -> Msg {
    if let Value::Blob(bytes) = v {
        return match from_byte_array::<'_, Msg>(&bytes) {
            Ok(msg) => msg,
            _ => Msg::default(),
        };
    }
    Msg::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::mail::Msg;
    use crate::constants::BUFFER_MAX_SIZE;
    use crate::events::DBEvent;
    use crate::Addr;
    use rand::{thread_rng, Rng};
    use std::iter::repeat;

    fn store_messages(actor_name: &str) -> (String, String) {
        //Add randomness to the message text
        let mut rng = thread_rng();
        let random: u64 = rng.gen();
        let message = format!("Actor message-{}", random);
        //Generate as many messages as required to flush buffer
        let messages = repeat(&message).take(BUFFER_MAX_SIZE);
        let messages: Vec<_> = messages
            .map(|msg| Msg::new_with_text(msg, "from", actor_name))
            .collect();

        let mut store = Store::new();
        let _ = store.setup();
        //Persist messages
        let rs = store.persist(Mail::Bulk(messages));
        assert!(rs.is_ok());
        let actor_id = Addr::new("actor").get_id().to_string();
        //Return random generated message and actor_id for actor_name
        (message.to_string(), actor_id)
    }
    #[test]
    fn select_from_inbox_test() -> Result<()> {
        let (message, actor_id) = store_messages("actor");

        let mut store = Store::new();
        let _ = store.setup();

        let rowids = store.rowids_of(&actor_id).unwrap();
        let msgs = store.from_messages(rowids).unwrap();
        let count = msgs
            .iter()
            .filter(|msg| msg.0.content_as_text() == Some(&message))
            .count();
        assert!(count == BUFFER_MAX_SIZE);
        Ok(())
    }

    #[test]
    fn read_inbox_from_test() {
        let (message, actor_id) = store_messages("actor");
        let mut store = Store::new();
        let _ = store.setup();
        let mut rowids = store.rowids_of(&actor_id).unwrap();
        let last = rowids.pop().unwrap();
        let msgs = store.read_inbox_from(&actor_id, last).unwrap();

        assert!(msgs[0].content_as_text() == Some(&message));
    }

    #[test]
    fn purge_inbox_of_test_1() {
        let actor_id = "1000".to_string();
        let mut store = Store::new();
        let _ = store.setup();
        assert_eq!(store.purge_inbox_of(&actor_id), Ok(()));
    }
    #[test]
    fn read_inbox_write_out_msg_test_1() {
        let _actor_id = "1000".to_string();
        let _read_count = 0;
        let mut store = Store::new();
        let _ = store.setup();
        /***let messages = store.read_inbox_full(&actor_id).unwrap();

        for _msg in &messages {
            read_count += 1;
        }
        println!("The msg read count: {:?}", read_count);***/
    }

    #[test]
    fn read_inbox_test1() {
        let actor_id = "1000".to_string();
        let mut read_count = 0;
        let mut store = Store::new();
        let _ = store.setup();
        let messages = store.read_inbox(&actor_id).unwrap();
        for msg in messages {
            println!("The msg: {:?}", msg);
            println!();
            println!();
            println!();
            read_count += 1;
        }
        println!("The msg read count: {:?}", read_count);
    }
    #[test]
    fn read_inbox_full_test1() {
        let _actor_id = "1000".to_string();
        let _read_count = 0;
        let mut store = Store::new();
        let _ = store.setup();
        /***let messages = store.read_inbox_full(&actor_id).unwrap();
        for msg in messages {
            println!("The msg: {:?}", msg);
            println!();
            println!();
            println!();
            read_count += 1;
        }
        println!("The msg read count: {:?}", read_count);***/
    }

    fn into_inbox_batch_func(num: u32) -> Result<()> {
        let mut store = Store::new();
        let _ = store.setup();
        let mut messages = Vec::<Msg>::with_capacity(num.try_into().unwrap());
        let mut rng = thread_rng();
        for _ in 0..num {
            let random_num: u64 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num);
            let msg = Msg::new_with_text(&msg_content, "from", "to");
            messages.push(msg);
        }
        let status = store.into_inbox_batch(messages.into_iter());
        assert!(status.is_ok());
        Ok(())
    }

    fn into_inbox_no_batch_func(num: u32) -> Result<()> {
        let mut store = Store::new();
        let _ = store.setup();

        let mut rng = thread_rng();
        for _ in 0..num {
            let random_num: u64 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num);
            let msg = Msg::new_with_text(&msg_content, "from", "to");
            let _status = store.into_inbox(msg);
        }
        Ok(())
    }

    #[test]
    fn into_inbox_no_batch_test_1() {
        let num = 100;
        //InvalidParameterCount
        //Err(SqliteFailure(
        //Err(ToSqlConversionFailure(TryFromIntError
        //Err(SqliteFailure(Error { code: ReadOnly, extended_code: 1032 }
        //Err(SqliteFailure(Error { code: TypeMismatch
        //Err(SqliteFailure(Error { code: ConstraintViolation, extended_code: 1555 },
        // Some("UNIQUE constraint failed: actors.actor_id")
        let _status = into_inbox_no_batch_func(num);
    }

    #[test]
    fn into_inbox_batch_test_1() {
        let num = 100;
        let status = into_inbox_batch_func(num);
        assert!(status.is_ok());
    }

    #[test]
    fn persist_builder_1001_test_1() -> Result<()> {
        let mut store = Store::new();
        let _ = store.setup();
        let identity = "1001".to_string();
        let insert = store.persist_builder(&identity, &r#"{"new_actor_builder":null}"#.to_string());
        println!("insert = {:?}", insert);
        assert!(insert.is_ok());
        Ok(())
    }
    #[test]
    fn remove_actor_permanent_1001_test_1() -> Result<()> {
        let mut store = Store::new();
        let _ = store.setup();
        let actor_id = "1001".to_string();
        let remove = store.remove_actor_permanent(&actor_id);
        println!("remove = {:?}", remove);
        assert!(remove.is_ok());
        Ok(())
    }

    #[test]
    fn actor_is_present_1001_test_1() -> Result<()> {
        let mut store = Store::new();
        let _ = store.setup();
        let actor_id = "1001".to_string();
        let present = store.actor_is_present(&actor_id);
        println!("present = {:?}", present);
        assert!(present.is_ok());
        Ok(())
    }

    #[test]
    fn retrieve_build_def_1001_test_1() -> Result<()> {
        let mut store = Store::new();
        let _ = store.setup();
        let actor_id = "1001".to_string();
        let build_def = store.retrieve_build_def(&actor_id);
        println!("build_def = {:?}", build_def);
        assert!(build_def.is_ok());
        Ok(())
    }

    #[test]
    fn serialize_db_event_test1() {
        let db_event = DBEvent(100);
        let json = serde_json::to_string(&db_event).unwrap();
        let expected = "[100]";
        assert_eq!(json, expected);
    }
}
