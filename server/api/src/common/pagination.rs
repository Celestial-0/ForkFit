use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl PaginationParams {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> u64 {
        // Enforce maximum size of 100 per page to protect DB performance
        self.per_page.unwrap_or(10).max(1).min(100)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.per_page()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl PaginationMeta {
    pub fn new(page: u64, per_page: u64, total: u64) -> Self {
        let total_pages = if per_page == 0 {
            0
        } else {
            (total + per_page - 1) / per_page
        };

        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_defaults() {
        let params = PaginationParams { page: None, per_page: None };
        assert_eq!(params.page(), 1);
        assert_eq!(params.per_page(), 10);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_pagination_limits() {
        let params = PaginationParams { page: Some(0), per_page: Some(0) };
        assert_eq!(params.page(), 1);
        assert_eq!(params.per_page(), 1);
        assert_eq!(params.offset(), 0);

        let params_large = PaginationParams { page: Some(5), per_page: Some(500) };
        assert_eq!(params_large.page(), 5);
        assert_eq!(params_large.per_page(), 100);
        assert_eq!(params_large.offset(), 400);
    }

    #[test]
    fn test_pagination_meta_calculation() {
        let meta = PaginationMeta::new(1, 10, 25);
        assert_eq!(meta.page, 1);
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total, 25);
        assert_eq!(meta.total_pages, 3);

        let meta_empty = PaginationMeta::new(1, 10, 0);
        assert_eq!(meta_empty.total_pages, 0);
    }
}
