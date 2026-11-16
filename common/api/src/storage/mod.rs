pub mod impls;

mod auth_storage;
pub use auth_storage::AuthStorage;

use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
}

pub struct StorageEntry {
    pub key: String,
    pub value: serde_json::Value,
}

impl StorageEntry {
    pub fn new(key: impl Into<String>, value: impl Serialize) -> Result<Self, StorageError> {
        Ok(Self {
            key: key.into(),
            value: serde_json::to_value(value)?,
        })
    }
}

pub trait Storage: Send + Sync + 'static {
    fn get<T>(&self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: DeserializeOwned;

    fn get_many<T>(&self, keys: &[&str]) -> Result<Vec<Option<T>>, StorageError>
    where
        T: DeserializeOwned,
    {
        keys.iter().map(|key| self.get(key)).collect()
    }

    fn set<T>(&self, key: &str, value: T) -> Result<(), StorageError>
    where
        T: Serialize;

    fn set_many<I>(&self, entries: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = StorageEntry>,
    {
        for entry in entries {
            self.set(&entry.key, entry.value)?;
        }

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), StorageError>;

    fn remove_many<I, K>(&self, keys: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        for key in keys {
            self.remove(key.as_ref())?;
        }

        Ok(())
    }
}
