mod common;

use axum::http::StatusCode;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use common::{SETUP_SECRET, TestApi, TestAuthCookies};

#[tokio::test]
async fn session_returns_instance_admin_role() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api, "admin@example.com").await;
    let user_id = current_user_id(&api, &cookies).await;

    sqlx::query(
        r#"
        UPDATE users
        SET is_admin = true
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(api.pool())
    .await
    .unwrap();

    let session = api.get_with_auth("/api/v1/auth/session", &cookies).await;
    assert_eq!(session.status(), StatusCode::OK);
    let body: Value = session.json().await;
    assert_eq!(body["user"]["is_admin"], true);

    api.cleanup().await;
}

#[tokio::test]
async fn instance_settings_require_admin() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api, "not-admin@example.com").await;

    let read = api
        .get_with_auth("/api/v1/settings/instance", &cookies)
        .await;
    assert_eq!(read.status(), StatusCode::FORBIDDEN);

    let update = api
        .put_json_with_auth(
            "/api/v1/settings/instance/sso/github",
            &json!({ "enabled": false }),
            &cookies,
        )
        .await;
    assert_eq!(update.status(), StatusCode::FORBIDDEN);

    api.cleanup().await;
}

#[tokio::test]
async fn admin_claim_requires_current_setup_secret() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api, "claim@example.com").await;

    let wrong = api
        .post_json_with_auth(
            "/api/v1/settings/instance/admin/claim",
            &json!({ "setup_secret": "wrong" }),
            &cookies,
        )
        .await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);

    let claimed = claim_admin(&api, &cookies).await;
    let body: Value = claimed.json().await;
    assert_eq!(body["user"]["is_admin"], true);

    let session = api.get_with_auth("/api/v1/auth/session", &cookies).await;
    assert_eq!(session.status(), StatusCode::OK);
    let body: Value = session.json().await;
    assert_eq!(body["user"]["is_admin"], true);

    api.cleanup().await;
}

#[tokio::test]
async fn admin_can_read_and_update_masked_sso_settings() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api, "sso-admin@example.com").await;
    claim_admin(&api, &cookies).await;

    let read = api
        .get_with_auth("/api/v1/settings/instance", &cookies)
        .await;
    assert_eq!(read.status(), StatusCode::OK);
    let body: Value = read.json().await;
    assert!(body["github"].get("client_secret").is_none());
    assert!(body["email"].get("api_key").is_none());

    let saved = api
        .put_json_with_auth(
            "/api/v1/settings/instance/sso/github",
            &json!({
                "enabled": true,
                "client_id": "github-client",
                "client_secret": "github-secret"
            }),
            &cookies,
        )
        .await;
    assert_eq!(saved.status(), StatusCode::OK);
    let body: Value = saved.json().await;
    assert_eq!(body["github"]["enabled"], true);
    assert_eq!(body["github"]["client_id"], "github-client");
    assert_eq!(body["github"]["has_client_secret"], true);
    assert!(body["github"].get("client_secret").is_none());

    let retained = api
        .put_json_with_auth(
            "/api/v1/settings/instance/sso/github",
            &json!({
                "enabled": true,
                "client_id": "github-client-renamed"
            }),
            &cookies,
        )
        .await;
    assert_eq!(retained.status(), StatusCode::OK);
    let body: Value = retained.json().await;
    assert_eq!(body["github"]["client_id"], "github-client-renamed");
    assert_eq!(body["github"]["has_client_secret"], true);

    api.cleanup().await;
}

#[tokio::test]
async fn admin_can_test_email_and_retain_saved_api_key() {
    let api = TestApi::spawn().await;
    let cookies = signup(&api, "email-admin@example.com").await;
    claim_admin(&api, &cookies).await;

    let saved = api
        .post_json_with_auth(
            "/api/v1/settings/instance/email/test",
            &json!({
                "resend_api_key": "re_settings",
                "from_name": "Fosslate",
                "from_email": "hello@example.com",
                "test_recipient": "admin@example.com"
            }),
            &cookies,
        )
        .await;
    assert_eq!(saved.status(), StatusCode::OK);
    let body: Value = saved.json().await;
    assert_eq!(body["email"]["has_api_key"], true);
    assert_eq!(body["email"]["from_email"], "hello@example.com");
    assert!(body["email"].get("api_key").is_none());

    let retained = api
        .post_json_with_auth(
            "/api/v1/settings/instance/email/test",
            &json!({
                "from_name": "Fosslate Mail",
                "from_email": "mail@example.com",
                "test_recipient": "ops@example.com"
            }),
            &cookies,
        )
        .await;
    assert_eq!(retained.status(), StatusCode::OK);
    let body: Value = retained.json().await;
    assert_eq!(body["email"]["has_api_key"], true);
    assert_eq!(body["email"]["from_name"], "Fosslate Mail");
    assert_eq!(body["email"]["last_test_recipient"], "ops@example.com");

    api.cleanup().await;
}

async fn claim_admin(api: &TestApi, cookies: &TestAuthCookies) -> common::TestResponse {
    api.post_json_with_auth(
        "/api/v1/settings/instance/admin/claim",
        &json!({ "setup_secret": SETUP_SECRET }),
        cookies,
    )
    .await
    .assert_status(StatusCode::OK)
}

async fn current_user_id(api: &TestApi, cookies: &TestAuthCookies) -> i64 {
    let session = api.get_with_auth("/api/v1/auth/session", cookies).await;
    assert_eq!(session.status(), StatusCode::OK);
    let body: Value = session.json().await;
    body["user"]["id"].as_i64().unwrap()
}

async fn signup(api: &TestApi, email: &str) -> TestAuthCookies {
    configure_email_delivery(api).await;
    api.post_json(
        "/api/v1/auth/signup/start",
        &json!({
            "email": email,
            "password": "Password!"
        }),
    )
    .await
    .assert_status(StatusCode::ACCEPTED);
    set_signup_code(api, email, "123456").await;

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
