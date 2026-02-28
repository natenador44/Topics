use ids::Id;
use list_filter::TopicListCriteria;
use model::{NewTopic, PatchTopic, Topic};
use result::{OptRepoResult, RepoResult};
use serde::Serialize;
use std::fmt::Debug;
use utoipa::ToSchema;

pub mod list_filter;
pub mod model;
pub mod result;

pub trait TopicEngine: Clone + Send + Sync + 'static {
    type TopicId: Id;
    type Repo: TopicRepository<TopicId = Self::TopicId>;
    // type Cache // bound not necessarily from this crate, since this will be common to all services

    fn repo(&self) -> Self::Repo;
}

// more reasons can be added, for example if we end up having restrictions on name or description
#[derive(Debug, Serialize, ToSchema, Copy, Clone, PartialEq, Eq)]
pub enum CreateManyFailReason {
    ServiceError,
    MissingName,
}

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub enum CreateManyTopicStatus<T> {
    Pending {
        name: String,
        description: Option<String>,
    },
    Success(Topic<T>),
    Fail {
        topic_name: Option<String>,
        topic_description: Option<String>,
        reason: CreateManyFailReason,
    },
}

pub trait TopicRepository: Send + Sync + Clone + 'static {
    type TopicId: Id;

    fn get(
        &self,
        id: Self::TopicId,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send;

    fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> impl Future<Output = RepoResult<Vec<Topic<Self::TopicId>>>> + Send;

    fn create(
        &self,
        new_topic: NewTopic,
    ) -> impl Future<Output = RepoResult<Topic<Self::TopicId>>> + Send;

    fn create_many(
        &self,
        topics: Vec<NewTopic>,
    ) -> impl Future<Output = RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>>> + Send;

    fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send;

    fn delete(&self, id: Self::TopicId) -> impl Future<Output = OptRepoResult<()>> + Send;
}

pub mod mock {
    use std::{
        fmt::Debug,
        marker::PhantomData,
        sync::{Arc, Mutex, atomic::AtomicUsize},
    };

    use ids::Id;
    use itertools::Itertools;

    use crate::{
        TopicRepository,
        list_filter::TopicListCriteria,
        mock::matchers::Matcher,
        model::{NewTopic, PatchTopic, Topic},
        result::{OptRepoResult, RepoResult},
    };

    pub trait MockFn<I>: Clone {
        type Args: Debug + 'static;
        type Output: Send + Sync + 'static;
        fn default_output() -> Self::Output;
    }

    pub mod matchers {
        use std::{fmt::Debug, marker::PhantomData, usize};

        use crate::mock::ThreadSafe;

        pub trait Matcher<T>: ThreadSafe + std::any::Any {
            fn matches(&self, val: &T) -> bool;
            fn expected(&self) -> String;
            fn priority(&self) -> usize;
            fn is_equivalent(&self, other: &dyn Matcher<T>) -> bool;
        }

        pub fn anything<T: ThreadSafe>() -> impl Matcher<T> {
            Anything(PhantomData)
        }

        pub fn eq<T: PartialEq + ThreadSafe + Debug>(val: T) -> impl Matcher<T> {
            Eq(val)
        }

        struct Anything<T>(PhantomData<T>);

        impl<T: ThreadSafe> Matcher<T> for Anything<T> {
            fn matches(&self, _: &T) -> bool {
                true
            }

            fn expected(&self) -> String {
                "anything".into()
            }

            fn priority(&self) -> usize {
                usize::MAX
            }

            fn is_equivalent(&self, other: &dyn Matcher<T>) -> bool {
                (other as &dyn std::any::Any)
                    .downcast_ref::<Anything<T>>()
                    .is_some()
            }
        }

        #[derive(Debug)]
        struct Eq<T: PartialEq + ThreadSafe + Debug>(T);
        impl<T: PartialEq + ThreadSafe + Debug> Matcher<T> for Eq<T> {
            fn matches(&self, val: &T) -> bool {
                &self.0 == val
            }

            fn expected(&self) -> String {
                format!("equal to {:?}", self.0)
            }

            fn priority(&self) -> usize {
                usize::MIN
            }

            fn is_equivalent(&self, other: &dyn Matcher<T>) -> bool {
                (other as &dyn std::any::Any)
                    .downcast_ref::<Eq<T>>()
                    .map_or(false, |Eq(val)| &self.0 == val)
            }
        }
    }

    pub trait ReturningFn<T>: Fn() -> T + ThreadSafe + 'static {}
    impl<F, T> ReturningFn<T> for F where F: Fn() -> T + ThreadSafe + 'static {}

    struct Returning<T>(Arc<dyn ReturningFn<T>>);
    impl<T> Clone for Returning<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    struct Scenario<I, F: MockFn<I>> {
        arg_matcher: Arc<dyn Matcher<F::Args>>,
        returning: Returning<F::Output>,
        invocation_count: Arc<AtomicUsize>,
    }
    impl<I, F: MockFn<I>> Clone for Scenario<I, F> {
        fn clone(&self) -> Self {
            Self {
                arg_matcher: Arc::clone(&self.arg_matcher),
                returning: self.returning.clone(),
                invocation_count: Arc::clone(&self.invocation_count),
            }
        }
    }

    pub struct ScenarioBuilder<'a, I, F: MockFn<I>> {
        mock: &'a mut Mock<I, F>,
        arg_matcher: Arc<dyn Matcher<F::Args>>,
    }

    impl<'a, I, F: MockFn<I>> ScenarioBuilder<'a, I, F> {
        pub fn then_return(self, value_fn: impl ReturningFn<F::Output>) {
            let mut scenarios = self.mock.scenarios.lock().expect("mutex is not poisoned");
            scenarios.push(Scenario {
                arg_matcher: self.arg_matcher,
                returning: Returning(Arc::new(value_fn)),
                invocation_count: Arc::new(AtomicUsize::new(0)),
            });
            // self.mock // do something like this if we have overlapping arg matchers (like eq and any() for the same function). We'd want the eq to match first
            //     .scenarios
            //     .sort_by(|s1, s2| s1.arg_matcher.priority().cmp(&s2.arg_matcher.priority()));
        }
    }

    pub struct Verification<I, F: MockFn<I>> {
        mock_name: &'static str,
        scenario: Option<Scenario<I, F>>,
    }

    impl<I, F: MockFn<I>> Verification<I, F> {
        pub fn was_called_only_once(&self) {
            self.was_called_n_times(1);
        }

        pub fn was_not_called(&self) {
            self.was_called_n_times(0);
        }

        pub fn was_called_n_times(&self, expected_count: usize) {
            let actual_call_count = self.scenario.as_ref().map_or(0, |s| {
                s.invocation_count
                    .load(std::sync::atomic::Ordering::Relaxed)
            });
            assert_eq!(
                expected_count, actual_call_count,
                "'{}' expected invocation count of {}, got {}",
                self.mock_name, expected_count, actual_call_count
            );
        }
    }

    pub struct Mock<I, F: MockFn<I>> {
        scenarios: Arc<Mutex<Vec<Scenario<I, F>>>>,
        name: &'static str,
    }

    impl<I, F: MockFn<I>> Clone for Mock<I, F> {
        fn clone(&self) -> Self {
            Self {
                scenarios: Arc::clone(&self.scenarios),
                name: self.name,
            }
        }
    }

    impl<I, F: MockFn<I>> Mock<I, F>
    where
        I: ThreadSafe + Debug + Default + Clone,
    {
        fn new(name: &'static str) -> Mock<I, F> {
            Self {
                scenarios: Default::default(),
                name,
            }
        }

        pub fn get() -> Mock<I, MockGet<I>> {
            Mock::<I, MockGet<I>>::new("get")
        }

        pub fn create() -> Mock<I, MockCreate<I>> {
            Mock::new("create")
        }

        // TODO I don't think this is how mocks are supposed to work.. idk, maybe.
        // I think it should be a "if args match this, return this" sort of thing.
        pub fn when_called_with(
            &mut self,
            arg_match: impl Matcher<F::Args> + 'static,
        ) -> ScenarioBuilder<'_, I, F> {
            ScenarioBuilder {
                mock: self,
                arg_matcher: Arc::new(arg_match),
            }
        }

        pub fn verify_call_with(
            &self,
            arg_match: impl Matcher<F::Args> + 'static,
        ) -> Verification<I, F> {
            let scenarios = self.scenarios.lock().expect("mutex not poisoned");
            let scenario = scenarios
                .iter()
                .find(|s| arg_match.is_equivalent(&*s.arg_matcher))
                .cloned();

            Verification {
                mock_name: self.name,
                scenario,
            }
        }

        fn find_matching_scenarios(&self, args: &F::Args) -> Vec<Scenario<I, F>> {
            let scenarios = self.scenarios.lock().expect("mutex is not poisoned");

            scenarios
                .iter()
                .filter(|s| {
                    println!("here we go blah blah");
                    s.arg_matcher.matches(args)
                })
                .map(|s| s.clone())
                .collect()
        }

        fn invoke(&self, args: F::Args) -> F::Output {
            let matching_scenarios = self.find_matching_scenarios(&args);

            match &*matching_scenarios {
                [] => panic!(
                    "No matching scenarios found for '{}' call. Expected call arguments to be one of the following: [{}]. Actual args were `{:?}`",
                    self.name,
                    matching_scenarios
                        .iter()
                        .map(|s| s.arg_matcher.expected())
                        .join(","),
                    &args,
                ),
                [scenario] => {
                    scenario
                        .invocation_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    scenario.returning.0()
                }
                [..] => panic!(
                    "Multiple scenarios matched the args {:?} for {}",
                    &args, self.name,
                ),
            }
        }
    }

    pub trait ThreadSafe: Send + Sync + 'static {}
    impl<T> ThreadSafe for T where T: Send + Sync + 'static {}

    #[derive(Clone, Default)]
    pub struct MockGet<I>(PhantomData<I>);
    impl<I> MockFn<I> for MockGet<I>
    where
        I: ThreadSafe + Debug + Default + Clone,
    {
        type Args = I;

        type Output = OptRepoResult<Topic<I>>;

        fn default_output() -> Self::Output {
            Ok(None)
        }
    }

    fn default_topic<I: Default>() -> Topic<I> {
        Topic::create(I::default(), "default topic".into(), None)
    }

    #[derive(Clone, Default)]
    pub struct MockCreate<I>(PhantomData<I>);
    impl<I> MockFn<I> for MockCreate<I>
    where
        I: ThreadSafe + Debug + Default + Clone,
    {
        type Args = NewTopic;

        type Output = RepoResult<Topic<I>>;

        fn default_output() -> Self::Output {
            Ok(default_topic())
        }
    }

    #[derive(Default)]
    pub struct MockList<I>(PhantomData<I>);
    impl<I> Clone for MockList<I> {
        fn clone(&self) -> Self {
            Self(PhantomData)
        }
    }

    impl<I> MockFn<I> for MockList<I>
    where
        I: ThreadSafe + Debug + Default + Clone,
    {
        type Args = TopicListCriteria;

        type Output = RepoResult<Vec<Topic<I>>>;

        fn default_output() -> Self::Output {
            Ok(vec![])
        }
    }

    #[derive(Clone, Default)]
    pub struct MockCreateMany<I>(PhantomData<I>);

    impl<I> MockFn<I> for MockCreateMany<I>
    where
        I: ThreadSafe + Debug + Default + Clone,
    {
        type Args = Vec<NewTopic>;

        type Output = RepoResult<Vec<RepoResult<Topic<I>>>>;

        fn default_output() -> Self::Output {
            Ok(vec![])
        }
    }

    #[derive(Clone, Default)]
    pub struct MockPatch<I>(PhantomData<I>);

    impl<I> MockFn<I> for MockPatch<I>
    where
        I: ThreadSafe + Debug + Default + Clone,
    {
        type Args = (I, PatchTopic);

        type Output = OptRepoResult<Topic<I>>;

        fn default_output() -> Self::Output {
            Ok(None)
        }
    }

    #[derive(Clone, Default)]
    pub struct MockDelete<I>(PhantomData<I>);

    impl<I> MockFn<I> for MockDelete<I>
    where
        I: ThreadSafe + Debug + Default + Clone,
    {
        type Args = I;

        type Output = OptRepoResult<()>;

        fn default_output() -> Self::Output {
            Ok(None)
        }
    }

    #[derive(Clone)]
    pub struct MockTopicRepository<I: ThreadSafe + Debug + Default + Clone> {
        pub get_mock: Mock<I, MockGet<I>>,
        pub list_mock: Mock<I, MockList<I>>,
        pub create_mock: Mock<I, MockCreate<I>>,
        pub create_many_mock: Mock<I, MockCreateMany<I>>,
        pub patch_mock: Mock<I, MockPatch<I>>,
        pub delete: Mock<I, MockDelete<I>>,
    }

    impl<I: ThreadSafe + Debug + Default + Clone> Default for MockTopicRepository<I> {
        fn default() -> Self {
            Self {
                get_mock: Mock::new("get"),
                list_mock: Mock::new("list"),
                create_mock: Mock::new("create"),
                create_many_mock: Mock::new("create_many"),
                patch_mock: Mock::new("patch"),
                delete: Mock::new("delete"),
            }
        }
    }

    impl<I: ThreadSafe + Clone + Debug + Default> MockTopicRepository<I> {}

    impl<I: Id + Default + 'static> TopicRepository for MockTopicRepository<I> {
        type TopicId = I;

        fn get(
            &self,
            id: Self::TopicId,
        ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send {
            let get = self.get_mock.clone();
            async move { get.invoke(id) }
        }

        fn list(
            &self,
            list_criteria: TopicListCriteria,
        ) -> impl Future<Output = RepoResult<Vec<Topic<Self::TopicId>>>> + Send {
            let list = self.list_mock.clone();
            async move { list.invoke(list_criteria) }
        }

        fn create(
            &self,
            new_topic: NewTopic,
        ) -> impl Future<Output = RepoResult<Topic<Self::TopicId>>> + Send {
            let create = self.create_mock.clone();
            async move { create.invoke(new_topic) }
        }

        fn create_many(
            &self,
            topics: Vec<NewTopic>,
        ) -> impl Future<Output = RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>>> + Send
        {
            let create_many = self.create_many_mock.clone();
            async move { create_many.invoke(topics) }
        }

        fn patch(
            &self,
            id: Self::TopicId,
            patch: PatchTopic,
        ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send {
            let p = self.patch_mock.clone();
            async move { p.invoke((id, patch)) }
        }

        fn delete(&self, id: Self::TopicId) -> impl Future<Output = OptRepoResult<()>> + Send {
            let delete = self.delete.clone();
            async move { delete.invoke(id) }
        }
    }
}
