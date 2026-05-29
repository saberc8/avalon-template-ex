use anyhow::{Context, Result};

const BCRYPT_PREFIX: &str = "{bcrypt}";

pub fn hash_password(raw_password: &str) -> Result<String> {
    let hash = bcrypt::hash(raw_password, bcrypt::DEFAULT_COST).context("hash bcrypt password")?;
    Ok(format!("{BCRYPT_PREFIX}{hash}"))
}

pub fn verify_password(raw_password: &str, encoded_password: &str) -> Result<bool> {
    let hash = encoded_password
        .strip_prefix(BCRYPT_PREFIX)
        .unwrap_or(encoded_password);

    bcrypt::verify(raw_password, hash).context("verify bcrypt password")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_prefixed_bcrypt_password() {
        let hash = "{bcrypt}$2a$10$4jGwK2BMJ7FgVR.mgwGodey8.xR8FLoU1XSXpxJ9nZQt.pufhasSa";
        assert!(verify_password("admin123", hash).unwrap());
    }

    #[test]
    fn hashes_with_existing_prefix() {
        let hash = hash_password("admin123").unwrap();

        assert!(hash.starts_with(BCRYPT_PREFIX));
        assert!(verify_password("admin123", &hash).unwrap());
    }
}
