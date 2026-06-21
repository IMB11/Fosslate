mod common;

use axum::http::StatusCode;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use common::TestApi;

#[tokio::test]
async fn signup_sets_session_cookies_and_protects_product_routes() {
    let api = TestApi::spawn().await;
    configure_email_delivery(&api).await;

    let public_list = api.get("/api/v1/projects").await;
    assert_eq!(public_list.status(), StatusCode::OK);

    let protected_write = api
        .post_json(
            "/api/v1/projects",
            &json!({
                "name": "No session",
                "icon_asset_id": null,
                "source_language": { "key": "en", "name": "English" }
            }),
        )
        .await;
    assert_eq!(protected_write.status(), StatusCode::UNAUTHORIZED);

    let email = "admin@example.com";
    let start = api
        .post_json(
            "/api/v1/auth/signup/start",
            &json!({
                "email": email,
                "password": "Password!"
            }),
        )
        .await;
    assert_eq!(start.status(), StatusCode::ACCEPTED);
    set_signup_code(&api, email, "123456").await;

    let signup = api
        .post_json(
            "/api/v1/auth/signup/complete",
            &json!({
                "email": email,
                "password": "Password!",
                "code": "123456"
            }),
        )
        .await;
    assert_eq!(signup.status(), StatusCode::OK);
    let cookies = signup.auth_cookies();

    let session = api.get_with_auth("/api/v1/auth/session", &cookies).await;
    assert_eq!(session.status(), StatusCode::OK);
    let body: Value = session.json().await;
    assert_eq!(body["user"]["email"], "admin@example.com");

    let missing_csrf = api
        .post_json_with_auth_without_csrf(
            "/api/v1/projects",
            &json!({
                "name": "No CSRF",
                "icon_asset_id": null,
                "source_language": { "key": "en", "name": "English" }
            }),
            &cookies,
        )
        .await;
    assert_eq!(missing_csrf.status(), StatusCode::FORBIDDEN);

    let non_admin = api
        .post_json_with_auth(
            "/api/v1/projects",
            &json!({
                "name": "No admin",
                "icon_asset_id": null,
                "source_language": { "key": "en", "name": "English" }
            }),
            &cookies,
        )
        .await;
    assert_eq!(non_admin.status(), StatusCode::FORBIDDEN);

    sqlx::query("UPDATE users SET is_admin = true WHERE email = $1")
        .bind(email)
        .execute(api.pool())
        .await
        .unwrap();

    let created = api
        .post_json_with_auth(
            "/api/v1/projects",
            &json!({
                "name": "Fosslate",
                "icon_asset_id": null,
                "source_language": { "key": "en", "name": "English" }
            }),
            &cookies,
        )
        .await;
    assert_eq!(created.status(), StatusCode::CREATED);

    api.cleanup().await;
}

#[tokio::test]
async fn refresh_rotates_refresh_token_and_rejects_old_one() {
    let api = TestApi::spawn().await;
    let signup = signup(&api, "refresh@example.com", "Password!").await;
    let cookies = signup.auth_cookies();

    let refresh = api
        .post_json_with_refresh_cookie("/api/v1/auth/session/refresh", &json!({}), &cookies)
        .await;
    assert_eq!(refresh.status(), StatusCode::OK);
    let rotated = refresh.auth_cookies();

    let old_refresh = api
        .post_json_with_refresh_cookie("/api/v1/auth/session/refresh", &json!({}), &cookies)
        .await;
    assert_eq!(old_refresh.status(), StatusCode::UNAUTHORIZED);

    let session = api.get_with_auth("/api/v1/auth/session", &rotated).await;
    assert_eq!(session.status(), StatusCode::OK);

    api.cleanup().await;
}

#[tokio::test]
async fn logout_revokes_current_session_and_clears_cookies() {
    let api = TestApi::spawn().await;
    let signup = signup(&api, "logout@example.com", "Password!").await;
    let cookies = signup.auth_cookies();

    let logout = api
        .post_json_with_auth("/api/v1/auth/logout", &json!({}), &cookies)
        .await;
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);

    let session = api.get_with_auth("/api/v1/auth/session", &cookies).await;
    assert_eq!(session.status(), StatusCode::UNAUTHORIZED);

    api.cleanup().await;
}

#[tokio::test]
async fn forgot_password_is_enumeration_safe_and_creates_token_for_existing_user() {
    let api = TestApi::spawn().await;
    configure_email_delivery(&api).await;
    signup(&api, "reset@example.com", "Password!").await;

    let existing = api
        .post_json(
            "/api/v1/auth/password/forgot",
            &json!({ "email": "reset@example.com" }),
        )
        .await;
    assert_eq!(existing.status(), StatusCode::ACCEPTED);

    let missing = api
        .post_json(
            "/api/v1/auth/password/forgot",
            &json!({ "email": "missing@example.com" }),
        )
        .await;
    assert_eq!(missing.status(), StatusCode::ACCEPTED);

    let token_count: i64 = sqlx::query_scalar("SELECT count(*) FROM password_reset_tokens")
        .fetch_one(api.pool())
        .await
        .unwrap();
    assert_eq!(token_count, 1);

    api.cleanup().await;
}

#[tokio::test]
async fn reset_password_uses_single_use_token_and_revokes_sessions() {
    let api = TestApi::spawn().await;
    let signup = signup(&api, "single-use@example.com", "Password!").await;
    let cookies = signup.auth_cookies();
    let body: Value = signup.json().await;
    let user_id = body["user"]["id"].as_i64().unwrap();

    let reset_token = "known-reset-token";
    sqlx::query(
        r#"
        INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, now() + interval '1 hour')
        "#,
    )
    .bind(user_id)
    .bind(token_hash(reset_token))
    .execute(api.pool())
    .await
    .unwrap();

    let reset = api
        .post_json(
            "/api/v1/auth/password/reset",
            &json!({
                "token": reset_token,
                "password": "Newpassword!",
                "password_confirmation": "Newpassword!"
            }),
        )
        .await;
    assert_eq!(reset.status(), StatusCode::NO_CONTENT);

    let old_session = api.get_with_auth("/api/v1/auth/session", &cookies).await;
    assert_eq!(old_session.status(), StatusCode::UNAUTHORIZED);

    let reused = api
        .post_json(
            "/api/v1/auth/password/reset",
            &json!({
                "token": reset_token,
                "password": "Anotherpass!",
                "password_confirmation": "Anotherpass!"
            }),
        )
        .await;
    assert_eq!(reused.status(), StatusCode::UNAUTHORIZED);

    let login = api
        .post_json(
            "/api/v1/auth/login",
            &json!({
                "email": "single-use@example.com",
                "password": "Newpassword!"
            }),
        )
        .await;
    assert_eq!(login.status(), StatusCode::OK);

    api.cleanup().await;
}

async fn signup(api: &TestApi, email: &str, password: &str) -> common::TestResponse {
    configure_email_delivery(api).await;
    api.post_json(
        "/api/v1/auth/signup/start",
        &json!({
            "email": email,
            "password": password
        }),
    )
    .await
    .assert_status(StatusCode::ACCEPTED);
    set_signup_code(api, email, "123456").await;

    api.post_json(
        "/api/v1/auth/signup/complete",
        &json!({
            "email": email,
            "password": password,
            "code": "123456"
        }),
    )
    .await
    .assert_status(StatusCode::OK)
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

fn token_hash(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
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
