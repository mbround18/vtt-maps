CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    discord_id TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL,
    avatar_url TEXT,
    email TEXT,
    role TEXT NOT NULL DEFAULT 'guest' CONSTRAINT users_role_check CHECK (
        role IN (
            'admin',
            'contributor',
            'guest'
        )
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_discord_id ON users (discord_id);