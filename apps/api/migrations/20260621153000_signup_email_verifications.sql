CREATE TABLE signup_email_verifications (
    id BIGSERIAL PRIMARY KEY,
    email CITEXT NOT NULL,
    code_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    requested_user_agent TEXT,
    requested_ip_address TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX signup_email_verifications_email_idx
    ON signup_email_verifications (email, created_at DESC);

CREATE UNIQUE INDEX signup_email_verifications_active_email_uidx
    ON signup_email_verifications (email)
    WHERE used_at IS NULL;
