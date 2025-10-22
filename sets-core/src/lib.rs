use crate::list_filter::SetListCriteria;
use crate::model::{NewSet, PatchSet, Set};
use crate::result::{OptRepoResult, RepoResult};
use engine::id::Id;

pub mod model;

pub mod list_filter;
pub mod result;
pub trait SetKey {
    type SetId: Id;
    type TopicId: Id;

    fn set_id(&self) -> Self::SetId;
    fn topic_id(&self) -> Self::TopicId;
}

pub trait SetEngine: Clone + Send + Sync + 'static {
    type SetKey: SetKey;
    type Repo: SetRepository<SetKey = Self::SetKey>;

    fn repo(&self) -> Self::Repo;
}

pub trait SetRepository: Clone + Send + Sync + 'static {
    type SetKey: SetKey;

    fn get(
        &self,
        key: Self::SetKey,
    ) -> impl Future<Output = OptRepoResult<Set<Self::SetKey>>> + Send;

    fn list(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        list_criteria: SetListCriteria,
    ) -> impl Future<Output = RepoResult<Vec<Set<Self::SetKey>>>> + Send;

    fn create(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        new_set: NewSet,
    ) -> impl Future<Output = RepoResult<Set<Self::SetKey>>> + Send;

    fn create_many(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        sets: Vec<NewSet>,
    ) -> impl Future<Output = RepoResult<Vec<Set<Self::SetKey>>>> + Send;

    fn patch(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        patch: PatchSet,
    ) -> impl Future<Output = OptRepoResult<Set<Self::SetKey>>> + Send;

    fn delete(&self, key: Self::SetKey) -> impl Future<Output = OptRepoResult<()>> + Send;
}
