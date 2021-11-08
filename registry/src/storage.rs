use common::{from_byte_array, option_of_bytes, Msg};
use constants::*;
use fallible_streaming_iterator::FallibleStreamingIterator;
use rusqlite::{
    named_params, params, types::Value, Connection, Error::InvalidQuery, Result, ToSql,
};
use std::collections::{HashMap, VecDeque};
use std::io::{Error, ErrorKind};
use std::str::FromStr;

pub(crate) struct WrappedConnection {
    inner: Connection,
}
impl WrappedConnection {
    pub(crate) fn new() -> Self {
        let result = Connection::open(DATABASE);
        if result.is_ok() {
            let inner = result.unwrap();
            //TODO check this value
            inner.set_prepared_statement_cache_capacity(100);
            Self { inner }
        } else {
            panic!("Failed to obtain db connection");
        }
    }
}
unsafe impl Send for WrappedConnection {}
unsafe impl Sync for WrappedConnection {}

unsafe impl Send for StorageContext {}
unsafe impl Sync for StorageContext {}

pub(crate) struct StorageContext {
    conn: WrappedConnection,
    inbox_insert_stmts: HashMap<String, String>,
    inbox_select_stmts: HashMap<String, String>,
    outbox_insert_stmts: HashMap<String, String>,
    outbox_select_stmts: HashMap<String, String>,
    actor_create_stmts: HashMap<String, String>,
}
impl std::fmt::Debug for StorageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageContextint")
            .field("inbox_insert_stmts", &self.inbox_insert_stmts)
            .field("inbox_select_stmts", &self.inbox_select_stmts)
            .field("outbox_insert_stmts", &self.outbox_insert_stmts)
            .field("actor_create_stmts", &self.actor_create_stmts)
            .finish()
    }
}
impl StorageContext {
    pub(crate) fn new() -> Self {
        Self {
            conn: WrappedConnection::new(),
            inbox_insert_stmts: HashMap::new(),
            outbox_insert_stmts: HashMap::new(),
            inbox_select_stmts: HashMap::new(),
            outbox_select_stmts: HashMap::new(),
            actor_create_stmts: HashMap::new(),
        }
    }
    pub(crate) fn crate_actors_table(&mut self) -> Result<()> {
        self.conn.inner.execute_batch(BEGIN_TRANSACTION)?;
        self.conn.inner.execute(ACTORS, [])?;
        self.conn.inner.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }
    pub(crate) fn setup(&mut self) -> Result<()> {
        self.crate_actors_table()?;
        let existing_actors = self.select_existing_actors()?;
        self.setup_inboxes(&existing_actors)?;
        self.setup_outboxes(&existing_actors)?;
        println!(
            "Setting up actors - existing count {}",
            existing_actors.len()
        );
        Ok(())
    }
    pub(crate) fn select_existing_actors(&mut self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .inner
            .prepare_cached("SELECT actor_id FROM actors")
            .ok();
        //TODO check capacity
        let mut actors = Vec::with_capacity(usize::from_str(FETCH_LIMIT).unwrap());
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
    pub(crate) fn setup_inboxes(&mut self, actor_ids: &Vec<String>) -> Result<()> {
        self.conn.inner.execute_batch(BEGIN_TRANSACTION)?;
        for actor_id in actor_ids {
            self.inbox_of(actor_id)?;
        }
        self.conn.inner.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }

    pub(crate) fn setup_outboxes(&mut self, actor_ids: &Vec<String>) -> Result<()> {
        self.conn.inner.execute_batch(BEGIN_TRANSACTION)?;
        for actor_id in actor_ids {
            self.outbox_of(actor_id)?;
        }
        self.conn.inner.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }

    pub(crate) fn inbox_of(&mut self, actor_id: &String) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS inbox_{} (msg_id TEXT PRIMARY KEY, msg BLOB)",
            actor_id
        );
        self.conn.inner.execute(&stmt, [])?;
        Ok(())
    }
    pub(crate) fn outbox_of(&mut self, actor_id: &String) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS outbox_{} (msg_id TEXT PRIMARY KEY, msg BLOB)",
            actor_id
        );
        self.conn.inner.execute(&stmt, [])?;
        Ok(())
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
                println!("Table exists");
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
    pub(crate) fn select_from_inbox(
        &mut self,
        actor_id: &String,
        msg_ids: Vec<&str>,
    ) -> Result<VecDeque<Msg>> {
        let mut count = 0;
        let size = msg_ids.len();
        let msg_ids_in = msg_ids
            .iter()
            .map(|id| {
                count += 1;
                let mut s = String::from("'");
                s.push_str(id);
                s.push('\'');
                if count < size {
                    s.push(',');
                }
                s
            })
            .collect::<String>();
        let stmt = format!(
            "SELECT msg FROM inbox_{} WHERE msg_id in ({})",
            actor_id, msg_ids_in
        );
        let mut stmt = self.conn.inner.prepare(&stmt)?;
        let mut rows = stmt.query([])?;
        let mut messages = VecDeque::new();
        while let Some(row) = rows.next()? {
            let value: Value = row.get(0)?;
            messages.push_front(value_to_msg(value));
        }
        Ok(messages)
    }

    pub(crate) fn into_outbox(&mut self, actor_id: &String, msg: Msg) -> Result<()> {
        let stmt = self
            .outbox_insert_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                format!(
                    "INSERT INTO outbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                    actor_id
                )
            });
        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        let msg_id = msg.id_as_string();
        let bytes = option_of_bytes(&msg);
        match stmt {
            Some(ref mut s) => s.execute(
                named_params! { ":msg_id": &msg_id as &dyn ToSql, ":msg": &bytes as &dyn ToSql },
            )?,
            None => panic!(),
        };
        Ok(())
    }

    pub(crate) fn persist_actor(&mut self, identity: &String, build_def: &String) -> Result<()> {
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

    pub(crate) fn into_inbox(&mut self, actor_id: &String, msg: Msg) -> Result<()> {
        let stmt = self
            .inbox_insert_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                format!(
                    "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                    actor_id
                )
            });
        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        let msg_id = msg.id_as_string();
        let bytes = option_of_bytes(&msg);
        match stmt {
            Some(ref mut s) => s.execute(
                named_params! { ":msg_id": &msg_id as &dyn ToSql, ":msg": &bytes as &dyn ToSql },
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
        let mut messages = VecDeque::with_capacity(usize::from_str(FETCH_LIMIT).unwrap());
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

    pub(crate) fn read_inbox_full(&mut self, actor_id: &String) -> Result<VecDeque<Msg>> {
        let stmt = self
            .inbox_select_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| format!("SELECT msg FROM inbox_{} ORDER BY rowid ASC", actor_id));
        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        let mut messages = VecDeque::with_capacity(usize::from_str(FETCH_LIMIT).unwrap());
        match stmt {
            Some(ref mut s) => {
                let rows = s.query_map([], |row| row.get(0))?;
                for row in rows {
                    let value: Value = row?;
                    messages.push_front(value_to_msg(value));
                }
            }
            None => panic!("Error draining inbox - CachedStatement not found"),
        }
        Ok(messages)
    }

    pub(crate) fn into_inbox_batch(
        &mut self,
        actor_id: &String,
        msg_itr: impl Iterator<Item = Msg>,
    ) -> Result<()> {
        self.conn.inner.execute_batch(BEGIN_TRANSACTION)?;
        let stmt = self
            .inbox_insert_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                format!(
                    "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                    actor_id
                )
            });
        let mut stmt = self.conn.inner.prepare_cached(stmt).ok();
        match stmt {
            Some(ref mut s) => {
                for msg in msg_itr {
                    let bytes = option_of_bytes(&msg);
                    let _status = s.execute(named_params! { ":msg_id": &msg.id_as_string() as &dyn ToSql, ":msg": &bytes as &dyn ToSql })?;
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
            _ => Msg::Blank,
        };
    }
    Msg::Blank
}

pub(crate) fn create_actor_inbox(actor_id: &String) -> Result<()> {
    let mut ctx = StorageContext::new();
    ctx.setup();
    ctx.inbox_of(actor_id)
}

pub(crate) fn create_actor_outbox(actor_id: &String) -> Result<()> {
    let mut ctx = StorageContext::new();
    ctx.setup();
    ctx.outbox_of(actor_id)
}
pub(crate) fn into_inbox(actor_id: &String, msg: Msg) -> Result<()> {
    let mut ctx = StorageContext::new();
    let _res = ctx.setup();
    ctx.into_inbox(actor_id, msg)
}

pub(crate) fn into_outbox(actor_id: &String, msg: Msg) -> Result<()> {
    let mut ctx = StorageContext::new();
    ctx.setup();
    ctx.into_outbox(actor_id, msg)
}

pub(crate) fn remove_db() -> std::io::Result<()> {
    std::fs::remove_file(DATABASE)
}
#[cfg(test)]
mod tests {
    use super::*;
    use common::{from_file_sync, Msg};
    use rand::{thread_rng, Rng};
    use std::fs::File;
    use std::io::BufWriter;

    fn composite_test() {}
    #[test]
    fn select_from_inbox_test_1() -> Result<()> {
        let actor_id = "1000".to_string();
        let msg_ids = vec![
            "12973981928118750491",
            "14312566721778882611",
            "4058720503399076582",
            "3311787687830812909",
        ];
        let mut ctx = StorageContext::new();
        ctx.setup();
        let messages = ctx.select_from_inbox(&actor_id, msg_ids)?;
        let messages: Vec<_> = messages.iter().map(|msg| msg.id_as_string()).collect();
        println!("The messages: {:?}", messages);
        Ok(())
    }
    #[test]
    fn delete_from_inbox_test1() {
        let actor_id = "1000".to_string();
        let msg_ids = vec!["16563997168647304630", "18086766434657795389"];
        let mut ctx = StorageContext::new();
        ctx.setup();
        assert_eq!(ctx.delete_from_inbox(&actor_id, msg_ids).ok(), Some(()));
    }

    #[test]
    fn purge_inbox_of_test_1() {
        let actor_id = "1000".to_string();
        let mut ctx = StorageContext::new();
        ctx.setup();
        assert_eq!(ctx.purge_inbox_of(&actor_id), Ok(()));
    }
    #[test]
    fn read_inbox_write_out_msg_test_1() -> std::io::Result<()> {
        let actor_id = "1000".to_string();
        let mut read_count = 0;
        let mut ctx = StorageContext::new();
        ctx.setup();
        let messages = ctx.read_inbox_full(&actor_id).unwrap();

        for msg in &messages {
            read_count += 1;
            if read_count == messages.len() {
                println!("The msg last msg: {:?}", msg);
                let file = File::create(msg.id_as_string()).expect("Failed to create file!");
                let mut writer = BufWriter::new(file);
                msg.write_sync(&mut writer)
                    .expect("Write failed - post reading inbox!");
                std::fs::rename(msg.id_as_string(), "last_message.txt")?;
            }
        }
        println!("The msg read count: {:?}", read_count);
        Ok(())
    }

    #[test]
    fn message_from_file_test_1() -> std::io::Result<()> {
        let msg: Msg = from_file_sync("last_message.txt")?;
        println!("Msg txt: {:?}", msg.content_as_text());
        Ok(())
    }

    #[test]
    fn read_inbox_test1() {
        let actor_id = "1000".to_string();
        let mut read_count = 0;
        let mut ctx = StorageContext::new();
        ctx.setup();
        let messages = ctx.read_inbox(&actor_id).unwrap();
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
        let actor_id = "1000".to_string();
        let mut read_count = 0;
        let mut ctx = StorageContext::new();
        ctx.setup();
        let messages = ctx.read_inbox_full(&actor_id).unwrap();
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
    fn create_actor_inbox_test1() {
        let inbox_create_result = create_actor_inbox(&"1000".to_string());
        assert_eq!(inbox_create_result, Ok(()));
    }
    #[test]
    fn create_actor_outbox_test1() {
        let outbox_create_result = create_actor_outbox(&"1000".to_string());
        assert_eq!(outbox_create_result, Ok(()));
    }

    fn into_inbox_batch_func(num: u32, actor_id: &String) -> Result<()> {
        let mut ctx = StorageContext::new();
        ctx.setup();
        ctx.inbox_of(actor_id);
        let mut messages = Vec::<Msg>::with_capacity(num.try_into().unwrap());
        let mut rng = thread_rng();
        for _ in 0..num {
            let random_num: u64 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num);
            let msg = Msg::new_with_text(&msg_content, "from", "to");
            messages.push(msg);
        }
        let status = ctx.into_inbox_batch(actor_id, messages.into_iter());
        println!("Batch insert final status: {:?}", status);

        Ok(())
    }

    fn into_inbox_no_batch_func(num: u32, actor_id: &String) -> Result<()> {
        let mut ctx = StorageContext::new();
        ctx.setup();
        ctx.inbox_of(actor_id);

        let mut rng = thread_rng();
        for _ in 0..num {
            let random_num: u64 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num);
            let msg = Msg::new_with_text(&msg_content, "from", "to");
            let status = ctx.into_inbox(actor_id, msg);
            println!("Many inserts no batching each status: {:?}", status);
        }
        Ok(())
    }

    #[test]
    fn into_inbox_no_batch_test_1() {
        let num = 1;
        let actor_id = "1000".to_string();
        //InvalidParameterCount
        //Err(SqliteFailure(
        //Err(ToSqlConversionFailure(TryFromIntError
        //Err(SqliteFailure(Error { code: ReadOnly, extended_code: 1032 }
        //Err(SqliteFailure(Error { code: TypeMismatch
        //Err(SqliteFailure(Error { code: ConstraintViolation, extended_code: 1555 },
        // Some("UNIQUE constraint failed: actors.actor_id")
        let status = into_inbox_no_batch_func(num, &actor_id);
        println!("Insert status each ? {:?}", status);
    }

    #[test]
    fn into_inbox_batch_test_1() {
        let num = 5;
        let actor_id = "1000".to_string();
        let status = into_inbox_batch_func(num, &actor_id);
        println!("Insert status all? {:?}", status);
    }

    #[test]
    fn persist_actor_1001_test_1() -> Result<()> {
        let mut ctx = StorageContext::new();
        ctx.setup();
        let identity = "1001".to_string();
        let insert = ctx.persist_actor(&identity, &r#"{"new_actor_builder":null}"#.to_string());
        println!("insert = {:?}", insert);
        assert!(insert.is_ok());
        Ok(())
    }
    #[test]
    fn remove_actor_permanent_1001_test_1() -> Result<()> {
        let mut ctx = StorageContext::new();
        ctx.setup();
        let actor_id = "1001".to_string();
        let remove = ctx.remove_actor_permanent(&actor_id);
        println!("remove = {:?}", remove);
        assert!(remove.is_ok());
        Ok(())
    }

    #[test]
    fn actor_is_present_1001_test_1() -> Result<()> {
        let mut ctx = StorageContext::new();
        ctx.setup();
        let actor_id = "1001".to_string();
        let present = ctx.actor_is_present(&actor_id);
        println!("present = {:?}", present);
        assert!(present.is_ok());
        Ok(())
    }

    #[test]
    fn retrieve_build_def_1001_test_1() -> Result<()> {
        let mut ctx = StorageContext::new();
        ctx.setup();
        let actor_id = "1001".to_string();
        let build_def = ctx.retrieve_build_def(&actor_id);
        println!("build_def = {:?}", build_def);
        assert!(build_def.is_ok());
        Ok(())
    }
}

pub(crate) mod constants {
    pub(crate) const DATABASE: &str = "arrows.db";
    pub(super) const FETCH_LIMIT: &str = "100";
    pub(super) const BEGIN_TRANSACTION: &str = "BEGIN TRANSACTION;";
    pub(super) const COMMIT_TRANSACTION: &str = "COMMIT TRANSACTION;";
    pub(super) const SELECT_ACTORS: &str = "SELECT actor_id FROM actors";
    //TODO check where its being used?
    pub(self) const DOES_TABLE_EXIST: &str =
        "SELECT count(1) FROM sqlite_master WHERE type='table' AND name=?";
    pub(super) const ACTORS: &str =
        "CREATE TABLE IF NOT EXISTS actors (actor_id TEXT PRIMARY KEY, build_def TEXT)";
    pub(super) const BUILD_DEF_INSERT: &str =
        "INSERT INTO actors (actor_id, build_def) VALUES (:actor_id, :build_def)";
    pub(super) const DELETE_ACTOR: &str = "DELETE FROM actors WHERE actor_id = ?";
    pub(super) const ACTOR_ROWID: &str = "SELECT rowid FROM actors WHERE actor_id = ?";
    pub(super) const BUILD_DEF: &str = "SELECT build_def FROM actors WHERE actor_id = ?";
}
