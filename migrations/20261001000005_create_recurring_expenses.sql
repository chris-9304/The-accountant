CREATE TABLE IF NOT EXISTS recurring_expenses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_name   VARCHAR(200) NOT NULL,
    category_id     UUID REFERENCES categories(id),
    average_amount  DECIMAL(15, 2) NOT NULL,
    min_amount      DECIMAL(15, 2),
    max_amount      DECIMAL(15, 2),
    frequency       VARCHAR(20) NOT NULL,
    last_seen       DATE,
    next_expected   DATE,
    account_id      UUID REFERENCES accounts(id),
    occurrence_count INTEGER NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    confidence      DECIMAL(3, 2) NOT NULL DEFAULT 0.50,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
