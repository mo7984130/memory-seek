use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        PoisonError, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug)]
pub enum DeferredFileError {
    Io(io::Error),
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    Poisoned,
}

impl fmt::Display for DeferredFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeferredFileError::Io(_) => write!(f, "IO 错误"),
            DeferredFileError::Serialize(_) => write!(f, "序列化错误"),
            DeferredFileError::Deserialize(_) => write!(f, "反序列化错误"),
            DeferredFileError::Poisoned => write!(f, "锁被污染"),
        }
    }
}

impl std::error::Error for DeferredFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DeferredFileError::Io(e) => Some(e),
            DeferredFileError::Serialize(e) | DeferredFileError::Deserialize(e) => Some(e),
            DeferredFileError::Poisoned => None,
        }
    }
}

impl From<io::Error> for DeferredFileError {
    fn from(e: io::Error) -> Self {
        DeferredFileError::Io(e)
    }
}
impl From<serde_json::Error> for DeferredFileError {
    fn from(e: serde_json::Error) -> Self {
        DeferredFileError::Serialize(e)
    }
}
impl<T> From<PoisonError<T>> for DeferredFileError {
    fn from(_: PoisonError<T>) -> Self {
        Self::Poisoned
    }
}

pub struct DeferredFile<T: Serialize + DeserializeOwned + Default> {
    path: PathBuf,
    data: RwLock<T>,
    dirty: AtomicBool,
}

impl<T: Serialize + DeserializeOwned + Default> DeferredFile<T> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DeferredFileError> {
        let path = path.as_ref().to_path_buf();
        let data = match fs::read(&path) {
            Ok(bytes) => {
                if bytes.is_empty() {
                    T::default()
                } else {
                    serde_json::from_slice(&bytes)?
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => T::default(),
            Err(e) => return Err(DeferredFileError::Io(e)),
        };
        Ok(Self {
            path,
            data: RwLock::new(data),
            dirty: AtomicBool::new(false),
        })
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.data.read().unwrap();
        f(&guard)
    }

    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let r = {
            let mut guard = self.data.write().unwrap();
            f(&mut guard)
        };
        self.dirty.store(true, Ordering::Release);
        r
    }

    pub fn save(&self) -> Result<(), DeferredFileError> {
        if !self.dirty.swap(false, Ordering::AcqRel) {
            return Ok(());
        }

        let result = self.do_save();

        if result.is_err() {
            self.dirty.store(true, Ordering::Release);
        }

        result
    }

    fn do_save(&self) -> Result<(), DeferredFileError> {
        let bytes = {
            let guard = self.data.read()?;
            serde_json::to_vec_pretty(&*guard)?
        };

        let parent = self.path.parent().unwrap_or(Path::new("."));

        fs::create_dir_all(parent)?;

        let tmp = self.path.with_extension("tmp");

        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }

        fs::rename(&tmp, &self.path)?;

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }
}

impl<T: Serialize + DeserializeOwned + Default> Drop for DeferredFile<T> {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            tracing::error!("保存 {} 失败: {e}", self.path.display());
        }
    }
}
