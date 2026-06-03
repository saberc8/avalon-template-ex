use crate::shared::error::AppError;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpAllowlistMode {
    Strict,
    Default,
    Open,
}

#[derive(Debug, Clone)]
pub struct HttpSafetyConfig {
    pub mode: HttpAllowlistMode,
    pub allowlist: Vec<String>,
}

impl Default for HttpSafetyConfig {
    fn default() -> Self {
        Self {
            mode: HttpAllowlistMode::Default,
            allowlist: Vec::new(),
        }
    }
}

pub fn validate_http_target(target: &str, config: &HttpSafetyConfig) -> Result<(), AppError> {
    let url = Url::parse(target).map_err(|_| AppError::bad_request("HTTP 任务 URL 格式不正确"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::bad_request("HTTP 任务只允许 http 或 https URL"));
    }

    if matches!(config.mode, HttpAllowlistMode::Open) {
        return Ok(());
    }

    let host = url
        .host_str()
        .ok_or_else(|| AppError::bad_request("HTTP 任务 URL 必须包含 Host"))?;
    let host = host.trim_matches(['[', ']']).to_ascii_lowercase();
    if matches!(config.mode, HttpAllowlistMode::Default) && is_default_allowed_host(&host) {
        return Ok(());
    }
    if config
        .allowlist
        .iter()
        .any(|allowed| host_matches(&host, allowed))
    {
        return Ok(());
    }

    Err(AppError::bad_request("HTTP 任务 URL 不在 allowlist 中"))
}

fn is_default_allowed_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

fn host_matches(host: &str, allowed: &str) -> bool {
    let allowed = allowed.trim().to_ascii_lowercase();
    if allowed.is_empty() {
        return false;
    }
    if let Some(suffix) = allowed.strip_prefix("*.") {
        return host.ends_with(&format!(".{suffix}"));
    }
    host == allowed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> HttpSafetyConfig {
        HttpSafetyConfig {
            mode: HttpAllowlistMode::Default,
            allowlist: vec!["api.example.com".to_owned()],
        }
    }

    #[test]
    fn default_mode_allows_localhost_and_configured_hosts() {
        let config = default_config();

        assert!(validate_http_target("http://localhost:4398/health", &config).is_ok());
        assert!(validate_http_target("https://api.example.com/jobs/run", &config).is_ok());
    }

    #[test]
    fn default_mode_rejects_unlisted_hosts() {
        let err =
            validate_http_target("https://evil.example.net/hit", &default_config()).unwrap_err();

        assert!(err.to_string().contains("allowlist"));
    }

    #[test]
    fn open_mode_allows_any_http_host_but_rejects_non_http_schemes() {
        let config = HttpSafetyConfig {
            mode: HttpAllowlistMode::Open,
            allowlist: Vec::new(),
        };

        assert!(validate_http_target("https://any.example.net/hit", &config).is_ok());
        assert!(validate_http_target("file:///etc/passwd", &config).is_err());
    }
}
