use crate::{
    adapters::postgres::PostgresAdapter,
    error::AppResult,
    models::User,
};

#[derive(Clone)]
pub struct UserService {
    postgres: PostgresAdapter,
}

impl UserService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn create_user(&self, username: String) -> AppResult<User> {
        Ok(self.postgres.create_user(username.trim()).await?)
    }

    pub async fn list_users(&self) -> AppResult<Vec<User>> {
        Ok(self.postgres.list_users().await?)
    }

    pub async fn get_user(&self, id: i64) -> AppResult<User> {
        Ok(self.postgres.get_user(id).await?)
    }
}
