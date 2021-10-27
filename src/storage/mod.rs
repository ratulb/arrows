use arrow_commons::{from_byte_array, option_of_bytes, Message};
use constants::*;

use rusqlite::{named_params, CachedStatement, Connection, Result, Statement, ToSql};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

pub(crate) struct StorageContext<'a> {
    conn: &'a Connection,
    inbox_insert_stmts: HashMap<u64, Option<CachedStatement<'a>>>,
    inbox_select_stmts: HashMap<u64, Option<CachedStatement<'a>>>,
    outbox_insert_stmts: HashMap<u64, Option<CachedStatement<'a>>>,
    outbox_select_stmts: HashMap<u64, Option<CachedStatement<'a>>>,

    select_stmnts: HashMap<u64, Option<Statement<'a>>>,
    create_outbox_stmnts: HashMap<u64, Option<bool>>,
}

impl<'a> StorageContext<'a> {
    fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            inbox_insert_stmts: HashMap::new(),
            outbox_insert_stmts: HashMap::new(),
            inbox_select_stmts: HashMap::new(),
            outbox_select_stmts: HashMap::new(),

            select_stmnts: HashMap::new(),
            create_outbox_stmnts: HashMap::new(),
        }
    }
    pub(crate) fn setup(&mut self) -> Result<()> {
        self.conn.execute_batch(BEGIN_TRANSACTION)?;
        self.conn.execute(ACTORS, [])?;
        let mut stmt = self.conn.prepare(SELECT_ACTORS)?;
        let mut actors = stmt.query([])?;
        while let Some(actor) = actors.next()? {
            let actor_id: u64 = actor.get(0)?;
            self.inbox_of(actor_id)?;
            self.outbox_of(actor_id)?;
        }
        self.conn.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }
    pub(crate) fn inbox_of(&mut self, actor_id: u64) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS inbox_{} (msg_id TEXT PRIMARY KEY, msg BLOB)",
            &actor_id.to_string()[..]
        );
        self.conn.execute(&stmt, [])?;
        Ok(())
    }
    pub(crate) fn outbox_of(&mut self, actor_id: u64) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS outbox_{} (msg_id INTEGER PRIMARY KEY, msg BLOB)",
            &actor_id.to_string()[..]
        );
        self.conn.execute(&stmt, [])?;
        Ok(())
    }

    pub(crate) fn into_outbox(&mut self, actor_id: u64, msg: Message) -> Result<()> {
        let stmt = self.outbox_insert_stmts.entry(actor_id).or_insert_with(|| {
            self.conn
                .prepare_cached(&format!(
                    "INSERT INTO outbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                    &actor_id.to_string()[..]
                ))
                .ok()
        });

        let msg_id = msg.get_id().clone().to_string();
        let bytes = option_of_bytes(&msg);
        match stmt {
            Some(ref mut s) => s.execute(named_params! { ":msg_id": msg_id, ":msg": bytes })?,
            None => panic!(),
        };
        Ok(())
    }

    pub(crate) fn into_inbox(&mut self, actor_id: u64, msg: Message) -> Result<()> {
        let stmt = self.inbox_insert_stmts.entry(actor_id).or_insert_with(|| {
            self.conn
                .prepare_cached(&format!(
                    "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                    &actor_id.to_string()[..]
                ))
                .ok()
        });

        let bytes = option_of_bytes(&msg);
        match stmt {
            Some(ref mut s) => s.execute(named_params! { ":msg_id": &msg.id_as_string() as &dyn ToSql, ":msg": &bytes as &dyn ToSql })?,
            None => panic!(),
        };
        Ok(())
    }
    pub(crate) fn drain_inbox(&mut self, actor_id: u64) -> Result<VecDeque<Message>> {
        let stmt = self.inbox_select_stmts.entry(actor_id).or_insert_with(|| {
            self.conn
                .prepare_cached(&format!(
                    "SELECT msg FROM inbox_{} ORDER BY rowid ASC LIMIT {}",
                    &actor_id.to_string()[..],
                    FETCH_LIMIT
                ))
                .ok()
        });
        let mut messages = VecDeque::with_capacity(usize::from_str(FETCH_LIMIT).unwrap());
        match stmt {
            Some(ref mut s) => {
                let rows = s.query_and_then([], |row| row.get::<_, Message>(0))?;
                for row in rows {
                    messages.push_front(row?);
                }
            }
            None => panic!("Error draining inbox - CachedStatement not found"),
        }
        return Ok(messages);
    }

    pub(crate) fn drain_inbox_full(&mut self, actor_id: u64) -> Result<VecDeque<Message>> {
        let stmt = self.inbox_select_stmts.entry(actor_id).or_insert_with(|| {
            self.conn
                .prepare_cached(&format!(
                    "SELECT msg FROM inbox_{} ORDER BY rowid ASC",
                    &actor_id.to_string()[..]
                ))
                .ok()
        });
        let mut messages = VecDeque::with_capacity(usize::from_str(FETCH_LIMIT).unwrap());
        match stmt {
            Some(ref mut s) => {
                let rows = s.query_and_then([], |row| row.get::<_, Message>(0))?;
                for row in rows {
                    messages.push_front(row?);
                }
            }
            None => panic!("Error draining inbox - CachedStatement not found"),
        }
        return Ok(messages);
    }

    pub(crate) fn create_inbox(&mut self, actor_id: u64) -> Result<()> {
        let stmnt = format!(
            "CREATE TABLE IF NOT EXISTS inbox_{} (id INTEGER PRIMARY KEY, msg_id TEXT , msg BLOB)",
            &actor_id.to_string()[..]
        );
        self.conn.execute(&stmnt, [])?;
        Ok(())
    }

    pub(crate) fn create_outbox(&mut self, actor_id: u64) -> Result<()> {
        if self.create_outbox_stmnts.get(&actor_id).is_none() {
            let stmnt = format!(
                "CREATE TABLE IF NOT EXISTS outbox_{} (id INTEGER PRIMARY KEY,name TEXT NOT NULL, data BLOB)",
                &actor_id.to_string()[..]
            );
            self.conn.execute(&stmnt, [])?;
            self.create_outbox_stmnts.insert(actor_id, Some(true));
        }
        Ok(())
    }

    pub(crate) fn into_inbox_batch(
        &mut self,
        actor_id: u64,
        msgItr: impl Iterator<Item = Message>,
    ) -> Result<()> {
        self.conn.execute_batch(BEGIN_TRANSACTION)?;
        let stmt = self.inbox_insert_stmts.entry(actor_id).or_insert_with(|| {
            self.conn
                .prepare_cached(&format!(
                    "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                    &actor_id.to_string()[..]
                ))
                .ok()
        });

        match stmt {
            Some(ref mut s) => {
                for msg in msgItr {
                    let bytes = option_of_bytes(&msg);
                    let status = s.execute(named_params! { ":msg_id": &msg.id_as_string() as &dyn ToSql, ":msg": &bytes as &dyn ToSql })?;
                    println!("Batch statement execution status: {:?}", status);
                }
            }
            None => panic!(),
        };
        self.conn.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }
}

pub(crate) fn create_actor_inbox(actor_id: u64) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.inbox_of(actor_id)
}

pub(crate) fn create_actor_outbox(actor_id: u64) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.outbox_of(actor_id)
}

pub(crate) fn into_inbox(actor_id: u64, msg: Message) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.into_inbox(actor_id, msg)
}
pub(crate) fn into_outbox(actor_id: u64, msg: Message) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.into_outbox(actor_id, msg)
}

pub(crate) fn remove_db() -> std::io::Result<()> {
    std::fs::remove_file(DATABASE)
}
/***
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
type CustomFromSql = dyn FromSql;

impl CustomFromSql for Message {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(byte_arr) if byte_arr.len() > 0 => {
                match from_byte_array::<'_, Message>(byte_arr) {
                    Ok(good) => Ok(good),
                    _ => Ok(Message::Blank),
                }
            }
            _ => Ok(Message::Blank),
        }
    }
}***/
//.map_err(|err| FromSqlError::Other(Box::new(err)))
#[cfg(test)]
mod tests {
    use super::*;
    use arrow_commons::{from_byte_array, from_file_sync, option_of_bytes, type_of, Message};
    use rand::{thread_rng, Rng};
    use rusqlite::types::ValueRef;
    use rusqlite::{params, DropBehavior, Result};
    use std::fs::File;
    use std::io::BufWriter;
    use std::{thread, time};

    #[test]
    fn setup_test_1() {
        let _actor_id: u64 = 1000;
    }
    #[test]
    fn read_inbox_write_out_msg_test_1() -> std::io::Result<()> {
        let actor_id = 1000;
        let mut read_count = 0;
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        let messages = ctx.drain_inbox_full(actor_id).unwrap();

        for msg in &messages {
            read_count += 1;
            if read_count == messages.len() - 1 {
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
        let msg: Message = from_file_sync("last_message.txt")?;
        println!("Message txt: {:?}", msg.content_as_text());
        Ok(())
    }

    #[test]
    fn read_inbox_test1() {
        let actor_id = 1000;
        let mut read_count = 0;
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        let messages = ctx.drain_inbox(actor_id).unwrap();
        for msg in messages {
            println!("The msg: {:?}", msg);
            println!("");
            println!("");
            println!("");
            read_count += 1;
        }
        println!("The msg read count: {:?}", read_count);
    }

    #[test]
    fn create_actor_inbox_test1() {
        let inbox_create_result = create_actor_inbox(1000);
        assert_eq!(inbox_create_result, Ok(()));
    }
    #[test]
    fn create_actor_outbox_test1() {
        let outbox_create_result = create_actor_outbox(1000);
        assert_eq!(outbox_create_result, Ok(()));
    }
    //#[test]
    fn into_inbox_test1() {
        let into_inbox_result =
            into_inbox(1000, Message::new_with_text("The test msg", "from", "to"));
        assert_eq!(into_inbox_result, Ok(()));
    }

    pub(crate) fn create_table_and_insert_message() -> Result<()> {
        let conn = Connection::open(DATABASE)?;
        //let mut stmt = conn.prepare("DROP TABLE IF EXISTS inbox")?;
        //stmt.execute(params![])?;
        conn.execute("CREATE TABLE IF NOT EXISTS inbox (msg BLOB)", [])?;
        let msg = Message::new_with_text("The test msg", "from", "to");
        let _id = msg.get_id();
        let msg: Option<Vec<u8>> = option_of_bytes(&msg);
        //let msg: Option<Vec<u8>> = Some(vec![1,2,3,4]);
        conn.execute("INSERT INTO inbox (msg) VALUES (?1)", params![msg])?;

        // let mut stmt = conn.prepare("SELECT name FROM inbox")?;
        // type_of(& stmt.query_map([], |row| row.get(0))?);

        Ok(())
    }
    #[test]
    fn read_from_inbox_test_1() -> Result<()> {
        let mut conn = Connection::open(DATABASE)?;
        let mut tx = conn.transaction()?;
        tx.set_drop_behavior(DropBehavior::Commit);
        let mut stmnt = tx.prepare_cached("SELECT msg FROM inbox LIMIT 1000")?;
        //let mut stmnt = tx.prepare_cached("SELECT msg FROM inbox")?;
        //let mut stmnt = conn.prepare("SELECT * FROM inbox where name = ?")?;
        let mut rows = stmnt.query([])?;
        //let mut rows = stmnt.query(rusqlite::params!["msg"])?;
        while let Some(row) = rows.next()? {
            type_of(&row.get_ref_unwrap(0));
            if let ValueRef::Blob(b) = row.get_ref_unwrap(0) {
                type_of(&b);
                let r: std::io::Result<Message> = from_byte_array(b);
                println!("{:?}", r.unwrap());
            }
        }
        Ok(())
    }

    #[test]
    fn drop_create_insert_test1() {
        let num = 100;
        for _ in 0..num {
            assert_eq!(create_table_and_insert_message(), Ok(()));
        }
    }

    fn insert_message_many_batching(num: u32, actor_id: u64) -> Result<()> {
        let conn = Connection::open(DATABASE)?;
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        ctx.inbox_of(actor_id);
        let mut messages = Vec::<Message>::with_capacity(num.try_into().unwrap());
        let mut rng = thread_rng();
        for _ in 0..num {
            let random_num: u64 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num.to_string());
            let msg = Message::new_with_text(&msg_content, "from", "to");
            messages.push(msg);
        }
        let status = ctx.into_inbox_batch(actor_id, messages.into_iter());
        println!("Batch insert final status: {:?}", status);

        Ok(())
    }

    fn insert_message_many_no_batching(num: u32, actor_id: u64) -> Result<()> {
        let conn = Connection::open(DATABASE)?;
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        ctx.inbox_of(actor_id);

        let mut rng = thread_rng();
        for _ in 0..num {
            let random_num: u64 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num.to_string());
            let msg = Message::new_with_text(&msg_content, "from", "to");
            let status = ctx.into_inbox(actor_id, msg);
            println!("Many inserts no batching each status: {:?}", status);
        }
        Ok(())
    }

    #[test]
    fn insert_message_many_no_batch_test_1() {
        let num = 500;
        let actor_id = 1000;
        //InvalidParameterCount
        //Err(SqliteFailure(
        //Err(ToSqlConversionFailure(TryFromIntError
        //Err(SqliteFailure(Error { code: ReadOnly, extended_code: 1032 }
        //Err(SqliteFailure(Error { code: TypeMismatch
        let status = insert_message_many_no_batching(num, actor_id);
        println!("Insert status each ? {:?}", status);
    }

    #[test]
    fn insert_message_many_batch_test_1() {
        let num = 500;
        let actor_id = 1000;
        let status = insert_message_many_batching(num, actor_id);
        println!("Insert status all? {:?}", status);
    }
}

mod constants {
    pub(super) const DATABASE: &str = "arrows.db";
    pub(super) const FETCH_LIMIT: &str = "1000";
    pub(super) const BEGIN_TRANSACTION: &str = "BEGIN TRANSACTION;";
    pub(super) const COMMIT_TRANSACTION: &str = "COMMIT TRANSACTION;";
    pub(super) const SELECT_ACTORS: &str = "SELECT actor_id FROM actors";
    pub(super) const ACTORS: &str =
        "CREATE TABLE IF NOT EXISTS actors (actor_id INTEGER PRIMARY KEY)";
}
