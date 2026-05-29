use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use tokio::{fs, io::AsyncWriteExt};

use crate::shared::error::AppError;

static TEST_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn default_root() -> PathBuf {
    TEST_ROOT
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("data/file"))
}

pub async fn save_bytes(
    root: impl AsRef<Path>,
    parent_path: &str,
    name: &str,
    bytes: &[u8],
) -> Result<String, AppError> {
    let relative_parent = normalized_parent_path(parent_path);
    let dir = root.as_ref().join(relative_parent.trim_start_matches('/'));
    fs::create_dir_all(&dir).await?;
    let full_path = dir.join(name);
    let mut file = fs::File::create(&full_path).await?;
    file.write_all(bytes).await?;

    Ok(join_logical_path(&relative_parent, name))
}

pub fn normalized_parent_path(value: &str) -> String {
    let mut value = value.trim().replace('\\', "/");
    if value.is_empty() {
        return "/".to_owned();
    }
    if !value.starts_with('/') {
        value.insert(0, '/');
    }
    while value.len() > 1 && value.ends_with('/') {
        value.pop();
    }
    value
}

pub fn join_logical_path(parent: &str, name: &str) -> String {
    let parent = normalized_parent_path(parent);
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}

#[cfg(test)]
pub fn set_test_root(path: PathBuf) {
    let _ = TEST_ROOT.set(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_path_is_normalized_for_local_storage() {
        assert_eq!(normalized_parent_path(""), "/");
        assert_eq!(normalized_parent_path("a/b/"), "/a/b");
        assert_eq!(join_logical_path("/a", "b.txt"), "/a/b.txt");
    }
}
