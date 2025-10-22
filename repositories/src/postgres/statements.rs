use error_stack::{Report, ResultExt};
use tokio_postgres::types::Type;
use tokio_postgres::{Client, Statement};

#[derive(Debug, thiserror::Error)]
#[error("failed to prepare topics statement")]
pub struct StatementPrepareError;

#[derive(Debug, Clone)]
pub struct TopicStatements {
    pub get: Statement,
    pub list: Statement,
    pub create: Statement,
    pub patch_name_desc: Statement,
    pub patch_name: Statement,
    pub patch_desc: Statement,
    pub delete: Statement,
}

impl TopicStatements {
    pub async fn prepare(client: &Client) -> Result<Self, Report<StatementPrepareError>> {
        Ok(Self {
            get: client
                .prepare_typed(
                    "select id, name, description, created, updated from topics where id = $1",
                    &[Type::UUID],
                )
                .await
                .change_context(StatementPrepareError)?,
            list: client
                .prepare_typed(
                    "select id, name, description, created, updated from topics offset $1 limit $2",
                    &[Type::INT8, Type::INT8],
                )
                .await
                .change_context(StatementPrepareError)?,
            create: client
                .prepare_typed(
                    "insert into topics (id, name, description) values ($1, $2, $3) returning id, name, description, created, updated",
                    &[Type::UUID, Type::VARCHAR, Type::VARCHAR],
                )
                .await
                .change_context(StatementPrepareError)?,
            patch_name_desc: client
                .prepare_typed(
                    "update topics set name = $1, description = $2, updated = now() where id = $3 returning id, name, description, created, updated",
                    &[Type::VARCHAR, Type::VARCHAR, Type::UUID],
                )
                .await
                .change_context(StatementPrepareError)?,
            patch_name: client
                .prepare_typed(
                    "update topics set name = $1, updated = now() where id = $2 returning id, name, description, created, updated",
                    &[Type::VARCHAR, Type::UUID],
                )
                .await
                .change_context(StatementPrepareError)?,
            patch_desc: client
                .prepare_typed(
                    "update topics set description = $1, updated = now() where id = $2 returning id, name, description, created, updated",
                    &[Type::VARCHAR, Type::UUID],
                )
                .await
                .change_context(StatementPrepareError)?,
            delete: client
                .prepare_typed(
                    "delete from topics where id = $1",
                    &[Type::UUID],
                )
                .await
                .change_context(StatementPrepareError)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetStatements {
    pub get: Statement,
    // pub list: Statement,
    pub create: Statement,
    // pub patch_name_desc: Statement,
    // pub patch_name: Statement,
    // pub patch_desc: Statement,
    // pub delete: Statement,
}

impl SetStatements {
    pub async fn prepare(client: &Client) -> Result<Self, Report<StatementPrepareError>> {
        Ok(Self {
            get: client
                .prepare_typed(
                    "select id, topic_id, name, description, created, updated from sets where id = $1 and topic_id = $2",
                    &[Type::UUID, Type::UUID]
                )
                .await
                .change_context(StatementPrepareError)?,
            create: client
                .prepare_typed(
                    "insert into sets (id, topic_id, name, description) values ($1, $2, $3, $4) returning id, topic_id, name, description, created, updated",
                    &[Type::UUID, Type::UUID, Type::VARCHAR, Type::VARCHAR],
                )
                .await
                .change_context(StatementPrepareError)?,
        })
    }
}
