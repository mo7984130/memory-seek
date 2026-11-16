use std::str::FromStr;

use http::Uri;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// API 基础 URL,例如 `http://127.0.0.1:8080/api`。
    ///
    /// 末尾斜杠会被自动裁剪,再与接口路径拼接。
    base_url: String,
}
impl ApiConfig {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, http::uri::InvalidUri> {
        let url = base_url.as_ref().trim_end_matches('/');
        Uri::from_str(url)?;

        Ok(Self {
            base_url: url.to_owned(),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.memory-seek.driftcloud.cn".to_owned(),
        }
    }
}
