use crate::models::User;

use super::PostgresAdapter;

impl PostgresAdapter {
    pub async fn create_user(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username)
            VALUES ($1)
            RETURNING id, username, created_at, updated_at
            "#,
        )
        .bind(username)
        .fetch_one(self.pool())
        .await
    }

    pub async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, created_at, updated_at
            FROM users
            ORDER BY id
            "#,
        )
        .fetch_all(self.pool())
        .await
    }

    pub async fn get_user(&self, id: i64) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool())
        .await
    }
}
