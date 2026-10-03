CREATE TABLE IF NOT EXISTS monthly_snapshots (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    month                   DATE NOT NULL UNIQUE,
    total_income            DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_expenses          DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_savings           DECIMAL(15, 2) NOT NULL DEFAULT 0,
    net_worth               DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_debt_owed         DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_debt_receivable   DECIMAL(15, 2) NOT NULL DEFAULT 0,
    account_balances        JSONB NOT NULL DEFAULT '{}',
    category_breakdown      JSONB NOT NULL DEFAULT '{}',
    top_expenses            JSONB NOT NULL DEFAULT '[]',
    health_score            INTEGER,
    budget_adherence_pct    DECIMAL(5, 2),
    savings_rate_pct        DECIMAL(5, 2),
    generated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS financial_health_scores (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    overall_score           INTEGER NOT NULL CHECK (overall_score >= 0 AND overall_score <= 100),
    savings_rate_score      INTEGER NOT NULL,
    debt_ratio_score        INTEGER NOT NULL,
    budget_adherence_score  INTEGER NOT NULL,
    emergency_fund_score    INTEGER NOT NULL,
    debt_collection_score   INTEGER NOT NULL,
    details                 JSONB NOT NULL DEFAULT '{}',
    calculated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS cash_flow_forecasts (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    forecast_date               DATE NOT NULL,
    projected_income            DECIMAL(15, 2) NOT NULL DEFAULT 0,
    projected_expenses          DECIMAL(15, 2) NOT NULL DEFAULT 0,
    projected_debt_payments     DECIMAL(15, 2) NOT NULL DEFAULT 0,
    projected_debt_collections  DECIMAL(15, 2) NOT NULL DEFAULT 0,
    projected_balance           DECIMAL(15, 2) NOT NULL DEFAULT 0,
    is_danger_zone              BOOLEAN NOT NULL DEFAULT false,
    danger_threshold            DECIMAL(15, 2),
    confidence                  DECIMAL(3, 2) NOT NULL DEFAULT 0.50,
    generated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_forecasts_date ON cash_flow_forecasts(forecast_date);
