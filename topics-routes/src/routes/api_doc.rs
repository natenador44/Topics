pub mod examples {
    use serde_json::Value;
    use std::sync::LazyLock;
    use topics_core::model::Topic;

    pub mod create {
        use super::*;
        use crate::routes::responses::BulkCreateResponse;
        use topics_core::{CreateManyFailReason, CreateManyTopicStatus};
        static BULK_ALL_SUCCESS: LazyLock<Value> = LazyLock::new(|| {
            serde_json::to_value(BulkCreateResponse::new(vec![
                CreateManyTopicStatus::Success(Topic::create(
                    "some-id1",
                    "example1".to_string(),
                    None,
                )),
                CreateManyTopicStatus::Success(Topic::create(
                    "some-id2",
                    "example2".to_string(),
                    None,
                )),
            ]))
            .expect("bulk create response is serializable to Value")
        });

        pub fn bulk_all_success() -> &'static Value {
            &*BULK_ALL_SUCCESS
        }

        static BULK_MIXED_SUCCESS: LazyLock<Value> = LazyLock::new(|| {
            serde_json::to_value(BulkCreateResponse::new(vec![
                CreateManyTopicStatus::Success(Topic::create(
                    "some-id1",
                    "example1".to_string(),
                    Some("this topic was successfully created".to_string()),
                )),
                CreateManyTopicStatus::Fail {
                    topic_name: Some("failed topic".to_string()),
                    topic_description: Some("this topic could not be created".to_string()),
                    reason: CreateManyFailReason::ServiceError,
                }, // this is the only reason so far
            ]))
            .expect("bulk create response is serializable to Value")
        });

        pub fn bulk_mixed_success() -> &'static Value {
            &*BULK_MIXED_SUCCESS
        }

        static BULK_NO_SUCCESS: LazyLock<Value> = LazyLock::new(|| {
            serde_json::to_value(BulkCreateResponse::<&'static str>::new(vec![
                CreateManyTopicStatus::Fail {
                    topic_name: Some("failed topic1".to_string()),
                    topic_description: Some("this topic could not be created".to_string()),
                    reason: CreateManyFailReason::ServiceError,
                }, // this is the only reason so far
                CreateManyTopicStatus::Fail {
                    topic_name: Some("failed topic2".to_string()),
                    topic_description: Some("this topic could not be created".to_string()),
                    reason: CreateManyFailReason::ServiceError,
                }, // this is the only reason so far
            ]))
            .expect("bulk create response is serializable to Value")
        });

        pub fn bulk_no_success() -> &'static Value {
            &*BULK_NO_SUCCESS
        }
    }
}
