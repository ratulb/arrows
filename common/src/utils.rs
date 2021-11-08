use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Result, Seek, Write};
use std::path::Path;

use std::convert::TryInto;

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

pub fn file_exists(file_path: &str) -> bool {
    Path::new(file_path).exists()
}

pub fn convert_to_arr<const N: usize>(v: Vec<u8>) -> [u8; N] {
    v.try_into().unwrap_or_else(|v: Vec<u8>| {
        panic!("Expected a Vec of length {} but it was {}", N, v.len())
    })
}

pub fn type_of<T>(_: &T) {
    println!("The type is {}", std::any::type_name::<T>());
}

pub fn compute_hash<T>(value: &T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub async fn to_file<T: Serialize>(value: T, file: &str) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &value)?;
    writer.flush()?;
    Ok(())
}
pub fn to_file_sync<T: Serialize>(value: T, file: &str) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &value)?;
    writer.flush()?;
    Ok(())
}

pub async fn to_stream<W: Seek + Write, T: Serialize>(value: T, writer: &mut W) -> Result<()> {
    serde_json::to_writer(writer, &value)?;
    Ok(())
}

pub async fn from_file<T: for<'de> Deserialize<'de>>(file: &str) -> Result<T> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let t: T = serde_json::from_reader(reader)?;
    Ok(t)
}
pub fn from_file_sync<T: for<'de> Deserialize<'de>>(file: &str) -> Result<T> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let t: T = serde_json::from_reader(reader)?;
    Ok(t)
}

pub fn option_of_bytes<T: ?Sized + std::fmt::Debug + Serialize>(t: &T) -> Option<Vec<u8>> {
    println!("The incoming t: {:?}", t);
    match serialize(t) {
        Ok(bytes) => Some(bytes),
        Err(err) => {
            eprintln!("Error serializing: {:?}", err);
            None
        }
    }
}

pub fn from_bytes<'a, T: std::fmt::Debug + Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    //use std::io::{Error, ErrorKind};
    use std::io::Error;
    match deserialize(bytes) {
        Ok(t) => Ok(t),
        Err(err) => {
            eprintln!("Error derializing: {:?}", err);
            //Err(Error::new(ErrorKind::Other, "Failed deserializing"))
            Err(Error::last_os_error())
        }
    }
}

pub fn from_byte_array<'a, T: std::fmt::Debug + Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    //use std::io::{Error, ErrorKind};
    use std::io::Error;
    match deserialize(bytes) {
        Ok(t) => Ok(t),
        Err(err) => {
            eprintln!("Error derializing: {:?}", err);
            //Err(Error::new(ErrorKind::Other, "Failed deserializing"))
            Err(Error::last_os_error())
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
