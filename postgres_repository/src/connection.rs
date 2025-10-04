use crate::{RepoInitErr, RepoInitResult};
use error_stack::ResultExt;
use std::sync::Arc;
use tokio_postgres::types::Type;
use tokio_postgres::{Client, Statement};

pub type PsqlClient = Arc<Client>;

#[derive(Debug, Clone)]
pub struct DbConnection {
    pub client: PsqlClient,
    pub statements: Arc<PreparedStatements>,
}

impl DbConnection {
    pub async fn new(client: Client) -> RepoInitResult<Self> {
        let client = PsqlClient::new(client);
        Ok(Self {
            statements: Arc::new(PreparedStatements::new(Arc::clone(&client)).await?),
            client,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PreparedStatements {
    pub topics: TopicPreparedStatements,
}

impl PreparedStatements {
    async fn new(client: PsqlClient) -> RepoInitResult<Self> {
        Ok(Self {
            topics: TopicPreparedStatements::new(client).await?,
        })
    }
}

#[derive(Debug, Clone)]
// NOTE: preparing statements ahead of time like this will cause errors if you change the tables while the service is running
pub struct TopicPreparedStatements {
    pub exists: Statement,
    pub find: Statement,
    pub create: Statement,
    pub name_desc_search: Statement,
    pub name_search: Statement,
    pub desc_search: Statement,
    pub full_search: Statement,
}

impl TopicPreparedStatements {
    async fn new(client: PsqlClient) -> RepoInitResult<Self> {
        let this = Self {
            exists: client
                .prepare_typed("select 1 from topics where id = $1", &[Type::UUID])
                .await
                .change_context(RepoInitErr)?,
            find: client
                .prepare_typed(
                    "select id, name, description, created, updated from topics where id = $1",
                    &[Type::UUID],
                )
                .await
                .change_context(RepoInitErr)?,
            create: client
                .prepare_typed(
                    "insert into topics (id, name, description) values ($1, $2, $3) returning created",
                    &[Type::UUID, Type::VARCHAR, Type::VARCHAR],
                )
                .await
                .change_context(RepoInitErr)?,
            name_desc_search: client.prepare_typed(
                "select id, name, description, created, updated from topics where name_tsv @@ plainto_tsquery($1) and description_tsv @@ plainto_tsquery($2) offset $3 limit $4",
                &[Type::VARCHAR, Type::VARCHAR, Type::OID, Type::OID],
            )
            .await.change_context(RepoInitErr)?,
            name_search: client.prepare_typed(
                "select id, name, description, created, updated from topics where name_tsv @@ plainto_tsquery($1) offset $2 limit $3",
                &[Type::VARCHAR, Type::OID, Type::OID],
            ).await.change_context(RepoInitErr)?,
            desc_search: client.prepare_typed(
                "select id, name, description, created, updated from topics where description_tsv @@ plainto_tsquery($1) offset $2 limit $3",
                &[Type::VARCHAR, Type::OID, Type::OID],
            ).await.change_context(RepoInitErr)?,
            full_search: client.prepare_typed(
                "select id, name, description, created, updated from topics offset $1 limit $2",
                &[Type::OID, Type::OID],
            ).await.change_context(RepoInitErr)?,
        };

        Ok(this)
    }
}
