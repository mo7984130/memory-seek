use std::any::Any;

use crate::{
    contextual::{ContextualError, ContextualResult},
    type_registry::TypeRegistry,
};

pub trait MissDepExt<T> {
    /// 取出依赖值，缺失时返回内部错误
    #[track_caller]
    fn miss_dep(self, requester_name: &'static str, dep_name: &'static str) -> ContextualResult<T>;
}
impl<T: Any + Send + Sync + Clone> MissDepExt<T> for Option<T> {
    #[inline]
    #[track_caller]
    fn miss_dep(self, requester_name: &'static str, dep_name: &'static str) -> ContextualResult<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(ContextualError::error_without_source(
                crate::contextual::Reason::BuildFail,
                format!("{} 需要 {}, 但缺失", requester_name, dep_name),
            )),
        }
    }
}

pub trait GetDepExt<T> {
    #[track_caller]
    fn get_dep(&self, requester_name: &'static str, dep_name: &'static str) -> ContextualResult<T>;
}
impl<T: Any + Send + Sync + Clone> GetDepExt<T> for TypeRegistry {
    #[inline]
    #[track_caller]
    fn get_dep(&self, requester_name: &'static str, dep_name: &'static str) -> ContextualResult<T> {
        self.get_cloned().miss_dep(requester_name, dep_name)
    }
}
