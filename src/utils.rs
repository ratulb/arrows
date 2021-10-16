use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Read, Result, Seek, Write};

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
