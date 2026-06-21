mod common;

use axum::http::StatusCode;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;

use common::{TestApi, TestAuthCookies};

#[tokio::test]
async fn migrations_create_core_schema_tables() {
    let api = TestApi::spawn().await;

    let tables = [
        "users",
        "project_language_proofreaders",
        "projects",
        "project_target_languages",
        "namespaces",
        "source_strings",
        "translations",
        "translation_votes",
        "translation_approvals",
        "current_translations",
        "namespace_language_stats",
        "instance_setup",
        "auth_provider_configs",
        "email_delivery_config",
        "auth_identities",
        "auth_sessions",
        "password_reset_tokens",
        "oauth_login_states",
        "auth_attempts",
        "signup_email_verifications",
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

    let is_admin_column = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'users'
              AND column_name = 'is_admin'
        ) AS exists
        "#,
    )
    .fetch_one(api.pool())
    .await
    .unwrap();
    assert!(is_admin_column.try_get::<bool, _>("exists").unwrap());

    api.cleanup().await;
}

#[tokio::test]
async fn users_can_be_created_listed_and_fetched() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api).await;

    let created = api
        .post_json_with_auth("/api/v1/users", &json!({ "username": "calum" }), &cookies)
        .await;
    assert_eq!(created.status(), StatusCode::CREATED);

    let user: Value = created.json().await;
    assert_eq!(user["username"], "calum");
    assert!(user["id"].as_i64().is_some());

    let list = api.get_with_auth("/api/v1/users", &cookies).await;
    assert_eq!(list.status(), StatusCode::OK);

    let users: Value = list.json().await;
    let users = users.as_array().unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["id"], user["id"]);
    assert_eq!(users[0]["username"], "calum");

    let fetched = api
        .get_with_auth(
            &format!("/api/v1/users/{}", user["id"].as_i64().unwrap()),
            &cookies,
        )
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
    let cookies = signup(&api).await;

    let first = api
        .post_json_with_auth(
            "/api/v1/users",
            &json!({ "username": "duplicate" }),
            &cookies,
        )
        .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let duplicate = api
        .post_json_with_auth(
            "/api/v1/users",
            &json!({ "username": "duplicate" }),
            &cookies,
        )
        .await;

    assert_ne!(duplicate.status(), StatusCode::CREATED);

    let list = api.get_with_auth("/api/v1/users", &cookies).await;
    assert_eq!(list.status(), StatusCode::OK);

    let users: Value = list.json().await;
    assert_eq!(users.as_array().unwrap().len(), 1);

    api.cleanup().await;
}

#[tokio::test]
async fn missing_user_returns_not_found() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api).await;

    let missing = api.get_with_auth("/api/v1/users/999999", &cookies).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);

    api.cleanup().await;
}

#[tokio::test]
async fn projects_can_be_created_listed_fetched_and_updated() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api).await;

    let created = create_project(&api, &cookies, "Fosslate", None, "en", "English").await;
    assert_eq!(created["name"], "Fosslate");
    assert_eq!(created["icon_asset_id"], Value::Null);
    assert_eq!(created["source_language"]["key"], "en");
    assert_eq!(created["source_language"]["name"], "English");

    let public_id = created["public_id"].as_str().unwrap();

    let list = api.get_with_auth("/api/v1/projects", &cookies).await;
    assert_eq!(list.status(), StatusCode::OK);

    let projects: Value = list.json().await;
    let projects = projects.as_array().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["public_id"], public_id);

    let fetched = api
        .get_with_auth(&format!("/api/v1/projects/{public_id}"), &cookies)
        .await;
    assert_eq!(fetched.status(), StatusCode::OK);

    let fetched_project: Value = fetched.json().await;
    assert_eq!(fetched_project["public_id"], public_id);
    assert_eq!(fetched_project["name"], "Fosslate");

    let updated = api
        .put_json_with_auth(
            &format!("/api/v1/projects/{public_id}"),
            &json!({
                "name": "Fosslate Updated",
                "icon_asset_id": 42,
                "source_language": {
                    "key": "en-GB",
                    "name": "English (UK)"
                }
            }),
            &cookies,
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
    let cookies = signup(&api).await;

    let project = create_project(&api, &cookies, "Disposable", Some(7), "fr", "French").await;
    let public_id = project["public_id"].as_str().unwrap();

    let deleted = api
        .delete_with_auth(&format!("/api/v1/projects/{public_id}"), &cookies)
        .await;
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let fetched = api
        .get_with_auth(&format!("/api/v1/projects/{public_id}"), &cookies)
        .await;
    assert_eq!(fetched.status(), StatusCode::NOT_FOUND);

    let list = api.get_with_auth("/api/v1/projects", &cookies).await;
    assert_eq!(list.status(), StatusCode::OK);

    let projects: Value = list.json().await;
    assert!(projects.as_array().unwrap().is_empty());

    let updated = api
        .put_json_with_auth(
            &format!("/api/v1/projects/{public_id}"),
            &json!({
                "name": "Should Not Return",
                "icon_asset_id": null,
                "source_language": {
                    "key": "fr",
                    "name": "French"
                }
            }),
            &cookies,
        )
        .await;
    assert_eq!(updated.status(), StatusCode::NOT_FOUND);

    api.cleanup().await;
}

async fn create_project(
    api: &TestApi,
    cookies: &TestAuthCookies,
    name: &str,
    icon_asset_id: Option<i64>,
    source_language_key: &str,
    source_language_name: &str,
) -> Value {
    let created = api
        .post_json_with_auth(
            "/api/v1/projects",
            &json!({
                "name": name,
                "icon_asset_id": icon_asset_id,
                "source_language": {
                    "key": source_language_key,
                    "name": source_language_name
                }
            }),
            cookies,
        )
        .await;

    assert_eq!(created.status(), StatusCode::CREATED);
    created.json().await
}

async fn signup(api: &TestApi) -> TestAuthCookies {
    configure_email_delivery(api).await;
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    api.post_json(
        "/api/v1/auth/signup/start",
        &json!({
            "email": email,
            "password": "Password!"
        }),
    )
    .await
    .assert_status(StatusCode::ACCEPTED);
    set_signup_code(api, &email, "123456").await;

    api.post_json(
        "/api/v1/auth/signup/complete",
        &json!({
            "email": email,
            "password": "Password!",
            "code": "123456"
        }),
    )
    .await
    .assert_status(StatusCode::OK)
    .auth_cookies()
}

async fn configure_email_delivery(api: &TestApi) {
    api.put_json_with_setup_secret("/api/v1/setup/sso/github", &json!({ "enabled": false }))
        .await
        .assert_status(StatusCode::OK);
    api.put_json_with_setup_secret("/api/v1/setup/sso/gitlab", &json!({ "enabled": false }))
        .await
        .assert_status(StatusCode::OK);
    api.post_json_with_setup_secret(
        "/api/v1/setup/email/test",
        &json!({
            "resend_api_key": "re_test",
            "from_name": "Fosslate",
            "from_email": "hello@example.com",
            "test_recipient": "admin@example.com"
        }),
    )
    .await
    .assert_status(StatusCode::OK);
}

async fn set_signup_code(api: &TestApi, email: &str, code: &str) {
    sqlx::query(
        r#"
        UPDATE signup_email_verifications
        SET code_hash = $2
        WHERE email = $1
          AND used_at IS NULL
        "#,
    )
    .bind(email)
    .bind(token_hash(&format!("{email}:{code}")))
    .execute(api.pool())
    .await
    .unwrap();
}

fn token_hash(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}
