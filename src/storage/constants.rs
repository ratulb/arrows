pub(crate) const DATABASE: &str = "arrows.db";
//pub(crate) const DATABASE_EVENTS: &str = "arrows_events.db";
pub(crate) const ARROWS_DB_PATH: &str = "ARROWS_DB_PATH";
pub(crate) const FETCH_LIMIT: &str = "1000";
pub(crate) const BUFFER_MAX_SIZE: usize = 5;
pub(crate) const EVENT_MAX_AGE: u64 = 1;
pub(crate) const INBOX: &str = "inbox";
pub(crate) const OUTBOX: &str = "outbox";
pub(crate) const BEGIN_TRANSACTION: &str = "BEGIN TRANSACTION;";
pub(crate) const COMMIT_TRANSACTION: &str = "COMMIT TRANSACTION;";
pub(crate) const SELECT_ACTORS: &str = "SELECT actor_id FROM actors";
//TODO check where its being used?
//pub(self) const DOES_TABLE_EXIST: &str =
//  "SELECT count(1) FROM sqlite_master WHERE type='table' AND name=?";
pub(crate) const INBOX_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS inbox(actor_id TEXT, msg_id TEXT, msg BLOB, PRIMARY KEY(actor_id, msg_id))";
pub(crate) const OUTBOX_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS outbox(actor_id TEXT, msg_id TEXT, msg BLOB, PRIMARY KEY(actor_id, msg_id))";

pub(crate) const ACTORS: &str =
    "CREATE TABLE IF NOT EXISTS actors (actor_id TEXT PRIMARY KEY, build_def TEXT)";
pub(crate) const INBOUNDS: &str =
    "CREATE TABLE IF NOT EXISTS inbounds (row_id INTEGER PRIMARY KEY, status TEXT DEFAULT 'N')";

pub(crate) const OUTBOUNDS: &str =
    "CREATE TABLE IF NOT EXISTS outbounds (row_id INTEGER PRIMARY KEY, status TEXT DEFAULT 'N')";
pub(crate) const BUILD_DEF_INSERT: &str =
    "INSERT INTO actors (actor_id, build_def) VALUES (:actor_id, :build_def)";
pub(crate) const INSERT_INTO_INBOX: &str =
    "INSERT INTO inbox(actor_id, msg_id, msg) VALUES(:actor_id, :msg_id, :msg)";

pub(crate) const INBOUND_INSERT: &str = "INSERT INTO inbounds (row_id) VALUES (:row_id)";
pub(crate) const OUTBOUND_INSERT: &str = "INSERT INTO outbounds (row_id) VALUES (:row_id)";
pub(crate) const DELETE_ACTOR: &str = "DELETE FROM actors WHERE actor_id = ?";
pub(crate) const ACTOR_ROWID: &str = "SELECT rowid FROM actors WHERE actor_id = ?";
pub(crate) const BUILD_DEF: &str = "SELECT build_def FROM actors WHERE actor_id = ?";

pub(crate) const INBOUND_SELECT: &str = "SELECT row_id FROM inbounds WHERE status ='N'";
pub(crate) const OUTBOUND_SELECT: &str = "SELECT row_id FROM outbounds WHERE status ='N'";
