use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use common_core::{
    contextual::{ContextualResult, Reason, ext::ResultExt},
    ext::result::ToOkExt,
};
use common_crypto::token_cipher::TokenCipher;
use serde::{Deserialize, Serialize};
use type_core::{UserId, viusal::VisualTokenStr};

#[derive(Serialize, Deserialize)]
pub enum ImageTokenType {
    Thumbnail,
    Preview,
    Original,
}

#[derive(Serialize, Deserialize)]
pub enum VideoTokenType {
    Thumbnail,
    Preview,
    Original,
}

#[derive(Serialize, Deserialize)]
pub enum VisualTokenKind {
    Image(ImageTokenType),
    Visual(ImageTokenType),
}

#[derive(Serialize, Deserialize)]
pub struct VisualToken {
    file_id: String,
    kind: VisualTokenKind,
    pub viewer_id: UserId,
}

impl VisualToken {
    pub fn encrypt(self, token_cipher: &TokenCipher) -> ContextualResult<VisualTokenStr> {
        let bytes =
            postcard::to_allocvec(&self).contextual_error(Reason::EncryptFail, "加密错误")?;
        let visual_token = token_cipher.encrypt(&bytes)?;
        VisualTokenStr::new(URL_SAFE_NO_PAD.encode(visual_token)).to_ok()
    }

    /// 解密外部不可信令牌；任何失败都返回 None
    pub fn decrypt(token_cipher: &TokenCipher, token_str: VisualTokenStr) -> Option<VisualToken> {
        let encrypted_str = URL_SAFE_NO_PAD.decode(token_str.as_str()).ok()?;
        let bytes = token_cipher.decrypt(&encrypted_str)?;
        postcard::from_bytes(&bytes).ok()
    }

    pub fn image_thumbnail(file_id: String, viewer_id: UserId) -> VisualToken {
        Self {
            file_id,
            kind: VisualTokenKind::Image(ImageTokenType::Thumbnail),
            viewer_id,
        }
    }
}
