mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::Row;

use common::TestApi;

#[tokio::test]
async fn migrations_create_core_schema_tables() {
    let api = TestApi::spawn().await;

    let tables = [
        "users",
        "projects",
        "project_target_languages",
        "namespaces",
        "source_strings",
        "translations",
        "translation_votes",
        "translation_approvals",
        "current_translations",
        "namespace_language_stats",
    ];

    for table in tables {
        let row = sqlx::query("SELECT to_regclass($1)::text AS table_name")
            .bind(format!("public.{table}"))
            .fetch_one(api.pool())
            .await
            .unwrap();

        assert_eq!(
            row.try_get::<Option<String>, _>("table_name").unwrap(),
            Some(table.to_owned()),
            "expected {table} table to exist after migrations",
        );
    }

    api.cleanup().await;
}

#[tokio::test]
async fn users_can_be_created_listed_and_fetched() {
    let api = TestApi::spawn().await;

    let created = api
        .post_json("/api/v1/users", &json!({ "username": "calum" }))
        .await;
    assert_eq!(created.status(), StatusCode::CREATED);

    let user: Value = created.json().await;
    assert_eq!(user["username"], "calum");
    assert!(user["id"].as_i64().is_some());

    let list = api.get("/api/v1/users").await;
    assert_eq!(list.status(), StatusCode::OK);

    let users: Value = list.json().await;
    let users = users.as_array().unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["id"], user["id"]);
    assert_eq!(users[0]["username"], "calum");

    let fetched = api
        .get(&format!("/api/v1/users/{}", user["id"].as_i64().unwrap()))
        .await;
    assert_eq!(fetched.status(), StatusCode::OK);

    let fetched_user: Value = fetched.json().await;
    assert_eq!(fetched_user["id"], user["id"]);
    assert_eq!(fetched_user["username"], "calum");

    api.cleanup().await;
}

#[tokio::test]
async fn duplicate_usernames_are_rejected() {
    let api = TestApi::spawn().await;

    let first = api
        .post_json("/api/v1/users", &json!({ "username": "duplicate" }))
        .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let duplicate = api
        .post_json("/api/v1/users", &json!({ "username": "duplicate" }))
        .await;

    assert_ne!(duplicate.status(), StatusCode::CREATED);

    let list = api.get("/api/v1/users").await;
    assert_eq!(list.status(), StatusCode::OK);

    let users: Value = list.json().await;
    assert_eq!(users.as_array().unwrap().len(), 1);

    api.cleanup().await;
}

#[tokio::test]
async fn missing_user_returns_not_found() {
    let api = TestApi::spawn().await;

    let missing = api.get("/api/v1/users/999999").await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);

    api.cleanup().await;
}

#[tokio::test]
async fn projects_can_be_created_listed_fetched_and_updated() {
    let api = TestApi::spawn().await;

    let created = create_project(&api, "Fosslate", None, "en", "English").await;
    assert_eq!(created["name"], "Fosslate");
    assert_eq!(created["icon_asset_id"], Value::Null);
    assert_eq!(created["source_language"]["key"], "en");
    assert_eq!(created["source_language"]["name"], "English");

    let public_id = created["public_id"].as_str().unwrap();

    let list = api.get("/api/v1/projects").await;
    assert_eq!(list.status(), StatusCode::OK);

    let projects: Value = list.json().await;
    let projects = projects.as_array().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["public_id"], public_id);

    let fetched = api.get(&format!("/api/v1/projects/{public_id}")).await;
    assert_eq!(fetched.status(), StatusCode::OK);

    let fetched_project: Value = fetched.json().await;
    assert_eq!(fetched_project["public_id"], public_id);
    assert_eq!(fetched_project["name"], "Fosslate");

    let updated = api
        .put_json(
            &format!("/api/v1/projects/{public_id}"),
            &json!({
                "name": "Fosslate Updated",
                "icon_asset_id": 42,
                "source_language": {
                    "key": "en-GB",
                    "name": "English (UK)"
                }
            }),
        )
        .await;
    assert_eq!(updated.status(), StatusCode::OK);

    let updated_project: Value = updated.json().await;
    assert_eq!(updated_project["public_id"], public_id);
    assert_eq!(updated_project["name"], "Fosslate Updated");
    assert_eq!(updated_project["icon_asset_id"], 42);
    assert_eq!(updated_project["source_language"]["key"], "en-GB");
    assert_eq!(updated_project["source_language"]["name"], "English (UK)");

    api.cleanup().await;
}

#[tokio::test]
async fn deleted_projects_are_hidden_from_reads_and_updates() {
    let api = TestApi::spawn().await;

    let project = create_project(&api, "Disposable", Some(7), "fr", "French").await;
    let public_id = project["public_id"].as_str().unwrap();

    let deleted = api.delete(&format!("/api/v1/projects/{public_id}")).await;
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let fetched = api.get(&format!("/api/v1/projects/{public_id}")).await;
    assert_eq!(fetched.status(), StatusCode::NOT_FOUND);

    let list = api.get("/api/v1/projects").await;
    assert_eq!(list.status(), StatusCode::OK);

    let projects: Value = list.json().await;
    assert!(projects.as_array().unwrap().is_empty());

    let updated = api
        .put_json(
            &format!("/api/v1/projects/{public_id}"),
            &json!({
                "name": "Should Not Return",
                "icon_asset_id": null,
                "source_language": {
                    "key": "fr",
                    "name": "French"
                }
            }),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::NOT_FOUND);

    api.cleanup().await;
}

async fn create_project(
    api: &TestApi,
    name: &str,
    icon_asset_id: Option<i64>,
    source_language_key: &str,
    source_language_name: &str,
) -> Value {
    let created = api
        .post_json(
            "/api/v1/projects",
            &json!({
                "name": name,
                "icon_asset_id": icon_asset_id,
                "source_language": {
                    "key": source_language_key,
                    "name": source_language_name
                }
            }),
        )
        .await;

    assert_eq!(created.status(), StatusCode::CREATED);
    created.json().await
}
