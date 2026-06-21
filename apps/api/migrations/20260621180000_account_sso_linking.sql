ALTER TABLE oauth_login_states
    ADD COLUMN action TEXT NOT NULL DEFAULT 'login' CHECK (action IN ('login', 'link')),
    ADD COLUMN user_id BIGINT REFERENCES users (id) ON DELETE CASCADE;

ALTER TABLE oauth_login_states
    ADD CONSTRAINT oauth_login_states_link_user_check
    CHECK (
        (action = 'login' AND user_id IS NULL)
        OR (action = 'link' AND user_id IS NOT NULL)
    );
