CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id)
        ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,

    access_token_hash BYTEA NOT NULL UNIQUE,
    access_expires_at TIMESTAMPTZ NOT NULL,

    refresh_token_hash BYTEA NOT NULL UNIQUE,
    refresh_expires_at TIMESTAMPTZ NOT NULL,
    absolute_expires_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX sessions_user_id_idx ON sessions(user_id);
