use std::mem::MaybeUninit;
use const_format::formatcp;
use crate::app::pagination::Pagination;

type MAX_FILTER_COUNT_TYPE = u8;
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
    const NONE: MAX_FILTER_COUNT_TYPE = 0;
}

pub trait SearchFilter {
    type Criteria;
    fn tag(&self) -> Tag;
    fn criteria(pagination: Pagination) -> Self::Criteria;
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
pub struct SearchCriteria<T, const N: usize> {
    inner: Box<SearchCriteriaInner<T, N>>,
}

impl<T, const N: usize> SearchCriteria<T, N> {
    pub fn new(pagination: Pagination, default_page_size: usize) -> Self {
        const { assert!(N <= MAX_FILTER_COUNT_TYPE::MAX as usize, "{}", formatcp!("SearchCriteria only supports sizes up to {}", MAX)) };
        Self {
            inner: Box::new(SearchCriteriaInner {
                filters: [ const { MaybeUninit::uninit() }; N],
                applied_count: 0,
                applied: Tag::NONE,
                pagination,
                default_page_size,
            })
        }
    }

    pub fn page(&self) -> usize {
        self.inner.pagination.page
    }

    pub fn page_size(&self) -> usize {
        self.inner.pagination.page_size.unwrap_or(self.inner.default_page_size)
    }

    pub fn filters(&self) -> &[T] {
        unsafe {
            std::mem::transmute(&self.inner.filters[..self.inner.applied_count as usize])
        }
    }
}

impl<T, const N: usize> SearchCriteria<T, N>
where T: SearchFilter,
{
    pub fn add(&mut self, filter: T) -> &mut Self {
        let tag = filter.tag() as MAX_FILTER_COUNT_TYPE;
        if tag & self.inner.applied == 0 {
            self.inner.applied |= tag;
            self.inner.filters[self.inner.applied_count as usize].write(filter);
            self.inner.applied_count += 1;
        }

        self
    }
}

struct SearchCriteriaInner<T, const N: usize> {
    filters: [MaybeUninit<T>; N],
    applied_count: MAX_FILTER_COUNT_TYPE,
    applied: MAX_FILTER_COUNT_TYPE,
    pagination: Pagination,
    default_page_size: usize,
}

impl<T, const N: usize> Drop for SearchCriteriaInner<T, N> {
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
    use std::collections::HashSet;
    use super::*;

    #[derive(Copy, Clone, PartialEq, Debug, Eq)]
    enum TestFilter {
        Test1, Test2, Test3
    }

    impl SearchFilter for TestFilter {
        type Criteria = SearchCriteria<Self, 3>;

        fn tag(&self) -> Tag {
            match self {
                TestFilter::Test1 => Tag::One,
                TestFilter::Test2 => Tag::Two,
                TestFilter::Test3 => Tag::Four,
            }
        }

        fn criteria(pagination: Pagination) -> Self::Criteria {
            SearchCriteria::new(pagination, 0)
        }
    }

    #[test]
    fn each_filter_can_only_be_applied_once() {
        let mut criteria = TestFilter::criteria(Pagination { page: 1, page_size: None });
        for _ in 0..10 {
            criteria.add(TestFilter::Test1);
        }

        for _ in 0..10 {
            criteria.add(TestFilter::Test2);
        }

        for _ in 0..10 {
            criteria.add(TestFilter::Test3);
        }

        assert_eq!(3, criteria.filters().len());
        assert_eq!(TestFilter::Test1, criteria.filters()[0]);
        assert_eq!(TestFilter::Test2, criteria.filters()[1]);
        assert_eq!(TestFilter::Test3, criteria.filters()[2]);
    }

    #[test]
    fn search_criteria_size_no_larger_than_hash_set() {
        assert!(size_of::<SearchCriteria<TestFilter, 3>>() <= size_of::<HashSet<TestFilter>>());
    }

    #[test]
    fn search_criteria_size_no_larger_than_vec() {
        assert!(size_of::<SearchCriteria<TestFilter, 3>>() <= size_of::<Vec<TestFilter>>());
    }
}