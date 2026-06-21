mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

use common::TestApi;

#[tokio::test]
async fn setup_check_returns_empty_status_for_required_or_complete() {
    let api = TestApi::spawn().await;

    let initial = api.get("/setup/check").await;
    assert_eq!(initial.status(), StatusCode::NO_CONTENT);

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
    api.post_json_with_setup_secret("/api/v1/setup/complete", &json!({}))
        .await
        .assert_status(StatusCode::OK);

    let complete = api.get("/setup/check").await;
    assert_eq!(complete.status(), StatusCode::NOT_FOUND);

    api.cleanup().await;
}

#[tokio::test]
async fn setup_routes_require_valid_bearer_secret() {
    let api = TestApi::spawn().await;

    let missing = api.get("/api/v1/setup/status").await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let malformed = api
        .get_with_authorization("/api/v1/setup/status", "Basic nope")
        .await;
    assert_eq!(malformed.status(), StatusCode::UNAUTHORIZED);

    let invalid = api
        .get_with_authorization("/api/v1/setup/status", "Bearer wrong")
        .await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);

    let valid = api.get_with_setup_secret("/api/v1/setup/status").await;
    assert_eq!(valid.status(), StatusCode::OK);

    api.cleanup().await;
}

#[tokio::test]
async fn verify_returns_initial_setup_status_and_callback_urls() {
    let api = TestApi::spawn().await;

    let response = api
        .post_json_with_setup_secret("/api/v1/setup/verify", &json!({}))
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await;
    assert_eq!(body["required"], true);
    assert_eq!(body["completed"], false);
    assert_eq!(body["next_step"], "github");
    assert_eq!(
        body["github"]["callback_url"],
        "http://localhost:3000/api/v1/auth/sso/github/callback"
    );
    assert_eq!(
        body["gitlab"]["callback_url"],
        "http://localhost:3000/api/v1/auth/sso/gitlab/callback"
    );
    assert_eq!(body["gitlab"]["base_url"], "https://gitlab.com");
    assert_eq!(body["email"]["configured"], false);

    api.cleanup().await;
}

#[tokio::test]
async fn setup_flow_enforces_step_order_and_masks_provider_secrets() {
    let api = TestApi::spawn().await;

    let early_gitlab = api
        .put_json_with_setup_secret("/api/v1/setup/sso/gitlab", &json!({ "enabled": false }))
        .await;
    assert_eq!(early_gitlab.status(), StatusCode::BAD_REQUEST);

    let github = api
        .put_json_with_setup_secret(
            "/api/v1/setup/sso/github",
            &json!({
                "enabled": true,
                "client_id": " github-client ",
                "client_secret": "github-secret"
            }),
        )
        .await;
    assert_eq!(github.status(), StatusCode::OK);

    let body: Value = github.json().await;
    assert_eq!(body["github"]["configured"], true);
    assert_eq!(body["github"]["client_id"], "github-client");
    assert_eq!(body["github"]["has_client_secret"], true);
    assert!(body["github"].get("client_secret").is_none());
    assert_eq!(body["next_step"], "gitlab");

    let gitlab = api
        .put_json_with_setup_secret(
            "/api/v1/setup/sso/gitlab",
            &json!({
                "enabled": true,
                "base_url": " https://gitlab.example.com/ ",
                "client_id": "gitlab-client",
                "client_secret": "gitlab-secret"
            }),
        )
        .await;
    assert_eq!(gitlab.status(), StatusCode::OK);

    let body: Value = gitlab.json().await;
    assert_eq!(body["gitlab"]["configured"], true);
    assert_eq!(body["gitlab"]["base_url"], "https://gitlab.example.com");
    assert_eq!(
        body["gitlab"]["scopes"],
        json!(["openid", "profile", "email"])
    );
    assert!(body["gitlab"].get("client_secret").is_none());
    assert_eq!(body["next_step"], "email");

    api.cleanup().await;
}

#[tokio::test]
async fn setup_flow_can_skip_sso_send_test_email_and_complete() {
    let api = TestApi::spawn().await;

    api.put_json_with_setup_secret("/api/v1/setup/sso/github", &json!({ "enabled": false }))
        .await
        .assert_status(StatusCode::OK);

    api.put_json_with_setup_secret("/api/v1/setup/sso/gitlab", &json!({ "enabled": false }))
        .await
        .assert_status(StatusCode::OK);

    let before_email_complete = api
        .post_json_with_setup_secret("/api/v1/setup/complete", &json!({}))
        .await;
    assert_eq!(before_email_complete.status(), StatusCode::BAD_REQUEST);

    let email = api
        .post_json_with_setup_secret(
            "/api/v1/setup/email/test",
            &json!({
                "resend_api_key": "re_test",
                "from_name": "Fosslate",
                "from_email": "hello@example.com",
                "test_recipient": "admin@example.com"
            }),
        )
        .await;
    assert_eq!(email.status(), StatusCode::OK);

    let body: Value = email.json().await;
    assert_eq!(body["email"]["configured"], true);
    assert_eq!(body["email"]["provider"], "resend");
    assert_eq!(body["email"]["has_api_key"], true);
    assert_eq!(body["email"]["last_test_message_id"], "test-message-id");
    assert_eq!(body["next_step"], "complete");

    let complete = api
        .post_json_with_setup_secret("/api/v1/setup/complete", &json!({}))
        .await;
    assert_eq!(complete.status(), StatusCode::OK);

    let body: Value = complete.json().await;
    assert_eq!(body["next"], "/");

    let after_complete = api.get_with_setup_secret("/api/v1/setup/status").await;
    assert_eq!(after_complete.status(), StatusCode::CONFLICT);

    let body: Value = after_complete.json().await;
    assert_eq!(body["error"], "setup_complete");

    api.cleanup().await;
}

#[tokio::test]
async fn email_test_rejects_invalid_email_addresses() {
    let api = TestApi::spawn().await;

    api.put_json_with_setup_secret("/api/v1/setup/sso/github", &json!({ "enabled": false }))
        .await
        .assert_status(StatusCode::OK);

    api.put_json_with_setup_secret("/api/v1/setup/sso/gitlab", &json!({ "enabled": false }))
        .await
        .assert_status(StatusCode::OK);

    let response = api
        .post_json_with_setup_secret(
            "/api/v1/setup/email/test",
            &json!({
                "resend_api_key": "re_test",
                "from_name": "Fosslate",
                "from_email": "not-an-email",
                "test_recipient": "admin@example.com"
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    api.cleanup().await;
}
