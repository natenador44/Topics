use rstest::rstest;
use testcontainers_modules::testcontainers::Image;
use topics_core::TopicRepository;
use super::{postgres, TestRuntime};

#[rstest]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn get_no_data_returns_none<C, R>(#[future(awt)] #[case] runtime: TestRuntime<C, R>)
where
    C: Image,
    R: TopicRepository,
{

}