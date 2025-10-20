use repositories::postgres::topics::{ConnectionDetails, TopicId, TopicRepo};
use rstest::{fixture, rstest};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};
use topics_core::TopicRepository;

struct TestRuntime {
    _container: ContainerAsync<Postgres>,
    repo: TopicRepo,
}

#[rstest]
#[tokio::test]
async fn get_no_data_returns_none(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let result = repo.get(TopicId::new()).await.unwrap();

    assert!(result.is_none());
}

#[fixture]
async fn runtime(#[future] container: ContainerAsync<Postgres>) -> TestRuntime {
    let container = container.await;
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let repo = TopicRepo::init_with_pool_size(
        ConnectionDetails::Url(format!(
            "postgresql://testuser:testpass@{host}:{port}/topics"
        )),
        1,
    )
    .await
    .unwrap();

    TestRuntime {
        _container: container,
        repo,
    }
}

#[fixture]
async fn container() -> ContainerAsync<Postgres> {
    Postgres::default()
        .with_db_name("topics")
        .with_user("testuser")
        .with_password("testpass")
        .start()
        .await
        .unwrap()
}
