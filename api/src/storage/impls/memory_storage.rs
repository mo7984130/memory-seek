use std::{collections::HashMap, sync::RwLock};

use crate::storage::{Storage, StorageError};

#[derive(Default)]
pub struct MemoryStorage {
    map: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn get<T>(&self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: serde::de::DeserializeOwned,
    {
        let map = self.map.read().unwrap_or_else(|e| e.into_inner());

        map.get(key)
            .map(|bytes| serde_json::from_slice(bytes).map_err(Into::into))
            .transpose()
    }

    fn set<T>(&self, key: &str, value: T) -> Result<(), StorageError>
    where
        T: serde::Serialize,
    {
        let bytes = serde_json::to_vec(&value)?;

        let mut map = self.map.write().unwrap_or_else(|e| e.into_inner());

        map.insert(key.to_owned(), bytes);

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        let mut map = self.map.write().unwrap_or_else(|e| e.into_inner());

        map.remove(key);

        Ok(())
    }
}
