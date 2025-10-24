use crate::postgres::insert_many::{InsertMany, InsertManyBuilder, value_set};
use crate::postgres::statements::SetStatements;
use crate::postgres::topics::TopicId;
use crate::postgres::{RepoInitErr, sanitize_pagination};
use deadpool_postgres::{Object, Pool};
use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};
use sets_core::list_filter::SetListCriteria;
use sets_core::model::{NewSet, PatchSet, Set};
use sets_core::result::{OptRepoResult, Reason, RepoResult, SetRepoError};
use sets_core::{SetKey, SetRepository};
use std::borrow::Borrow;
use tokio_postgres::Row;
use tokio_postgres::error::SqlState;
use tokio_stream::StreamExt;
use tracing::warn;
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

#[derive(Debug)]
pub struct PostgresSetKey(pub TopicId, pub SetId);
impl SetKey for PostgresSetKey {
    type SetId = SetId;
    type TopicId = TopicId;

    fn set_id(&self) -> Self::SetId {
        self.1
    }

    fn topic_id(&self) -> Self::TopicId {
        self.0
    }
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

        Ok(Self {
            statements: SetStatements::prepare(client)
                .await
                .change_context(RepoInitErr::sets())?,
            pool,
        })
    }

    async fn client(&self, on_err: SetRepoError) -> RepoResult<Object> {
        self.pool.get().await.change_context(on_err)
    }
}

enum GetOutcome {
    SetNotFound,
    SetFound(Set<PostgresSetKey>),
}

impl From<Row> for GetOutcome {
    fn from(row: Row) -> Self {
        let set_exists: bool = row.get("set_exists");

        if !set_exists {
            Self::SetNotFound
        } else {
            Self::SetFound(row_to_set(row))
        }
    }
}

fn row_to_set(row: impl Borrow<Row>) -> Set<PostgresSetKey> {
    let row = row.borrow();
    Set {
        key: PostgresSetKey(TopicId(row.get("topic_id")), SetId(row.get("id"))),
        name: row.get("name"),
        description: row.get("description"),
        created: row.get("created"),
        updated: row.get("updated"),
    }
}

impl SetRepository for SetRepo {
    type SetKey = PostgresSetKey;

    async fn get(&self, key: Self::SetKey) -> OptRepoResult<Set<Self::SetKey>> {
        let result = self
            .client(SetRepoError::Get(Reason::Db))
            .await?
            .query_opt(&self.statements.get, &[&key.topic_id().0, &key.set_id().0])
            .await
            .change_context(SetRepoError::Get(Reason::Db))?
            .map(GetOutcome::from);

        match result {
            // no topics in database
            None => Err(SetRepoError::Get(Reason::TopicNotFound).into_report()),
            // topic in database, but could not find a set with this id associated with the topic id
            Some(GetOutcome::SetNotFound) => Ok(None),
            Some(GetOutcome::SetFound(set)) => Ok(Some(set)),
        }
    }

    async fn list(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        list_criteria: SetListCriteria,
    ) -> RepoResult<Vec<Set<Self::SetKey>>> {
        let pagination =
            sanitize_pagination(&list_criteria, SetRepoError::List(Reason::Validation))?;

        // TODO find a way to do this with RowStream so we're not allocating a new Vec
        let rows = self
            .client(SetRepoError::List(Reason::Db))
            .await?
            .query(
                &self.statements.list,
                &[&topic_id.0, &pagination.page, &pagination.page_size],
            )
            .await
            .change_context(SetRepoError::List(Reason::Db))?;

        // with the structure of the query, we'll get one row of null column values
        // if the topic does exist. So if the query is empty, this topic didn't exist
        if rows.is_empty() {
            Err(SetRepoError::List(Reason::TopicNotFound).into_report())
        } else {
            let first = &rows[0];

            let set_id: Option<Uuid> = first.get("id");
            let sets = if set_id.is_some() {
                rows.into_iter().map(row_to_set).collect()
            } else {
                vec![]
            };
            Ok(sets)
        }
    }

    async fn create(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        new_set: NewSet,
    ) -> RepoResult<Set<Self::SetKey>> {
        let set_id = SetId::new();
        let result = self
            .client(SetRepoError::Create(Reason::Db))
            .await?
            .query_one(
                &self.statements.create,
                &[&set_id.0, &topic_id.0, &new_set.name, &new_set.description],
            )
            .await;

        match result {
            Ok(row) => Ok(row_to_set(row)),
            Err(e)
                if e.code().map_or(false, |c| {
                    c.code() == SqlState::FOREIGN_KEY_VIOLATION.code()
                }) =>
            {
                Err(e.into_report()).change_context(SetRepoError::Create(Reason::TopicNotFound))
            }
            Err(e) => Err(e.into_report()).change_context(SetRepoError::Create(Reason::Db)),
        }
    }

    async fn create_many(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        sets: Vec<NewSet>,
    ) -> RepoResult<Vec<RepoResult<Set<Self::SetKey>>>> {
        let Some(insert_many) = generate_insert_many(topic_id, sets) else {
            warn!("no set requests sent to data layer, not creating any new topics");
            return Ok(vec![]);
        };

        let results = self
            .client(SetRepoError::CreateMany(Reason::Db))
            .await?
            .query_raw(&insert_many.query, insert_many.params())
            .await
            .change_context(SetRepoError::CreateMany(Reason::Db))?
            .collect::<Vec<_>>()
            .await;

        let mut set_results = Vec::with_capacity(results.len());

        for (i, row_result) in results.into_iter().enumerate() {
            match row_result {
                Ok(row) => set_results.push(Ok(row_to_set(row))),
                Err(e)
                    if e.code().map_or(false, |c| {
                        c.code() == SqlState::FOREIGN_KEY_VIOLATION.code()
                    }) =>
                {
                    return Err(SetRepoError::CreateMany(Reason::TopicNotFound).into_report());
                }
                Err(e) => {
                    warn!("Result row {i} failed: {e}");
                    set_results.push(Err(SetRepoError::CreateMany(Reason::Db).into_report()));
                }
            }
        }

        Ok(set_results)
    }

    async fn patch(
        &self,
        topic_id: <Self::SetKey as SetKey>::TopicId,
        patch: PatchSet,
    ) -> OptRepoResult<Set<Self::SetKey>> {
        todo!()
    }

    async fn delete(&self, key: Self::SetKey) -> OptRepoResult<()> {
        todo!()
    }
}

fn generate_insert_many(topic_id: TopicId, sets: Vec<NewSet>) -> Option<InsertMany> {
    let mut set_iter = sets.into_iter();

    let first = set_iter.next()?;

    let mut builder = InsertManyBuilder::new(
        "sets",
        ["id", "topic_id", "name", "description"],
        value_set![SetId::new().0 => Uuid, topic_id.0 => Uuid, first.name => String, first.description => Option<String>],
    );

    for set in set_iter {
        builder.add_value_set(value_set![SetId::new().0 => Uuid, topic_id.0 => Uuid, set.name => String, set.description => Option<String>]);
    }

    builder.returning(&[
        "id",
        "topic_id",
        "name",
        "description",
        "created",
        "updated",
    ]);

    Some(builder.build())
}
