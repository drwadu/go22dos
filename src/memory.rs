use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::prelude::*;
use thiserror::Error;

use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Error)]
pub enum MemoryError {
    JsonError(#[from] serde_json::Error),
    IoError(#[from] std::io::Error),
    Unknown,
}
impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Result<T> = std::result::Result<T, MemoryError>;
type Data<T, I> = std::collections::HashMap<T, Vec<I>>;

pub trait Serializable:
    Eq
    + PartialEq
    + std::hash::Hash
    + Clone
    + std::fmt::Display
    + AsRef<str>
    + DeserializeOwned
    + Serialize
{
}

impl<T> Serializable for T where
    T: Eq
        + PartialEq
        + std::hash::Hash
        + Clone
        + std::fmt::Display
        + AsRef<str>
        + DeserializeOwned
        + Serialize
{
}

#[derive(Debug, Clone)]
pub struct Memory<K, V>
where
    K: Serializable,
    V: Serializable,
{
    pub data: Data<K, V>,
    pub idxs: Vec<K>,
}

impl<K: Serializable, V: Serializable> Memory<K, V> {
    fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
            idxs: Vec::new(),
        }
    }

    fn serialize(&self) -> Option<String> {
        serde_json::to_string(&self.data).ok()
    }
}

pub fn deserialize<K, V>(path: &'_ str) -> Option<Memory<K, V>>
where
    K: Serializable,
    V: Serializable,
{
    let contents = std::fs::read_to_string(path).ok()?.clone();
    serde_json::from_str::<Data<K, V>>(&contents)
        .map(|data| {
            let idxs = data.keys().cloned().collect::<Vec<_>>();
            Memory { data, idxs }
        })
        .ok()
}

pub trait MemoryManagement<K, V>
where
    K: Serializable,
    V: Serializable,
{
    fn lock_unwrap(&self) -> MutexGuard<'_, Memory<K, V>>;
    fn save(&self, to: String) -> Result<()>;
    fn add_item(&mut self, on: usize, item: V) -> Result<()>;
    fn delete_item(&mut self, on_topic: usize, on_item: usize) -> Result<V>;
    fn access_item(&mut self, on_topic: usize, on_item: usize) -> Result<V>;
    fn add_topic(&mut self, topic_id: impl Into<K>) -> Result<()>;
    fn delete_topic(&mut self, on: usize) -> Result<()>;
}
impl<K, V> MemoryManagement<K, V> for Arc<Mutex<Memory<K, V>>>
where
    K: Serializable + std::fmt::Debug,
    V: Serializable + std::fmt::Debug,
{
    fn lock_unwrap(&self) -> MutexGuard<'_, Memory<K, V>> {
        self.lock().expect("mutex lock is poisoned")
    }

    fn save(&self, to: String) -> Result<()> {
        let ctx = self.lock_unwrap();
        let mut f = File::create(to).map_err(MemoryError::IoError)?;
        let data = ctx.serialize().ok_or(MemoryError::Unknown)?;
        f.write_all(data.as_bytes()).map_err(MemoryError::IoError)
    }

    fn add_item(&mut self, on_topic: usize, item: V) -> Result<()> {
        let mut ctx = self.lock_unwrap();
        let idxs = ctx.idxs.clone();
        idxs.get(on_topic)
            .and_then(|topic| ctx.data.get_mut(topic))
            .map(|items| items.push(item))
            .ok_or(MemoryError::Unknown)
    }

    fn access_item(&mut self, on_topic: usize, on_item: usize) -> Result<V> {
        let mut ctx = self.lock_unwrap();
        let idxs = ctx.idxs.clone();
        idxs.get(on_topic)
            .and_then(|topic| ctx.data.get_mut(topic))
            .map(|item| item.get(on_item))
            .flatten()
            .ok_or(MemoryError::Unknown)
            .cloned()
    }

    fn delete_item(&mut self, on_topic: usize, on_item: usize) -> Result<V> {
        let mut ctx = self.lock_unwrap();
        let idxs = ctx.idxs.clone();
        idxs.get(on_topic)
            .and_then(|topic| ctx.data.get_mut(topic))
            .map(|item| item.remove(on_item)) // items displayed in order
            .ok_or(MemoryError::Unknown)
    }

    fn add_topic(&mut self, topic_id: impl Into<K>) -> Result<()> {
        let mut ctx = self.lock_unwrap();
        let topic = topic_id.into();
        // TODO:
        match ctx.data.keys().any(|topic_| *topic_ == topic) {
            true => Err(MemoryError::Unknown),
            _ => {
                ctx.idxs.push(topic.clone());
                if ctx.data.insert(topic, vec![]).is_none() {
                    return Ok(());
                } else {
                    Err(MemoryError::Unknown)
                }
            }
        }
    }

    fn delete_topic(&mut self, on: usize) -> Result<()> {
        let mut ctx = self.lock_unwrap();
        let mut idxs = ctx.idxs.clone();
        let topic = idxs.get(on);
        let res = topic
            .map(|topic_id| ctx.data.remove(topic_id))
            .map(|_| ())
            .ok_or(MemoryError::Unknown);
        if res.is_ok() {
            //#[allow(mutable_borrow_reservation_conflict)]
            idxs.remove(idxs.iter().position(|t| t == topic.unwrap()).unwrap());
            ctx.idxs = idxs;
        }

        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read() {
        let mem = deserialize::<String, String>(crate::config::DATA);
        dbg!(mem);
    }
}
