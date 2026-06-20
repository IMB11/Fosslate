use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct Language {
    /// Stable language key, usually a BCP 47-style locale such as `en`, `fr-FR`, or `pt-BR`.
    pub key: String,
    /// Human-readable language name shown in the UI.
    pub name: String,
}
