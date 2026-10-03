CREATE TABLE IF NOT EXISTS tags (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(50) NOT NULL UNIQUE,
    color       VARCHAR(7),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
