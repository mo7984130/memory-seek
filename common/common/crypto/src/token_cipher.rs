use std::sync::Arc;

use aes_siv::{KeyInit, siv::Aes256Siv};
use common_core::contextual::{ContextualResult, Reason, ext::ResultExt};
use hkdf::Hkdf;
use serde::Deserialize;
use sha2::Sha256;
use tracing::debug;

const HKDF_KEY_INFO: &[u8] = b"aes-256-siv-key";
const TOKEN_AAD: &[u8] = b"memory-seek-visual-token-v1";

pub struct TokenCipher {
    key: [u8; 64],
}

#[derive(Debug, Deserialize)]
pub struct TokenCipherConfig {
    pub key: String,
    pub salt: String,
}

pub type SharedTokenCipher = Arc<TokenCipher>;

impl TokenCipher {
    /// 从配置结构体创建 TokenCipher
    pub fn new(config: &TokenCipherConfig) -> Self {
        let key = Self::derive_key(config.key.as_bytes(), config.salt.as_bytes());

        Self { key }
    }

    /// 根据保存的密钥创建独立的 AES-SIV 实例
    fn cipher(&self) -> Aes256Siv {
        Aes256Siv::new_from_slice(&self.key).expect("AES-256-SIV requires a 64-byte key")
    }

    /// 确定性加密
    pub fn encrypt(&self, plaintext: &[u8]) -> ContextualResult<Vec<u8>> {
        let mut cipher = self.cipher();

        cipher
            .encrypt([TOKEN_AAD], plaintext)
            .contextual_error(Reason::EncryptFail, "AES-SIV 加密失败")
    }

    /// 解密
    pub fn decrypt(&self, ciphertext: &[u8]) -> Option<Vec<u8>> {
        let mut cipher = self.cipher();
        match cipher.decrypt([TOKEN_AAD], ciphertext) {
            Ok(v) => Some(v),
            Err(e) => {
                debug!("解密错误: {}", e);
                None
            }
        }
    }

    /// 使用 HKDF-SHA256 派生 AES-256-SIV 密钥
    fn derive_key(raw_key: &[u8], salt: &[u8]) -> [u8; 64] {
        let hk = Hkdf::<Sha256>::new(Some(salt), raw_key);

        let mut derived = [0u8; 64];

        hk.expand(HKDF_KEY_INFO, &mut derived)
            .expect("64-byte HKDF expand won't fail");

        derived
    }
}
