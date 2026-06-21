use serde::Serialize;

use crate::models::{AuthProviderSetupStatus, EmailDeliverySetupStatus};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct InstanceSettings {
    pub github: AuthProviderSetupStatus,
    pub gitlab: AuthProviderSetupStatus,
    pub email: EmailDeliverySetupStatus,
}
