use serde::Deserialize;
use utoipa::ToSchema;

const fn default_page() -> usize {
    1
}

#[derive(Debug, Deserialize, ToSchema, PartialEq, Eq)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: usize,
    pub page_size: Option<usize>,
}
