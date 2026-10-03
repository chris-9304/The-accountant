CREATE TABLE IF NOT EXISTS debt_payments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    debt_id     UUID NOT NULL REFERENCES debts(id) ON DELETE CASCADE,
    amount      DECIMAL(15, 2) NOT NULL,
    payment_date DATE NOT NULL,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_debt_payments_debt ON debt_payments(debt_id);
