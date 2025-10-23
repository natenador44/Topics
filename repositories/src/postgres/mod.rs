// #[cfg(feature = "postgres-topics")]
pub mod initializer;
pub mod sets;
mod statements;
pub mod topics;

use engine::list_criteria::ListCriteria;
use error_stack::Report;
use std::error::Error;

macro_rules! validate_pagination_field {
    ($field_name:literal, $field:expr; $e:expr) => {{
        use error_stack::{IntoReport, ResultExt};

        if $field > i64::MAX as u64 {
            return Err(IntoReport::into_report($e)).attach_with(|| {
                format!(
                    "{} '{}' is too large and is not supported",
                    $field_name, $field
                )
            });
        } else {
            $field as i64
        }
    }};

    ($field_name:literal, $field:expr => $map:expr; $e:expr) => {{
        use error_stack::{IntoReport, ResultExt};
        if $field > i64::MAX as u64 {
            return Err(IntoReport::into_report($e)).attach_with(|| {
                format!(
                    "{} '{}' is too large and is not supported",
                    $field_name, $field
                )
            });
        } else {
            $map as i64
        }
    }};
}

use validate_pagination_field;

struct SanitizedPagination {
    pub page: i64,
    pub page_size: i64,
}

fn sanitize_pagination<const N: usize, T, E: Error + Send + Sync + Copy + 'static>(
    list_criteria: &ListCriteria<T, N>,
    sanitation_err: E,
) -> Result<SanitizedPagination, Report<E>> {
    let page = validate_pagination_field!("page", list_criteria.page() => list_criteria.page().saturating_sub(1); sanitation_err);
    let page_size =
        validate_pagination_field!("page_size", list_criteria.page_size(); sanitation_err);

    Ok(SanitizedPagination { page, page_size })
}

pub enum ConnectionDetails {
    Url(String),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize postgres {0} repo")]
pub struct RepoInitErr(&'static str);
impl RepoInitErr {
    fn topics() -> Self {
        Self("topics")
    }

    fn sets() -> Self {
        Self("sets")
    }

    fn all() -> Self {
        Self("all")
    }
}
