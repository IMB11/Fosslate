use crate::{
    error::AppResult,
    services::{Services, maintenance::RebuildResult},
};

#[allow(dead_code)]
pub async fn rebuild_namespace_language_stats(services: &Services) -> AppResult<RebuildResult> {
    services.maintenance.rebuild_namespace_language_stats().await
}
