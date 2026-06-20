use crate::{adapters::postgres::PostgresAdapter, error::AppResult};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct RebuildResult {
    pub rows_written: u64,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct MaintenanceService {
    postgres: PostgresAdapter,
}

#[allow(dead_code)]
impl MaintenanceService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn rebuild_current_translations(&self) -> AppResult<RebuildResult> {
        let mut tx = self.postgres.begin().await?;
        let rows_written = self
            .postgres
            .rebuild_current_translations_in_tx(&mut tx)
            .await?;
        tx.commit().await?;

        Ok(RebuildResult { rows_written })
    }

    pub async fn rebuild_namespace_language_stats(&self) -> AppResult<RebuildResult> {
        let mut tx = self.postgres.begin().await?;
        let rows_written = self
            .postgres
            .rebuild_namespace_language_stats_in_tx(&mut tx)
            .await?;
        tx.commit().await?;

        Ok(RebuildResult { rows_written })
    }
}
