use crate::compute_hash;
use lazy_static::lazy_static;
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::env;
use std::hash::Hash;
use std::net::{IpAddr, SocketAddr};

lazy_static! {
    pub static ref PORT: u16 = env::var("port").unwrap_or("7171".to_string())[..]
        .parse()
        .unwrap();
    pub static ref ADDR: &'static str = Box::leak(
        env::var("ip_addr")
            .unwrap_or("127.0.0.1".to_string())
            .into_boxed_str()
    );
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Scheme {
    Email,
    Inprocess,
    Http,
    Https,
    Tcp,
    Grpc,
    Udp,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Address {
    id: u64,
    name: String,
    class: Option<String>,
    ns: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    proto: Option<Scheme>,
}

impl Address {
    pub fn new(name: &str) -> Self {
        let mut addr = Self {
            id: 0,
            name: name.to_string(),
            class: Some("default".to_string()),
            ns: Some("system".to_string()),
            host: Some((&ADDR).to_string()),
            port: Some(*PORT),
            proto: Some(Scheme::Inprocess),
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
    fn addr_hash(addr: &mut Address) {
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
}

impl Default for Address {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            class: None,
            ns: None,
            host: None,
            port: None,
            proto: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_addr_test1() {
        let addr1 = Address::new("add1");
        let addr2 = Address::new("add1");
        assert_eq!(addr1.id, addr2.id);
    }
    #[test]
    fn create_addr_test2() {
        let addr1 = Address::new("add1");
        println!(
            "address is local: {}",
            addr1.get_socket_addr().unwrap().ip().is_loopback()
        );
    }
    #[test]
    fn create_addr_test3() {
        let addr2 = Address::new("add2");
        assert_eq!(addr2.is_local(), true);
    }
    #[test]
    fn create_addr_change_port_test_1() {
        use std::env;
        env::set_var("port", "7171");
        let mut addr = Address::new("addr");
        let id = addr.get_id();
        assert_eq!(addr.is_local(), true);
        addr.with_port(7171);
        assert_eq!(addr.get_id(), id);
        addr.with_port(7172);
        assert_ne!(addr.get_id(), id);
    }
    #[test]
    fn create_addr_change_ip_test1() {
        let mut addr = Address::new("add");
        assert_eq!(addr.is_local(), true);
        let id = addr.get_id();
        addr.with_ip("300.300.300.300");
        assert_eq!(addr.get_id(), id);
        addr.with_ip("10.160.0.2");
        assert_eq!(addr.is_local(), true);
    }

    #[test]
    fn check_hostip_and_port_test1() {
        let addr = Address::new("add");
        println!("{:?}", addr);
    }
}
