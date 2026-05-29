use serde::{Deserialize, Serialize};

pub const DEFAULT_PAGE: u64 = 1;
pub const DEFAULT_PAGE_SIZE: u64 = 10;
pub const MAX_PAGE_SIZE: u64 = 100;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
}

impl PageQuery {
    pub fn offset(self) -> i64 {
        let query = self.normalized();
        let offset = query
            .page
            .saturating_sub(1)
            .saturating_mul(query.size)
            .min(i64::MAX as u64);

        offset as i64
    }

    pub fn limit(self) -> i64 {
        self.normalized().size as i64
    }

    pub fn normalized(self) -> Self {
        let page = if self.page == 0 {
            DEFAULT_PAGE
        } else {
            self.page
        };
        let size = if self.size == 0 {
            DEFAULT_PAGE_SIZE
        } else {
            self.size.min(MAX_PAGE_SIZE)
        };

        Self { page, size }
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
    DEFAULT_PAGE
}

fn default_size() -> u64 {
    DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_page_and_size_are_normalized() {
        let query = PageQuery { page: 0, size: 0 };

        assert_eq!(query.offset(), 0);
        assert_eq!(query.limit(), default_size() as i64);
    }

    #[test]
    fn huge_size_is_clamped() {
        let query = PageQuery {
            page: 1,
            size: u64::MAX,
        };

        assert_eq!(query.limit(), 100);
    }

    #[test]
    fn huge_page_uses_overflow_resistant_offset() {
        let query = PageQuery {
            page: u64::MAX,
            size: u64::MAX,
        };

        assert_eq!(query.offset(), i64::MAX);
    }
}
