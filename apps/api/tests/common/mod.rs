#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode},
};
use fosslate_api::{
    app::{self, AppState},
    config::Config,
    db,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

pub struct TestApp {
    pub db: TemporaryDatabase,
    app: Router,
}

pub struct TestApi {
    inner: TestApp,
}

pub struct TestResponse {
    response: Response<Body>,
}

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

    pub async fn post_json(&self, path: &str, body: &Value) -> TestResponse {
        TestResponse {
            response: self
                .inner
                .request_json(Method::POST, path, body.clone())
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

    pub async fn delete(&self, path: &str) -> TestResponse {
        TestResponse {
            response: self.inner.request_empty(Method::DELETE, path).await,
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
}

pub async fn spawn_app() -> TestApp {
    TestApp::new().await
}

impl TestApp {
    pub async fn spawn() -> Self {
        Self::new().await
    }

    pub async fn new() -> Self {
        let db = TemporaryDatabase::create().await;
        let config = Config {
            database_url: db.database_url.clone(),
            api_host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            api_port: 0,
            cors_allowed_origin: None,
        };
        let app = app::build(AppState::new(db.pool.clone()), &config);
        Self { db, app }
    }

    pub async fn cleanup(self) {
        self.db.cleanup().await;
    }

    pub async fn request_json(&self, method: Method, path: &str, body: Value) -> Response<Body> {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_empty(&self, method: Method, path: &str) -> Response<Body> {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap();

        self.app.clone().oneshot(request).await.unwrap()
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
