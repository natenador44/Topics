use serde::Deserialize;
use utoipa::ToSchema;

const fn default_page() -> u64 {
    1
}

#[derive(Debug, Deserialize, ToSchema, PartialEq, Eq)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u64,
    pub page_size: Option<u64>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: Default::default(),
            page_size: Default::default(),
        }
    }
}

impl Pagination {
    pub fn with_default_page_size(page: u64) -> Self {
        Self {
            page,
            page_size: None,
        }
    }

    pub fn with_page_size(page: u64, page_size: u64) -> Self {
        Self {
            page,
            page_size: Some(page_size),
        }
    }
}
