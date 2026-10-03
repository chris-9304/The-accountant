CREATE TABLE IF NOT EXISTS debt_limits (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    max_total_debt  DECIMAL(15, 2) NOT NULL,
    max_single_debt DECIMAL(15, 2),
    effective_from  DATE NOT NULL,
    effective_until DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
