use crate::pagination::Pagination;
use const_format::formatcp;
use std::mem::MaybeUninit;

type MaxFilterCountType = u8;
const MAX: u8 = u8::MAX;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Tag {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
    ThirtyTwo = 32,
    SixtyFour = 64,
    OneTwentyEight = 128,
}

impl Tag {
    const NONE: MaxFilterCountType = 0;
}

pub trait SearchFilter {
    const MAX_FILTER_COUNT: usize;
    type Criteria;
    fn tag(&self) -> Tag;
    fn criteria(pagination: Pagination, default_page_size: u64) -> Self::Criteria;
}

/*
is this better than just a Vec<Filter<T>>
where Filter is
enum Filter<T> { Pagination(Pagination), Other(T) }?

That doesn't let us prevent duplicate filters being applied, and adds
additional checks when going through the filters. It is a lot simpler though, and takes up
less space on the heap

How about HashSet?
 */
/// `N` cannot be larger than u8::MAX
/// ```compile_fail
/// enum TestFilter { Test1 }
/// let _ = SearchCriteria::<TestFilter, 256>::new(Pagination { page: 1, page_size: None }, 0);

#[derive(Debug, PartialEq, Eq)]
pub struct ListCriteria<T, const N: usize> {
    inner: Box<SearchCriteriaInner<T, N>>,
}

impl<T, const N: usize> ListCriteria<T, N> {
    pub fn new(pagination: Pagination, default_page_size: u64) -> Self {
        const {
            assert!(
                N <= MaxFilterCountType::MAX as usize,
                "{}",
                formatcp!("SearchCriteria only supports sizes up to {}", MAX)
            )
        };
        Self {
            inner: Box::new(SearchCriteriaInner {
                filters: None,
                pagination,
                default_page_size,
            }),
        }
    }

    pub fn page(&self) -> u64 {
        self.inner.pagination.page
    }

    pub fn page_size(&self) -> u64 {
        self.inner
            .pagination
            .page_size
            .unwrap_or(self.inner.default_page_size)
    }

    pub fn filters(&self) -> Option<&[T]> {
        self.inner.filters.as_ref().map(|f| f.get())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SearchCriteriaInner<T, const N: usize> {
    filters: Option<SearchCriteriaFilters<T, N>>,
    pagination: Pagination,
    default_page_size: u64,
}

impl<T, const N: usize> ListCriteria<T, N>
where
    T: SearchFilter,
{
    pub fn add(&mut self, filter: T) -> &mut Self {
        let filters = self
            .inner
            .filters
            .get_or_insert_with(SearchCriteriaFilters::new);

        let tag = filter.tag() as MaxFilterCountType;

        if tag & filters.applied == 0 {
            filters.applied |= tag;
            filters.filters[filters.applied_count as usize].write(filter);
            filters.applied_count += 1;
        }

        self
    }

    pub fn with(mut self, filter: T) -> Self {
        self.add(filter);
        self
    }
}

#[derive(Debug)]
struct SearchCriteriaFilters<T, const N: usize> {
    filters: [MaybeUninit<T>; N],
    applied_count: MaxFilterCountType,
    applied: MaxFilterCountType,
}

impl<T, const N: usize> PartialEq for SearchCriteriaFilters<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.applied == other.applied
            && self.applied_count == other.applied_count
            && self.get() == other.get()
    }
}

impl<T, const N: usize> Eq for SearchCriteriaFilters<T, N> where T: Eq {}

impl<T, const N: usize> SearchCriteriaFilters<T, N> {
    fn new() -> Self {
        Self {
            filters: [const { MaybeUninit::uninit() }; N],
            applied_count: 0,
            applied: Tag::NONE,
        }
    }

    fn get(&self) -> &[T] {
        unsafe { std::mem::transmute(&self.filters[..self.applied_count as usize]) }
    }
}

impl<T, const N: usize> Drop for SearchCriteriaFilters<T, N> {
    fn drop(&mut self) {
        for i in 0..self.applied_count {
            // SAFETY we've kept track of the number of filters applied, stored from left to right
            // in the array.
            unsafe { self.filters[i as usize].assume_init_drop() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[derive(Copy, Clone, PartialEq, Debug, Eq)]
    enum TestSearch {
        Test1,
        Test2,
        Test3,
    }

    impl SearchFilter for TestSearch {
        const MAX_FILTER_COUNT: usize = 3;
        type Criteria = ListCriteria<Self, { Self::MAX_FILTER_COUNT }>;

        fn tag(&self) -> Tag {
            match self {
                TestSearch::Test1 => Tag::One,
                TestSearch::Test2 => Tag::Two,
                TestSearch::Test3 => Tag::Four,
            }
        }

        fn criteria(pagination: Pagination, default_page_size: u64) -> Self::Criteria {
            ListCriteria::new(pagination, default_page_size)
        }
    }

    #[test]
    fn each_filter_can_only_be_applied_once() {
        let mut criteria = TestSearch::criteria(
            Pagination {
                page: 1,
                page_size: None,
            },
            0,
        );
        for _ in 0..10 {
            criteria.add(TestSearch::Test1);
        }

        for _ in 0..10 {
            criteria.add(TestSearch::Test2);
        }

        for _ in 0..10 {
            criteria.add(TestSearch::Test3);
        }

        assert_eq!(3, criteria.filters().unwrap().len());
        assert_eq!(TestSearch::Test1, criteria.filters().unwrap()[0]);
        assert_eq!(TestSearch::Test2, criteria.filters().unwrap()[1]);
        assert_eq!(TestSearch::Test3, criteria.filters().unwrap()[2]);
    }

    #[test]
    fn search_criteria_size_no_larger_than_hash_set() {
        let hash_set_size = size_of::<HashSet<TestSearch>>();
        let criteria_size = size_of::<ListCriteria<TestSearch, 3>>();
        assert!(
            criteria_size <= hash_set_size,
            "{} <= {} failed",
            criteria_size,
            hash_set_size
        );
    }

    #[test]
    fn search_criteria_size_no_larger_than_vec() {
        let vec_size = size_of::<Vec<TestSearch>>();
        let criteria_size = size_of::<ListCriteria<TestSearch, 3>>();
        assert!(
            criteria_size <= vec_size,
            "{} <= {} failed",
            criteria_size,
            vec_size
        );
    }
}
