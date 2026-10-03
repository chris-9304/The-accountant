CREATE TABLE IF NOT EXISTS transactions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID NOT NULL REFERENCES accounts(id),
    category_id         UUID REFERENCES categories(id),
    statement_id        UUID REFERENCES statements(id) ON DELETE SET NULL,
    amount              DECIMAL(15, 2) NOT NULL,
    transaction_type    VARCHAR(10) NOT NULL,
    description         TEXT,
    merchant_name       VARCHAR(200),
    reference_number    VARCHAR(100),
    transaction_date    DATE NOT NULL,
    source              VARCHAR(20) NOT NULL,
    balance_after       DECIMAL(15, 2),
    notes               TEXT,
    is_recurring        BOOLEAN NOT NULL DEFAULT false,
    recurring_id        UUID REFERENCES recurring_expenses(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_transactions_account_date ON transactions(account_id, transaction_date DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(transaction_date DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_statement ON transactions(statement_id);
