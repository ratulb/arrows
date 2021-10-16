use crate::compute_hash;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Scheme {
    Email,
    Inprocess,
    Http,
    Https,
    Tcp,
    Grpc,
    Udp,
}
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Address {
    id: u64,
    name: String,
    class: Option<String>,
    ns: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    proto: Option<Scheme>,
    parent: Option<String>,
}

impl Address {
    pub fn new(name: &str) -> Self {
        let mut addr = Self {
            id: 0,
            name: name.to_string(),
            class: Some("default".to_string()),
            ns: Some("system".to_string()),
            host: Some("127.0.0.1".to_string()),
            port: Some(7171),
            proto: Some(Scheme::Inprocess),
            parent: None,
        };
        addr.id = compute_hash(&addr);
        addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_file;
    #[test]
    fn create_addr_test1() {
        let addr1 = Address::new("add1");
        let addr2 = Address::new("add1");
        assert_eq!(addr1.id, addr2.id);
    }
    #[test]
    fn create_addr_test2() {
        let addr1 = Address::new("add1");
        to_file(addr1, "addr1.json");
    }
}
