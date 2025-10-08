use crate::postgres::connection::PsqlClient;
use crate::{RepoInitErr, RepoInitResult};
use connection::DbConnection;
use engine::error::{EntityRepoError, RepoResult, SetRepoError, TopicRepoError};
use engine::models::{Entity, EntityId, Set, SetId, Topic, TopicId};
use engine::repository::entities::{EntityUpdate, ExistingEntityRepository};
use engine::repository::sets::{ExistingSetRepository, SetUpdate};
use engine::repository::topics::{ExistingTopicRepository, TopicUpdate};
use engine::repository::{
    EntitiesRepository, IdentifiersRepository, SetsRepository, TopicsRepository,
};
use engine::search_criteria::SearchCriteria;
use engine::search_filters::{
    EntitySearchCriteria, SetSearchCriteria, TopicFilter, TopicSearchCriteria,
};
use error_stack::{FutureExt, IntoReport, Report, ResultExt};
use optional_field::Field;
use serde_json::Value;
use tokio::task::JoinSet;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, GenericClient, NoTls, Row, Statement, connect};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, instrument, warn};

mod connection;
mod migration;

#[cfg(test)]
mod tests;

pub enum ConnectionDetails {
    Url(String),
    Individual {
        user: String,
        password: String,
        database: String,
        port: u16,
        host: String,
    },
}

pub async fn init(
    runtime: tokio::runtime::Handle,
    connection_details: ConnectionDetails,
) -> RepoInitResult<TopicRepo> {
    let (mut client, connection) = match connection_details {
        ConnectionDetails::Url(url) => connect(&url, NoTls).await.change_context(RepoInitErr)?,
        ConnectionDetails::Individual { .. } => {
            todo!("individual connection components not supported yet")
        }
    };

    runtime.spawn(async move {
        if let Err(e) = connection.await {
            error!("postgres connection error: {e:?}");
        }
    });

    migration::run(&mut client).await?;

    Ok(TopicRepo {
        connection: DbConnection::new(client).await?,
    })
}

#[derive(Debug, Clone)]
pub struct TopicRepo {
    connection: DbConnection,
}

fn row_to_topic(row: Row) -> Topic {
    Topic {
        id: TopicId(row.get("id")),
        name: row.get("name"),
        description: row.get("description"),
        created: row.get("created"),
        updated: row.get("updated"),
    }
}

impl TopicsRepository for TopicRepo {
    type ExistingTopic = ExistingTopicRepo;

    #[instrument(skip_all, name = "repo#expect_existing")]
    async fn expect_existing(
        &self,
        topic_id: TopicId,
    ) -> RepoResult<Option<Self::ExistingTopic>, TopicRepoError> {
        debug!("expecting topic to exist...");
        Ok(self
            .connection
            .client
            .query_opt(&self.connection.statements.topics.exists, &[&topic_id.0])
            .await
            .change_context(TopicRepoError::Exists)?
            .map(|_| ExistingTopicRepo {
                topic_id,
                conn: self.connection.clone(),
            }))
    }

    #[instrument(skip_all, name = "repo#find")]
    async fn find(&self, topic_id: TopicId) -> RepoResult<Option<Topic>, TopicRepoError> {
        debug!("grabbing topic data");
        let result = self
            .connection
            .client
            .query_opt(&self.connection.statements.topics.find, &[&topic_id.0])
            .await
            .change_context(TopicRepoError::Get)?;

        Ok(result.map(row_to_topic))
    }

    #[instrument(skip_all, name = "repo#create")]
    async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> RepoResult<Topic, TopicRepoError> {
        let id = TopicId::new();
        debug!("creating new topic with id {id}");
        let row = self
            .connection
            .client
            .query_one(
                &self.connection.statements.topics.create,
                &[&id.0, &name, &description],
            )
            .await
            .change_context(TopicRepoError::Create)?;

        // may eventually grab id, name, and description from the insert call
        // this prevents extra memory allocation though
        Ok(Topic {
            id,
            name,
            description,
            created: row.get("created"),
            updated: None,
        })
    }

    #[instrument(skip_all, name = "repo#search")]
    async fn search(
        &self,
        topic_search_criteria: TopicSearchCriteria,
    ) -> RepoResult<Vec<Topic>, TopicRepoError> {
        let client = &self.connection.client;
        let statements = &self.connection.statements;
        let offset = topic_search_criteria.page().saturating_sub(1); // assuming users send '1' to specify the first page... we'll see if this sticks, not sure what standard is
        let page_size = topic_search_criteria.page_size();
        debug!(
            "user specified page: {}, actual offset: {offset}, page size: {page_size}, number of search filters: {}",
            topic_search_criteria.page(),
            topic_search_criteria
                .filters()
                .map(|f| f.len())
                .unwrap_or(0)
        );

        let result = match topic_search_criteria.filters() {
            Some(
                [TopicFilter::Name(name), TopicFilter::Description(desc)]
                | [TopicFilter::Description(desc), TopicFilter::Name(name)],
            ) => client
                .query_raw(
                    &statements.topics.name_desc_search,
                    slice_iter(&[name, desc, &offset, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
            Some([TopicFilter::Name(name)]) => client
                .query_raw(
                    &statements.topics.name_search,
                    slice_iter(&[name, &offset, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
            Some([TopicFilter::Description(desc)]) => client
                .query_raw(
                    &statements.topics.desc_search,
                    slice_iter(&[desc, &offset, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
            Some(_) => unreachable!("currently only two topic filters exist"),
            None => client
                .query_raw(
                    &statements.topics.full_search,
                    slice_iter(&[&offset, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
        };

        result
            .map(|r| r.map(row_to_topic).change_context(TopicRepoError::Search))
            .collect()
            .await
    }
}

pub struct ExistingTopicRepo {
    conn: DbConnection,
    topic_id: TopicId,
}

impl ExistingTopicRepository for ExistingTopicRepo {
    type SetRepo = SetRepo;
    type IdentifierRepo = IdentifierRepo;

    fn sets(&self) -> Self::SetRepo {
        SetRepo {
            conn: self.conn.clone(),
            topic_id: self.topic_id,
        }
    }

    fn identifiers(&self) -> Self::IdentifierRepo {
        todo!()
    }

    #[instrument(skip_all, name = "repo#delete")]
    async fn delete(&self) -> RepoResult<(), TopicRepoError> {
        self.conn
            .client
            .execute(&self.conn.statements.topics.delete, &[&self.topic_id.0])
            .await
            .change_context(TopicRepoError::Delete)?;
        Ok(())
    }

    #[instrument(skip_all, name = "repo#update")]
    async fn update(&self, topic: TopicUpdate) -> RepoResult<Topic, TopicRepoError> {
        let row = match (topic.name, topic.description) {
            (Some(n), Field::Present(d)) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.topics.update_name_desc,
                    &[&n, &d, &self.topic_id.0],
                )
                .await
                .change_context(TopicRepoError::Update)?,
            (Some(n), Field::Missing) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.topics.update_name,
                    &[&n, &self.topic_id.0],
                )
                .await
                .change_context(TopicRepoError::Update)?,
            (None, Field::Present(d)) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.topics.update_desc,
                    &[&d, &self.topic_id.0],
                )
                .await
                .change_context(TopicRepoError::Update)?,
            _ => {
                debug!("no changes requested, only reading topic");
                self.conn
                    .client
                    .query_one(&self.conn.statements.topics.find, &[&self.topic_id.0])
                    .await
                    .change_context(TopicRepoError::Search)?
            }
        };

        Ok(row_to_topic(row))
    }
}

pub struct SetRepo {
    conn: DbConnection,
    topic_id: TopicId,
}

fn row_to_set(row: Row) -> Set {
    Set {
        id: SetId(row.get("id")),
        topic_id: TopicId(row.get("topic_id")),
        name: row.get("name"),
        description: row.get("description"),
        created: row.get("created"),
        updated: row.get("updated"),
    }
}

fn row_to_entity(row: Row) -> Entity {
    Entity {
        id: EntityId(row.get("id")),
        set_id: SetId(row.get("set_id")),
        applied_identifiers: Vec::new(), // not implemented yet
        payload: row.get("payload"),
        created: row.get("created"),
        updated: row.get("updated"),
    }
}

impl SetsRepository for SetRepo {
    type ExistingSet = ExistingSetRepo;

    async fn expect_existing(
        &self,
        set_id: SetId,
    ) -> RepoResult<Option<Self::ExistingSet>, SetRepoError> {
        debug!("expecting entity to exist...");
        Ok(self
            .conn
            .client
            .query_opt(&self.conn.statements.sets.exists, &[&set_id.0])
            .await
            .change_context(SetRepoError::Exists)?
            .map(|_| ExistingSetRepo {
                conn: self.conn.clone(),
                topic_id: self.topic_id,
                set_id,
            }))
    }

    async fn find(&self, set_id: SetId) -> RepoResult<Option<Set>, SetRepoError> {
        let row = self
            .conn
            .client
            .query_opt(&self.conn.statements.sets.find, &[&set_id.0])
            .await
            .change_context(SetRepoError::Get)?;

        Ok(row.map(row_to_set))
    }

    async fn create(
        &self,
        name: String,
        description: Option<String>,
        initial_entity_payloads: Vec<Value>,
    ) -> RepoResult<Set, SetRepoError> {
        let client = &self.conn.client;
        let statements = &self.conn.statements;
        let set_id = SetId::new();

        let row = client
            .query_one(
                &statements.sets.create,
                &[&set_id.0, &self.topic_id.0, &name, &description],
            )
            .await
            .change_context(SetRepoError::Create)?;

        if !initial_entity_payloads.is_empty() {
            process_entities(
                initial_entity_payloads,
                EntityRepo {
                    conn: self.conn.clone(),
                    topic_id: self.topic_id,
                    set_id,
                },
            )
            .await?;
        }

        Ok(row_to_set(row))
    }

    async fn search(
        &self,
        search_criteria: SetSearchCriteria,
    ) -> RepoResult<Vec<Set>, SetRepoError> {
        // TODO actual search criteria
        let offset = search_criteria.page().saturating_sub(1); // assuming users send '1' to specify the first page... we'll see if this sticks, not sure what standard is
        let page_size = search_criteria.page_size();
        debug!(
            "user specified page: {}, actual offset: {offset}, page size: {page_size}, number of search filters: {}",
            search_criteria.page(),
            search_criteria.filters().map(|f| f.len()).unwrap_or(0)
        );

        let result = self
            .conn
            .client
            .query_raw(
                &self.conn.statements.sets.full_search,
                slice_iter(&[&offset, &page_size]),
            )
            .await
            .change_context(SetRepoError::Search)?;

        result
            .map(|r| r.map(row_to_set).change_context(SetRepoError::Search))
            .collect()
            .await
    }
}

async fn process_entities(
    payloads: Vec<Value>,
    entity_repo: EntityRepo,
) -> RepoResult<(), SetRepoError> {
    let mut futs = payloads
        .into_iter()
        .map(|p| {
            let repo = entity_repo.clone();
            async move { repo.create(p).await }
        })
        .collect::<JoinSet<_>>();

    while let Some(entity) = futs.join_next().await {
        let entity = match entity {
            Ok(entity) => entity.change_context(SetRepoError::Create)?,
            Err(e) => {
                warn!(
                    "failed to join on entity creation futures: {:?}",
                    e.into_report()
                );
                continue;
            }
        };

        info!("successfully created entity: {}", entity.id);
    }

    Ok(())
}

pub struct ExistingSetRepo {
    conn: DbConnection,
    topic_id: TopicId,
    set_id: SetId,
}

impl ExistingSetRepository for ExistingSetRepo {
    type EntitiesRepo = EntityRepo;

    fn entities(&self) -> Self::EntitiesRepo {
        EntityRepo {
            conn: self.conn.clone(),
            topic_id: self.topic_id,
            set_id: self.set_id,
        }
    }

    async fn delete(&self) -> RepoResult<(), SetRepoError> {
        // delete all entities in the set first
        // then delete the set
        // eventually we'll have a 'soft delete' option since this is so destructive
        self.entities()
            .delete_all_in_set()
            .await
            .change_context(SetRepoError::Delete)?;

        let delete_count = self
            .conn
            .client
            .execute(&self.conn.statements.sets.delete, &[&self.set_id.0])
            .await
            .change_context(SetRepoError::Delete)?;

        info!("deleted {} sets", delete_count);
        Ok(())
    }

    async fn update(&self, set: SetUpdate) -> RepoResult<Set, SetRepoError> {
        match (set.name, set.description) {
            (Some(n), Field::Present(Some(d))) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.sets.update_name_desc,
                    &[&n, &d, &self.set_id.0],
                )
                .await
                .change_context(SetRepoError::Update)
                .map(row_to_set),
            (Some(n), _) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.sets.update_name,
                    &[&n, &self.set_id.0],
                )
                .await
                .change_context(SetRepoError::Update)
                .map(row_to_set),
            (_, Field::Present(Some(d))) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.sets.update_desc,
                    &[&d, &self.set_id.0],
                )
                .await
                .change_context(SetRepoError::Update)
                .map(row_to_set),
            _ => {
                info!("no update properties specified, just getting set as is");
                self.conn
                    .client
                    .query_one(&self.conn.statements.sets.find, &[&self.set_id.0])
                    .await
                    .change_context(SetRepoError::Update)
                    .map(row_to_set)
            }
        }
    }
}

#[derive(Clone)]
pub struct EntityRepo {
    conn: DbConnection,
    topic_id: TopicId,
    set_id: SetId,
}

impl EntitiesRepository for EntityRepo {
    type ExistingEntityRepo = ExistingEntityRepo;

    async fn expect_existing(
        &self,
        entity_id: EntityId,
    ) -> RepoResult<Option<Self::ExistingEntityRepo>, EntityRepoError> {
        debug!("expecting entity to exist...");
        Ok(self
            .conn
            .client
            .query_opt(
                &self.conn.statements.entities.exists,
                &[&entity_id.0, &self.set_id.0],
            )
            .await
            .change_context(EntityRepoError::Exists)?
            .map(|_| ExistingEntityRepo {
                conn: self.conn.clone(),
                topic_id: self.topic_id,
                set_id: self.set_id,
                entity_id,
            }))
    }

    async fn search(
        &self,
        search_criteria: EntitySearchCriteria,
    ) -> RepoResult<Vec<Entity>, EntityRepoError> {
        // TODO actual search criteria
        let offset = search_criteria.page().saturating_sub(1); // assuming users send '1' to specify the first page... we'll see if this sticks, not sure what standard is
        let page_size = search_criteria.page_size();
        debug!(
            "user specified page: {}, actual offset: {offset}, page size: {page_size}, number of search filters: {}",
            search_criteria.page(),
            search_criteria.filters().map(|f| f.len()).unwrap_or(0)
        );

        let result = self
            .conn
            .client
            .query_raw(
                &self.conn.statements.entities.full_search,
                slice_iter(&[&self.set_id.0, &offset, &page_size]),
            )
            .await
            .change_context(EntityRepoError::Search)?;

        result
            .map(|r| r.map(row_to_entity).change_context(EntityRepoError::Search))
            .collect()
            .await
    }

    async fn find(&self, entity_id: EntityId) -> RepoResult<Option<Entity>, EntityRepoError> {
        let entity = self
            .conn
            .client
            .query_opt(
                &self.conn.statements.entities.find,
                &[&entity_id.0, &self.set_id.0],
            )
            .await
            .change_context(EntityRepoError::Get)?
            .map(row_to_entity);
        Ok(entity)
    }

    async fn create(&self, payload: Value) -> RepoResult<Entity, EntityRepoError> {
        let entity_id = EntityId::new();
        let row = self
            .conn
            .client
            .query_one(
                &self.conn.statements.entities.create,
                &[&entity_id.0, &self.set_id.0, &payload],
            )
            .await
            .change_context(EntityRepoError::Create)?;
        Ok(row_to_entity(row))
    }

    async fn delete_all_in_set(&self) -> RepoResult<(), EntityRepoError> {
        let removed_entity_count = self
            .conn
            .client
            .execute(
                &self.conn.statements.entities.delete_all_in_set,
                &[&self.set_id.0],
            )
            .await
            .change_context(EntityRepoError::Delete)?;

        info!("removed {} entities", removed_entity_count);
        Ok(())
    }
}

pub struct ExistingEntityRepo {
    conn: DbConnection,
    topic_id: TopicId,
    set_id: SetId,
    entity_id: EntityId,
}

impl ExistingEntityRepository for ExistingEntityRepo {
    async fn delete(&self) -> RepoResult<(), EntityRepoError> {
        let deleted_count = self
            .conn
            .client
            .execute(
                &self.conn.statements.entities.delete,
                &[&self.entity_id.0, &self.set_id.0],
            )
            .await
            .change_context(EntityRepoError::Delete)?;

        info!("deleted {} entities", deleted_count);
        Ok(())
    }

    async fn update(&self, entity_update: EntityUpdate) -> RepoResult<Entity, EntityRepoError> {
        let row = if let Some(payload) = entity_update.payload {
            self.conn
                .client
                .query_one(
                    &self.conn.statements.entities.update_payload,
                    &[&payload, &self.entity_id.0, &self.set_id.0],
                )
                .await
                .change_context(EntityRepoError::Update)?
        } else {
            info!("updated requested without update parameters, just getting entity");
            self.conn
                .client
                .query_one(
                    &self.conn.statements.entities.find,
                    &[&self.entity_id.0, &self.set_id.0],
                )
                .await
                .change_context(EntityRepoError::Update)?
        };

        Ok(row_to_entity(row))
    }
}

pub struct IdentifierRepo; // TODO postgres pool

impl IdentifiersRepository for IdentifierRepo {}

fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}
