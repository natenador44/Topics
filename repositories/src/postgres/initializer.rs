use crate::postgres::sets::SetRepo;
use crate::postgres::topics::TopicRepo;
use crate::postgres::{ConnectionDetails, RepoInitErr};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use error_stack::{IntoReport, Report, ResultExt};
use refinery::{Migration, Runner};
use std::str::FromStr;
use tokio_postgres::{Config, NoTls};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/postgres/migrations");
}

pub trait Init {
    type Repo;
    fn init(self, pool: Pool) -> impl Future<Output = Result<Self::Repo, Report<RepoInitErr>>>;
    fn migrations(&self) -> Result<Vec<Migration>, Report<RepoInitErr>>;
}
impl Init for () {
    type Repo = ();

    async fn init(self, _: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        Err(RepoInitErr("unknown").into_report())
            .attach("init was called without specifing a repo to initialize")
    }

    fn migrations(&self) -> Result<Vec<Migration>, Report<RepoInitErr>> {
        Err(RepoInitErr("unknown").into_report())
            .attach("migrations were gathered without specifing a repo to initialize")
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

    fn migrations(&self) -> Result<Vec<Migration>, Report<RepoInitErr>> {
        let mut migrations = self.0.migrations()?;
        migrations.extend(self.1.migrations()?);
        Ok(migrations)
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

    fn migrations(&self) -> Result<Vec<Migration>, Report<RepoInitErr>> {
        let mut migrations = self.0.migrations()?;
        migrations.extend(self.1.migrations()?);
        migrations.extend(self.2.migrations()?);
        Ok(migrations)
    }
}

pub struct TopicInit;
impl Init for TopicInit {
    type Repo = TopicRepo;

    async fn init(self, pool: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        TopicRepo::new(pool).await
    }

    fn migrations(&self) -> Result<Vec<Migration>, Report<RepoInitErr>> {
        refinery::load_sql_migrations("./src/postgres/migrations/topics")
            .change_context(RepoInitErr::topics())
    }
}

pub struct SetInit;
impl Init for SetInit {
    type Repo = SetRepo;

    async fn init(self, pool: Pool) -> Result<Self::Repo, Report<RepoInitErr>> {
        SetRepo::new(pool).await
    }

    fn migrations(&self) -> Result<Vec<Migration>, Report<RepoInitErr>> {
        refinery::load_sql_migrations("./src/postgres/migrations/sets")
            .change_context(RepoInitErr::sets())
    }
}

pub struct RepoInitializer<T: Init = ()> {
    initializer: T,
}

impl<T> RepoInitializer<T>
where
    T: Init,
{
    pub async fn init(
        self,
        connection_details: ConnectionDetails,
        pool_size: Option<usize>,
    ) -> Result<T::Repo, Report<RepoInitErr>> {
        let config = match connection_details {
            ConnectionDetails::Url(url) => {
                Config::from_str(&url).change_context(RepoInitErr::all())?
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
        let pool = pool_builder.build().change_context(RepoInitErr::all())?;

        self.run_migrations(&pool).await?;

        self.initializer.init(pool).await
    }

    // split into a separate method in case `pool_size` is 1. We need to get a hold of the only
    // connection, run the migrations, then drop it so it's freed up for initialization
    async fn run_migrations(&self, pool: &Pool) -> Result<(), Report<RepoInitErr>> {
        let mut handle = pool.get().await.change_context(RepoInitErr::sets())?;

        let client = &mut **handle;

        let migrations = self.initializer.migrations()?;

        Runner::new(&migrations)
            .run_async(client)
            .await
            .change_context(RepoInitErr::all())?;
        Ok(())
    }
}

impl Default for RepoInitializer<()> {
    fn default() -> Self {
        Self { initializer: () }
    }
}

impl RepoInitializer<()> {
    pub fn with_topics(self) -> RepoInitializer<TopicInit> {
        RepoInitializer {
            initializer: TopicInit,
        }
    }

    /// Sets currently rely on Topics as well, so we force both
    pub fn with_sets(self) -> RepoInitializer<(TopicInit, SetInit)> {
        RepoInitializer {
            initializer: (TopicInit, SetInit),
        }
    }
}

impl RepoInitializer<TopicInit> {
    pub fn with_sets(self) -> RepoInitializer<(TopicInit, SetInit)> {
        RepoInitializer {
            initializer: (TopicInit, SetInit),
        }
    }
}
