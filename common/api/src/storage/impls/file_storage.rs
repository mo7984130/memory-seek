use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::RwLock,
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::storage::{Storage, StorageEntry, StorageError};

pub struct FileStorage {
    path: PathBuf,
    data: RwLock<HashMap<String, Value>>,
}

impl FileStorage {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();

        let data = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => HashMap::new(),
            Err(error) => return Err(error.into()),
        };

        Ok(Self {
            path,
            data: RwLock::new(data),
        })
    }

    fn save(&self, data: &HashMap<String, Value>) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(data)?;

        let parent = self.path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;

        let temp_path = self.path.with_extension("tmp");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        file.write_all(&bytes)?;
        file.sync_all()?;

        fs::rename(&temp_path, &self.path)?;

        Ok(())
    }
}

impl Storage for FileStorage {
    fn get<T>(&self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: DeserializeOwned,
    {
        let data = self.data.read().unwrap();

        data.get(key)
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }

    fn set<T>(&self, key: &str, value: T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(value)?;

        let mut data = self.data.write().unwrap();

        let old = data.insert(key.to_owned(), value);

        if let Err(error) = self.save(&data) {
            if let Some(old) = old {
                data.insert(key.to_owned(), old);
            } else {
                data.remove(key);
            }

            return Err(error);
        }

        Ok(())
    }

    fn set_many<I>(&self, entries: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = StorageEntry>,
    {
        let mut data = self.data.write().unwrap();
        let mut next = data.clone();

        for entry in entries {
            next.insert(entry.key, entry.value);
        }

        self.save(&next)?;
        *data = next;

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        let mut data = self.data.write().unwrap();

        let Some(old) = data.remove(key) else {
            return Ok(());
        };

        if let Err(error) = self.save(&data) {
            data.insert(key.to_owned(), old);
            return Err(error);
        }

        Ok(())
    }

    fn remove_many<I, K>(&self, keys: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        let keys = keys
            .into_iter()
            .map(|key| key.as_ref().to_owned())
            .collect::<Vec<_>>();

        let mut data = self.data.write().unwrap();
        let mut next = data.clone();

        let mut changed = false;

        for key in keys {
            changed |= next.remove(&key).is_some();
        }

        if !changed {
            return Ok(());
        }

        self.save(&next)?;
        *data = next;

        Ok(())
    }
}
