mod miss_dep_ext;
pub use miss_dep_ext::{GetDepExt, MissDepExt};

mod app_state;
pub use app_state::FromRegistry;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct TypeRegistry {
    values: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// 以值的类型为键插入或覆盖条目
    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        self.values.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// 按类型取出值的引用，不存在或类型不符时返回 `None`
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub fn get_cloned<T: Any + Send + Sync + Clone>(&self) -> Option<T> {
        self.get::<T>().cloned()
    }
}
