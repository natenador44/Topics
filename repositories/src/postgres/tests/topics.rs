use crate::postgres::{ConnectionDetails, init};
use engine::models::TopicId;
use engine::repository::TopicsRepository;
use rstest::*;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use tokio::runtime::Handle;

#[rstest]
#[tokio::test]
async fn get_topic_no_data_returns_none(container: Postgres) {
    let node = container.start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();

    let repo = init(
        Handle::current(),
        ConnectionDetails::Url(format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            port
        )),
    )
    .await
    .unwrap();

    let topic = repo.find(TopicId::new()).await.unwrap();
    assert!(topic.is_none());
}

#[fixture]
fn container() -> Postgres {
    Postgres::default()
}
