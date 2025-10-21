use testcontainers_modules::testcontainers::{ContainerAsync, Image};
use topics_core::TopicRepository;

mod topics;

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
        where F: Fn() -> R::TopicId + 'static,
    {
        Self {_container: container, repo, new_id_fn: Box::new(new_id_fn) }
    }

    fn generate_new_id(&self) -> R::TopicId {
        (self.new_id_fn)()
    }
}



