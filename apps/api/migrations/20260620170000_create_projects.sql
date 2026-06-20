CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER users_set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE projects (
    id BIGSERIAL PRIMARY KEY,
    public_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    icon_asset_id BIGINT,
    source_language_key TEXT NOT NULL,
    source_language_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER projects_set_updated_at
BEFORE UPDATE ON projects
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE project_target_languages (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects (id),
    language_key TEXT NOT NULL,
    language_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (id, project_id)
);

CREATE UNIQUE INDEX project_target_languages_active_key_uidx
ON project_target_languages (project_id, language_key)
WHERE deleted_at IS NULL;

CREATE TRIGGER project_target_languages_set_updated_at
BEFORE UPDATE ON project_target_languages
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE namespaces (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects (id),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (id, project_id)
);

CREATE UNIQUE INDEX namespaces_active_name_uidx
ON namespaces (project_id, name)
WHERE deleted_at IS NULL;

CREATE TRIGGER namespaces_set_updated_at
BEFORE UPDATE ON namespaces
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE source_strings (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL,
    namespace_id BIGINT NOT NULL,
    identifier TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (id, project_id, namespace_id),
    FOREIGN KEY (namespace_id, project_id)
        REFERENCES namespaces (id, project_id)
);

CREATE UNIQUE INDEX source_strings_active_identifier_uidx
ON source_strings (namespace_id, identifier)
WHERE deleted_at IS NULL;

CREATE INDEX source_strings_namespace_id_idx
ON source_strings (namespace_id, id)
WHERE deleted_at IS NULL;

CREATE TRIGGER source_strings_set_updated_at
BEFORE UPDATE ON source_strings
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE translations (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL,
    namespace_id BIGINT NOT NULL,
    string_id BIGINT NOT NULL,
    target_language_id BIGINT NOT NULL,
    author_user_id BIGINT NOT NULL REFERENCES users (id),
    value TEXT NOT NULL,
    rating_score INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (id, string_id, target_language_id),
    FOREIGN KEY (string_id, project_id, namespace_id)
        REFERENCES source_strings (id, project_id, namespace_id),
    FOREIGN KEY (target_language_id, project_id)
        REFERENCES project_target_languages (id, project_id)
);

CREATE INDEX translations_string_language_idx
ON translations (string_id, target_language_id)
WHERE deleted_at IS NULL;

CREATE INDEX translations_best_candidate_idx
ON translations (
    string_id,
    target_language_id,
    rating_score DESC,
    created_at ASC,
    id ASC
)
WHERE deleted_at IS NULL;

CREATE INDEX translations_project_namespace_language_idx
ON translations (project_id, namespace_id, target_language_id, string_id)
WHERE deleted_at IS NULL;

CREATE TABLE translation_votes (
    translation_id BIGINT NOT NULL REFERENCES translations (id),
    user_id BIGINT NOT NULL REFERENCES users (id),
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (translation_id, user_id)
);

CREATE TRIGGER translation_votes_set_updated_at
BEFORE UPDATE ON translation_votes
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE translation_approvals (
    string_id BIGINT NOT NULL,
    target_language_id BIGINT NOT NULL,
    translation_id BIGINT NOT NULL,
    approved_by_user_id BIGINT NOT NULL REFERENCES users (id),
    approved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (string_id, target_language_id),
    FOREIGN KEY (translation_id, string_id, target_language_id)
        REFERENCES translations (id, string_id, target_language_id)
);

CREATE TABLE current_translations (
    project_id BIGINT NOT NULL,
    namespace_id BIGINT NOT NULL,
    string_id BIGINT NOT NULL,
    target_language_id BIGINT NOT NULL,
    current_translation_id BIGINT,
    approved_translation_id BIGINT,
    best_rated_translation_id BIGINT,
    candidate_count INTEGER NOT NULL DEFAULT 0 CHECK (candidate_count >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (string_id, target_language_id),
    FOREIGN KEY (string_id, project_id, namespace_id)
        REFERENCES source_strings (id, project_id, namespace_id),
    FOREIGN KEY (target_language_id, project_id)
        REFERENCES project_target_languages (id, project_id),
    FOREIGN KEY (current_translation_id, string_id, target_language_id)
        REFERENCES translations (id, string_id, target_language_id),
    FOREIGN KEY (approved_translation_id, string_id, target_language_id)
        REFERENCES translations (id, string_id, target_language_id),
    FOREIGN KEY (best_rated_translation_id, string_id, target_language_id)
        REFERENCES translations (id, string_id, target_language_id),
    CHECK (
        current_translation_id IS NULL
        OR current_translation_id = approved_translation_id
        OR current_translation_id = best_rated_translation_id
    )
);

CREATE INDEX current_translations_target_string_idx
ON current_translations (target_language_id, string_id);

CREATE TABLE namespace_language_stats (
    project_id BIGINT NOT NULL,
    namespace_id BIGINT NOT NULL,
    target_language_id BIGINT NOT NULL,
    string_count INTEGER NOT NULL DEFAULT 0 CHECK (string_count >= 0),
    translated_count INTEGER NOT NULL DEFAULT 0 CHECK (translated_count >= 0),
    approved_count INTEGER NOT NULL DEFAULT 0 CHECK (approved_count >= 0),
    candidate_count INTEGER NOT NULL DEFAULT 0 CHECK (candidate_count >= 0),
    missing_count INTEGER NOT NULL DEFAULT 0 CHECK (missing_count >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (namespace_id, target_language_id),
    FOREIGN KEY (namespace_id, project_id)
        REFERENCES namespaces (id, project_id),
    FOREIGN KEY (target_language_id, project_id)
        REFERENCES project_target_languages (id, project_id)
);
