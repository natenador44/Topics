use std::sync::Arc;

use repositories::postgres::topics::{TopicId, TopicRepo};
use rstest::fixture;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};
use tokio_postgres::{Client, NoTls, connect};

#[rstest]
#[tokio::test]
async fn get_no_data_returns_none(#[future] client: Arc<Client>) {
    let client = client.await;
    let repo = TopicRepo::new(Arc::clone(&client)).await.unwrap();

    let result = repo.get(TopicId::new()).unwrap();

    assert!(result.is_none());
}

#[fixture]
async fn client(#[future] container: ContainerAsync<Postgres>) -> Arc<Client> {
    let container = container.await;
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let (client, conn) = connect(&format!("postgresql://{host}:{port}/topics"), NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection await error: {e:?}");
        }
    });

    client
}

#[fixture]
async fn container() -> ContainerAsync<Postgres> {
    Postgres::default()
        .with_db_name("topics")
        .start()
        .await
        .unwrap()
}
