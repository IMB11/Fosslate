use crate::{
    error::AppResult,
    services::{Services, maintenance::RebuildResult},
};

#[allow(dead_code)]
pub async fn rebuild_current_translations(services: &Services) -> AppResult<RebuildResult> {
    services.maintenance.rebuild_current_translations().await
}
