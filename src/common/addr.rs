use crate::common::config::Config;
use crate::{compute_hash, option_of_bytes};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

static WILDCARD_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

///`Addr` - An actor address with name, node ip and port

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
            host: Some(Config::get_shared().host().to_string()),
            port: Some(Config::get_shared().port()),
        };
        Self::addr_hash(&mut addr);
        addr
    }
    pub fn remote(name: &str, hostport: &str) -> Self {
        let mut addr = Self::new(name);
        let mut hostport = hostport.split(':');
        let host = hostport.next().map(|host| host.to_string());
        let port = hostport
            .next()
            .map(|port| port.parse::<u16>().unwrap_or(7171));
        addr.host = host;
        addr.port = port;
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
            return Some(SocketAddr::new(h[..].parse().ok()?, self.port?));
        }
        None
    }

    pub fn get_host_ip(&self) -> IpAddr {
        match self.get_host() {
            Some(host) => match host.parse::<Ipv4Addr>() {
                Ok(ip) => IpAddr::V4(ip),
                Err(err) => panic!("{}", err),
            },
            None => panic!(),
        }
    }

    pub fn is_ip_local(ip: IpAddr) -> bool {
        Config::get_shared()
            .host()
            .to_string()
            .parse()
            .map_or(false, |parsed: IpAddr| parsed == ip)
    }

    pub fn is_local_ip(&self) -> bool {
        let host_ip = self.get_host_ip();
        if host_ip.is_loopback() || host_ip == WILDCARD_IP {
            true
        } else {
            local_ip().map_or(false, |local_ip| local_ip == host_ip)
        }
    }

    pub fn is_local_port(&self) -> bool {
        match self.get_port() {
            Some(port) => port == Config::get_shared().port(),
            None => false,
        }
    }

    pub fn is_local(&self) -> bool {
        self.is_local_ip() && self.is_local_port()
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

impl std::fmt::Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Addr")
            .field("name", &self.name)
            .field("host", self.host.as_ref().unwrap_or(&"not set".to_string()))
            .field("port", &self.port)
            .finish()
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
        assert!(addr1.is_local_ip());
    }
    #[test]
    fn create_addr_test3() {
        let addr2 = Addr::new("add2");
        assert!(addr2.is_local());
    }
    #[test]
    fn create_addr_change_port_test_1() {
        use std::env;
        env::set_var("PORT", "6161");
        let mut addr = Addr::new("addr");
        let id = addr.get_id();
        assert!(addr.is_local());
        addr.with_port(7171);
        assert_ne!(addr.get_id(), id);
        addr.with_port(6161);
        assert_eq!(addr.get_id(), id);
    }
    #[test]
    fn create_addr_change_ip_test1() {
        let mut addr = Addr::new("add");
        assert!(addr.is_local());
        let id = addr.get_id();
        //Set an invalid ip - that should not alter anything
        addr.with_ip("300.300.300.300");
        assert_eq!(addr.get_id(), id);
        addr.with_ip("10.160.0.2");
        assert!(addr.is_local());
        println!("{}", addr);
    }
}
