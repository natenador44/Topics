use crate::postgres::RepoInitErr;
use crate::postgres::statements::{SetStatements, TopicStatements};
use crate::postgres::topics::TopicId;
use deadpool_postgres::Pool;
use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use sets_core::SetRepository;
use sets_core::list_filter::SetListCriteria;
use sets_core::model::{NewSet, PatchSet, Set};
use sets_core::result::{OptRepoResult, RepoResult};
use tokio_postgres::Client;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct SetId(Uuid);

impl SetId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn new_with(id: Uuid) -> Self {
        Self(id)
    }
}

pub struct SetKey(SetId, TopicId);
impl sets_core::SetKey for SetKey {
    type SetId = SetId;
    type TopicId = TopicId;

    fn set_id(&self) -> Self::SetId {
        self.0
    }

    fn topic_id(&self) -> Self::TopicId {
        self.1
    }
}

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/postgres/migrations/sets");
}

async fn run_migrations(client: &mut Client) -> Result<(), Report<RepoInitErr>> {
    embedded::migrations::runner()
        .run_async(client)
        .await
        .change_context(RepoInitErr::sets())?;
    Ok(())
}

#[derive(Clone)]
pub struct SetRepo {
    pool: Pool,
    statements: SetStatements,
}

impl SetRepo {
    pub async fn new(pool: Pool) -> Result<Self, Report<RepoInitErr>> {
        let mut handle = pool.get().await.change_context(RepoInitErr::sets())?;

        let client = &mut *(&mut *handle);

        run_migrations(client).await?;

        Ok(Self {
            statements: SetStatements::prepare(client)
                .await
                .change_context(RepoInitErr::sets())?,
            pool,
        })
    }
}

impl SetRepository for SetRepo {
    type SetKey = SetKey;

    async fn get(&self, key: Self::SetKey) -> OptRepoResult<Set<Self::SetKey>> {
        todo!()
    }

    async fn list(
        &self,
        topic_id: <Self::SetKey as sets_core::SetKey>::TopicId,
        list_criteria: SetListCriteria,
    ) -> RepoResult<Vec<Set<Self::SetKey>>> {
        todo!()
    }

    async fn create(
        &self,
        topic_id: <Self::SetKey as sets_core::SetKey>::TopicId,
        new_set: NewSet,
    ) -> RepoResult<Set<Self::SetKey>> {
        todo!()
    }

    async fn create_many(
        &self,
        topic_id: <Self::SetKey as sets_core::SetKey>::TopicId,
        sets: Vec<NewSet>,
    ) -> RepoResult<Vec<Set<Self::SetKey>>> {
        todo!()
    }

    async fn patch(
        &self,
        topic_id: <Self::SetKey as sets_core::SetKey>::TopicId,
        patch: PatchSet,
    ) -> OptRepoResult<Set<Self::SetKey>> {
        todo!()
    }

    async fn delete(&self, key: Self::SetKey) -> OptRepoResult<()> {
        todo!()
    }
}
