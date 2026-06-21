CREATE TABLE instance_setup (
    id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO instance_setup (id)
VALUES (1)
ON CONFLICT (id) DO NOTHING;

CREATE TRIGGER instance_setup_set_updated_at
BEFORE UPDATE ON instance_setup
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE auth_provider_configs (
    provider TEXT PRIMARY KEY CHECK (provider IN ('github', 'gitlab')),
    enabled BOOLEAN NOT NULL DEFAULT false,
    skipped_at TIMESTAMPTZ,
    base_url TEXT,
    client_id TEXT,
    client_secret_ciphertext BYTEA,
    scopes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    configured_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        enabled = false
        OR (
            client_id IS NOT NULL
            AND client_secret_ciphertext IS NOT NULL
        )
    )
);

CREATE TRIGGER auth_provider_configs_set_updated_at
BEFORE UPDATE ON auth_provider_configs
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE email_delivery_config (
    id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    provider TEXT NOT NULL DEFAULT 'resend' CHECK (provider = 'resend'),
    api_key_ciphertext BYTEA NOT NULL,
    from_name TEXT NOT NULL,
    from_email TEXT NOT NULL,
    last_test_recipient TEXT NOT NULL,
    last_tested_at TIMESTAMPTZ NOT NULL,
    last_test_message_id TEXT NOT NULL,
    configured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER email_delivery_config_set_updated_at
BEFORE UPDATE ON email_delivery_config
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
