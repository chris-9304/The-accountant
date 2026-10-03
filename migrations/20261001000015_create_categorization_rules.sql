CREATE TABLE IF NOT EXISTS categorization_rules (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern     VARCHAR(200) NOT NULL,
    category_id UUID NOT NULL REFERENCES categories(id),
    priority    INTEGER NOT NULL DEFAULT 0,
    match_type  VARCHAR(20) NOT NULL DEFAULT 'contains',
    is_active   BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_rules_active ON categorization_rules(is_active, priority DESC);
