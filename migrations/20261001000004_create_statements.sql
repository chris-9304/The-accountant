CREATE TABLE IF NOT EXISTS statements (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID NOT NULL REFERENCES accounts(id),
    file_name           VARCHAR(255) NOT NULL,
    file_path           TEXT NOT NULL,
    file_format         VARCHAR(10) NOT NULL,
    file_hash           VARCHAR(64) NOT NULL,
    bank_format         VARCHAR(50),
    period_start        DATE,
    period_end          DATE,
    total_transactions  INTEGER DEFAULT 0,
    total_credits       DECIMAL(15, 2) DEFAULT 0,
    total_debits        DECIMAL(15, 2) DEFAULT 0,
    status              VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message       TEXT,
    parsed_at           TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_statements_file_hash ON statements(file_hash);
CREATE INDEX IF NOT EXISTS idx_statements_account ON statements(account_id);
