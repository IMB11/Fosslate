use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::app::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::auth::finish_sso,
        crate::routes::auth::forgot_password,
        crate::routes::auth::get_auth_providers,
        crate::routes::auth::get_auth_session,
        crate::routes::auth::login,
        crate::routes::auth::logout,
        crate::routes::auth::refresh_auth_session,
        crate::routes::auth::reset_password,
        crate::routes::auth::complete_signup,
        crate::routes::auth::start_signup,
        crate::routes::auth::start_sso,
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
        crate::routes::setup::check_setup_required,
        crate::routes::setup::complete_setup,
        crate::routes::setup::get_setup_status,
        crate::routes::setup::save_github_sso_setup,
        crate::routes::setup::save_gitlab_sso_setup,
        crate::routes::setup::test_email_delivery_setup,
        crate::routes::setup::verify_setup_secret,
        crate::routes::settings::claim_instance_admin,
        crate::routes::settings::get_instance_settings,
        crate::routes::settings::save_instance_sso_provider,
        crate::routes::settings::test_instance_email_delivery,
        crate::routes::stats::list_namespace_language_stats,
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
            crate::models::AuthProvidersResponse,
            crate::models::AuthUser,
            crate::models::AuthUserResponse,
            crate::models::InstanceSettings,
            crate::models::Language,
            crate::models::Namespace,
            crate::models::NamespaceLanguageStats,
            crate::models::Project,
            crate::models::ProjectTargetLanguage,
            crate::models::AuthProviderSetupStatus,
            crate::models::EmailDeliverySetupStatus,
            crate::models::SetupCompleteResponse,
            crate::models::SetupStatus,
            crate::models::SetupStep,
            crate::models::SsoProviderAvailability,
            crate::models::SsoProviders,
            crate::models::SourceString,
            crate::models::Translation,
            crate::models::User,
            crate::routes::approvals::ApproveTranslationRequest,
            crate::routes::auth::ForgotPasswordRequest,
            crate::routes::auth::LoginRequest,
            crate::routes::auth::ResetPasswordRequest,
            crate::routes::auth::SignupCompleteRequest,
            crate::routes::auth::SignupStartRequest,
            crate::routes::health::HealthResponse,
            crate::routes::languages::AddTargetLanguageRequest,
            crate::routes::meta::DatabaseStatus,
            crate::routes::meta::MetaResponse,
            crate::routes::namespaces::NamespaceRequest,
            crate::routes::projects::CreateProjectRequest,
            crate::routes::projects::UpdateProjectRequest,
            crate::routes::setup::SaveSsoProviderRequest,
            crate::routes::setup::TestEmailDeliveryRequest,
            crate::routes::settings::ClaimInstanceAdminRequest,
            crate::routes::settings::SaveInstanceSsoProviderRequest,
            crate::routes::settings::TestInstanceEmailDeliveryRequest,
            crate::routes::strings::SourceStringRequest,
            crate::routes::translations::CreateTranslationRequest,
            crate::routes::users::CreateUserRequest,
            crate::routes::votes::SetVoteRequest,
        )
    ),
    tags(
        (name = "approvals", description = "Approve translation candidates and remove approvals for string/language pairs."),
        (name = "auth", description = "Password, session, password reset, and SSO authentication endpoints."),
        (name = "health", description = "Process-level and dependency health endpoints."),
        (name = "languages", description = "Manage target languages within a project."),
        (name = "meta", description = "Application metadata and dependency status endpoints."),
        (name = "namespaces", description = "Group source strings within a project."),
        (name = "projects", description = "Create, read, update, and soft-delete translation projects."),
        (name = "setup", description = "First-run admin setup, SSO provider configuration, and Resend email delivery setup."),
        (name = "settings", description = "Authenticated user and instance settings."),
        (name = "stats", description = "Read project and namespace translation coverage stats."),
        (name = "strings", description = "Manage source-language strings within namespaces."),
        (name = "translations", description = "Create, list, and soft-delete candidate translations."),
        (name = "users", description = "Temporary users used as authors, voters, and reviewers."),
        (name = "votes", description = "Cast or replace votes on translation candidates."),
    ),
    info(
        title = "Fosslate API",
        description = "HTTP API for Fosslate setup, projects, namespaces, source strings, target languages, translation candidates, votes, approvals, users, and health checks.",
        version = env!("CARGO_PKG_VERSION"),
    )
)]
struct ApiDoc;

pub fn document() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_json))
        .merge(Scalar::with_url("/docs", document()))
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(document())
}
