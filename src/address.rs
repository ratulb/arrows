use crate::compute_hash;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

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
pub struct Address<'a> {
    id: u64,
    #[serde(borrow)]
    name: &'a str,
    class: Option<&'a str>,
    #[serde(borrow)]
    ns: Option<&'a str>,
    #[serde(borrow)]
    host: Option<&'a str>,
    port: Option<u16>,
    proto: Option<Scheme>,
    #[serde(borrow)]
    parent: Option<&'a str>,
}

impl<'a> Address<'a> {
    pub fn new(name: &'a str) -> Self {
        let mut addr = Self {
            id: 0,
            name,
            class: Some("default"),
            ns: Some("system"),
            host: Some("127.0.0.1"),
            port: Some(7171),
            proto: Some(Scheme::Inprocess),
            parent: None,
        };
        addr.id = compute_hash(&addr);
        addr
    }
    pub fn get_name(&'a self) -> &'a str {
        self.name
    }
}

impl<'a> Default for Address<'a> {
    fn default() -> Self {
        Self {
            id: 0,
            name: "",
            class: None,
            ns: None,
            host: None,
            port: None,
            proto: None,
            parent: None,
        }
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
