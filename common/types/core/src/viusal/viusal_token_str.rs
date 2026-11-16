use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct VisualTokenStr(String);

impl VisualTokenStr {
    pub fn new(encrypted: impl Into<String>) -> Self {
        Self(encrypted.into())
    }

    /// 取得加密字符串。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Deref for VisualTokenStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for VisualTokenStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for VisualTokenStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self)
    }
}

impl From<String> for VisualTokenStr {
    /// 从已加密字符串构造
    fn from(value: String) -> Self {
        Self(value)
    }
}
