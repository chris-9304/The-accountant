CREATE TABLE IF NOT EXISTS debts (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    counterparty      VARCHAR(200) NOT NULL,
    amount            DECIMAL(15, 2) NOT NULL,
    remaining_amount  DECIMAL(15, 2) NOT NULL,
    debt_type         VARCHAR(20) NOT NULL,
    reason            TEXT,
    deadline          DATE,
    interest_rate     DECIMAL(5, 2) NOT NULL DEFAULT 0,
    status            VARCHAR(20) NOT NULL DEFAULT 'active',
    priority          VARCHAR(10) NOT NULL DEFAULT 'medium',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_debts_status ON debts(status);
CREATE INDEX IF NOT EXISTS idx_debts_deadline ON debts(deadline);
CREATE INDEX IF NOT EXISTS idx_debts_type ON debts(debt_type);
