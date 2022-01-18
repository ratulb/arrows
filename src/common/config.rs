use lazy_static::lazy_static;
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
    listen_addr: String,
    resident_listener: String,
}

impl Config {
    pub fn get_shared() -> RwLockReadGuard<'static, Self> {
        CONFIG.read()
    }
    pub fn from_env() -> Config {
        let port: u16 = env::var("port").unwrap_or_else(|_| "7171".to_string())[..]
            .parse()
            .unwrap();
        let host = env::var("ip_addr").unwrap_or_else(|_| "127.0.0.1".to_string());
        let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/tmp".to_string());
        let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:7171".to_string());
        let resident_listener = env::var("resident_listener").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                WINDOWS.to_string()
            } else {
                LINUX.to_string()
            }
        });
        Self {
            host,
            port,
            db_path,
            listen_addr,
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

    pub fn listen_addr(&self) -> &str {
        &self.listen_addr
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

    pub fn set_listen_addr(&mut self, listen_addr: &str) {
        self.listen_addr = listen_addr.to_string();
    }

    pub fn set_resident_listener(&mut self, resident_listener: &str) {
        self.resident_listener = resident_listener.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_default_test() {
        for _i in 0..10000 {
            let config = Config::get_shared();
            println!("{:?}", config);
        }
    }
    #[test]
    fn config_from_env_test() {
        for _i in 0..10000 {
            let config = Config::from_env();
            println!("{:?}", config);
        }
    }
}
