pub(crate) const DATABASE: &str = "arrows.db";
pub(crate) const ARROWS_DB_PATH: &str = "ARROWS_DB_PATH";
pub(crate) const FETCH_LIMIT: usize = 1000;
pub(crate) const BUFFER_MAX_SIZE: usize = 1;
pub(crate) const ACTOR_BUFFER_SIZE: usize = 1;
pub(crate) const TABLE_MESSAGES: &str = "messages";
//In seconds
pub(crate) const EVENT_MAX_AGE: u64 = 1;
pub(crate) const TX_BEGIN: &str = "BEGIN TRANSACTION;";
pub(crate) const TX_COMMIT: &str = "COMMIT TRANSACTION;";
pub(crate) const SELECT_ACTORS: &str = "SELECT actor_id FROM actors";
//TODO check where its being used?
//pub(self) const DOES_TABLE_EXIST: &str =
//  "SELECT count(1) FROM sqlite_master WHERE type='table' AND name=?";
pub(crate) const MESSAGES: &str =
"CREATE TABLE IF NOT EXISTS messages (actor_id TEXT, msg_id TEXT, msg_seq INTEGER,inbound INTEGER DEFAULT 1, msg BLOB, PRIMARY KEY (actor_id, msg_id, msg_seq))";

pub(crate) const ACTORS: &str =
    "CREATE TABLE IF NOT EXISTS actors (actor_id TEXT PRIMARY KEY, addr BLOB DEFAULT NULL, actor_def TEXT, state BLOB DEFAULT NULL, msg_seq INTEGER DEFAULT 0)";
pub(crate) const EVENTS: &str =
    "CREATE TABLE IF NOT EXISTS events (row_id INTEGER PRIMARY KEY, status TEXT DEFAULT 'N')";

pub(crate) const ACTOR_DEF_INSERT: &str =
    "INSERT INTO actors (actor_id, addr, actor_def) VALUES (:actor_id, :addr, :actor_def)";
pub(crate) const INSERT_INTO_MESSAGES: &str =
"INSERT INTO messages (actor_id, msg_id, msg_seq, msg) VALUES(:actor_id, :msg_id,(SELECT IFNULL(MAX(msg_seq), 0) + 1 FROM messages where actor_id = :actor_id), :msg)";

pub(crate) const EVENTS_INSERT: &str = "INSERT INTO events (row_id) VALUES (:row_id)";
pub(crate) const DELETE_ACTOR: &str = "DELETE FROM actors WHERE actor_id = ?";
pub(crate) const ACTOR_ROWID: &str = "SELECT rowid FROM actors WHERE actor_id = ?";
pub(crate) const ACTOR_DEF: &str = "SELECT addr, actor_def, msg_seq FROM actors WHERE actor_id = ?";

pub(crate) const EVENTS_SELECT: &str = "SELECT row_id FROM events WHERE status ='N'";

pub(crate) const MIN_MSG_SEQ: &str = "SELECT MIN(msg_seq), M.rowid, E.row_id  FROM  messages M, events E WHERE M.actor_id = ? AND M.inbound = 1 AND M.rowid=E.row_id AND E.status = 'N'";

pub(crate) const UPDATE_EVENTS: &str = "UPDATE events SET status = 'Y' WHERE row_id = ?";
pub(crate) const UPDATE_ACTOR_EVENT_SEQ: &str = "UPDATE actors SET msg_seq = ? WHERE actor_id = ?";
