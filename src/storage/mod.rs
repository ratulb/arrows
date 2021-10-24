use crate::{from_byte_array, option_of_bytes, Message};
use constants::*;

use rusqlite::{
    named_params, types::ValueRef, CachedStatement, Connection, Result, Row, Statement,
};
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
            "CREATE TABLE IF NOT EXISTS inbox_{} (msg_id INTEGER PRIMARY KEY, msg BLOB)",
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

    pub(crate) fn into_outbox(&mut self, actor_id: u64, msg: Message<'_>) -> Result<()> {
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

    pub(crate) fn into_inbox(&mut self, actor_id: u64, msg: Message<'_>) -> Result<()> {
        let stmt = self.inbox_insert_stmts.entry(actor_id).or_insert_with(|| {
            self.conn
                .prepare_cached(&format!(
                    "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
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

    pub(crate) fn drain_inbox(&mut self, actor_id: u64) -> Result<VecDeque<Message<'a>>> {
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
                let rows = s.query_and_then([], |row| row.get::<_, Message<'_>>(0))?;
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

pub(crate) fn into_inbox(actor_id: u64, msg: Message<'_>) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.into_inbox(actor_id, msg)
}
pub(crate) fn into_outbox(actor_id: u64, msg: Message<'_>) -> Result<()> {
    let conn = Connection::open(DATABASE)?;
    let mut ctx = StorageContext::new(&conn);
    ctx.setup();
    ctx.into_outbox(actor_id, msg)
}

pub(crate) fn remove_db() -> std::io::Result<()> {
    std::fs::remove_file(DATABASE)
}

use crate::type_of;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult};
impl FromSql for Message<'_> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Ok(Message::Blank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_byte_array, option_of_bytes, type_of, Message};
    use rand::{thread_rng, Rng};
    use rusqlite::types::ValueRef;
    use rusqlite::{params, DropBehavior, Result};

    #[test]
    fn setup_test_1() {
        let _actor_id: u64 = 1000;
    }

    #[test]
    fn read_inbox_test1() {
        let actor_id = 1000;
        let conn = Connection::open(DATABASE).unwrap();
        let mut ctx = StorageContext::new(&conn);
        ctx.setup();
        let messages = ctx.drain_inbox(actor_id).unwrap();
        for msg in messages {
            println!("The msg: {:?}", msg);
        }
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
    fn read_from_inbox() -> Result<()> {
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
                let r: std::io::Result<Message<'_>> = from_byte_array(b);
                println!("{:?}", r.unwrap());
            }
        }
        Ok(())
    }

    #[test]
    fn drop_create_insert_test1() {
        let num = 10;
        for _ in 0..num {
            assert_eq!(create_table_and_insert_message(), Ok(()));
        }
    }

    fn insert_message_batch(num: u32) -> Result<()> {
        println!("What is happening1:wq0?");
        let conn = Connection::open(DATABASE)?;
        //let mut tx = conn.transaction()?;
        //tx.set_drop_behavior(DropBehavior::Commit);
        //set_prepared_statement_cache_capacity(&self, capacity: usize)
        println!("What is happening200?");
        let mut rng = thread_rng();
        let mut stmt = conn.prepare_cached("INSERT INTO inbox (msg_id, msg) VALUES (?, ?)")?;
        println!("What is happening1:wq300");
        for _ in 0..num {
            let random_num: u64 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num.to_string());
            let msg = Message::new_with_text(&msg_content, "from", "to");
            let id = msg.get_id();
            let msg: Option<Vec<u8>> = option_of_bytes(&msg);
            println!("What is happening?");
            //stmnt.insert([msg, id])?;
            stmt.execute(params![&id, &msg])?;
        }

        Ok(())
    }
    #[test]
    fn insert_message_batch_test_1() {
        let num = 100;
        //InvalidParameterCount
        //Err(SqliteFailure(
        //Err(ToSqlConversionFailure(TryFromIntError
        //Err(SqliteFailure(Error { code: ReadOnly, extended_code: 1032 }
        //Err(SqliteFailure(Error { code: TypeMismatch
        let r = insert_message_batch(num);
        println!("What is the matter? {:?}", r);
    }
}

mod constants {
    pub(super) const DATABASE: &str = "arrows.db";
    pub(super) const FETCH_LIMIT: &str = "100";
    pub(super) const BEGIN_TRANSACTION: &str = "BEGIN TRANSACTION;";
    pub(super) const COMMIT_TRANSACTION: &str = "COMMIT TRANSACTION;";
    pub(super) const SELECT_ACTORS: &str = "SELECT actor_id FROM actors";
    pub(super) const ACTORS: &str =
        "CREATE TABLE IF NOT EXISTS actors (actor_id INTEGER PRIMARY KEY)";
}
