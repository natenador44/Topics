use crate::postgres::sets::SetRepo;
use crate::postgres::topics::TopicRepo;
use crate::postgres::{ConnectionDetails, RepoInitErr, RepoMigrationErr};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use error_stack::{IntoReport, Report, ResultExt};
use std::str::FromStr;
use tokio_postgres::{Client, Config, NoTls};
use tracing::debug;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/postgres/migrations");
}

pub trait Init {
    type Repo;
    fn init(self, pool: Pool) -> impl Future<Output = Result<Self::Repo, Report<RepoInitErr>>>;
    fn run_migrations(
        &self,
        client: &mut Client,
    ) -> impl Future<Output = Result<(), Report<RepoMigrationErr>>>;
}
impl Init for () {
    type Repo = ();

    async fn init(self, _: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        Err(RepoInitErr("unknown").into_report())
            .attach("init was called without specifing a repo to initialize")
    }

    async fn run_migrations(&self, _: &mut Client) -> Result<(), Report<RepoMigrationErr>> {
        Err(RepoMigrationErr.into_report())
            .attach("migrations were invoked without specifying a repo to initialize")
    }
}

impl<T1, T2> Init for (T1, T2)
where
    T1: Init,
    T2: Init,
{
    type Repo = (T1::Repo, T2::Repo);

    async fn init(self, pool: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        let r1 = self.0.init(pool.clone()).await?;
        let r2 = self.1.init(pool).await?;
        Ok((r1, r2))
    }

    async fn run_migrations(&self, client: &mut Client) -> Result<(), Report<RepoMigrationErr>> {
        self.0.run_migrations(client).await?;
        self.1.run_migrations(client).await
    }
}

impl<T1, T2, T3> Init for (T1, T2, T3)
where
    T1: Init,
    T2: Init,
    T3: Init,
{
    type Repo = (T1::Repo, T2::Repo, T3::Repo);

    async fn init(self, pool: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        let r1 = self.0.init(pool.clone()).await?;
        let r2 = self.1.init(pool.clone()).await?;
        let r3 = self.2.init(pool).await?;
        Ok((r1, r2, r3))
    }

    async fn run_migrations(&self, client: &mut Client) -> Result<(), Report<RepoMigrationErr>> {
        self.0.run_migrations(client).await?;
        self.1.run_migrations(client).await?;
        self.2.run_migrations(client).await
    }
}

pub struct TopicInit;
impl Init for TopicInit {
    type Repo = TopicRepo;

    async fn init(self, pool: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        TopicRepo::new(pool).await
    }

    async fn run_migrations(&self, client: &mut Client) -> Result<(), Report<RepoMigrationErr>> {
        embedded::migrations::runner()
            .run_async(client)
            .await
            .change_context(RepoMigrationErr)
            .attach("topics repo")?;
        Ok(())
    }
}

pub struct SetInit;
impl Init for SetInit {
    type Repo = SetRepo;

    async fn init(self, pool: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        SetRepo::new(pool).await
    }

    async fn run_migrations(&self, client: &mut Client) -> Result<(), Report<RepoMigrationErr>> {
        embedded::migrations::runner()
            .run_async(client)
            .await
            .change_context(RepoMigrationErr)
            .attach("sets repo")?;
        Ok(())
    }
}

pub struct RepoCreator<T: Init = ()> {
    initializer: T,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create repos")]
pub struct RepoCreationErr;

impl<T> RepoCreator<T>
where
    T: Init,
{
    pub async fn create(
        self,
        connection_details: ConnectionDetails,
        pool_size: Option<usize>,
    ) -> Result<T::Repo, Report<RepoCreationErr>> {
        let config = match connection_details {
            ConnectionDetails::Url(url) => {
                Config::from_str(&url).change_context(RepoCreationErr)?
            }
        };

        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(config, NoTls, mgr_config);
        let mut pool_builder = Pool::builder(mgr);
        if let Some(pool_size) = pool_size {
            pool_builder = pool_builder.max_size(pool_size);
        }
        debug!("building connection pool..");
        let pool = pool_builder.build().change_context(RepoCreationErr)?;
        debug!("connection pool built, running migrations");

        self.run_migrations(&pool)
            .await
            .change_context(RepoCreationErr)?;

        self.initializer
            .init(pool)
            .await
            .change_context(RepoCreationErr)
    }

    // split into a separate method in case `pool_size` is 1. We need to get a hold of the only
    // connection, run the migrations, then drop it so it's freed up for initialization
    async fn run_migrations(&self, pool: &Pool) -> Result<(), Report<RepoMigrationErr>> {
        let mut handle = pool.get().await.change_context(RepoMigrationErr)?;

        let client = &mut **handle;

        self.initializer.run_migrations(client).await
    }
}

impl Default for RepoCreator<()> {
    fn default() -> Self {
        Self { initializer: () }
    }
}

impl RepoCreator<()> {
    pub fn with_topics(self) -> RepoCreator<TopicInit> {
        RepoCreator {
            initializer: TopicInit,
        }
    }

    /// Sets currently rely on Topics as well, so we force both
    pub fn with_sets(self) -> RepoCreator<(TopicInit, SetInit)> {
        RepoCreator {
            initializer: (TopicInit, SetInit),
        }
    }
}

impl RepoCreator<TopicInit> {
    pub fn with_sets(self) -> RepoCreator<(TopicInit, SetInit)> {
        RepoCreator {
            initializer: (TopicInit, SetInit),
        }
    }
}
