use std::{collections::HashMap, path::Path};

use common_core::util::DeferredFile;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::storage::{Storage, StorageEntry, StorageError};

pub struct FileStorage {
    inner: DeferredFile<HashMap<String, Value>>,
}

impl FileStorage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Ok(Self {
            inner: DeferredFile::load(path)?,
        })
    }
}

impl Storage for FileStorage {
    fn get<T>(&self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: DeserializeOwned,
    {
        self.inner
            .with(|data| {
                data.get(key)
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()
            })
            .map_err(Into::into)
    }

    fn set<T>(&self, key: &str, value: T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(value)?;

        self.inner.update(|data| {
            data.insert(key.to_owned(), value);
        });

        Ok(())
    }

    fn set_many<I>(&self, entries: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = StorageEntry>,
    {
        self.inner.update(|data| {
            for entry in entries {
                data.insert(entry.key, entry.value);
            }
        });

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        self.inner.update(|data| {
            data.remove(key);
        });

        Ok(())
    }

    fn remove_many<I, K>(&self, keys: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.inner.update(|data| {
            for key in keys {
                data.remove(key.as_ref());
            }
        });

        Ok(())
    }

    fn flush(&self) -> Result<(), StorageError> {
        self.inner.save()
    }
}
