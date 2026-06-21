#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, Response, StatusCode, header::SET_COOKIE},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use fosslate_api::{
    adapters::resend::EmailDeliveryClient,
    app::{self, AppState},
    config::Config,
    db,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

pub struct TestApp {
    pub db: TemporaryDatabase,
    app: Router,
    authenticate_requests: bool,
}

pub struct TestApi {
    inner: TestApp,
}

pub struct TestResponse {
    response: Response<Body>,
}

pub const SETUP_SECRET: &str = "test-setup-secret";

impl TestApi {
    pub async fn spawn() -> Self {
        Self {
            inner: TestApp::new().await,
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.inner.db.pool
    }

    pub async fn cleanup(self) {
        self.inner.cleanup().await;
    }

    pub async fn get(&self, path: &str) -> TestResponse {
        TestResponse {
            response: self.inner.request_empty(Method::GET, path).await,
        }
    }

    pub async fn get_with_setup_secret(&self, path: &str) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_empty_with_setup_secret(Method::GET, path)
                .await,
        }
    }

    pub async fn get_with_authorization(&self, path: &str, authorization: &str) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_empty_with_authorization(Method::GET, path, authorization)
                .await,
        }
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json(Method::POST, path, body.clone())
                .await,
        }
    }

    pub async fn post_json_with_setup_secret(&self, path: &str, body: &Value) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json_with_setup_secret(Method::POST, path, body.clone())
                .await,
        }
    }

    pub async fn put_json(&self, path: &str, body: &Value) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json(Method::PUT, path, body.clone())
                .await,
        }
    }

    pub async fn put_json_with_setup_secret(&self, path: &str, body: &Value) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json_with_setup_secret(Method::PUT, path, body.clone())
                .await,
        }
    }

    pub async fn delete(&self, path: &str) -> TestResponse {
        TestResponse {
            response: self.inner.request_empty(Method::DELETE, path).await,
        }
    }

    pub async fn get_with_auth(&self, path: &str, cookies: &TestAuthCookies) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_empty_with_auth(Method::GET, path, cookies, false)
                .await,
        }
    }

    pub async fn post_json_with_auth(
        &self,
        path: &str,
        body: &Value,
        cookies: &TestAuthCookies,
    ) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json_with_auth(Method::POST, path, body.clone(), cookies, true)
                .await,
        }
    }

    pub async fn put_json_with_auth(
        &self,
        path: &str,
        body: &Value,
        cookies: &TestAuthCookies,
    ) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json_with_auth(Method::PUT, path, body.clone(), cookies, true)
                .await,
        }
    }

    pub async fn delete_with_auth(&self, path: &str, cookies: &TestAuthCookies) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_empty_with_auth(Method::DELETE, path, cookies, true)
                .await,
        }
    }

    pub async fn post_json_with_auth_without_csrf(
        &self,
        path: &str,
        body: &Value,
        cookies: &TestAuthCookies,
    ) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json_with_auth(Method::POST, path, body.clone(), cookies, false)
                .await,
        }
    }

    pub async fn post_json_with_refresh_cookie(
        &self,
        path: &str,
        body: &Value,
        cookies: &TestAuthCookies,
    ) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json_with_refresh_cookie(Method::POST, path, body.clone(), cookies)
                .await,
        }
    }
}

impl TestResponse {
    pub fn status(&self) -> StatusCode {
        self.response.status()
    }

    pub fn assert_status(self, expected: StatusCode) -> Self {
        assert_eq!(self.status(), expected);
        self
    }

    pub async fn json<T: DeserializeOwned>(self) -> T {
        let (_, body) = response_json(self.response).await;
        serde_json::from_value(body).unwrap()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }

    pub fn auth_cookies(&self) -> TestAuthCookies {
        TestAuthCookies::from_headers(self.response.headers())
    }
}

#[derive(Debug, Clone)]
pub struct TestAuthCookies {
    pub access: String,
    pub refresh: String,
    pub csrf: String,
}

impl TestAuthCookies {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let mut access = None;
        let mut refresh = None;
        let mut csrf = None;

        for value in headers.get_all(SET_COOKIE) {
            let value = value.to_str().unwrap();
            let cookie = value.split(';').next().unwrap();
            let (name, value) = cookie.split_once('=').unwrap();
            match name {
                "fs_access" => access = Some(value.to_owned()),
                "fs_refresh" => refresh = Some(value.to_owned()),
                "fs_csrf" => csrf = Some(value.to_owned()),
                _ => {}
            }
        }

        Self {
            access: access.expect("missing fs_access cookie"),
            refresh: refresh.expect("missing fs_refresh cookie"),
            csrf: csrf.expect("missing fs_csrf cookie"),
        }
    }

    pub fn cookie_header(&self) -> String {
        format!(
            "fs_access={}; fs_refresh={}; fs_csrf={}",
            self.access, self.refresh, self.csrf
        )
    }

    pub fn refresh_cookie_header(&self) -> String {
        format!("fs_refresh={}", self.refresh)
    }
}

pub async fn spawn_app() -> TestApp {
    TestApp::new_authenticated().await
}

impl TestApp {
    pub async fn spawn() -> Self {
        Self::new_authenticated().await
    }

    pub async fn new() -> Self {
        Self::new_with_authentication(false).await
    }

    pub async fn new_authenticated() -> Self {
        Self::new_with_authentication(true).await
    }

    async fn new_with_authentication(authenticate_requests: bool) -> Self {
        let db = TemporaryDatabase::create().await;
        let config = Config {
            database_url: db.database_url.clone(),
            api_host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            api_port: 0,
            cors_allowed_origin: None,
            public_app_url: "http://localhost:3000".to_owned(),
            public_api_url: "http://localhost:3000".to_owned(),
            resend_api_url: "https://api.resend.com/emails".to_owned(),
        };
        let app = app::build(
            AppState::with_email_delivery(
                db.pool.clone(),
                &config,
                SETUP_SECRET.to_owned(),
                "test-secrets-key".to_owned(),
                EmailDeliveryClient::static_success("test-message-id"),
            ),
            &config,
        );
        Self {
            db,
            app,
            authenticate_requests,
        }
    }

    pub async fn cleanup(self) {
        self.db.cleanup().await;
    }

    pub async fn request_json(&self, method: Method, path: &str, body: Value) -> Response<Body> {
        let mut builder = Request::builder()
            .method(&method)
            .uri(path)
            .header("content-type", "application/json");

        if let Some(cookies) = self
            .auth_cookies_for_request(method.clone(), path, Some(&body))
            .await
        {
            builder = builder.header("cookie", cookies.cookie_header());
            if requires_csrf(&method) {
                builder = builder.header("x-csrf-token", &cookies.csrf);
            }
        }

        let request = builder.body(Body::from(body.to_string())).unwrap();

        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_empty(&self, method: Method, path: &str) -> Response<Body> {
        let mut builder = Request::builder().method(&method).uri(path);

        if let Some(cookies) = self
            .auth_cookies_for_request(method.clone(), path, None)
            .await
        {
            builder = builder.header("cookie", cookies.cookie_header());
            if requires_csrf(&method) {
                builder = builder.header("x-csrf-token", &cookies.csrf);
            }
        }

        let request = builder.body(Body::empty()).unwrap();

        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_json_with_setup_secret(
        &self,
        method: Method,
        path: &str,
        body: Value,
    ) -> Response<Body> {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {SETUP_SECRET}"))
            .body(Body::from(body.to_string()))
            .unwrap();

        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_empty_with_setup_secret(
        &self,
        method: Method,
        path: &str,
    ) -> Response<Body> {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", format!("Bearer {SETUP_SECRET}"))
            .body(Body::empty())
            .unwrap();

        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_empty_with_authorization(
        &self,
        method: Method,
        path: &str,
        authorization: &str,
    ) -> Response<Body> {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", authorization)
            .body(Body::empty())
            .unwrap();

        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_json_with_auth(
        &self,
        method: Method,
        path: &str,
        body: Value,
        cookies: &TestAuthCookies,
        include_csrf: bool,
    ) -> Response<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .header("cookie", cookies.cookie_header());

        if include_csrf {
            builder = builder.header("x-csrf-token", &cookies.csrf);
        }

        let request = builder.body(Body::from(body.to_string())).unwrap();
        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_json_with_refresh_cookie(
        &self,
        method: Method,
        path: &str,
        body: Value,
        cookies: &TestAuthCookies,
    ) -> Response<Body> {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .header("cookie", cookies.refresh_cookie_header())
            .body(Body::from(body.to_string()))
            .unwrap();

        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_empty_with_auth(
        &self,
        method: Method,
        path: &str,
        cookies: &TestAuthCookies,
        include_csrf: bool,
    ) -> Response<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("cookie", cookies.cookie_header());

        if include_csrf {
            builder = builder.header("x-csrf-token", &cookies.csrf);
        }

        let request = builder.body(Body::empty()).unwrap();
        self.app.clone().oneshot(request).await.unwrap()
    }

    async fn auth_cookies_for_request(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Option<TestAuthCookies> {
        if !self.authenticate_requests || is_public_path(path) {
            return None;
        }

        let requested_user_id = body.and_then(user_id_from_body);
        let user_id = match requested_user_id {
            Some(user_id) if self.user_exists(user_id).await => user_id,
            _ => self.default_auth_user_id().await,
        };

        let cookies = self.ensure_test_session(user_id).await;
        if requires_csrf(&method) || matches!(method, Method::GET | Method::HEAD) {
            Some(cookies)
        } else {
            Some(cookies)
        }
    }

    async fn default_auth_user_id(&self) -> i64 {
        sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO users (username, email, email_verified_at, is_admin)
            VALUES ('__test_auth_user', '__test_auth_user@example.test', now(), true)
            ON CONFLICT (username) DO UPDATE
            SET email = EXCLUDED.email,
                is_admin = true
            RETURNING id
            "#,
        )
        .fetch_one(self.pool())
        .await
        .unwrap()
    }

    async fn user_exists(&self, user_id: i64) -> bool {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM users WHERE id = $1
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .unwrap()
    }

    async fn ensure_test_session(&self, user_id: i64) -> TestAuthCookies {
        let cookies = TestAuthCookies {
            access: format!("test-access-{user_id}"),
            refresh: format!("test-refresh-{user_id}"),
            csrf: format!("test-csrf-{user_id}"),
        };

        sqlx::query(
            r#"
            INSERT INTO auth_sessions (
                user_id,
                access_token_hash,
                refresh_token_hash,
                csrf_token_hash,
                access_expires_at,
                refresh_expires_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                now() + interval '15 minutes',
                now() + interval '30 days'
            )
            ON CONFLICT (access_token_hash) DO UPDATE
            SET
                refresh_token_hash = EXCLUDED.refresh_token_hash,
                csrf_token_hash = EXCLUDED.csrf_token_hash,
                access_expires_at = EXCLUDED.access_expires_at,
                refresh_expires_at = EXCLUDED.refresh_expires_at,
                revoked_at = NULL
            "#,
        )
        .bind(user_id)
        .bind(token_hash(&cookies.access))
        .bind(token_hash(&cookies.refresh))
        .bind(token_hash(&cookies.csrf))
        .execute(self.pool())
        .await
        .unwrap();

        cookies
    }

    pub fn db(&self) -> &PgPool {
        &self.db.pool
    }

    pub fn pool(&self) -> &PgPool {
        &self.db.pool
    }

    pub async fn get(&self, path: &str) -> TestResponse {
        TestResponse {
            response: self.request_empty(Method::GET, path).await,
        }
    }

    pub async fn post_json(&self, path: &str, body: Value) -> TestResponse {
        TestResponse {
            response: self.request_json(Method::POST, path, body).await,
        }
    }

    pub async fn put_json(&self, path: &str, body: Value) -> TestResponse {
        TestResponse {
            response: self.request_json(Method::PUT, path, body).await,
        }
    }

    pub async fn delete(&self, path: &str) -> TestResponse {
        TestResponse {
            response: self.request_empty(Method::DELETE, path).await,
        }
    }

    pub async fn get_typed<T: DeserializeOwned>(
        &self,
        path: &str,
        expected_status: StatusCode,
    ) -> T {
        self.get(path)
            .await
            .assert_status(expected_status)
            .json()
            .await
    }

    pub async fn post_typed<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        expected_status: StatusCode,
    ) -> T {
        self.post_json(path, serde_json::to_value(body).unwrap())
            .await
            .assert_status(expected_status)
            .json()
            .await
    }

    pub async fn put_typed<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        expected_status: StatusCode,
    ) -> T {
        self.put_json(path, serde_json::to_value(body).unwrap())
            .await
            .assert_status(expected_status)
            .json()
            .await
    }

    pub async fn post_json_expect_status<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        expected_status: StatusCode,
    ) {
        self.post_json(path, serde_json::to_value(body).unwrap())
            .await
            .assert_status(expected_status);
    }

    pub async fn put_json_expect_status<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        expected_status: StatusCode,
    ) {
        self.put_json(path, serde_json::to_value(body).unwrap())
            .await
            .assert_status(expected_status);
    }

    pub async fn delete_expect_status(&self, path: &str, expected_status: StatusCode) {
        self.delete(path).await.assert_status(expected_status);
    }

    pub async fn delete_typed<T: DeserializeOwned>(
        &self,
        path: &str,
        expected_status: StatusCode,
    ) -> T {
        self.delete(path)
            .await
            .assert_status(expected_status)
            .json()
            .await
    }

    pub async fn create_user(&self, username: &str) -> i64 {
        let response = self
            .request_json(
                Method::POST,
                "/api/v1/users",
                json!({ "username": username }),
            )
            .await;
        let (status, body) = response_json(response).await;
        assert_eq!(status, StatusCode::CREATED, "{body:#?}");
        body["id"].as_i64().unwrap()
    }

    pub async fn create_project(&self, name: &str) -> TestProject {
        let response = self
            .request_json(
                Method::POST,
                "/api/v1/projects",
                json!({
                    "name": name,
                    "icon_asset_id": null,
                    "source_language": {
                        "key": "en-GB",
                        "name": "English"
                    }
                }),
            )
            .await;
        let (status, body) = response_json(response).await;
        assert_eq!(status, StatusCode::CREATED, "{body:#?}");
        TestProject {
            id: body["id"].as_i64().unwrap(),
            public_id: body["public_id"].as_str().unwrap().to_owned(),
        }
    }

    pub async fn add_language(&self, project_public_id: &str, key: &str, name: &str) -> i64 {
        let response = self
            .request_json(
                Method::POST,
                &format!("/api/v1/projects/{project_public_id}/languages"),
                json!({
                    "language": {
                        "key": key,
                        "name": name
                    }
                }),
            )
            .await;
        let (status, body) = response_json(response).await;
        assert_eq!(status, StatusCode::CREATED, "{body:#?}");
        body["id"].as_i64().unwrap()
    }

    pub async fn create_namespace(&self, project_public_id: &str, name: &str) -> i64 {
        let response = self
            .request_json(
                Method::POST,
                &format!("/api/v1/projects/{project_public_id}/namespaces"),
                json!({ "name": name }),
            )
            .await;
        let (status, body) = response_json(response).await;
        assert_eq!(status, StatusCode::CREATED, "{body:#?}");
        body["id"].as_i64().unwrap()
    }

    pub async fn create_string(
        &self,
        project_public_id: &str,
        namespace_id: i64,
        identifier: &str,
        value: &str,
    ) -> i64 {
        let response = self
            .request_json(
                Method::POST,
                &format!("/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings"),
                json!({
                    "identifier": identifier,
                    "value": value
                }),
            )
            .await;
        let (status, body) = response_json(response).await;
        assert_eq!(status, StatusCode::CREATED, "{body:#?}");
        body["id"].as_i64().unwrap()
    }

    pub async fn create_translation(
        &self,
        project_public_id: &str,
        string_id: i64,
        target_language_id: i64,
        author_user_id: i64,
        value: &str,
    ) -> i64 {
        let response = self
            .request_json(
                Method::POST,
                &format!("/api/v1/projects/{project_public_id}/strings/{string_id}/translations"),
                json!({
                    "target_language_id": target_language_id,
                    "author_user_id": author_user_id,
                    "value": value
                }),
            )
            .await;
        let (status, body) = response_json(response).await;
        assert_eq!(status, StatusCode::CREATED, "{body:#?}");
        body["id"].as_i64().unwrap()
    }
}

pub struct TestProject {
    pub id: i64,
    pub public_id: String,
}

pub struct TemporaryDatabase {
    pub pool: PgPool,
    pub database_name: String,
    pub database_url: String,
    root_database_url: String,
}

impl TemporaryDatabase {
    pub async fn create() -> Self {
        dotenvy::dotenv().ok();
        let root_database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL is required for API tests");
        let database_name = format!("fosslate_test_{}", Uuid::new_v4().simple());

        let root_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&root_database_url)
            .await
            .expect("failed to connect to root test database");

        sqlx::query(AssertSqlSafe(format!(
            r#"CREATE DATABASE "{database_name}""#
        )))
        .execute(&root_pool)
        .await
        .expect("failed to create temporary test database");
        root_pool.close().await;

        let mut database_url = Url::parse(&root_database_url).expect("DATABASE_URL is invalid");
        database_url.set_path(&format!("/{database_name}"));
        let database_url = database_url.to_string();

        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&database_url)
            .await
            .expect("failed to connect to temporary test database");
        db::run_migrations(&pool)
            .await
            .expect("failed to run migrations on temporary test database");

        Self {
            pool,
            database_name,
            database_url,
            root_database_url,
        }
    }

    pub async fn cleanup(self) {
        self.pool.close().await;
        let root_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&self.root_database_url)
            .await
            .expect("failed to reconnect to root test database");
        sqlx::query(
            r#"
            SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = $1
              AND pid <> pg_backend_pid()
            "#,
        )
        .bind(&self.database_name)
        .execute(&root_pool)
        .await
        .expect("failed to terminate temporary database connections");
        sqlx::query(AssertSqlSafe(format!(
            r#"DROP DATABASE IF EXISTS "{}""#,
            self.database_name
        )))
        .execute(&root_pool)
        .await
        .expect("failed to drop temporary test database");
        root_pool.close().await;
    }
}

pub async fn response_json(response: Response<Body>) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    if bytes.is_empty() {
        return (status, Value::Null);
    }
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn is_public_path(path: &str) -> bool {
    if !path.starts_with("/api/v1/") {
        return true;
    }
    path == "/api/v1/meta"
        || path.starts_with("/api/v1/auth/")
        || path.starts_with("/api/v1/setup/")
}

fn requires_csrf(method: &Method) -> bool {
    !matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

fn user_id_from_body(body: &Value) -> Option<i64> {
    body.get("author_user_id")
        .or_else(|| body.get("approved_by_user_id"))
        .or_else(|| body.get("user_id"))
        .and_then(Value::as_i64)
}

fn token_hash(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}
