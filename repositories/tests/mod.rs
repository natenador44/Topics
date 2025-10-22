mod sets;
mod topics;

mod postgres {
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    pub async fn container() -> ContainerAsync<Postgres> {
        Postgres::default()
            .with_db_name("topics")
            .with_user("testuser")
            .with_password("testpass")
            .start()
            .await
            .unwrap()
    }
}
