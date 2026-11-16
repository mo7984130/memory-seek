use rand::{Rng, RngExt, rng};

/// 生成随机十六进制小写字符串；实际输出长度为 `len / 2 * 2`
#[inline]
pub fn generate_random_str(len: usize) -> String {
    let mut key = vec![0u8; len / 2];
    rng().fill_bytes(&mut key);
    hex::encode(key)
}

/// 生成随机大写字母与数字字符串(A-Z + 0-9)，适用于验证码、邀请码等场景
pub fn generate_random_uppercase_str(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..len)
        .map(|_| CHARSET[rng().random_range(0..CHARSET.len())] as char)
        .collect()
}
