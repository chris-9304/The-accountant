CREATE TABLE IF NOT EXISTS budgets (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    month               DATE NOT NULL UNIQUE,
    total_spending_limit DECIMAL(15, 2) NOT NULL,
    savings_target      DECIMAL(15, 2) NOT NULL DEFAULT 0,
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS budget_categories (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id        UUID NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
    category_id      UUID NOT NULL REFERENCES categories(id),
    allocated_amount DECIMAL(15, 2) NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(budget_id, category_id)
);
