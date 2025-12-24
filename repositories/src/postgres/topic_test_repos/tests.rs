use routing::pagination::Pagination;
use topics_core::{TopicRepository, list_filter::TopicListCriteria, model::NewTopic};

use crate::postgres::{topic_test_repos::InMemoryTopicsRepo, topics::TopicId};

const DEFAULT_PAGE_SIZE: u64 = 25;

#[tokio::test]
async fn in_memory_get_without_create_returns_none() {
    let repo = InMemoryTopicsRepo::default();

    assert_eq!(None, repo.get(TopicId::new()).await.unwrap())
}

#[tokio::test]
async fn in_memory_list_without_create_returns_empty_list() {
    let repo = InMemoryTopicsRepo::default();

    assert!(
        repo.list(TopicListCriteria::new(
            Pagination::default(),
            DEFAULT_PAGE_SIZE,
        ))
        .await
        .unwrap()
        .is_empty()
    );
}

#[tokio::test]
async fn in_memory_create_returns_topic_with_specified_attributes_and_adds_to_underlying_datastructure()
 {
    let repo = InMemoryTopicsRepo::default();
    let topic = repo
        .create(NewTopic::new("test", Some("test desc")))
        .await
        .unwrap();

    let db = repo.db.read().await;

    assert_eq!(Some(&topic), db.get(&topic.id));
}
