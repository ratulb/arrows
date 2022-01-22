//! # Config
//!The centralized configuration construct - to run multiple instances of the system
//!supply these settings at startup

use lazy_static::lazy_static;
use local_ip_address::local_ip;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use std::env;
use std::hash::Hash;
lazy_static! {
//The shared config - gets initialized at the system start
     static ref CONFIG: RwLock<Config> = RwLock::new(Config::from_env());
}
//Dev binary path on windows
static WINDOWS: &str = "target\\debug\\arrows.exe";
//Dev binary path on linux
static LINUX: &str = "target/debug/arrows";

///The config struct
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Config {
    host: String,
    port: u16,
    db_path: String,
    resident_listener: String,
    db_buff_size: usize,
}

impl Config {
    ///Get the read lock
    pub fn get_shared() -> RwLockReadGuard<'static, Self> {
        CONFIG.read()
    }

    ///Retrieve env settings at start
    pub fn from_env() -> Config {
        let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/tmp".to_string());
        let resident_listener = env::var("resident_listener").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                WINDOWS.to_string()
            } else {
                LINUX.to_string()
            }
        });

        let (host, port) = match env::var("LISTEN_ADDR") {
            Ok(address) => {
                let mut hostport = address.split(':');
                let host = hostport.next().unwrap_or("0.0.0.0");
                let port = hostport.next().unwrap_or("7171");
                (host.to_string(), port.to_string())
            }
            Err(err) => {
                eprintln!("{}", err);
                let port = env::var("PORT").unwrap_or_else(|_| "7171".to_string());
                match local_ip() {
                    Ok(ip) => (ip.to_string(), port),
                    Err(err) => {
                        eprintln!("{}", err);
                        ("0.0.0.0".to_string(), port)
                    }
                }
            }
        };

        let port: u16 = port.parse().expect("port num");
        let db_buff_size: usize = env::var("db_buff_size")
            .unwrap_or("1".to_string())
            .parse()
            .expect("db_buff_size");

        Self {
            host,
            port,
            db_path,
            resident_listener,
            db_buff_size,
        }
    }
    ///Reinit based on user supplied config when the CLI is run
    pub fn re_init(config: Config) {
        let mut current = CONFIG.write();
        *current = config;
    }
    ///The host ip
    pub fn host(&self) -> &str {
        &self.host
    }
    ///Port the listener start with - default is 7171
    pub fn port(&self) -> u16 {
        self.port
    }
    ///Embedded sqlite db path
    pub fn db_path(&self) -> &str {
        &self.db_path
    }
    ///Location of listener binary
    pub fn resident_listener(&self) -> &str {
        &self.resident_listener
    }
    ///Gets reinitialzed based on user supplied IP:PORT - Addrs get created based on this
    pub fn set_host(&mut self, host: &str) {
        self.host = host.to_string();
    }
    ///Alter the port - Addrs reflect this
    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }
    ///Alter it if running multiple arrows instances on the same node
    pub fn set_db_path(&mut self, db_path: &str) {
        self.db_path = db_path.to_string();
    }
    ///Depends on what profile we are running under. Final location of the listener binary.
    pub fn set_resident_listener(&mut self, resident_listener: &str) {
        self.resident_listener = resident_listener.to_string();
    }
    ///How much buffering the backing store should instead of executing database operations
    ///for every transaction that takes place in the system.
    ///
    ///Configurable via `db_buffer_size`.

    pub fn db_buff_size(&self) -> usize {
        self.db_buff_size
    }
    ///Set the db_buff_size. It should update all cached values. Currently its loaded at
    ///system start time
    ///
    pub fn set_db_buff_size(&mut self, buff_size: usize) {
        self.db_buff_size = buff_size;
    }
}
