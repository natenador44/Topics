use std::fmt::Debug;

pub trait Repository: Clone + Send + Sync + Debug {
    type TopicRepo: TopicRepository;
    type IdentifierRepo: IdentifierRepository;
    type SetRepo: SetRepository;

    fn topic_repository(&self) -> &Self::TopicRepo;
    fn identifier_repository(&self) -> &Self::IdentifierRepo;
    fn set_repository(&self) -> &Self::SetRepo;
}

pub trait TopicRepository {}

pub trait IdentifierRepository {}

pub trait SetRepository {}
