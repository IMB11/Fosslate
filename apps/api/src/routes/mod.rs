use axum::{routing::get, Router};

use crate::{app::AppState, openapi};

pub mod approvals;
pub mod health;
pub mod languages;
pub mod meta;
pub mod namespaces;
pub mod projects;
pub mod strings;
pub mod translations;
pub mod users;
pub mod votes;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/meta", get(meta::meta))
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
