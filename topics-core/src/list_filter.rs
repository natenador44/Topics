use engine::Pagination;
use engine::list_criteria::{ListCriteria, ListFilter, Tag};

pub enum TopicFilter {
    Name(String),
}

impl ListFilter for TopicFilter {
    const MAX_FILTER_COUNT: usize = MAX_FILTER_COUNT;
    type Criteria = TopicListCriteria;

    fn tag(&self) -> Tag {
        match self {
            TopicFilter::Name(_) => Tag::One,
        }
    }

    fn criteria(pagination: Pagination, default_page_size: u64) -> Self::Criteria {
        TopicListCriteria::new(pagination, default_page_size)
    }
}

pub type TopicListCriteria = ListCriteria<TopicFilter, MAX_FILTER_COUNT>;

const MAX_FILTER_COUNT: usize = 1;
