use serde::Deserialize;
use utoipa::ToSchema;

const fn default_page() -> u32 {
    1
}

#[derive(Debug, Deserialize, ToSchema, PartialEq, Eq)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,
    pub page_size: Option<u32>,
}
