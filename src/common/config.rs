use lazy_static::lazy_static;
use local_ip_address::local_ip;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use std::env;
use std::hash::Hash;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::from_env());
}

static WINDOWS: &str = "target\\debug\\arrows.exe";
static LINUX: &str = "target/debug/arrows";

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Config {
    host: String,
    port: u16,
    db_path: String,
    resident_listener: String,
}

impl Config {
    pub fn get_shared() -> RwLockReadGuard<'static, Self> {
        CONFIG.read()
    }
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
                let port = env::var("PORT").unwrap_or("7171".to_string());
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
        Self {
            host,
            port,
            db_path,
            resident_listener,
        }
    }

    pub fn re_init(config: Config) {
        let mut current = CONFIG.write();
        *current = config;
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    pub fn resident_listener(&self) -> &str {
        &self.resident_listener
    }

    pub fn set_host(&mut self, host: &str) {
        self.host = host.to_string();
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn set_db_path(&mut self, db_path: &str) {
        self.db_path = db_path.to_string();
    }

    pub fn set_resident_listener(&mut self, resident_listener: &str) {
        self.resident_listener = resident_listener.to_string();
    }
}
