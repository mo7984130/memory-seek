#[derive(Debug, Clone, Default)]
pub enum ChangedValue<T> {
    #[default]
    Unchanged,
    Changed(T),
}
pub use ChangedValue::Changed;
pub use ChangedValue::Unchanged;

impl<T> ChangedValue<T> {
    /// 返回是否为已变更状态
    #[inline]
    pub fn is_changed(&self) -> bool {
        match self {
            Unchanged => false,
            Changed(_) => true,
        }
    }

    /// 返回是否为未变更状态
    #[inline]
    pub fn is_unchanged(&self) -> bool {
        match self {
            Unchanged => true,
            Changed(_) => false,
        }
    }
}
