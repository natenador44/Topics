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
pub struct TopicPreparedStatements {
    pub exists: Statement,
}

impl TopicPreparedStatements {
    async fn new(client: PsqlClient) -> RepoInitResult<Self> {
        let this = Self {
            exists: client
                .prepare_typed("select 1 from topics where topic_id = $1", &[Type::UUID])
                .await
                .change_context(RepoInitErr)?,
        };

        Ok(this)
    }
}
