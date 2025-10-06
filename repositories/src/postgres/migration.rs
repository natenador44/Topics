use crate::{RepoInitErr, RepoInitResult};
use error_stack::ResultExt;
use tokio_postgres::Client;

mod embedded {
    use refinery::embed_migrations;
    // need to force recompilation of this (add a line or whitespace and build/run again, or run cargo clean
    embed_migrations!("./src/postgres/migrations/");
}

pub async fn run(client: &mut Client) -> RepoInitResult<()> {
    embedded::migrations::runner()
        .run_async(client)
        .await
        .change_context(RepoInitErr)?;
    Ok(())
}
