use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
}

impl PageQuery {
    pub fn offset(self) -> i64 {
        ((self.page.saturating_sub(1)) * self.size) as i64
    }

    pub fn limit(self) -> i64 {
        self.size as i64
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageResult<T> {
    pub list: Vec<T>,
    pub total: i64,
}

impl<T> PageResult<T> {
    pub fn new(list: Vec<T>, total: i64) -> Self {
        Self { list, total }
    }
}

fn default_page() -> u64 {
    1
}

fn default_size() -> u64 {
    10
}
