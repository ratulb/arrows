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
    #[test]
    fn create_addr_test1() {
        let addr1 = Address::new("add1");
        let addr2 = Address::new("add2");
        println!("Address: {:?}", addr1);
        println!("Address: {:?}", addr2);
        assert_eq!(addr1.id, addr2.id);
    }
}

mod addresses {
    use super::*;
    use std::alloc::{alloc, dealloc, Layout};
    
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub(super) struct Addresses {
        #[serde(skip_serializing,skip_deserializing)]
        ptr: *mut Address,
        len: usize,
    }

    impl Addresses {
        pub fn new(len: usize) -> Self {
            let ptr = unsafe {
                let layout = Layout::from_size_align_unchecked(len, std::mem::size_of::<Address>());
                alloc(layout) as *mut Address
            };
            Self { ptr, len }
        }
        pub fn get(&self, idx: usize) -> Option<&Address> {
            if idx < self.len {
                unsafe { Some(&*(self.ptr.add(idx))) }
            } else {
                None
            }
        }
        pub fn get_mut(&self, idx: usize) -> Option<&mut Address> {
            if idx < self.len {
                unsafe { Some(&mut *(self.ptr.add(idx))) }
            } else {
                None
            }
        }
        pub fn len(&self) -> usize {
            self.len
        }
    }

    impl Drop for Addresses {
        fn drop(&mut self) {
            unsafe {
                dealloc(
                    self.ptr as *mut u8,
                    Layout::from_size_align_unchecked(self.len, std::mem::size_of::<Address>()),
                )
            };
        }
    }

    impl std::ops::Index<usize> for Addresses {
        type Output = Address;
        fn index(&self, index: usize) -> &Self::Output {
            self.get(index).unwrap()
        }
    }
    impl std::ops::IndexMut<usize> for Addresses {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            self.get_mut(index).unwrap()
        }
    }
}
