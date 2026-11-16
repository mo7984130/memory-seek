use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPage<T, C> {
    pub records: Vec<T>,
    pub next_cursor: Option<C>,
    pub has_more: bool,
}

impl<T, C> CursorPage<T, C> {
    /// 创建空分页结果
    #[inline]
    pub fn empty() -> Self {
        Self {
            records: vec![],
            next_cursor: None,
            has_more: false,
        }
    }

    /// 替换分页记录，保留游标与分页状态
    #[inline]
    pub fn replace_records<U>(self, records: Vec<U>) -> CursorPage<U, C> {
        CursorPage {
            records,
            next_cursor: self.next_cursor,
            has_more: self.has_more,
        }
    }

    /// 映射分页记录，保留游标与分页状态
    #[inline]
    pub fn map_records<U, F>(self, map_records: F) -> CursorPage<U, C>
    where
        F: FnOnce(Vec<T>) -> Vec<U>,
    {
        let CursorPage {
            records,
            next_cursor,
            has_more,
        } = self;
        CursorPage {
            records: map_records(records),
            next_cursor,
            has_more,
        }
    }

    /// 仅在存在下一页时用最后一条记录生成下一页游标
    #[inline]
    pub fn with_next_cursor<C2, F>(self, get_cursor: F) -> CursorPage<T, C2>
    where
        F: FnOnce(&T) -> C2,
    {
        let next_cursor = if self.has_more {
            self.records.last().map(get_cursor)
        } else {
            None
        };
        CursorPage {
            records: self.records,
            next_cursor,
            has_more: self.has_more,
        }
    }

    /// 附加记录总数，供确实需要总数的接口使用。
    #[inline]
    pub fn with_total(self, total_count: u64) -> CursorPageWithTotal<T, C> {
        CursorPageWithTotal {
            page: self,
            total_count: Some(total_count),
        }
    }

    /// 包装为带总数的分页；`total_count` 为 `None` 表示本页不携带总数。
    #[inline]
    pub fn with_optional_total(self, total_count: Option<u64>) -> CursorPageWithTotal<T, C> {
        CursorPageWithTotal {
            page: self,
            total_count,
        }
    }
}

impl<T> CursorPage<T, ()> {
    /// 从多取一条的查询结果构造分页并截断超出页大小的记录
    #[inline]
    pub fn from_oversize(mut records: Vec<T>, size: u64) -> Self {
        let size = size as usize;
        let has_more = records.len() > size;
        if has_more {
            records.truncate(size);
        }
        Self {
            records,
            next_cursor: None,
            has_more,
        }
    }

    /// 从记录构造分页；`has_more` 为 false，`next_cursor` 为 `None`
    #[inline]
    pub fn from_records(records: Vec<T>) -> Self {
        Self {
            records,
            next_cursor: None,
            has_more: false,
        }
    }

    /// 从记录与是否还有下一页标志构造分页；`next_cursor` 为 `None`
    #[inline]
    pub fn from_has_more(records: Vec<T>, has_more: bool) -> Self {
        Self {
            records,
            next_cursor: None,
            has_more,
        }
    }
}

/// 带记录总数的游标分页。
///
/// `total_count` 只在首屏（无游标）时计算；后续页为 `None` 并在序列化时省略。
#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPageWithTotal<T, C> {
    pub page: CursorPage<T, C>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u64>,
}

impl<T, C> CursorPageWithTotal<T, C> {
    /// 从分页与总数构造结果
    #[inline]
    pub fn new(page: CursorPage<T, C>, total_count: Option<u64>) -> Self {
        Self { page, total_count }
    }

    /// 创建空分页结果，保留本页是否携带总数
    #[inline]
    pub fn empty(total_count: Option<u64>) -> Self {
        Self {
            page: CursorPage::empty(),
            total_count,
        }
    }

    /// 替换分页记录，保留游标、分页状态与总数
    #[inline]
    pub fn replace_records<U>(self, records: Vec<U>) -> CursorPageWithTotal<U, C> {
        CursorPageWithTotal {
            page: self.page.replace_records(records),
            total_count: self.total_count,
        }
    }

    /// 映射分页记录，保留游标、分页状态与总数
    #[inline]
    pub fn map_records<U, F>(self, map_records: F) -> CursorPageWithTotal<U, C>
    where
        F: FnOnce(Vec<T>) -> Vec<U>,
    {
        CursorPageWithTotal {
            page: self.page.map_records(map_records),
            total_count: self.total_count,
        }
    }

    /// 仅在存在下一页时用最后一条记录生成下一页游标
    #[inline]
    pub fn with_next_cursor<C2, F>(self, get_cursor: F) -> CursorPageWithTotal<T, C2>
    where
        F: FnOnce(&T) -> C2,
    {
        CursorPageWithTotal {
            page: self.page.with_next_cursor(get_cursor),
            total_count: self.total_count,
        }
    }
}
