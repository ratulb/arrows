use crate::{from_byte_array, from_bytes, option_of_bytes, Message};
use rusqlite::{named_params, params, CachedStatement, Connection, Result, Statement};
use std::collections::HashMap;

const BEGIN_TRANSACTION: &'static str = "BEGIN TRANSACTION;";
const COMMIT_TRANSACTION: &'static str = "COMMIT TRANSACTION;";

pub(crate) struct StorageContext<'a> {
    conn: &'a Connection,
    insert_stmts: HashMap<u64, Option<CachedStatement<'a>>>,
    select_stmnts: HashMap<u64, Option<Statement<'a>>>,
    create_outbox_stmnts: HashMap<u64, Option<bool>>,
}

impl<'a> StorageContext<'a> {
    pub(crate) fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            insert_stmts: HashMap::new(),
            select_stmnts: HashMap::new(),
            create_outbox_stmnts: HashMap::new(),
        }
    }
    pub(crate) fn setup(&mut self) -> Result<()> {
        self.conn.execute_batch(BEGIN_TRANSACTION)?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS actors (actor_id INTEGER PRIMARY KEY)",
            [],
        )?;
        let mut stmt = self.conn.prepare("SELECT actor_id FROM actors")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let actor_id: u64 = row.get(0)?;
            self.inbox_for_actor(actor_id)?;
            self.outbox_for_actor(actor_id)?;
        }
        self.conn.execute_batch(COMMIT_TRANSACTION)?;

        Ok(())
    }
    pub(crate) fn inbox_for_actor(&mut self, actor_id: u64) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS inbox_{} (msg_id INTEGER PRIMARY KEY, msg BLOB)",
            &actor_id.to_string()[..]
        );
        self.conn.execute(&stmt, [])?;
        Ok(())
    }
    pub(crate) fn outbox_for_actor(&mut self, actor_id: u64) -> Result<()> {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS outbox_{} (msg_id INTEGER PRIMARY KEY, msg BLOB)",
            &actor_id.to_string()[..]
        );
        self.conn.execute(&stmt, [])?;
        Ok(())
    }

    pub(crate) fn insert_into_inbox(&mut self, actor_id: u64, msg: Message) -> Result<()> {
        let mut stmt = self.insert_stmts.entry(actor_id).or_insert_with(|| {
            Some(
                self.conn
                    .prepare_cached(&format!(
                        "INSERT INTO inbox_{} (msg_id, msg) VALUES (:msg_id, :msg)",
                        &actor_id.to_string()[..]
                    ))
                    .ok()?,
            )
        });

        let msg_id = msg.get_id().clone().to_string();
        let bytes = option_of_bytes(&msg);
        match stmt {
            Some(ref mut s) => s.execute(named_params! { ":msg_id": msg_id, ":msg": bytes })?,
            None => panic!(),
        };
        Ok(())
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
    let conn = Connection::open("arrows.db")?;
    let mut ctx = StorageContext::new(&conn);
    ctx.create_inbox(actor_id)
}

pub(crate) fn create_actor_outbox(actor_id: u64) -> Result<()> {
    let conn = Connection::open("arrows.db")?;
    let mut ctx = StorageContext::new(&conn);
    ctx.create_outbox(actor_id)
}
pub(crate) fn insert_into_inbox(actor_id: u64, msg: Message) -> Result<()> {
    let conn = Connection::open("arrows.db")?;
    let mut ctx = StorageContext::new(&conn);
    ctx.insert_into_inbox(actor_id, msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{option_of_bytes, type_of, Message};
    use rand::{thread_rng, Rng};
    use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
    use rusqlite::{DropBehavior, Result};

    use rusqlite::MappedRows;
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
    #[test]
    fn insert_into_inbox_test1() {
        let insert_into_inbox_result =
            insert_into_inbox(1000, Message::new_with_text("The test msg", "from", "to"));
        assert_eq!(insert_into_inbox_result, Ok(()));
    }

    pub(crate) fn create_table_and_insert_message() -> Result<()> {
        let conn = Connection::open("arrows.db")?;
        //let mut stmt = conn.prepare("DROP TABLE IF EXISTS inbox")?;
        //stmt.execute(params![])?;
        conn.execute("CREATE TABLE IF NOT EXISTS inbox (msg BLOB)", [])?;
        let msg = Message::new_with_text("The test msg", "from", "to");
        let id = msg.get_id();
        let msg: Option<Vec<u8>> = option_of_bytes(&msg);
        //let msg: Option<Vec<u8>> = Some(vec![1,2,3,4]);
        conn.execute("INSERT INTO inbox (msg) VALUES (?1)", params![msg])?;

        // let mut stmt = conn.prepare("SELECT name FROM inbox")?;
        // type_of(& stmt.query_map([], |row| row.get(0))?);

        Ok(())
    }
    #[test]
    fn read_from_inbox() -> Result<()> {
        let mut conn = Connection::open("arrows.db")?;
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
                //println!("b then is: {:?}", b);
                let r: std::io::Result<Message> = from_byte_array(b);
                println!("{:?}", r.unwrap());
            }
        }
        //tx.commit()
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
        let mut conn = Connection::open("arrows.db")?;
        let mut tx = conn.transaction()?;
        tx.set_drop_behavior(DropBehavior::Commit);
        //set_prepared_statement_cache_capacity(&self, capacity: usize)
        let mut rng = thread_rng();
        let mut stmnt = tx.prepare_cached("INSERT INTO inbox (msg) VALUES (?)")?;
        for _ in 0..num {
            let random_num: i32 = rng.gen();
            let msg_content = format!("The test msg-{}", random_num.to_string());
            let msg = Message::new_with_text(&msg_content, "from", "to");
            //let id = msg.get_id();
            let msg: Option<Vec<u8>> = option_of_bytes(&msg);
            stmnt.execute([msg])?;
        }

        Ok(())
    }
    #[test]
    fn insert_message_batch_test_1() {
        let num = 100000;
        insert_message_batch(num);
    }
}
