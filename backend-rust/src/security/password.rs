use bcrypt::{hash, verify, DEFAULT_COST};

/// 密码校验接口，便于后续扩展实现。
pub trait PasswordVerifier {
    /// 校验明文密码与存储密码是否匹配。
    fn verify(&self, raw: &str, encoded: &str) -> anyhow::Result<bool>;
}

/// 使用 BCrypt 的密码校验器，兼容 Spring Security `{bcrypt}` 前缀。
#[derive(Clone)]
pub struct BcryptVerifier;

impl BcryptVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordVerifier for BcryptVerifier {
    fn verify(&self, raw: &str, mut encoded: &str) -> anyhow::Result<bool> {
        if encoded.is_empty() {
            return Ok(false);
        }
        // 去掉可选的 "{bcrypt}" 前缀，兼容 Java/Go 存储格式
        const PREFIX: &str = "{bcrypt}";
        if encoded.starts_with(PREFIX) {
            encoded = &encoded[PREFIX.len()..];
        }
        match verify(raw, encoded) {
            Ok(ok) => Ok(ok),
            Err(e) => {
                // 非匹配错误视为校验失败，其余错误向上抛出
                if e.to_string().contains("invalid password") {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
        }
    }
}

/// 简单封装一个哈希函数，供创建用户/重置密码使用。
/// 这里保持与 Java/Go 一致：使用 `{bcrypt}` 前缀。
pub fn bcrypt_hash(raw: &str) -> anyhow::Result<String> {
    if raw.trim().is_empty() {
        anyhow::bail!("密码不能为空");
    }
    let h = hash(raw, DEFAULT_COST)?;
    Ok(format!("{{bcrypt}}{}", h))
}

