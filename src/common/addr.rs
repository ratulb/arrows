use crate::{compute_hash, option_of_bytes};
use lazy_static::lazy_static;
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::env;
use std::hash::Hash;
use std::net::{IpAddr, SocketAddr};

lazy_static! {
    pub static ref PORT: u16 = env::var("port").unwrap_or_else(|_| "7171".to_string())[..]
        .parse()
        .unwrap();
    pub static ref ADDR: &'static str = Box::leak(
        env::var("ip_addr")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .into_boxed_str()
    );
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, Hash, Default)]
pub struct Addr {
    id: u64,
    name: String,
    class: Option<String>,
    ns: Option<String>,
    host: Option<String>,
    port: Option<u16>,
}

impl Addr {
    pub fn new(name: &str) -> Self {
        let mut addr = Self {
            id: 0,
            name: name.to_string(),
            class: Some("default".to_string()),
            ns: Some("system".to_string()),
            host: Some((&ADDR).to_string()),
            port: Some(*PORT),
        };
        Self::addr_hash(&mut addr);
        addr
    }
    pub fn with_port(&mut self, port: u16) {
        self.port = Some(port);
        Self::addr_hash(self);
    }
    pub fn with_ip(&mut self, ip: &str) {
        let parseable: Result<IpAddr, _> = ip.parse();
        if parseable.is_ok() {
            self.host = Some(ip.to_string());
            Self::addr_hash(self);
        } else {
            eprintln!("Could not parse given ip: {:?}", ip);
        }
    }
    fn addr_hash(addr: &mut Addr) {
        addr.id = 0;
        addr.id = compute_hash(&addr);
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_id(&self) -> u64 {
        self.id
    }
    pub fn get_socket_addr(&self) -> Option<SocketAddr> {
        if let Some(h) = &self.host {
            return Some(SocketAddr::new(
                h[..].parse().ok().unwrap(),
                self.port.unwrap(),
            ));
        }
        None
    }
    pub fn is_local(&self) -> bool {
        match self.get_socket_addr() {
            None => false,
            Some(sa) => {
                if sa.ip().is_loopback() {
                    true
                } else {
                    let local_ip = local_ip();
                    local_ip.is_ok() && sa.ip() == local_ip.unwrap()
                }
            }
        }
    }

    pub fn get_host(&self) -> Option<&String> {
        self.host.as_ref()
    }
    pub fn get_port(&self) -> Option<u16> {
        self.port
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        option_of_bytes(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_addr_test1() {
        let addr1 = Addr::new("add1");
        let addr2 = Addr::new("add1");
        assert_eq!(addr1.id, addr2.id);
    }
    #[test]
    fn create_addr_test2() {
        let addr1 = Addr::new("add1");
        assert!(addr1.get_socket_addr().unwrap().ip().is_loopback());
    }
    #[test]
    fn create_addr_test3() {
        let addr2 = Addr::new("add2");
        assert!(addr2.is_local());
    }
    #[test]
    fn create_addr_change_port_test_1() {
        use std::env;
        env::set_var("port", "7171");
        let mut addr = Addr::new("addr");
        let id = addr.get_id();
        assert!(addr.is_local());
        addr.with_port(7171);
        assert_eq!(addr.get_id(), id);
        addr.with_port(7172);
        assert_ne!(addr.get_id(), id);
    }
    #[test]
    fn create_addr_change_ip_test1() {
        let mut addr = Addr::new("add");
        assert!(addr.is_local());
        let id = addr.get_id();
        addr.with_ip("300.300.300.300");
        assert_eq!(addr.get_id(), id);
        addr.with_ip("10.160.0.2");
        assert!(addr.is_local());
    }
}
