mod common;

use axum::http::StatusCode;
use fosslate_api::{
    jobs::{rebuild_projections, rebuild_stats},
    services::Services,
};
use serde_json::{Value, json};

#[derive(Debug, sqlx::FromRow)]
struct StatsRow {
    string_count: i32,
    translated_count: i32,
    approved_count: i32,
    candidate_count: i32,
    missing_count: i32,
}

struct Fixture {
    project_public_id: String,
    namespace_id: i64,
    target_language_id: i64,
    author_user_id: i64,
    approver_user_id: i64,
}

#[tokio::test]
async fn stats_count_strings_translations_approvals_candidates_and_missing_strings() {
    let app = common::spawn_app().await;
    let fixture = create_fixture(&app).await;

    let first_string_id = app
        .create_string(
            &fixture.project_public_id,
            fixture.namespace_id,
            "home.title",
            "Home",
        )
        .await;
    app.create_string(
        &fixture.project_public_id,
        fixture.namespace_id,
        "home.subtitle",
        "Subtitle",
    )
    .await;
    let first_translation_id = app
        .create_translation(
            &fixture.project_public_id,
            first_string_id,
            fixture.target_language_id,
            fixture.author_user_id,
            "Bienvenue",
        )
        .await;
    app.create_translation(
        &fixture.project_public_id,
        first_string_id,
        fixture.target_language_id,
        fixture.author_user_id,
        "Accueil",
    )
    .await;

    assert_stats(
        &app,
        &fixture,
        StatsRow {
            string_count: 2,
            translated_count: 1,
            approved_count: 0,
            candidate_count: 2,
            missing_count: 1,
        },
    )
    .await;

    approve_translation(&app, &fixture, first_translation_id).await;

    assert_stats(
        &app,
        &fixture,
        StatsRow {
            string_count: 2,
            translated_count: 1,
            approved_count: 1,
            candidate_count: 2,
            missing_count: 1,
        },
    )
    .await;

    app.cleanup().await;
}

#[tokio::test]
async fn stats_do_not_inflate_string_count_and_update_after_deletions() {
    let app = common::spawn_app().await;
    let fixture = create_fixture(&app).await;

    let first_string_id = app
        .create_string(
            &fixture.project_public_id,
            fixture.namespace_id,
            "home.title",
            "Home",
        )
        .await;
    let second_string_id = app
        .create_string(
            &fixture.project_public_id,
            fixture.namespace_id,
            "home.subtitle",
            "Subtitle",
        )
        .await;
    let first_translation_id = app
        .create_translation(
            &fixture.project_public_id,
            first_string_id,
            fixture.target_language_id,
            fixture.author_user_id,
            "Bienvenue",
        )
        .await;
    app.create_translation(
        &fixture.project_public_id,
        first_string_id,
        fixture.target_language_id,
        fixture.author_user_id,
        "Accueil",
    )
    .await;
    let second_translation_id = app
        .create_translation(
            &fixture.project_public_id,
            second_string_id,
            fixture.target_language_id,
            fixture.author_user_id,
            "Sous-titre",
        )
        .await;
    approve_translation(&app, &fixture, first_translation_id).await;

    assert_stats(
        &app,
        &fixture,
        StatsRow {
            string_count: 2,
            translated_count: 2,
            approved_count: 1,
            candidate_count: 3,
            missing_count: 0,
        },
    )
    .await;

    app.delete(&format!(
        "/api/v1/projects/{}/translations/{second_translation_id}",
        fixture.project_public_id
    ))
    .await
    .assert_status(StatusCode::NO_CONTENT);

    assert_stats(
        &app,
        &fixture,
        StatsRow {
            string_count: 2,
            translated_count: 1,
            approved_count: 1,
            candidate_count: 2,
            missing_count: 1,
        },
    )
    .await;

    app.delete(&format!(
        "/api/v1/projects/{}/strings/{second_string_id}",
        fixture.project_public_id
    ))
    .await
    .assert_status(StatusCode::NO_CONTENT);

    assert_stats(
        &app,
        &fixture,
        StatsRow {
            string_count: 1,
            translated_count: 1,
            approved_count: 1,
            candidate_count: 2,
            missing_count: 0,
        },
    )
    .await;

    app.cleanup().await;
}

#[tokio::test]
async fn rebuild_jobs_are_idempotent_and_repair_stale_projection_rows() {
    let app = common::spawn_app().await;
    let fixture = create_fixture(&app).await;

    let first_string_id = app
        .create_string(
            &fixture.project_public_id,
            fixture.namespace_id,
            "home.title",
            "Home",
        )
        .await;
    let second_string_id = app
        .create_string(
            &fixture.project_public_id,
            fixture.namespace_id,
            "home.subtitle",
            "Subtitle",
        )
        .await;
    let first_translation_id = app
        .create_translation(
            &fixture.project_public_id,
            first_string_id,
            fixture.target_language_id,
            fixture.author_user_id,
            "Bienvenue",
        )
        .await;
    app.create_translation(
        &fixture.project_public_id,
        second_string_id,
        fixture.target_language_id,
        fixture.author_user_id,
        "Sous-titre",
    )
    .await;
    approve_translation(&app, &fixture, first_translation_id).await;

    sqlx::query("DELETE FROM current_translations")
        .execute(app.pool())
        .await
        .unwrap();
    sqlx::query(
        r#"
        UPDATE namespace_language_stats
        SET string_count = 99,
            translated_count = 99,
            approved_count = 99,
            candidate_count = 99,
            missing_count = 99
        WHERE namespace_id = $1
          AND target_language_id = $2
        "#,
    )
    .bind(fixture.namespace_id)
    .bind(fixture.target_language_id)
    .execute(app.pool())
    .await
    .unwrap();

    let services = Services::new(app.pool().clone());
    rebuild_projections::rebuild_current_translations(&services)
        .await
        .unwrap();
    rebuild_stats::rebuild_namespace_language_stats(&services)
        .await
        .unwrap();

    assert_current_row_count(&app, &fixture, 2).await;
    assert_stats(
        &app,
        &fixture,
        StatsRow {
            string_count: 2,
            translated_count: 2,
            approved_count: 1,
            candidate_count: 2,
            missing_count: 0,
        },
    )
    .await;

    rebuild_projections::rebuild_current_translations(&services)
        .await
        .unwrap();
    rebuild_stats::rebuild_namespace_language_stats(&services)
        .await
        .unwrap();

    assert_current_row_count(&app, &fixture, 2).await;
    assert_stats(
        &app,
        &fixture,
        StatsRow {
            string_count: 2,
            translated_count: 2,
            approved_count: 1,
            candidate_count: 2,
            missing_count: 0,
        },
    )
    .await;

    app.cleanup().await;
}

#[tokio::test]
async fn openapi_json_contains_route_groups_and_core_paths() {
    let app = common::spawn_app().await;

    let document: Value = app
        .get("/openapi.json")
        .await
        .assert_status(StatusCode::OK)
        .json()
        .await;

    let tags = document["tags"].as_array().unwrap();
    for tag in [
        "approvals",
        "health",
        "languages",
        "meta",
        "namespaces",
        "projects",
        "strings",
        "translations",
        "users",
        "votes",
    ] {
        assert!(
            tags.iter().any(|entry| entry["name"] == tag),
            "missing OpenAPI tag {tag}"
        );
    }

    let paths = document["paths"].as_object().unwrap();
    for path in [
        "/api/v1/users",
        "/api/v1/projects",
        "/api/v1/projects/{project_public_id}/languages",
        "/api/v1/projects/{project_public_id}/namespaces",
        "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings",
        "/api/v1/projects/{project_public_id}/strings/{string_id}/translations",
        "/api/v1/projects/{project_public_id}/translations/{translation_id}/vote",
        "/api/v1/projects/{project_public_id}/translations/{translation_id}/approval",
        "/api/v1/projects/{project_public_id}/strings/{string_id}/approvals/{target_language_id}",
    ] {
        assert!(paths.contains_key(path), "missing OpenAPI path {path}");
    }

    app.cleanup().await;
}

async fn create_fixture(app: &common::TestApp) -> Fixture {
    let author_user_id = app.create_user("stats_author").await;
    let approver_user_id = app.create_user("stats_approver").await;
    let project = app.create_project("Stats Project").await;
    let target_language_id = app
        .add_language(&project.public_id, "fr-FR", "French")
        .await;
    let namespace_id = app.create_namespace(&project.public_id, "common").await;

    Fixture {
        project_public_id: project.public_id,
        namespace_id,
        target_language_id,
        author_user_id,
        approver_user_id,
    }
}

async fn approve_translation(
    app: &common::TestApp,
    fixture: &Fixture,
    translation_id: i64,
) {
    app.put_json(
        &format!(
            "/api/v1/projects/{}/translations/{translation_id}/approval",
            fixture.project_public_id
        ),
        json!({ "approved_by_user_id": fixture.approver_user_id }),
    )
    .await
    .assert_status(StatusCode::OK);
}

async fn assert_stats(app: &common::TestApp, fixture: &Fixture, expected: StatsRow) {
    let actual = sqlx::query_as::<_, StatsRow>(
        r#"
        SELECT string_count, translated_count, approved_count, candidate_count, missing_count
        FROM namespace_language_stats
        WHERE namespace_id = $1
          AND target_language_id = $2
        "#,
    )
    .bind(fixture.namespace_id)
    .bind(fixture.target_language_id)
    .fetch_one(app.pool())
    .await
    .unwrap();

    assert_eq!(actual.string_count, expected.string_count);
    assert_eq!(actual.translated_count, expected.translated_count);
    assert_eq!(actual.approved_count, expected.approved_count);
    assert_eq!(actual.candidate_count, expected.candidate_count);
    assert_eq!(actual.missing_count, expected.missing_count);
}

async fn assert_current_row_count(app: &common::TestApp, fixture: &Fixture, expected: i64) {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM current_translations
        WHERE namespace_id = $1
          AND target_language_id = $2
        "#,
    )
    .bind(fixture.namespace_id)
    .bind(fixture.target_language_id)
    .fetch_one(app.pool())
    .await
    .unwrap();

    assert_eq!(count, expected);
}
