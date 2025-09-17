#[derive(Debug, Clone)]
pub struct SetService<T> {
    repo: T,
}

impl<T> SetService<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }
}
