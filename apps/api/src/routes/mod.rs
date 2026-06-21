use axum::{Router, routing::get};

use crate::{app::AppState, openapi};

pub mod approvals;
pub mod auth;
pub mod health;
pub mod languages;
pub mod meta;
pub mod namespaces;
pub mod projects;
pub mod settings;
pub mod setup;
pub mod stats;
pub mod strings;
pub mod translations;
pub mod users;
pub mod votes;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/setup/check", get(setup::check_setup_required))
        .route("/api/v1/meta", get(meta::meta))
        .route("/api/v1/auth/providers", get(auth::get_auth_providers))
        .route(
            "/api/v1/auth/signup/start",
            axum::routing::post(auth::start_signup),
        )
        .route(
            "/api/v1/auth/signup/complete",
            axum::routing::post(auth::complete_signup),
        )
        .route("/api/v1/auth/login", axum::routing::post(auth::login))
        .route("/api/v1/auth/session", get(auth::get_auth_session))
        .route(
            "/api/v1/auth/session/refresh",
            axum::routing::post(auth::refresh_auth_session),
        )
        .route("/api/v1/auth/logout", axum::routing::post(auth::logout))
        .route(
            "/api/v1/auth/password/forgot",
            axum::routing::post(auth::forgot_password),
        )
        .route(
            "/api/v1/auth/password/reset",
            axum::routing::post(auth::reset_password),
        )
        .route(
            "/api/v1/auth/sso/{provider}/start",
            get(auth::start_sso),
        )
        .route(
            "/api/v1/auth/sso/{provider}/callback",
            get(auth::finish_sso),
        )
        .route("/api/v1/setup/verify", axum::routing::post(setup::verify_setup_secret))
        .route("/api/v1/setup/status", get(setup::get_setup_status))
        .route(
            "/api/v1/setup/sso/github",
            axum::routing::put(setup::save_github_sso_setup),
        )
        .route(
            "/api/v1/setup/sso/gitlab",
            axum::routing::put(setup::save_gitlab_sso_setup),
        )
        .route(
            "/api/v1/setup/email/test",
            axum::routing::post(setup::test_email_delivery_setup),
        )
        .route(
            "/api/v1/setup/complete",
            axum::routing::post(setup::complete_setup),
        )
        .route(
            "/api/v1/settings/instance/admin/claim",
            axum::routing::post(settings::claim_instance_admin),
        )
        .route(
            "/api/v1/settings/instance",
            get(settings::get_instance_settings),
        )
        .route(
            "/api/v1/settings/instance/sso/{provider}",
            axum::routing::put(settings::save_instance_sso_provider),
        )
        .route(
            "/api/v1/settings/instance/email/test",
            axum::routing::post(settings::test_instance_email_delivery),
        )
        .route(
            "/api/v1/users",
            get(users::list_users).post(users::create_user),
        )
        .route("/api/v1/users/{user_id}", get(users::get_user))
        .route(
            "/api/v1/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/api/v1/projects/{project_public_id}",
            get(projects::get_project)
                .put(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/api/v1/projects/{project_public_id}/languages",
            get(languages::list_target_languages).post(languages::add_target_language),
        )
        .route(
            "/api/v1/projects/{project_public_id}/languages/{target_language_id}",
            axum::routing::delete(languages::remove_target_language),
        )
        .route(
            "/api/v1/projects/{project_public_id}/namespaces",
            get(namespaces::list_namespaces).post(namespaces::create_namespace),
        )
        .route(
            "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}",
            get(namespaces::get_namespace)
                .put(namespaces::update_namespace)
                .delete(namespaces::delete_namespace),
        )
        .route(
            "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings",
            get(strings::list_source_strings).post(strings::create_source_string),
        )
        .route(
            "/api/v1/projects/{project_public_id}/stats/namespaces",
            get(stats::list_namespace_language_stats),
        )
        .route(
            "/api/v1/projects/{project_public_id}/strings/{string_id}",
            get(strings::get_source_string)
                .put(strings::update_source_string)
                .delete(strings::delete_source_string),
        )
        .route(
            "/api/v1/projects/{project_public_id}/strings/{string_id}/translations",
            get(translations::list_translations).post(translations::create_translation),
        )
        .route(
            "/api/v1/projects/{project_public_id}/translations/{translation_id}",
            axum::routing::delete(translations::delete_translation),
        )
        .route(
            "/api/v1/projects/{project_public_id}/translations/{translation_id}/vote",
            axum::routing::put(votes::set_vote),
        )
        .route(
            "/api/v1/projects/{project_public_id}/translations/{translation_id}/approval",
            axum::routing::put(approvals::approve_translation),
        )
        .route(
            "/api/v1/projects/{project_public_id}/strings/{string_id}/approvals/{target_language_id}",
            axum::routing::delete(approvals::remove_approval),
        )
        .merge(openapi::router())
}
