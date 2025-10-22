use engine::Pagination;
use engine::list_criteria::{ListCriteria, ListFilter, Tag};

const MAX_FILTER_COUNT: usize = 1;
pub type SetListCriteria = ListCriteria<SetFilter, MAX_FILTER_COUNT>;

pub enum SetFilter {
    Name(String),
}

impl ListFilter for SetFilter {
    const MAX_FILTER_COUNT: usize = MAX_FILTER_COUNT;
    type Criteria = ListCriteria<SetFilter, MAX_FILTER_COUNT>;

    fn tag(&self) -> Tag {
        match self {
            SetFilter::Name(_) => Tag::One,
        }
    }

    fn criteria(pagination: Pagination, default_page_size: u64) -> Self::Criteria {
        ListCriteria::new(pagination, default_page_size)
    }
}
