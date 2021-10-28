use arrows_common::{from_byte_array, option_of_bytes, Message};
use constants::*;
use rusqlite::{
    named_params, params, types::Value, CachedStatement, Connection, Result, Statement, ToSql,
};
use std::collections::{HashMap, VecDeque};
use std::io::{Error, ErrorKind};
use std::str::FromStr;

pub(crate) struct StorageContext<'a> {
    conn: &'a Connection,
    inbox_insert_stmts: HashMap<String, Option<CachedStatement<'a>>>,
    inbox_select_stmts: HashMap<String, Option<CachedStatement<'a>>>,
    outbox_insert_stmts: HashMap<String, Option<CachedStatement<'a>>>,
    outbox_select_stmts: HashMap<String, Option<CachedStatement<'a>>>,

    select_stmnts: HashMap<String, Option<Statement<'a>>>,
    create_outbox_stmnts: HashMap<String, Option<bool>>,
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
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let actor_id: String = row.get(0)?;
            self.inbox_of(&actor_id)?;
            self.outbox_of(&actor_id)?;
        }
        self.conn.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }
    pub(crate) fn inbox_of(&mut self, actor_id: &String) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS inbox_{} (msg_id TEXT PRIMARY KEY, msg BLOB)",
            actor_id
        );
        self.conn.execute(&stmt, [])?;
        Ok(())
    }
    pub(crate) fn purge_inbox_of(&mut self, actor_id: &String) -> Result<()> {
        let stmt = format!(
            "SELECT count(1) FROM sqlite_master WHERE type='table' AND name='inbox_{}'",
            actor_id
        );
        let mut stmt = self.conn.prepare(&stmt)?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let value: usize = row.get(0)?;
            if value == 1 {
                println!("Table exists");
                let stmt = format!("DELETE FROM inbox_{}", actor_id);
                match self.conn.execute(&stmt, []) {
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
            .execute_batch(BEGIN_TRANSACTION)
            .map_err(sql_to_io);
        let mut stmt = self.conn.prepare_cached(&stmt).map_err(sql_to_io)?;
        for msg_id in msg_ids {
            stmt.execute(params![msg_id]).map_err(sql_to_io);
        }
        self.conn
            .execute_batch(COMMIT_TRANSACTION)
            .map_err(sql_to_io);
        Ok(())
    }
    pub(crate) fn select_from_inbox(
        &mut self,
        actor_id: &String,
        msg_ids: Vec<&str>,
    ) -> Result<VecDeque<Message>> {
        let mut count = 0;
        let size = msg_ids.len();
        let msg_ids_in = msg_ids
            .iter()
            .map(|id| {
                count += 1;
                let mut s = String::from("'");
                s.push_str(id);
                s.push_str("'");
                if count < size {
                    s.push_str(",");
                }
                s
            })
            .collect::<String>();
        let stmt = format!(
            "SELECT msg FROM inbox_{} WHERE msg_id in ({})",
            actor_id, msg_ids_in
        );
        let mut stmt = self.conn.prepare(&stmt)?;
        let mut rows = stmt.query([])?;
        let mut messages = VecDeque::new();
        while let Some(row) = rows.next()? {
            let value: Value = row.get(0)?;
            messages.push_front(value_to_msg(value));
        }
        Ok(messages)
    }

    pub(crate) fn outbox_of(&mut self, actor_id: &String) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS outbox_{} (msg_id TEXT PRIMARY KEY, msg BLOB)",
            actor_id
        );
        self.conn.execute(&stmt, [])?;
        Ok(())
    }

    pub(crate) fn into_outbox(&mut self, actor_id: &String, msg: Message) -> Result<()> {
        let stmt = self
            .outbox_insert_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                self.conn
                    .prepare_cached(&format!(
                        "INSERT INTO outbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                        actor_id
                    ))
                    .ok()
            });

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

    pub(crate) fn into_inbox(&mut self, actor_id: &String, msg: Message) -> Result<()> {
        let stmt = self
            .inbox_insert_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                self.conn
                    .prepare_cached(&format!(
                        "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                        actor_id
                    ))
                    .ok()
            });
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
    pub(crate) fn read_inbox(&mut self, actor_id: &String) -> Result<VecDeque<Message>> {
        let stmt = self
            .inbox_select_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                self.conn
                    .prepare_cached(&format!(
                        "SELECT msg FROM inbox_{} ORDER BY rowid ASC LIMIT {}",
                        actor_id, FETCH_LIMIT
                    ))
                    .ok()
            });

        let mut messages = VecDeque::with_capacity(usize::from_str(FETCH_LIMIT).unwrap());
        match stmt {
            Some(ref mut s) => {
                //let rows = s.query_and_then([], |row| row.get::<_, Message>(0))?;
                let mut rows = s.query_map([], |row| row.get(0))?;
                for row in rows {
                    let value: Value = row?;
                    messages.push_front(value_to_msg(value));
                }
            }
            None => {
                panic!("Error draining inbox - CachedStatement not found")
            }
        }
        return Ok(messages);
    }

    pub(crate) fn read_inbox_full(&mut self, actor_id: &String) -> Result<VecDeque<Message>> {
        let stmt = self
            .inbox_select_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                self.conn
                    .prepare_cached(&format!(
                        "SELECT msg FROM inbox_{} ORDER BY rowid ASC",
                        actor_id
                    ))
                    .ok()
            });
        let mut messages = VecDeque::with_capacity(usize::from_str(FETCH_LIMIT).unwrap());
        match stmt {
            Some(ref mut s) => {
                let mut rows = s.query_map([], |row| row.get(0))?;
                for row in rows {
                    let value: Value = row?;
                    messages.push_front(value_to_msg(value));
                }
            }
            None => panic!("Error draining inbox - CachedStatement not found"),
        }
        return Ok(messages);
    }

    pub(crate) fn into_inbox_batch(
        &mut self,
        actor_id: &String,
        msg_itr: impl Iterator<Item = Message>,
    ) -> Result<()> {
        self.conn.execute_batch(BEGIN_TRANSACTION)?;
        let stmt = self
            .inbox_insert_stmts
            .entry(actor_id.to_string())
            .or_insert_with(|| {
                self.conn
                    .prepare_cached(&format!(
                        "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                        actor_id
                    ))
                    .ok()
            });

        match stmt {
            Some(ref mut s) => {
                for msg in msg_itr {
                    let bytes = option_of_bytes(&msg);
                    let status = s.execute(named_params! { ":msg_id": &msg.id_as_string() as &dyn ToSql, ":msg": &bytes as &dyn ToSql })?;
                }
            }
            None => panic!(),
        };
        self.conn.execute_batch(COMMIT_TRANSACTION)?;
        Ok(())
    }
}
pub(crate) fn sql_to_io(err: rusqlite::Error) -> std::io::Error {
    eprintln!("rusqlite::Error has occured: {:?}", err);
    Error::new(ErrorKind::Other, "rusqlite error")
}

pub(crate) fn value_to_msg(v: Value) -> Message {
    if let Value::Blob(bytes) = v {
        return match from_byte_array::<'_, Message>(&bytes) {
            Ok(msg) => msg,
            _ => Message::Blank,
        };
    }
    Message::Blank
}

pub(crate) fn create_actor_inbox(actor_id: &String) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.inbox_of(actor_id)
}

pub(crate) fn create_actor_outbox(actor_id: &String) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.outbox_of(actor_id)
}

pub(crate) fn into_inbox(actor_id: &String, msg: Message) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.into_inbox(actor_id, msg)
}
pub(crate) fn into_outbox(actor_id: &String, msg: Message) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.into_outbox(actor_id, msg)
}

pub(crate) fn remove_db() -> std::io::Result<()> {
    std::fs::remove_file(DATABASE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrows_common::{from_file_sync, Message};
    use rand::{thread_rng, Rng};
    use rusqlite::Connection;
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
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        let messages = ctx.select_from_inbox(&actor_id, msg_ids)?;
        let messages: Vec<_> = messages
            .iter()
            .map(|msg| msg.id_as_string().to_string())
            .collect();
        println!("The messages: {:?}", messages);
        Ok(())
    }
    #[test]
    fn delete_from_inbox_test1() {
        let actor_id = "1000".to_string();
        let msg_ids = vec!["16563997168647304630", "18086766434657795389"];
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        assert_eq!(ctx.delete_from_inbox(&actor_id, msg_ids).ok(), Some(()));
    }

    #[test]
    fn purge_inbox_of_test_1() {
        let actor_id = "1000".to_string();
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        assert_eq!(ctx.purge_inbox_of(&actor_id), Ok(()));
    }
    #[test]
    fn read_inbox_write_out_msg_test_1() -> std::io::Result<()> {
        let actor_id = "1000".to_string();
        let mut read_count = 0;
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
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
        let msg: Message = from_file_sync("last_message.txt")?;
        println!("Message txt: {:?}", msg.content_as_text());
        Ok(())
    }

    #[test]
    fn read_inbox_test1() {
        let actor_id = "1000".to_string();
        let mut read_count = 0;
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        let messages = ctx.read_inbox(&actor_id).unwrap();
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
    fn read_inbox_full_test1() {
        let actor_id = "1000".to_string();
        let mut read_count = 0;
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        let messages = ctx.read_inbox_full(&actor_id).unwrap();
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
        let inbox_create_result = create_actor_inbox(&"1000".to_string());
        assert_eq!(inbox_create_result, Ok(()));
    }
    #[test]
    fn create_actor_outbox_test1() {
        let outbox_create_result = create_actor_outbox(&"1000".to_string());
        assert_eq!(outbox_create_result, Ok(()));
    }

    fn into_inbox_batch_func(num: u32, actor_id: &String) -> Result<()> {
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

    fn into_inbox_no_batch_func(num: u32, actor_id: &String) -> Result<()> {
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
    fn into_inbox_no_batch_test_1() {
        let num = 1;
        let actor_id = "1000".to_string();
        //InvalidParameterCount
        //Err(SqliteFailure(
        //Err(ToSqlConversionFailure(TryFromIntError
        //Err(SqliteFailure(Error { code: ReadOnly, extended_code: 1032 }
        //Err(SqliteFailure(Error { code: TypeMismatch
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
}

mod constants {
    pub(super) const DATABASE: &str = "arrows.db";
    pub(super) const FETCH_LIMIT: &str = "100";
    pub(super) const BEGIN_TRANSACTION: &str = "BEGIN TRANSACTION;";
    pub(super) const COMMIT_TRANSACTION: &str = "COMMIT TRANSACTION;";
    pub(super) const SELECT_ACTORS: &str = "SELECT actor_id FROM actors";
    pub(self) const DOES_TABLE_EXIST: &str =
        "SELECT count(1) FROM sqlite_master WHERE type='table' AND name=?";
    pub(super) const ACTORS: &str = "CREATE TABLE IF NOT EXISTS actors (actor_id TEXT PRIMARY KEY)";
}
