use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct CurrentTranslation {
    /// Internal numeric project ID that owns this projection row.
    pub project_id: i64,
    /// Internal numeric namespace ID that contains the source string.
    pub namespace_id: i64,
    /// Internal numeric source string ID.
    pub string_id: i64,
    /// Internal numeric target language ID.
    pub target_language_id: i64,
    /// Translation candidate currently selected for display. Approved candidates win over best-rated candidates.
    pub current_translation_id: Option<i64>,
    /// Approved translation candidate ID, if a reviewer has approved one.
    pub approved_translation_id: Option<i64>,
    /// Highest-rated active translation candidate ID, if any candidates exist.
    pub best_rated_translation_id: Option<i64>,
    /// Number of active candidate translations for this source string and target language.
    pub candidate_count: i32,
    /// Time this projection row was last recalculated.
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
