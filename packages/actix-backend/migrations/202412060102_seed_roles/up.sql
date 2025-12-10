CREATE TABLE IF NOT EXISTS roles (
    name TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO
    roles (name, description)
VALUES (
        'admin',
        'Full catalog access including rebuild + admin tooling'
    ),
    (
        'contributor',
        'Can upload assets and manage metadata edits'
    ),
    (
        'guest',
        'Read-only access to browse and download maps'
    ) ON CONFLICT (name) DO NOTHING;

ALTER TABLE users
ADD CONSTRAINT fk_users_role FOREIGN KEY (role) REFERENCES roles (name) ON UPDATE CASCADE ON DELETE RESTRICT;