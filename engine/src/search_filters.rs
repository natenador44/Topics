use crate::models::IdentifierId;
use crate::pagination::Pagination;
use crate::search_criteria::{SearchCriteria, SearchFilter, Tag};

const TOPIC_SEARCH_FILTER_MAX: usize = 2;
pub type TopicSearchCriteria = SearchCriteria<TopicFilter, { TOPIC_SEARCH_FILTER_MAX }>;

#[derive(Debug, PartialEq, Eq)]
pub enum TopicFilter {
    Name(String),
    Description(String),
}

impl SearchFilter for TopicFilter {
    const MAX_FILTER_COUNT: usize = TOPIC_SEARCH_FILTER_MAX;
    type Criteria = TopicSearchCriteria;

    fn tag(&self) -> Tag {
        match self {
            TopicFilter::Name(_) => Tag::One,
            TopicFilter::Description(_) => Tag::Two,
        }
    }

    fn criteria(pagination: Pagination, default_page_size: u32) -> Self::Criteria {
        SearchCriteria::new(pagination, default_page_size)
    }
}

const MAX_SEARCH_FILTER_COUNT: usize = 3;
pub type SetSearchCriteria = SearchCriteria<SetFilter, MAX_SEARCH_FILTER_COUNT>;

pub enum SetFilter {
    Name(String),
    EntityText(String),
    Identifiers(Vec<IdentifierId>),
}

impl SearchFilter for SetFilter {
    const MAX_FILTER_COUNT: usize = MAX_SEARCH_FILTER_COUNT;
    type Criteria = SetSearchCriteria;

    fn tag(&self) -> Tag {
        match self {
            SetFilter::Name(_) => Tag::One,
            SetFilter::EntityText(_) => Tag::Two,
            SetFilter::Identifiers(_) => Tag::Four,
        }
    }

    fn criteria(pagination: Pagination, default_page_size: u32) -> Self::Criteria {
        SearchCriteria::new(pagination, default_page_size)
    }
}
