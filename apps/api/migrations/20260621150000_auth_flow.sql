CREATE EXTENSION IF NOT EXISTS citext;

ALTER TABLE users
    ADD COLUMN email CITEXT,
    ADD COLUMN password_hash TEXT,
    ADD COLUMN email_verified_at TIMESTAMPTZ,
    ADD COLUMN avatar_url TEXT,
    ADD COLUMN last_login_at TIMESTAMPTZ,
    ADD COLUMN disabled_at TIMESTAMPTZ;

CREATE UNIQUE INDEX users_email_uidx
ON users (email)
WHERE email IS NOT NULL;

CREATE TABLE auth_identities (
    provider TEXT NOT NULL CHECK (provider IN ('github', 'gitlab')),
    provider_user_id TEXT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    email CITEXT,
    username TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, provider_user_id),
    UNIQUE (provider, user_id)
);

CREATE TRIGGER auth_identities_set_updated_at
BEFORE UPDATE ON auth_identities
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE auth_sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    access_token_hash TEXT NOT NULL UNIQUE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    csrf_token_hash TEXT NOT NULL,
    access_expires_at TIMESTAMPTZ NOT NULL,
    refresh_expires_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    user_agent TEXT,
    ip_address TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (access_expires_at < refresh_expires_at)
);

CREATE INDEX auth_sessions_active_access_idx
ON auth_sessions (access_token_hash)
WHERE revoked_at IS NULL;

CREATE INDEX auth_sessions_active_refresh_idx
ON auth_sessions (refresh_token_hash)
WHERE revoked_at IS NULL;

CREATE INDEX auth_sessions_user_idx
ON auth_sessions (user_id)
WHERE revoked_at IS NULL;

CREATE TRIGGER auth_sessions_set_updated_at
BEFORE UPDATE ON auth_sessions
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE password_reset_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    requested_user_agent TEXT,
    requested_ip_address TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX password_reset_tokens_user_idx
ON password_reset_tokens (user_id, created_at DESC);

CREATE TABLE oauth_login_states (
    id BIGSERIAL PRIMARY KEY,
    state_hash TEXT NOT NULL UNIQUE,
    provider TEXT NOT NULL CHECK (provider IN ('github', 'gitlab')),
    pkce_verifier_ciphertext BYTEA NOT NULL,
    redirect_to TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX oauth_login_states_active_idx
ON oauth_login_states (state_hash)
WHERE used_at IS NULL;

CREATE TABLE auth_attempts (
    id BIGSERIAL PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('login', 'password_reset', 'sso')),
    email CITEXT,
    ip_address TEXT,
    success BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX auth_attempts_kind_email_idx
ON auth_attempts (kind, email, created_at DESC);
