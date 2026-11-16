use std::str::FromStr;

use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version};
use common_core::contextual::{
    ContextualError, ContextualResult, Reason,
    ext::{EmitIfErrExt, ResultExt},
};

#[cfg(feature = "tokio")]
/// 全进程共享的密码验证并发信号量:上限为可用并行数,防止 CPU 密集的哈希抢占 runtime。
static PASSWORD_VERIFY_SEM: std::sync::LazyLock<tokio::sync::Semaphore> =
    std::sync::LazyLock::new(|| {
        tokio::sync::Semaphore::new(
            std::thread::available_parallelism()
                .expect("获取可用并行数错误")
                .into(),
        )
    });

#[cfg(feature = "tokio")]
/// 获取全局密码验证并发信号量；调用方需在 CPU 密集的哈希前获取许可,drop 时释放
pub fn password_verify_semaphore() -> &'static tokio::sync::Semaphore {
    &PASSWORD_VERIFY_SEM
}

pub const PASSWORD_HASHER: PasswordHasher = PasswordHasher::Argon2id(Argon2idConfig {
    m_cost: 16 * 1024,
    t_cost: 2,
    p_cost: 1,
});

/// 哈希算法类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordHasher {
    Bcrypt(BcryptConfig),
    Argon2id(Argon2idConfig),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BcryptConfig {
    pub cost: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argon2idConfig {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl PasswordHasher {
    /// 从哈希字符串前缀识别密码哈希算法及其参数，无法识别时返回 `None`
    pub fn detect(hash: &str) -> Option<Self> {
        if hash.starts_with("$2") {
            let cost = bcrypt::HashParts::from_str(hash).ok()?.get_cost();
            Some(Self::Bcrypt(BcryptConfig { cost }))
        } else if hash.starts_with("$argon2") {
            let parsed = PasswordHash::new(hash).ok()?;
            let m_cost = parsed.params.get_decimal("m")?;
            let t_cost = parsed.params.get_decimal("t")?;
            let p_cost = parsed.params.get_decimal("p")?;
            Some(Self::Argon2id(Argon2idConfig {
                m_cost,
                t_cost,
                p_cost,
            }))
        } else {
            None
        }
    }

    /// 使用当前算法对明文密码进行哈希
    pub fn hash(&self, password: &str) -> ContextualResult<String> {
        match self {
            Self::Bcrypt(cfg) => bcrypt::hash(password, cfg.cost)
                .contextual_error(Reason::PasswordHasherError, "Bcrypt 计算失败"),
            Self::Argon2id(cfg) => {
                let hash = argon2::PasswordHasher::hash_password(
                    &Self::argon2_hasher(cfg)?,
                    password.as_bytes(),
                )
                .contextual_error(Reason::PasswordHasherError, "Argon2id 计算失败")?
                .to_string();
                Ok(hash)
            }
        }
    }

    /// 验证明文密码与哈希值是否匹配
    pub fn verify(&self, password: &str, hash: &str) -> ContextualResult<bool> {
        match self {
            Self::Bcrypt(_) => bcrypt::verify(password, hash)
                .contextual_error(Reason::PasswordHasherError, "Bcrypt 密码验证失败"),
            Self::Argon2id(cfg) => {
                let hasher = Self::argon2_hasher(cfg)?;
                let parsed = PasswordHash::new(hash)
                    .contextual_error(Reason::PasswordHasherError, "解析 Argon2 哈希失败")?;
                match hasher.verify_password(password.as_bytes(), &parsed) {
                    Ok(()) => Ok(true),
                    Err(password_hash::Error::PasswordInvalid) => Ok(false),
                    Err(error) => Err(ContextualError::error(
                        Reason::PasswordHasherError,
                        "Argon2 密码验证失败",
                        error,
                    )),
                }
            }
        }
    }

    /// 自动检测哈希算法并验证密码是否匹配，返回是否匹配与识别到的算法
    pub fn verify_and_detect(
        password: &str,
        hash: &str,
    ) -> ContextualResult<(bool, PasswordHasher)> {
        match PasswordHasher::detect(hash) {
            Some(alg) => {
                let result = alg.verify(password, hash)?;
                Ok((result, alg))
            }
            None => Err(ContextualError::error_without_source(
                Reason::PasswordHasherError,
                "密码算法检测失败",
            )),
        }
    }

    /// 根据配置创建 Argon2id 哈希器实例
    fn argon2_hasher(cfg: &Argon2idConfig) -> ContextualResult<Argon2<'static>> {
        let params = Params::new(cfg.m_cost, cfg.t_cost, cfg.p_cost, None)
            .contextual_error(Reason::PasswordHasherError, "创建 Argon2 参数失败")?;
        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }

    /// 执行恒定耗时的假验证以抵御基于时序的用户枚举攻击；用户不存在时调用，使响应耗时与密码错误时一致
    pub fn dummy_verify() {
        // 与真实验证路径(constants::PasswordHasher: argon2id m=16384, t=2, p=1)对齐,
        // 对假密码做一次实际哈希, 使耗时与“验证真实存在用户密码”一致。
        // 注意: 不能改用更慢/更快的算法(如 bcrypt cost 12),
        // 否则时序差异反而暴露账号是否存在。
        Self::Argon2id(Argon2idConfig {
            m_cost: 16 * 1024,
            t_cost: 2,
            p_cost: 1,
        })
        .hash("dummy")
        .emit_if_err();
    }
}
