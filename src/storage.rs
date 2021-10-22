use rusqlite::{Connection, Result, Statement};
use std::collections::HashMap;

pub(crate) struct StorageContext<'a> {
    conn: &'a Connection,
    insert_stmnts: HashMap<u64, Option<Statement<'a>>>,
    select_stmnts: HashMap<u64, Option<Statement<'a>>>,
    create_inbox_stmnts: HashMap<u64, Option<bool>>,
    create_outbox_stmnts: HashMap<u64, Option<bool>>,
}

impl<'a> StorageContext<'a> {
    pub(crate) fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            insert_stmnts: HashMap::new(),
            select_stmnts: HashMap::new(),
            create_inbox_stmnts: HashMap::new(),
            create_outbox_stmnts: HashMap::new(),
        }
    }

    pub(crate) fn create_inbox(&mut self, actor_id: u64) -> Result<()> {
        if self.create_inbox_stmnts.get(&actor_id).is_none() {
            let stmnt = format!(
                "CREATE TABLE IF NOT EXISTS inbox_{} (id INTEGER PRIMARY KEY,name TEXT NOT NULL, data BLOB)",
                &actor_id.to_string()[..]
            );
            self.conn.execute(&stmnt, [])?;
            self.create_inbox_stmnts.insert(actor_id, Some(true));
        }
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
