mod common;

use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct ApiLanguage {
    key: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ApiProject {
    id: i64,
    public_id: Uuid,
    name: String,
    source_language: ApiLanguage,
}

#[derive(Debug, Deserialize)]
struct ApiTargetLanguage {
    id: i64,
    project_id: i64,
    language: ApiLanguage,
}

#[derive(Debug, Deserialize)]
struct ApiNamespace {
    id: i64,
    project_id: i64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ApiSourceString {
    id: i64,
    project_id: i64,
    namespace_id: i64,
    identifier: String,
    value: String,
}

#[derive(Debug, sqlx::FromRow)]
struct NamespaceLanguageStatsRow {
    string_count: i32,
    translated_count: i32,
    approved_count: i32,
    candidate_count: i32,
    missing_count: i32,
}

#[tokio::test]
async fn target_languages_can_be_added_listed_removed_and_readded() {
    let app = common::spawn_app().await;
    let project = create_project(&app, "Target language project").await;
    let other_project = create_project(&app, "Other target language project").await;

    let english = add_language(&app, project.public_id, "en-GB", "English (UK)").await;
    assert_eq!(english.project_id, project.id);
    assert_eq!(english.language.key, "en-GB");
    assert_eq!(english.language.name, "English (UK)");

    let custom = add_language(&app, project.public_id, "pirate-mode", "Pirate mode").await;
    assert_eq!(custom.language.key, "pirate-mode");

    let duplicate = app
        .post_json(
            &format!("/api/v1/projects/{}/languages", project.public_id),
            language_request("en-GB", "English (UK)"),
        )
        .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let same_key_other_project =
        add_language(&app, other_project.public_id, "en-GB", "English (UK)").await;
    assert_eq!(same_key_other_project.project_id, other_project.id);

    let languages: Vec<ApiTargetLanguage> = app
        .get(&format!("/api/v1/projects/{}/languages", project.public_id))
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(languages.len(), 2);
    assert_eq!(languages[0].id, english.id);
    assert_eq!(languages[1].id, custom.id);

    app.delete(&format!(
        "/api/v1/projects/{}/languages/{}",
        project.public_id, english.id
    ))
    .await
    .assert_status(StatusCode::NO_CONTENT);

    let languages: Vec<ApiTargetLanguage> = app
        .get(&format!("/api/v1/projects/{}/languages", project.public_id))
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0].id, custom.id);

    let readded = add_language(&app, project.public_id, "en-GB", "English (UK)").await;
    assert_ne!(readded.id, english.id);
    assert_eq!(readded.language.key, "en-GB");

    app.cleanup().await;
}

#[tokio::test]
async fn namespaces_follow_project_scope_and_soft_delete_rules() {
    let app = common::spawn_app().await;
    let project = create_project(&app, "Namespace project").await;
    let other_project = create_project(&app, "Other namespace project").await;

    let namespace = create_namespace(&app, project.public_id, "common").await;
    assert_eq!(namespace.project_id, project.id);
    assert_eq!(namespace.name, "common");

    let duplicate = app
        .post_json(
            &format!("/api/v1/projects/{}/namespaces", project.public_id),
            json!({ "name": "common" }),
        )
        .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let same_name_other_project = create_namespace(&app, other_project.public_id, "common").await;
    assert_eq!(same_name_other_project.project_id, other_project.id);

    let namespaces: Vec<ApiNamespace> = app
        .get(&format!(
            "/api/v1/projects/{}/namespaces",
            project.public_id
        ))
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces[0].id, namespace.id);

    let fetched: ApiNamespace = app
        .get(&format!(
            "/api/v1/projects/{}/namespaces/{}",
            project.public_id, namespace.id
        ))
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(fetched.id, namespace.id);

    app.get(&format!(
        "/api/v1/projects/{}/namespaces/{}",
        other_project.public_id, namespace.id
    ))
    .await
    .assert_status(StatusCode::NOT_FOUND);

    let updated: ApiNamespace = app
        .put_json(
            &format!(
                "/api/v1/projects/{}/namespaces/{}",
                project.public_id, namespace.id
            ),
            json!({ "name": "shared" }),
        )
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(updated.name, "shared");

    app.put_json(
        &format!(
            "/api/v1/projects/{}/namespaces/{}",
            other_project.public_id, namespace.id
        ),
        json!({ "name": "wrong-project" }),
    )
    .await
    .assert_status(StatusCode::NOT_FOUND);

    app.delete(&format!(
        "/api/v1/projects/{}/namespaces/{}",
        project.public_id, namespace.id
    ))
    .await
    .assert_status(StatusCode::NO_CONTENT);

    app.get(&format!(
        "/api/v1/projects/{}/namespaces/{}",
        project.public_id, namespace.id
    ))
    .await
    .assert_status(StatusCode::NOT_FOUND);

    let recreated = create_namespace(&app, project.public_id, "shared").await;
    assert_ne!(recreated.id, namespace.id);
    assert_eq!(recreated.name, "shared");

    app.cleanup().await;
}

#[tokio::test]
async fn source_strings_follow_namespace_scope_keyset_pagination_and_stats() {
    let app = common::spawn_app().await;
    let project = create_project(&app, "String project").await;
    let other_project = create_project(&app, "Other string project").await;
    let language = add_language(&app, project.public_id, "fr-FR", "French").await;

    let namespace = create_namespace(&app, project.public_id, "common").await;
    let other_namespace = create_namespace(&app, project.public_id, "dashboard").await;
    let other_project_namespace = create_namespace(&app, other_project.public_id, "common").await;

    let hello = create_source_string(&app, project.public_id, namespace.id, "hello", "Hello").await;
    assert_eq!(hello.project_id, project.id);
    assert_eq!(hello.namespace_id, namespace.id);

    assert_namespace_stats(
        &app,
        namespace.id,
        language.id,
        NamespaceLanguageStatsRow {
            string_count: 1,
            translated_count: 0,
            approved_count: 0,
            candidate_count: 0,
            missing_count: 1,
        },
    )
    .await;

    let duplicate = app
        .post_json(
            &format!(
                "/api/v1/projects/{}/namespaces/{}/strings",
                project.public_id, namespace.id
            ),
            source_string_request("hello", "Hello again"),
        )
        .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let same_identifier_different_namespace = create_source_string(
        &app,
        project.public_id,
        other_namespace.id,
        "hello",
        "Hello",
    )
    .await;
    assert_eq!(
        same_identifier_different_namespace.namespace_id,
        other_namespace.id
    );

    let bye = create_source_string(&app, project.public_id, namespace.id, "bye", "Bye").await;
    let thanks =
        create_source_string(&app, project.public_id, namespace.id, "thanks", "Thanks").await;

    let first_page: Vec<ApiSourceString> = app
        .get(&format!(
            "/api/v1/projects/{}/namespaces/{}/strings?limit=2",
            project.public_id, namespace.id
        ))
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(first_page.len(), 2);
    assert_eq!(first_page[0].id, hello.id);
    assert_eq!(first_page[1].id, bye.id);

    let second_page: Vec<ApiSourceString> = app
        .get(&format!(
            "/api/v1/projects/{}/namespaces/{}/strings?after_id={}&limit=2",
            project.public_id, namespace.id, bye.id
        ))
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].id, thanks.id);

    let fetched: ApiSourceString = app
        .get(&format!(
            "/api/v1/projects/{}/strings/{}",
            project.public_id, hello.id
        ))
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(fetched.identifier, "hello");
    assert_eq!(fetched.value, "Hello");

    let updated: ApiSourceString = app
        .put_json(
            &format!(
                "/api/v1/projects/{}/strings/{}",
                project.public_id, hello.id
            ),
            source_string_request("hello.title", "Hello title"),
        )
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;
    assert_eq!(updated.identifier, "hello.title");
    assert_eq!(updated.value, "Hello title");

    app.get(&format!(
        "/api/v1/projects/{}/namespaces/{}/strings",
        other_project.public_id, namespace.id
    ))
    .await
    .assert_status(StatusCode::NOT_FOUND);

    app.post_json(
        &format!(
            "/api/v1/projects/{}/namespaces/{}/strings",
            other_project.public_id, namespace.id
        ),
        source_string_request("wrong-project", "Wrong project"),
    )
    .await
    .assert_status(StatusCode::NOT_FOUND);

    let other_project_string = create_source_string(
        &app,
        other_project.public_id,
        other_project_namespace.id,
        "hello",
        "Hello",
    )
    .await;
    assert_eq!(other_project_string.project_id, other_project.id);

    app.delete(&format!(
        "/api/v1/projects/{}/strings/{}",
        project.public_id, updated.id
    ))
    .await
    .assert_status(StatusCode::NO_CONTENT);

    app.get(&format!(
        "/api/v1/projects/{}/strings/{}",
        project.public_id, updated.id
    ))
    .await
    .assert_status(StatusCode::NOT_FOUND);

    assert_namespace_stats(
        &app,
        namespace.id,
        language.id,
        NamespaceLanguageStatsRow {
            string_count: 2,
            translated_count: 0,
            approved_count: 0,
            candidate_count: 0,
            missing_count: 2,
        },
    )
    .await;

    let recreated = create_source_string(
        &app,
        project.public_id,
        namespace.id,
        "hello.title",
        "Hello again",
    )
    .await;
    assert_ne!(recreated.id, updated.id);
    assert_eq!(recreated.identifier, "hello.title");

    app.cleanup().await;
}

async fn create_project(app: &common::TestApp, name: &str) -> ApiProject {
    let project: ApiProject = app
        .post_json(
            "/api/v1/projects",
            json!({
                "name": name,
                "icon_asset_id": null,
                "source_language": language_value("en", "English")
            }),
        )
        .await
        .assert_status(StatusCode::CREATED)
        .json()
        .await;

    assert_eq!(project.name, name);
    assert_eq!(project.source_language.key, "en");
    project
}

async fn add_language(
    app: &common::TestApp,
    project_public_id: Uuid,
    key: &str,
    name: &str,
) -> ApiTargetLanguage {
    app.post_json(
        &format!("/api/v1/projects/{project_public_id}/languages"),
        language_request(key, name),
    )
    .await
    .assert_status(StatusCode::CREATED)
    .json()
    .await
}

async fn create_namespace(
    app: &common::TestApp,
    project_public_id: Uuid,
    name: &str,
) -> ApiNamespace {
    app.post_json(
        &format!("/api/v1/projects/{project_public_id}/namespaces"),
        json!({ "name": name }),
    )
    .await
    .assert_status(StatusCode::CREATED)
    .json()
    .await
}

async fn create_source_string(
    app: &common::TestApp,
    project_public_id: Uuid,
    namespace_id: i64,
    identifier: &str,
    value: &str,
) -> ApiSourceString {
    app.post_json(
        &format!("/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings"),
        source_string_request(identifier, value),
    )
    .await
    .assert_status(StatusCode::CREATED)
    .json()
    .await
}

async fn assert_namespace_stats(
    app: &common::TestApp,
    namespace_id: i64,
    target_language_id: i64,
    expected: NamespaceLanguageStatsRow,
) {
    let actual = sqlx::query_as::<_, NamespaceLanguageStatsRow>(
        r#"
        SELECT string_count, translated_count, approved_count, candidate_count, missing_count
        FROM namespace_language_stats
        WHERE namespace_id = $1
          AND target_language_id = $2
        "#,
    )
    .bind(namespace_id)
    .bind(target_language_id)
    .fetch_one(app.db())
    .await
    .expect("namespace language stats row should exist");

    assert_eq!(actual.string_count, expected.string_count);
    assert_eq!(actual.translated_count, expected.translated_count);
    assert_eq!(actual.approved_count, expected.approved_count);
    assert_eq!(actual.candidate_count, expected.candidate_count);
    assert_eq!(actual.missing_count, expected.missing_count);
}

fn language_request(key: &str, name: &str) -> Value {
    json!({ "language": language_value(key, name) })
}

fn language_value(key: &str, name: &str) -> Value {
    json!({ "key": key, "name": name })
}

fn source_string_request(identifier: &str, value: &str) -> Value {
    json!({ "identifier": identifier, "value": value })
}
