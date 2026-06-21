use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod common;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageBody {
    key: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct CreateUserBody {
    username: String,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    id: i64,
}

#[derive(Debug, Serialize)]
struct CreateProjectBody {
    name: String,
    icon_asset_id: Option<i64>,
    source_language: LanguageBody,
}

#[derive(Debug, Deserialize)]
struct ProjectResponse {
    id: i64,
    public_id: Uuid,
}

#[derive(Debug, Serialize)]
struct AddTargetLanguageBody {
    language: LanguageBody,
}

#[derive(Debug, Deserialize)]
struct TargetLanguageResponse {
    id: i64,
    project_id: i64,
}

#[derive(Debug, Serialize)]
struct NamespaceBody {
    name: String,
}

#[derive(Debug, Deserialize)]
struct NamespaceResponse {
    id: i64,
    project_id: i64,
}

#[derive(Debug, Serialize)]
struct SourceStringBody {
    identifier: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct SourceStringResponse {
    id: i64,
    project_id: i64,
    namespace_id: i64,
}

#[derive(Debug, Serialize)]
struct CreateTranslationBody {
    target_language_id: i64,
    author_user_id: i64,
    value: String,
}

#[derive(Debug, Deserialize)]
struct TranslationResponse {
    id: i64,
    project_id: i64,
    namespace_id: i64,
    string_id: i64,
    target_language_id: i64,
    author_user_id: i64,
    value: String,
    rating_score: i32,
}

#[derive(Debug, Serialize)]
struct SetVoteBody {
    user_id: i64,
    vote: i16,
}

#[derive(Debug, Serialize)]
struct ApproveTranslationBody {
    approved_by_user_id: i64,
}

#[derive(Debug, Deserialize)]
struct CurrentTranslationResponse {
    project_id: i64,
    namespace_id: i64,
    string_id: i64,
    target_language_id: i64,
    current_translation_id: Option<i64>,
    approved_translation_id: Option<i64>,
    best_rated_translation_id: Option<i64>,
    candidate_count: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct CurrentTranslationRow {
    project_id: i64,
    namespace_id: i64,
    string_id: i64,
    target_language_id: i64,
    current_translation_id: Option<i64>,
    approved_translation_id: Option<i64>,
    best_rated_translation_id: Option<i64>,
    candidate_count: i32,
}

#[derive(Debug)]
struct Seed {
    user_id: i64,
    reviewer_id: i64,
    project_id: i64,
    project_public_id: Uuid,
    namespace_id: i64,
    string_id: i64,
    target_language_id: i64,
}

#[tokio::test]
async fn create_and_list_translations_enforces_project_language_and_user_constraints() {
    let app = common::TestApp::spawn().await;
    let seed = seed_project(&app, "constraints").await;
    let other = seed_project(&app, "other-project").await;

    let translation = create_translation(&app, &seed, "Bonjour").await;
    assert_eq!(translation.project_id, seed.project_id);
    assert_eq!(translation.namespace_id, seed.namespace_id);
    assert_eq!(translation.string_id, seed.string_id);
    assert_eq!(translation.target_language_id, seed.target_language_id);
    assert_eq!(translation.author_user_id, seed.user_id);
    assert_eq!(translation.value, "Bonjour");
    assert_eq!(translation.rating_score, 0);

    let translations: Vec<TranslationResponse> = app
        .get_typed(
            &format!(
                "/api/v1/projects/{}/strings/{}/translations?target_language_id={}",
                seed.project_public_id, seed.string_id, seed.target_language_id
            ),
            StatusCode::OK,
        )
        .await;
    assert_eq!(translations.len(), 1);
    assert_eq!(translations[0].id, translation.id);

    app.post_json_expect_status(
        &format!(
            "/api/v1/projects/{}/strings/{}/translations",
            other.project_public_id, seed.string_id
        ),
        &CreateTranslationBody {
            target_language_id: seed.target_language_id,
            author_user_id: seed.user_id,
            value: "wrong project".to_string(),
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.post_json_expect_status(
        &format!(
            "/api/v1/projects/{}/strings/{}/translations",
            seed.project_public_id, seed.string_id
        ),
        &CreateTranslationBody {
            target_language_id: other.target_language_id,
            author_user_id: seed.user_id,
            value: "wrong language".to_string(),
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.post_json_expect_status(
        &format!(
            "/api/v1/projects/{}/strings/{}/translations",
            seed.project_public_id, seed.string_id
        ),
        &CreateTranslationBody {
            target_language_id: seed.target_language_id,
            author_user_id: 9_999_999,
            value: "missing user".to_string(),
        },
        StatusCode::FORBIDDEN,
    )
    .await;

    app.cleanup().await;
}

#[tokio::test]
async fn translations_order_deterministically_and_soft_delete_updates_current_translation() {
    let app = common::TestApp::spawn().await;
    let seed = seed_project(&app, "ordering-delete").await;

    let first = create_translation(&app, &seed, "first").await;
    let second = create_translation(&app, &seed, "second").await;

    let listed = list_translations(&app, &seed).await;
    assert_eq!(ids(&listed), vec![first.id, second.id]);

    let voted_second = vote(&app, seed.project_public_id, second.id, seed.user_id, 1).await;
    assert_eq!(voted_second.rating_score, 1);

    let listed = list_translations(&app, &seed).await;
    assert_eq!(ids(&listed), vec![second.id, first.id]);

    let voted_first = vote(&app, seed.project_public_id, first.id, seed.user_id, 1).await;
    assert_eq!(voted_first.rating_score, 1);

    let listed = list_translations(&app, &seed).await;
    assert_eq!(ids(&listed), vec![first.id, second.id]);

    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("current translation should exist");
    assert_eq!(current.current_translation_id, Some(first.id));
    assert_eq!(current.best_rated_translation_id, Some(first.id));
    assert_eq!(current.approved_translation_id, None);
    assert_eq!(current.candidate_count, 2);

    app.delete_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}",
            seed.project_public_id, first.id
        ),
        StatusCode::NO_CONTENT,
    )
    .await;

    let listed = list_translations(&app, &seed).await;
    assert_eq!(ids(&listed), vec![second.id]);

    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("current translation should remain after deleting one candidate");
    assert_eq!(current.current_translation_id, Some(second.id));
    assert_eq!(current.best_rated_translation_id, Some(second.id));
    assert_eq!(current.candidate_count, 1);

    app.delete_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}",
            seed.project_public_id, second.id
        ),
        StatusCode::NO_CONTENT,
    )
    .await;

    let listed = list_translations(&app, &seed).await;
    assert!(listed.is_empty());

    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("the existing sparse row is nulled when the last candidate is deleted");
    assert_eq!(current.current_translation_id, None);
    assert_eq!(current.best_rated_translation_id, None);
    assert_eq!(current.approved_translation_id, None);
    assert_eq!(current.candidate_count, 0);

    app.cleanup().await;
}

#[tokio::test]
async fn votes_apply_score_deltas_repeated_votes_bad_votes_and_project_scope() {
    let app = common::TestApp::spawn().await;
    let seed = seed_project(&app, "votes").await;
    let other = seed_project(&app, "votes-other").await;

    let first = create_translation(&app, &seed, "first").await;
    let second = create_translation(&app, &seed, "second").await;

    let first = vote(&app, seed.project_public_id, first.id, seed.user_id, 1).await;
    assert_eq!(first.rating_score, 1);

    let first = vote(&app, seed.project_public_id, first.id, seed.user_id, 1).await;
    assert_eq!(first.rating_score, 1);

    let first = vote(&app, seed.project_public_id, first.id, seed.user_id, -1).await;
    assert_eq!(first.rating_score, -1);

    let second = vote(&app, seed.project_public_id, second.id, seed.user_id, 1).await;
    assert_eq!(second.rating_score, 1);

    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("current translation should exist");
    assert_eq!(current.current_translation_id, Some(second.id));
    assert_eq!(current.best_rated_translation_id, Some(second.id));
    assert_eq!(current.candidate_count, 2);

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/vote",
            seed.project_public_id, second.id
        ),
        &SetVoteBody {
            user_id: seed.reviewer_id,
            vote: 0,
        },
        StatusCode::BAD_REQUEST,
    )
    .await;

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/vote",
            other.project_public_id, second.id
        ),
        &SetVoteBody {
            user_id: seed.reviewer_id,
            vote: 1,
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.cleanup().await;
}

#[tokio::test]
async fn translation_mutations_reject_deleted_source_strings_and_target_languages() {
    let app = common::TestApp::spawn().await;
    let deleted_string_seed = seed_project(&app, "deleted-string-parent").await;
    let deleted_string_translation =
        create_translation(&app, &deleted_string_seed, "Bonjour").await;

    app.delete_expect_status(
        &format!(
            "/api/v1/projects/{}/strings/{}",
            deleted_string_seed.project_public_id, deleted_string_seed.string_id
        ),
        StatusCode::NO_CONTENT,
    )
    .await;

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/vote",
            deleted_string_seed.project_public_id, deleted_string_translation.id
        ),
        &SetVoteBody {
            user_id: deleted_string_seed.user_id,
            vote: 1,
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/approval",
            deleted_string_seed.project_public_id, deleted_string_translation.id
        ),
        &ApproveTranslationBody {
            approved_by_user_id: deleted_string_seed.reviewer_id,
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    let deleted_language_seed = seed_project(&app, "deleted-language-parent").await;
    let deleted_language_translation =
        create_translation(&app, &deleted_language_seed, "Bonjour").await;

    app.delete_expect_status(
        &format!(
            "/api/v1/projects/{}/languages/{}",
            deleted_language_seed.project_public_id, deleted_language_seed.target_language_id
        ),
        StatusCode::NO_CONTENT,
    )
    .await;

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/vote",
            deleted_language_seed.project_public_id, deleted_language_translation.id
        ),
        &SetVoteBody {
            user_id: deleted_language_seed.user_id,
            vote: 1,
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/approval",
            deleted_language_seed.project_public_id, deleted_language_translation.id
        ),
        &ApproveTranslationBody {
            approved_by_user_id: deleted_language_seed.reviewer_id,
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.cleanup().await;
}

#[tokio::test]
async fn approvals_override_reapprove_remove_fallback_missing_and_wrong_project() {
    let app = common::TestApp::spawn().await;
    let seed = seed_project(&app, "approvals").await;
    let other = seed_project(&app, "approvals-other").await;

    let first = create_translation(&app, &seed, "first").await;
    let second = create_translation(&app, &seed, "second").await;

    vote(&app, seed.project_public_id, second.id, seed.user_id, 1).await;

    let approved_first = approve(&app, seed.project_public_id, first.id, seed.reviewer_id).await;
    assert_current_response(
        &approved_first,
        &seed,
        Some(first.id),
        Some(first.id),
        Some(second.id),
        2,
    );

    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("approval should upsert current translation");
    assert_current_row(
        &current,
        &seed,
        Some(first.id),
        Some(first.id),
        Some(second.id),
        2,
    );

    let approved_second = approve(&app, seed.project_public_id, second.id, seed.reviewer_id).await;
    assert_current_response(
        &approved_second,
        &seed,
        Some(second.id),
        Some(second.id),
        Some(second.id),
        2,
    );

    let removed: CurrentTranslationResponse = app
        .delete_typed(
            &format!(
                "/api/v1/projects/{}/strings/{}/approvals/{}",
                seed.project_public_id, seed.string_id, seed.target_language_id
            ),
            StatusCode::OK,
        )
        .await;
    assert_current_response(&removed, &seed, Some(second.id), None, Some(second.id), 2);

    app.delete_expect_status(
        &format!(
            "/api/v1/projects/{}/strings/{}/approvals/{}",
            seed.project_public_id, seed.string_id, seed.target_language_id
        ),
        StatusCode::NOT_FOUND,
    )
    .await;

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/approval",
            other.project_public_id, first.id
        ),
        &ApproveTranslationBody {
            approved_by_user_id: seed.reviewer_id,
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.put_json_expect_status(
        &format!(
            "/api/v1/projects/{}/translations/{}/approval",
            seed.project_public_id, 9_999_999
        ),
        &ApproveTranslationBody {
            approved_by_user_id: seed.reviewer_id,
        },
        StatusCode::NOT_FOUND,
    )
    .await;

    app.cleanup().await;
}

#[tokio::test]
async fn current_translations_are_sparse_until_first_candidate_and_track_candidate_count() {
    let app = common::TestApp::spawn().await;
    let seed = seed_project(&app, "sparse-current").await;

    assert!(
        current_translation(&app, seed.string_id, seed.target_language_id)
            .await
            .is_none()
    );

    let first = create_translation(&app, &seed, "first").await;
    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("first candidate should create sparse current row");
    assert_current_row(&current, &seed, Some(first.id), None, Some(first.id), 1);

    let second = create_translation(&app, &seed, "second").await;
    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("second candidate should update current row");
    assert_current_row(&current, &seed, Some(first.id), None, Some(first.id), 2);

    vote(&app, seed.project_public_id, second.id, seed.user_id, 1).await;
    let current = current_translation(&app, seed.string_id, seed.target_language_id)
        .await
        .expect("vote should recompute current row");
    assert_current_row(&current, &seed, Some(second.id), None, Some(second.id), 2);

    app.cleanup().await;
}

async fn seed_project(app: &common::TestApp, label: &str) -> Seed {
    let suffix = Uuid::new_v4().simple().to_string();
    let user = create_user(app, &format!("{label}-author-{suffix}")).await;
    let reviewer = create_user(app, &format!("{label}-reviewer-{suffix}")).await;
    let project = create_project(app, &format!("{label}-project-{suffix}")).await;
    let target_language = add_target_language(app, project.public_id, "fr", "French").await;
    let namespace = create_namespace(app, project.public_id, &format!("{label}-namespace")).await;
    let source_string = create_source_string(
        app,
        project.public_id,
        namespace.id,
        &format!("{label}.hello"),
        "Hello",
    )
    .await;

    assert_eq!(target_language.project_id, project.id);
    assert_eq!(namespace.project_id, project.id);
    assert_eq!(source_string.project_id, project.id);
    assert_eq!(source_string.namespace_id, namespace.id);

    Seed {
        user_id: user.id,
        reviewer_id: reviewer.id,
        project_id: project.id,
        project_public_id: project.public_id,
        namespace_id: namespace.id,
        string_id: source_string.id,
        target_language_id: target_language.id,
    }
}

async fn create_user(app: &common::TestApp, username: &str) -> UserResponse {
    app.post_typed(
        "/api/v1/users",
        &CreateUserBody {
            username: username.to_string(),
        },
        StatusCode::CREATED,
    )
    .await
}

async fn create_project(app: &common::TestApp, name: &str) -> ProjectResponse {
    app.post_typed(
        "/api/v1/projects",
        &CreateProjectBody {
            name: name.to_string(),
            icon_asset_id: None,
            source_language: LanguageBody {
                key: "en".to_string(),
                name: "English".to_string(),
            },
        },
        StatusCode::CREATED,
    )
    .await
}

async fn add_target_language(
    app: &common::TestApp,
    project_public_id: Uuid,
    key: &str,
    name: &str,
) -> TargetLanguageResponse {
    app.post_typed(
        &format!("/api/v1/projects/{project_public_id}/languages"),
        &AddTargetLanguageBody {
            language: LanguageBody {
                key: key.to_string(),
                name: name.to_string(),
            },
        },
        StatusCode::CREATED,
    )
    .await
}

async fn create_namespace(
    app: &common::TestApp,
    project_public_id: Uuid,
    name: &str,
) -> NamespaceResponse {
    app.post_typed(
        &format!("/api/v1/projects/{project_public_id}/namespaces"),
        &NamespaceBody {
            name: name.to_string(),
        },
        StatusCode::CREATED,
    )
    .await
}

async fn create_source_string(
    app: &common::TestApp,
    project_public_id: Uuid,
    namespace_id: i64,
    identifier: &str,
    value: &str,
) -> SourceStringResponse {
    app.post_typed(
        &format!("/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings"),
        &SourceStringBody {
            identifier: identifier.to_string(),
            value: value.to_string(),
        },
        StatusCode::CREATED,
    )
    .await
}

async fn create_translation(
    app: &common::TestApp,
    seed: &Seed,
    value: &str,
) -> TranslationResponse {
    app.post_typed(
        &format!(
            "/api/v1/projects/{}/strings/{}/translations",
            seed.project_public_id, seed.string_id
        ),
        &CreateTranslationBody {
            target_language_id: seed.target_language_id,
            author_user_id: seed.user_id,
            value: value.to_string(),
        },
        StatusCode::CREATED,
    )
    .await
}

async fn list_translations(app: &common::TestApp, seed: &Seed) -> Vec<TranslationResponse> {
    app.get_typed(
        &format!(
            "/api/v1/projects/{}/strings/{}/translations?target_language_id={}",
            seed.project_public_id, seed.string_id, seed.target_language_id
        ),
        StatusCode::OK,
    )
    .await
}

async fn vote(
    app: &common::TestApp,
    project_public_id: Uuid,
    translation_id: i64,
    user_id: i64,
    vote: i16,
) -> TranslationResponse {
    app.put_typed(
        &format!("/api/v1/projects/{project_public_id}/translations/{translation_id}/vote"),
        &SetVoteBody { user_id, vote },
        StatusCode::OK,
    )
    .await
}

async fn approve(
    app: &common::TestApp,
    project_public_id: Uuid,
    translation_id: i64,
    approved_by_user_id: i64,
) -> CurrentTranslationResponse {
    app.put_typed(
        &format!("/api/v1/projects/{project_public_id}/translations/{translation_id}/approval"),
        &ApproveTranslationBody {
            approved_by_user_id,
        },
        StatusCode::OK,
    )
    .await
}

async fn current_translation(
    app: &common::TestApp,
    string_id: i64,
    target_language_id: i64,
) -> Option<CurrentTranslationRow> {
    sqlx::query_as::<_, CurrentTranslationRow>(
        r#"
        SELECT
            project_id,
            namespace_id,
            string_id,
            target_language_id,
            current_translation_id,
            approved_translation_id,
            best_rated_translation_id,
            candidate_count
        FROM current_translations
        WHERE string_id = $1
          AND target_language_id = $2
        "#,
    )
    .bind(string_id)
    .bind(target_language_id)
    .fetch_optional(app.pool())
    .await
    .expect("current_translations query should succeed")
}

fn ids(translations: &[TranslationResponse]) -> Vec<i64> {
    translations
        .iter()
        .map(|translation| translation.id)
        .collect()
}

fn assert_current_response(
    current: &CurrentTranslationResponse,
    seed: &Seed,
    current_translation_id: Option<i64>,
    approved_translation_id: Option<i64>,
    best_rated_translation_id: Option<i64>,
    candidate_count: i32,
) {
    assert_eq!(current.project_id, seed.project_id);
    assert_eq!(current.namespace_id, seed.namespace_id);
    assert_eq!(current.string_id, seed.string_id);
    assert_eq!(current.target_language_id, seed.target_language_id);
    assert_eq!(current.current_translation_id, current_translation_id);
    assert_eq!(current.approved_translation_id, approved_translation_id);
    assert_eq!(current.best_rated_translation_id, best_rated_translation_id);
    assert_eq!(current.candidate_count, candidate_count);
}

fn assert_current_row(
    current: &CurrentTranslationRow,
    seed: &Seed,
    current_translation_id: Option<i64>,
    approved_translation_id: Option<i64>,
    best_rated_translation_id: Option<i64>,
    candidate_count: i32,
) {
    assert_eq!(current.project_id, seed.project_id);
    assert_eq!(current.namespace_id, seed.namespace_id);
    assert_eq!(current.string_id, seed.string_id);
    assert_eq!(current.target_language_id, seed.target_language_id);
    assert_eq!(current.current_translation_id, current_translation_id);
    assert_eq!(current.approved_translation_id, approved_translation_id);
    assert_eq!(current.best_rated_translation_id, best_rated_translation_id);
    assert_eq!(current.candidate_count, candidate_count);
}
