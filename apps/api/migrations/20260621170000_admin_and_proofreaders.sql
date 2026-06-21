ALTER TABLE users
ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE project_language_proofreaders (
    project_id BIGINT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    target_language_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    granted_by_user_id BIGINT REFERENCES users (id) ON DELETE SET NULL,
    PRIMARY KEY (project_id, target_language_id, user_id),
    FOREIGN KEY (target_language_id, project_id)
        REFERENCES project_target_languages (id, project_id)
        ON DELETE CASCADE
);

CREATE INDEX project_language_proofreaders_user_idx
ON project_language_proofreaders (user_id, project_id, target_language_id);
