use crate::{RepoInitErr, RepoInitResult};
use error_stack::ResultExt;
use tokio_postgres::{Client, Config};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/migrations/");
}

pub async fn run(client: &mut Client) -> RepoInitResult<()> {
    embedded::migrations::runner()
        .run_async(client)
        .await
        .change_context(RepoInitErr)?;
    Ok(())
}
