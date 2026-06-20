use sqlx::{Postgres, Transaction};

use super::PostgresAdapter;

impl PostgresAdapter {
    pub async fn get_translation_vote_for_update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        translation_id: i64,
        user_id: i64,
    ) -> Result<Option<i16>, sqlx::Error> {
        sqlx::query_scalar::<_, i16>(
            r#"
            SELECT vote
            FROM translation_votes
            WHERE translation_id = $1
              AND user_id = $2
            FOR UPDATE
            "#,
        )
        .bind(translation_id)
        .bind(user_id)
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn upsert_translation_vote_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        translation_id: i64,
        user_id: i64,
        vote: i16,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO translation_votes (translation_id, user_id, vote)
            VALUES ($1, $2, $3)
            ON CONFLICT (translation_id, user_id)
            DO UPDATE SET vote = EXCLUDED.vote
            "#,
        )
        .bind(translation_id)
        .bind(user_id)
        .bind(vote)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
