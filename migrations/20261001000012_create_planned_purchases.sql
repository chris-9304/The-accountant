CREATE TABLE IF NOT EXISTS planned_purchases (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_name                 VARCHAR(200) NOT NULL,
    estimated_cost            DECIMAL(15, 2) NOT NULL,
    urgency                   VARCHAR(20) NOT NULL DEFAULT 'normal',
    target_date               DATE,
    recommended_purchase_date DATE,
    funding_strategy          VARCHAR(20),
    funding_details           JSONB,
    category_id               UUID REFERENCES categories(id),
    status                    VARCHAR(20) NOT NULL DEFAULT 'planned',
    notes                     TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
