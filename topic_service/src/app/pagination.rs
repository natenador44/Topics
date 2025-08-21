use serde::{Deserialize, Deserializer, de::Visitor};
use utoipa::ToSchema;

const fn default_page() -> usize {
    1
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct Pagination {
    #[serde(default = "default_page")]
    // #[serde(deserialize_with = "parse_page")]
    pub page: usize,
    pub page_size: Option<usize>,
}

fn parse_page<'de, D>(d: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(PageVisitor)
}

struct PageVisitor;

impl<'de> Visitor<'de> for PageVisitor {
    type Value = usize;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Expected a non-zero integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(|e| serde::de::Error::custom(e))
    }
}
