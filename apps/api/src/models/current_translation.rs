use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct CurrentTranslation {
    pub project_id: i64,
    pub namespace_id: i64,
    pub string_id: i64,
    pub target_language_id: i64,
    pub current_translation_id: Option<i64>,
    pub approved_translation_id: Option<i64>,
    pub best_rated_translation_id: Option<i64>,
    pub candidate_count: i32,
    pub updated_at: DateTime<Utc>,
}

pub fn select_current_translation(
    approved_translation_id: Option<i64>,
    best_rated_translation_id: Option<i64>,
) -> Option<i64> {
    approved_translation_id.or(best_rated_translation_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_translation_wins_over_best_rated_translation() {
        assert_eq!(select_current_translation(Some(1), Some(2)), Some(1));
        assert_eq!(select_current_translation(None, Some(2)), Some(2));
    }
}
