use axum::{routing::get, Json, Router};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::app::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::approvals::approve_translation,
        crate::routes::approvals::remove_approval,
        crate::routes::health::health,
        crate::routes::languages::add_target_language,
        crate::routes::languages::list_target_languages,
        crate::routes::languages::remove_target_language,
        crate::routes::meta::meta,
        crate::routes::namespaces::create_namespace,
        crate::routes::namespaces::delete_namespace,
        crate::routes::namespaces::get_namespace,
        crate::routes::namespaces::list_namespaces,
        crate::routes::namespaces::update_namespace,
        crate::routes::projects::create_project,
        crate::routes::projects::delete_project,
        crate::routes::projects::get_project,
        crate::routes::projects::list_projects,
        crate::routes::projects::update_project,
        crate::routes::strings::create_source_string,
        crate::routes::strings::delete_source_string,
        crate::routes::strings::get_source_string,
        crate::routes::strings::list_source_strings,
        crate::routes::strings::update_source_string,
        crate::routes::translations::create_translation,
        crate::routes::translations::delete_translation,
        crate::routes::translations::list_translations,
        crate::routes::users::create_user,
        crate::routes::users::get_user,
        crate::routes::users::list_users,
        crate::routes::votes::set_vote,
    ),
    components(
        schemas(
            crate::error::ErrorBody,
            crate::models::CurrentTranslation,
            crate::models::Language,
            crate::models::Namespace,
            crate::models::Project,
            crate::models::ProjectTargetLanguage,
            crate::models::SourceString,
            crate::models::Translation,
            crate::models::User,
            crate::routes::approvals::ApproveTranslationRequest,
            crate::routes::health::HealthResponse,
            crate::routes::languages::AddTargetLanguageRequest,
            crate::routes::meta::DatabaseStatus,
            crate::routes::meta::MetaResponse,
            crate::routes::namespaces::NamespaceRequest,
            crate::routes::projects::CreateProjectRequest,
            crate::routes::projects::UpdateProjectRequest,
            crate::routes::strings::SourceStringRequest,
            crate::routes::translations::CreateTranslationRequest,
            crate::routes::users::CreateUserRequest,
            crate::routes::votes::SetVoteRequest,
        )
    ),
    tags(
        (name = "approvals", description = "Translation approval endpoints"),
        (name = "health", description = "Health check endpoints"),
        (name = "languages", description = "Project target language endpoints"),
        (name = "meta", description = "Application metadata endpoints"),
        (name = "namespaces", description = "Namespace endpoints"),
        (name = "projects", description = "Project endpoints"),
        (name = "strings", description = "Source string endpoints"),
        (name = "translations", description = "Translation candidate endpoints"),
        (name = "users", description = "Temporary user endpoints"),
        (name = "votes", description = "Translation vote endpoints"),
    ),
    info(
        title = "Fosslate API",
        description = "HTTP API for Fosslate",
        version = env!("CARGO_PKG_VERSION"),
    )
)]
struct ApiDoc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_json))
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
