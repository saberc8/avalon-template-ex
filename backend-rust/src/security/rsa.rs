use anyhow::Context;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use rsa::pkcs8::DecodePrivateKey;
use rsa::RsaPrivateKey;

/// RSA 解密器，用于解密前端 RSA+Base64 加密的密码。
/// 私钥格式为 Base64 编码的 PKCS#8，与 Java/Hutool 配置保持一致。
#[derive(Clone)]
pub struct RsaDecryptor {
    inner: RsaPrivateKey,
}

impl RsaDecryptor {
    /// 从 Base64 字符串构造 RSA 解密器。
    pub fn from_base64(b64_key: &str) -> anyhow::Result<Self> {
        if b64_key.trim().is_empty() {
            anyhow::bail!("RSA 私钥为空");
        }
        let der = B64
            .decode(b64_key.trim())
            .context("解析 RSA 私钥 Base64 失败")?;
        let key = RsaPrivateKey::from_pkcs8_der(&der)
            .context("解析 PKCS#8 私钥失败")?;
        Ok(Self { inner: key })
    }

    /// 解密 Base64 编码的密文，返回明文字符串。
    pub fn decrypt_base64(&self, cipher_b64: &str) -> anyhow::Result<String> {
        use rsa::Pkcs1v15Encrypt;

        let cipher_bytes = B64
            .decode(cipher_b64.trim())
            .context("解析密码密文 Base64 失败")?;

        // 使用 PKCS#1 v1.5 解密，与 Java 默认配置兼容。
        let padding = Pkcs1v15Encrypt;
        let plain = self
            .inner
            .decrypt(padding, &cipher_bytes)
            .context("RSA 解密失败")?;
        String::from_utf8(plain).context("RSA 解密结果不是有效 UTF-8 字符串")
    }
}

