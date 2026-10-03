CREATE TABLE IF NOT EXISTS alerts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type      VARCHAR(50) NOT NULL,
    title           VARCHAR(200) NOT NULL,
    message         TEXT NOT NULL,
    severity        VARCHAR(10) NOT NULL DEFAULT 'info',
    is_read         BOOLEAN NOT NULL DEFAULT false,
    is_dismissed    BOOLEAN NOT NULL DEFAULT false,
    reference_type  VARCHAR(50),
    reference_id    UUID,
    triggered_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_alerts_unread ON alerts(is_read, is_dismissed) WHERE NOT is_read AND NOT is_dismissed;
CREATE INDEX IF NOT EXISTS idx_alerts_triggered ON alerts(triggered_at DESC);
