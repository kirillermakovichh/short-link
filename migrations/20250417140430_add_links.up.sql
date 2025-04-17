-- Add up migration script here
CREATE TABLE links (
    id TEXT PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    redirect_url TEXT NOT NULL,
    label TEXT NOT NULL,
    views BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    last_view TIMESTAMPTZ
);