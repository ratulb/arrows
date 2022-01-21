use crate::Result;
use bincode::{deserialize, serialize};

use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;

use std::hash::{Hash, Hasher};

///Compute the hash for a struct like [Addr](crate::common::addr::Addr)
pub fn compute_hash<T>(value: &T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

///Convert a type into byte representation such as [Addr](crate::common::addr::Addr)
///The type needs to be serde json serializable - uses bincode underneath.
///
pub fn option_of_bytes<T: ?Sized + std::fmt::Debug + Serialize>(t: &T) -> Option<Vec<u8>> {
    match serialize(t) {
        Ok(bytes) => Some(bytes),
        Err(err) => {
            eprintln!("Error serializing: {}", err);
            None
        }
    }
}
///Reconstruct a type from array of bytes such as [Msg](crate::common::mail::Mail)
///Reconstructed type must be serde json Derserialize
pub fn from_bytes<'a, T: std::fmt::Debug + Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    match deserialize(bytes) {
        Ok(t) => Ok(t),
        Err(err) => {
            eprintln!("Error derializing: {}", err);
            let err = Into::<bincode::ErrorKind>::into(*err);
            let err: Error = err.into();
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    #[test]
    fn from_bytes_test_1() {
        #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
        struct Complex<T> {
            inner: T,
            elems: Vec<Simple>,
        }
        impl<T> Complex<T> {
            fn get_inner(&self) -> &T {
                &self.inner
            }
        }
        #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
        struct Inner {
            name: String,
            children: Vec<String>,
            male: bool,
            age: u8,
        }
        #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
        struct Simple {
            e1: i32,
            e2: usize,
            e3: Option<bool>,
        }
        let simple = Simple {
            e1: 42,
            e2: 999,
            e3: Some(false),
        };
        let inner = Inner {
            name: "Some body".to_string(),
            children: vec!["Some value".to_string()],
            male: true,
            age: 99,
        };
        let complex = Complex {
            inner,
            elems: vec![simple],
        };
        let option_of_bytes = option_of_bytes(&complex);
        let from_bytes: Complex<Inner> = from_bytes(&option_of_bytes.unwrap()).unwrap();
        assert_eq!(complex, from_bytes);
        assert_eq!(complex.inner, *from_bytes.get_inner());
    }
}
