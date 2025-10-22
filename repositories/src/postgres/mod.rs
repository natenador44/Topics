// #[cfg(feature = "postgres-topics")]
pub mod initializer;
pub mod sets;
mod statements;
pub mod topics;

pub enum ConnectionDetails {
    Url(String),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize postgres {0} repo")]
pub struct RepoInitErr(&'static str);
impl RepoInitErr {
    fn topics() -> Self {
        Self("topics")
    }

    fn sets() -> Self {
        Self("sets")
    }

    fn all() -> Self {
        Self("all")
    }
}
