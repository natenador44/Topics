use testcontainers_modules::testcontainers::{ContainerAsync, Image};
use topics_core::TopicRepository;

mod topics;
mod sets;

struct TestRuntime<C, R>
where
    C: Image,
    R: TopicRepository,
{
    _container: ContainerAsync<C>,
    repo: R,
    new_id_fn: Box<dyn Fn() -> R::TopicId>,
}

impl<C, R> TestRuntime<C, R>
where
    C: Image,
    R: TopicRepository,
{
    fn new<F>(container: ContainerAsync<C>, repo: R, new_id_fn: F) -> Self
    where
        F: Fn() -> R::TopicId + 'static,
    {
        Self {
            _container: container,
            repo,
            new_id_fn: Box::new(new_id_fn),
        }
    }

    fn generate_new_id(&self) -> R::TopicId {
        (self.new_id_fn)()
    }
}

mod mongo {
    use crate::TestRuntime;
    use bson::oid::ObjectId;
    use testcontainers_modules::mongo::Mongo;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use repositories::mongodb::topics as mongo_repo;

    pub async fn runtime() -> TestRuntime<Mongo, mongo_repo::TopicRepo> {
        let mongo_container = container().await;
        let host = mongo_container.get_host().await.unwrap();
        let port = mongo_container.get_host_port_ipv4(27017).await.unwrap();

        let repo = mongo_repo::TopicRepo::init(mongo_repo::ConnectionDetails::Url(format!(
            "mongodb://{host}:{port}/?authSource=admin"
        )))
            .await
            .unwrap();

        TestRuntime::new(mongo_container, repo, || {
            mongo_repo::TopicId::new_with(ObjectId::new())
        })
    }

    async fn container() -> ContainerAsync<Mongo> {
        Mongo::default().start().await.unwrap()
    }
}

mod postgres {
    use crate::TestRuntime;
    use repositories::postgres::topics::{TopicId, TopicRepo};
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use repositories::postgres::ConnectionDetails;
    use repositories::postgres::topics as postgres_repo;

    pub async fn runtime() -> TestRuntime<Postgres, postgres_repo::TopicRepo> {
        let container = container().await;
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

        TestRuntime::new(container, repo, || TopicId::new())
    }

    async fn container() -> ContainerAsync<Postgres> {
        Postgres::default()
            .with_db_name("topics")
            .with_user("testuser")
            .with_password("testpass")
            .start()
            .await
            .unwrap()
    }
}