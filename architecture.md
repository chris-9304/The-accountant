# The Accountant — Architecture Document

> A personal finance management system that acts as your digital accountant.
> Rust backend · PostgreSQL · Docker-first · REST API

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Tech Stack](#2-tech-stack)
3. [Project Structure](#3-project-structure)
4. [Architecture Layers](#4-architecture-layers)
5. [Database Schema](#5-database-schema)
6. [Core Modules](#6-core-modules)
7. [Bank Statement Pipeline](#7-bank-statement-pipeline)
8. [API Design](#8-api-design)
9. [Docker & Deployment](#9-docker--deployment)
10. [Future: Native Apps](#10-future-native-apps)

---

## 1. System Overview

The Accountant is a personal finance management platform designed to give complete visibility and control over income, expenses, debt, budgets, and financial planning. It automatically ingests bank statements, categorizes transactions, tracks debt obligations, forecasts cash flow, and provides actionable financial insights.

### Core Capabilities

| Module | Description |
|---|---|
| **Dashboard** | At-a-glance view of all accounts, total balance, total debt, financial health score, alerts, and trends |
| **Expense Management** | Automatic PDF/CSV/Excel statement parsing, multi-account support (bank, UPI Lite, cash), auto-categorization |
| **Debt Management** | Track money owed and receivable, deadlines, partial payments, debt limits, net position after settlements |
| **Budget & Savings** | Monthly spending limits, savings targets, per-category budgets, budget adherence tracking |
| **Purchase Planning** | Planned purchase queue with smart funding advice (debt vs. savings vs. income) |
| **Income Optimization** | Debt leverage analysis, profit maximization strategies based on current financial state |
| **Recurring Detection** | Auto-detect subscriptions, EMIs, and recurring payments from transaction history |
| **Cash Flow Forecasting** | 30/60/90-day balance projections with danger zone warnings |
| **Financial Health Score** | Composite 0–100 score based on savings rate, debt ratios, budget adherence, emergency fund |
| **Alerts & Reminders** | Debt deadlines, budget overruns, low balance, unusual spending, subscription price changes |
| **Analytics & Trends** | Month-over-month comparisons, category breakdowns, spending insights |
| **Monthly Snapshots** | Auto-generated monthly financial reports with opening/closing balances |
| **Tags & Notes** | Flexible transaction tagging beyond categories (e.g., `#trip-goa`, `#birthday`) |
| **Tax-Ready Export** | CSV/PDF export grouped by category for tax filing and personal records |

### Account Types

The system supports multiple account types that transactions flow through:

- **Bank Account** — Primary account. Transactions imported automatically from bank statements (PDF/CSV/Excel).
- **UPI Lite** — Lightweight UPI balance. Can be tracked manually or via statements if available.
- **Cash** — Physical cash. All transactions input and managed manually.

---

## 2. Tech Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Language** | Rust | Performance, safety, strong type system for financial calculations |
| **Web Framework** | Axum 0.7+ | Modern async framework, tower ecosystem, excellent ergonomics |
| **Database** | PostgreSQL 16+ | ACID compliance, JSONB for flexible data, excellent for financial data |
| **ORM / Query** | SQLx 0.7+ | Compile-time checked queries, async, native PostgreSQL support |
| **PDF Parsing** | `pdf-extract` + `lopdf` | Rust-native PDF text extraction. Fallback: `pdftotext` (poppler-utils) in Docker |
| **CSV Parsing** | `csv` crate | Standard CSV parsing for bank statement CSV imports |
| **Excel Parsing** | `calamine` crate | Read `.xlsx` / `.xls` bank statement exports |
| **Serialization** | `serde` + `serde_json` | Industry standard for Rust serialization |
| **Date/Time** | `chrono` | Timezone-aware date/time handling |
| **Decimal Math** | `rust_decimal` | Exact decimal arithmetic — no floating point errors for money |
| **Scheduling** | `tokio-cron-scheduler` | Background jobs for recurring detection, alerts, snapshots |
| **Logging** | `tracing` + `tracing-subscriber` | Structured async-aware logging |
| **Containerization** | Docker + Docker Compose | OrbStack / Podman compatible |
| **Migrations** | `sqlx-cli` | SQL migration management |

### Key Rust Crates (Cargo.toml)

```toml
[dependencies]
# Web framework
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "limit"] }

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "uuid",
    "chrono",
    "rust_decimal",
    "json"
] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Date/Time & IDs
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

# Money — NEVER use f64 for currency
rust_decimal = { version = "1", features = ["db-postgres", "serde-with-str"] }

# PDF / CSV / Excel parsing
pdf-extract = "0.7"
lopdf = "0.34"
csv = "1"
calamine = "0.26"

# Configuration
dotenvy = "0.15"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
thiserror = "2"
anyhow = "1"

# Pattern matching for categorization
regex = "1"

# Background scheduling
tokio-cron-scheduler = "0.13"

# PDF report generation
printpdf = "0.7"
```

> **CRITICAL**: All monetary values MUST use `rust_decimal::Decimal`, never `f32`/`f64`. This is a financial application — floating point errors are unacceptable.

---

## 3. Project Structure

```
the-accountant/
├── Cargo.toml
├── Cargo.lock
├── docker-compose.yml
├── Dockerfile
├── .env.example
├── architecture.md            ← This file
├── claude.md                  ← Instructions for Claude
├── migrations/
│   ├── 001_create_accounts.sql
│   ├── 002_create_categories.sql
│   ├── 003_create_statements.sql
│   ├── 004_create_transactions.sql
│   ├── 005_create_tags.sql
│   ├── 006_create_debts.sql
│   ├── 007_create_budgets.sql
│   ├── 008_create_planned_purchases.sql
│   ├── 009_create_recurring_expenses.sql
│   ├── 010_create_alerts.sql
│   ├── 011_create_monthly_snapshots.sql
│   ├── 012_create_health_scores.sql
│   ├── 013_create_cash_flow_forecasts.sql
│   ├── 014_create_categorization_rules.sql
│   └── 015_seed_default_categories.sql
├── src/
│   ├── main.rs                 ← Entry point: init DB, start Axum server, start scheduler
│   ├── lib.rs                  ← Re-exports for testing
│   ├── config.rs               ← Environment config (DB URL, port, upload dir, etc.)
│   ├── errors.rs               ← Unified error types with Axum IntoResponse
│   ├── app_state.rs            ← Shared application state (DB pool, config)
│   │
│   ├── models/                 ← Domain models (structs matching DB + API DTOs)
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── transaction.rs
│   │   ├── category.rs
│   │   ├── tag.rs
│   │   ├── statement.rs
│   │   ├── debt.rs
│   │   ├── budget.rs
│   │   ├── planned_purchase.rs
│   │   ├── recurring_expense.rs
│   │   ├── alert.rs
│   │   ├── snapshot.rs
│   │   ├── health_score.rs
│   │   ├── cash_flow.rs
│   │   ├── categorization_rule.rs
│   │   └── dashboard.rs        ← Dashboard aggregate DTOs
│   │
│   ├── repositories/           ← Database access layer (SQL queries via SQLx)
│   │   ├── mod.rs
│   │   ├── account_repo.rs
│   │   ├── transaction_repo.rs
│   │   ├── category_repo.rs
│   │   ├── tag_repo.rs
│   │   ├── statement_repo.rs
│   │   ├── debt_repo.rs
│   │   ├── budget_repo.rs
│   │   ├── purchase_repo.rs
│   │   ├── recurring_repo.rs
│   │   ├── alert_repo.rs
│   │   ├── snapshot_repo.rs
│   │   ├── health_score_repo.rs
│   │   ├── forecast_repo.rs
│   │   └── rule_repo.rs
│   │
│   ├── services/               ← Business logic layer
│   │   ├── mod.rs
│   │   ├── dashboard_service.rs
│   │   ├── account_service.rs
│   │   ├── transaction_service.rs
│   │   ├── debt_service.rs
│   │   ├── budget_service.rs
│   │   ├── purchase_advisor.rs
│   │   ├── categorizer.rs       ← Rule-based auto-categorization engine
│   │   ├── recurring_detector.rs ← Subscription/recurring payment detection
│   │   ├── cash_flow_forecast.rs
│   │   ├── health_score.rs
│   │   ├── alert_service.rs
│   │   ├── snapshot_service.rs
│   │   └── export_service.rs    ← CSV/PDF report generation
│   │
│   ├── parsers/                ← Bank statement parsing infrastructure
│   │   ├── mod.rs              ← Parser trait + factory
│   │   ├── pdf_extractor.rs    ← Raw PDF → text extraction
│   │   ├── csv_extractor.rs    ← CSV file reading
│   │   ├── excel_extractor.rs  ← Excel file reading
│   │   ├── statement_parser.rs ← Orchestrator: file → parsed transactions
│   │   └── bank_formats/      ← Bank-specific format parsers
│   │       ├── mod.rs          ← BankFormat trait + auto-detection
│   │       ├── sbi.rs
│   │       ├── hdfc.rs
│   │       ├── icici.rs
│   │       ├── axis.rs
│   │       ├── kotak.rs
│   │       └── generic.rs     ← Fallback generic parser
│   │
│   ├── api/                   ← HTTP layer (Axum handlers + routing)
│   │   ├── mod.rs
│   │   ├── router.rs          ← All route definitions
│   │   ├── middleware.rs       ← Request logging, error handling
│   │   ├── responses.rs       ← Standardized API response wrapper
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── dashboard_handler.rs
│   │       ├── account_handler.rs
│   │       ├── transaction_handler.rs
│   │       ├── statement_handler.rs
│   │       ├── debt_handler.rs
│   │       ├── budget_handler.rs
│   │       ├── purchase_handler.rs
│   │       ├── category_handler.rs
│   │       ├── tag_handler.rs
│   │       ├── recurring_handler.rs
│   │       ├── alert_handler.rs
│   │       ├── report_handler.rs
│   │       └── rule_handler.rs
│   │
│   └── scheduler/             ← Background job scheduler
│       ├── mod.rs
│       ├── recurring_detector_job.rs  ← Nightly: scan for new recurring patterns
│       ├── alert_checker_job.rs       ← Hourly: check for deadline/budget alerts
│       ├── snapshot_generator_job.rs  ← Monthly: generate financial snapshot
│       └── forecast_job.rs            ← Weekly: regenerate cash flow forecasts
│
└── tests/
    ├── common/
    │   └── mod.rs              ← Test helpers, DB setup/teardown
    ├── api/                    ← Integration tests for API endpoints
    ├── services/               ← Unit tests for business logic
    ├── parsers/                ← Unit tests for statement parsing
    └── fixtures/               ← Sample bank statements (PDF, CSV) for testing
```

---

## 4. Architecture Layers

The application follows a strict **layered architecture** with unidirectional dependencies:

```
┌─────────────────────────────────────────────────────┐
│                    API Layer                         │
│     (Axum handlers, routing, request/response)      │
├─────────────────────────────────────────────────────┤
│                  Service Layer                       │
│          (Business logic, orchestration)             │
├──────────────────────┬──────────────────────────────┤
│   Repository Layer   │      Parser Layer             │
│   (SQLx queries)     │  (PDF/CSV/Excel → structs)    │
├──────────────────────┴──────────────────────────────┤
│               Infrastructure                         │
│     (PostgreSQL, File System, Scheduler)             │
└─────────────────────────────────────────────────────┘
```

### Layer Rules

1. **API Layer** → Calls **Service Layer** only. Never touches the database directly.
2. **Service Layer** → Calls **Repository Layer** and **Parser Layer**. Contains all business rules and validation.
3. **Repository Layer** → Executes SQL queries. Returns domain models. No business logic.
4. **Parser Layer** → Converts raw file data into structured transaction data. No database access.

### Dependency Injection

All layers receive their dependencies through Axum's `State` extractor:

```rust
// app_state.rs
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: AppConfig,
    pub upload_dir: String,
}
```

Handlers receive `State<AppState>` and construct services/repositories as needed. There is no DI framework — Rust's type system and module structure handle this cleanly.

---

## 5. Database Schema

All tables use `UUID` primary keys, `TIMESTAMPTZ` timestamps, and `DECIMAL(15,2)` for monetary values.

### Entity Relationship Diagram

```
accounts ──────────< transactions >────── categories
    │                     │                    │
    │                     │                    │
    └──< statements       ├──< transaction_tags >── tags
                          │
                          │
debts ──────────< debt_payments
    │
debt_limits

budgets ──────< budget_categories >── categories

planned_purchases ──── categories

recurring_expenses ──── categories ──── accounts

categorization_rules ──── categories

alerts (standalone, references any entity via reference_type + reference_id)

monthly_snapshots (standalone, monthly aggregates)

financial_health_scores (standalone, periodic scores)

cash_flow_forecasts (standalone, projected daily balances)
```

### Table Definitions

#### `accounts`
```sql
CREATE TABLE accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(100) NOT NULL,
    account_type    VARCHAR(50) NOT NULL,  -- 'bank', 'upi_lite', 'cash'
    balance         DECIMAL(15, 2) NOT NULL DEFAULT 0,
    currency        VARCHAR(3) NOT NULL DEFAULT 'INR',
    bank_name       VARCHAR(100),          -- NULL for cash/upi_lite
    account_number  VARCHAR(50),           -- masked, last 4 digits
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `categories`
```sql
CREATE TABLE categories (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) NOT NULL,
    parent_id   UUID REFERENCES categories(id) ON DELETE SET NULL,
    icon        VARCHAR(50),
    color       VARCHAR(7),               -- hex color e.g. '#FF5733'
    is_income   BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default seed categories:
-- Income: Salary, Freelance, Interest, Refunds, Gifts Received
-- Expense: Food & Dining, Groceries, Transport, Rent, Utilities, Shopping,
--          Entertainment, Health, Education, Subscriptions, Personal Care,
--          Travel, Gifts & Donations, EMI, Insurance, Miscellaneous
```

#### `statements`
```sql
CREATE TABLE statements (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID NOT NULL REFERENCES accounts(id),
    file_name           VARCHAR(255) NOT NULL,
    file_path           TEXT NOT NULL,
    file_format         VARCHAR(10) NOT NULL,      -- 'pdf', 'csv', 'xlsx'
    file_hash           VARCHAR(64) NOT NULL,      -- SHA-256 to prevent duplicate imports
    bank_format         VARCHAR(50),               -- detected bank format e.g. 'hdfc', 'sbi'
    period_start        DATE,
    period_end          DATE,
    total_transactions  INTEGER DEFAULT 0,
    total_credits       DECIMAL(15, 2) DEFAULT 0,
    total_debits        DECIMAL(15, 2) DEFAULT 0,
    status              VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- status: 'pending' → 'processing' → 'completed' | 'failed' | 'duplicate'
    error_message       TEXT,
    parsed_at           TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_statements_file_hash ON statements(file_hash);
```

#### `transactions`
```sql
CREATE TABLE transactions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID NOT NULL REFERENCES accounts(id),
    category_id         UUID REFERENCES categories(id),
    statement_id        UUID REFERENCES statements(id) ON DELETE SET NULL,
    amount              DECIMAL(15, 2) NOT NULL,
    transaction_type    VARCHAR(10) NOT NULL,       -- 'credit', 'debit'
    description         TEXT,
    merchant_name       VARCHAR(200),
    reference_number    VARCHAR(100),
    transaction_date    DATE NOT NULL,
    source              VARCHAR(20) NOT NULL,        -- 'manual', 'statement'
    balance_after       DECIMAL(15, 2),
    notes               TEXT,
    is_recurring        BOOLEAN NOT NULL DEFAULT false,
    recurring_id        UUID REFERENCES recurring_expenses(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transactions_account_date ON transactions(account_id, transaction_date DESC);
CREATE INDEX idx_transactions_category ON transactions(category_id);
CREATE INDEX idx_transactions_date ON transactions(transaction_date DESC);
CREATE INDEX idx_transactions_statement ON transactions(statement_id);
```

#### `tags` and `transaction_tags`
```sql
CREATE TABLE tags (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(50) NOT NULL UNIQUE,
    color       VARCHAR(7),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE transaction_tags (
    transaction_id  UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    tag_id          UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (transaction_id, tag_id)
);
```

#### `debts` and `debt_payments`
```sql
CREATE TABLE debts (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    counterparty      VARCHAR(200) NOT NULL,       -- person/entity name
    amount            DECIMAL(15, 2) NOT NULL,     -- original amount
    remaining_amount  DECIMAL(15, 2) NOT NULL,     -- current outstanding
    debt_type         VARCHAR(20) NOT NULL,         -- 'i_owe', 'owed_to_me'
    reason            TEXT,
    deadline          DATE,
    interest_rate     DECIMAL(5, 2) NOT NULL DEFAULT 0,
    status            VARCHAR(20) NOT NULL DEFAULT 'active',
    -- status: 'active', 'partially_paid', 'settled', 'overdue', 'written_off'
    priority          VARCHAR(10) NOT NULL DEFAULT 'medium',
    -- priority: 'low', 'medium', 'high', 'critical'
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE debt_payments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    debt_id     UUID NOT NULL REFERENCES debts(id) ON DELETE CASCADE,
    amount      DECIMAL(15, 2) NOT NULL,
    payment_date DATE NOT NULL,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_debts_status ON debts(status);
CREATE INDEX idx_debts_deadline ON debts(deadline);
```

#### `debt_limits`
```sql
CREATE TABLE debt_limits (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    max_total_debt  DECIMAL(15, 2) NOT NULL,  -- max total debt I can take
    max_single_debt DECIMAL(15, 2),           -- max single debt amount
    effective_from  DATE NOT NULL,
    effective_until DATE,                      -- NULL = currently active
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `budgets` and `budget_categories`
```sql
CREATE TABLE budgets (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    month               DATE NOT NULL UNIQUE,      -- first day of month, e.g. '2026-10-01'
    total_spending_limit DECIMAL(15, 2) NOT NULL,
    savings_target      DECIMAL(15, 2) NOT NULL DEFAULT 0,
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE budget_categories (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id        UUID NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
    category_id      UUID NOT NULL REFERENCES categories(id),
    allocated_amount DECIMAL(15, 2) NOT NULL,
    UNIQUE(budget_id, category_id)
);
```

#### `planned_purchases`
```sql
CREATE TABLE planned_purchases (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_name                 VARCHAR(200) NOT NULL,
    estimated_cost            DECIMAL(15, 2) NOT NULL,
    urgency                   VARCHAR(20) NOT NULL DEFAULT 'normal',
    -- urgency: 'low', 'normal', 'high', 'critical'
    target_date               DATE,
    recommended_purchase_date DATE,           -- computed by purchase advisor
    funding_strategy          VARCHAR(20),    -- 'savings', 'debt', 'income', 'mixed'
    funding_details           JSONB,          -- breakdown of how to fund
    category_id               UUID REFERENCES categories(id),
    status                    VARCHAR(20) NOT NULL DEFAULT 'planned',
    -- status: 'planned', 'saving', 'ready', 'purchased', 'cancelled'
    notes                     TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `recurring_expenses`
```sql
CREATE TABLE recurring_expenses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_name   VARCHAR(200) NOT NULL,
    category_id     UUID REFERENCES categories(id),
    average_amount  DECIMAL(15, 2) NOT NULL,
    min_amount      DECIMAL(15, 2),
    max_amount      DECIMAL(15, 2),
    frequency       VARCHAR(20) NOT NULL,
    -- frequency: 'weekly', 'biweekly', 'monthly', 'quarterly', 'yearly'
    last_seen       DATE,
    next_expected   DATE,
    account_id      UUID REFERENCES accounts(id),
    occurrence_count INTEGER NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    confidence      DECIMAL(3, 2) NOT NULL DEFAULT 0.50,  -- 0.00 to 1.00
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `alerts`
```sql
CREATE TABLE alerts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type      VARCHAR(50) NOT NULL,
    -- types: 'debt_deadline', 'debt_overdue', 'budget_overrun', 'budget_warning',
    --        'low_balance', 'unusual_spending', 'subscription_change',
    --        'recurring_new', 'collection_due', 'debt_limit_warning'
    title           VARCHAR(200) NOT NULL,
    message         TEXT NOT NULL,
    severity        VARCHAR(10) NOT NULL DEFAULT 'info',
    -- severity: 'info', 'warning', 'critical'
    is_read         BOOLEAN NOT NULL DEFAULT false,
    is_dismissed    BOOLEAN NOT NULL DEFAULT false,
    reference_type  VARCHAR(50),   -- 'debt', 'budget', 'account', 'transaction', 'recurring_expense'
    reference_id    UUID,
    triggered_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alerts_unread ON alerts(is_read, is_dismissed) WHERE NOT is_read AND NOT is_dismissed;
CREATE INDEX idx_alerts_triggered ON alerts(triggered_at DESC);
```

#### `monthly_snapshots`
```sql
CREATE TABLE monthly_snapshots (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    month                   DATE NOT NULL UNIQUE,
    total_income            DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_expenses          DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_savings           DECIMAL(15, 2) NOT NULL DEFAULT 0,
    net_worth               DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_debt_owed         DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_debt_receivable   DECIMAL(15, 2) NOT NULL DEFAULT 0,
    account_balances        JSONB NOT NULL DEFAULT '{}',
    -- e.g. {"bank": 50000, "upi_lite": 2000, "cash": 5000}
    category_breakdown      JSONB NOT NULL DEFAULT '{}',
    -- e.g. {"Food": 8000, "Transport": 3000, ...}
    top_expenses            JSONB NOT NULL DEFAULT '[]',
    health_score            INTEGER,
    budget_adherence_pct    DECIMAL(5, 2),
    savings_rate_pct        DECIMAL(5, 2),
    generated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `financial_health_scores`
```sql
CREATE TABLE financial_health_scores (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    overall_score           INTEGER NOT NULL CHECK (overall_score >= 0 AND overall_score <= 100),
    savings_rate_score      INTEGER NOT NULL,  -- 0-100
    debt_ratio_score        INTEGER NOT NULL,  -- 0-100
    budget_adherence_score  INTEGER NOT NULL,  -- 0-100
    emergency_fund_score    INTEGER NOT NULL,  -- 0-100
    debt_collection_score   INTEGER NOT NULL,  -- 0-100
    details                 JSONB NOT NULL DEFAULT '{}',
    calculated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `cash_flow_forecasts`
```sql
CREATE TABLE cash_flow_forecasts (
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

CREATE INDEX idx_forecasts_date ON cash_flow_forecasts(forecast_date);
```

#### `categorization_rules`
```sql
CREATE TABLE categorization_rules (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern     VARCHAR(200) NOT NULL,      -- keyword or regex
    category_id UUID NOT NULL REFERENCES categories(id),
    priority    INTEGER NOT NULL DEFAULT 0, -- higher = checked first
    match_type  VARCHAR(20) NOT NULL DEFAULT 'contains',
    -- match_type: 'exact', 'contains', 'regex', 'starts_with'
    is_active   BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed examples:
-- ('swiggy', food_category_id, 10, 'contains')
-- ('zomato', food_category_id, 10, 'contains')
-- ('uber', transport_category_id, 10, 'contains')
-- ('ola', transport_category_id, 10, 'contains')
-- ('netflix', subscriptions_category_id, 10, 'contains')
-- ('spotify', subscriptions_category_id, 10, 'contains')
-- ('amazon', shopping_category_id, 5, 'contains')
```

---

## 6. Core Modules

### 6.1 Dashboard Service

Aggregates data from all other services into a single dashboard response:

```rust
// Returned by GET /api/dashboard
pub struct DashboardResponse {
    // Account Overview
    pub total_balance: Decimal,          // sum of all account balances
    pub accounts: Vec<AccountSummary>,   // each account with balance

    // Debt Overview
    pub total_debt_owed: Decimal,        // total I owe others
    pub total_debt_receivable: Decimal,  // total others owe me
    pub net_debt_position: Decimal,      // receivable - owed
    pub upcoming_debt_deadlines: Vec<DebtDeadline>,

    // Budget Overview (current month)
    pub budget_spent: Decimal,
    pub budget_limit: Decimal,
    pub budget_remaining: Decimal,
    pub savings_target: Decimal,
    pub savings_actual: Decimal,

    // Financial Health
    pub health_score: u32,
    pub health_trend: Trend,             // up, down, stable

    // Balance After Settlements
    pub projected_balance_after_debts: Decimal,

    // Recent Activity
    pub recent_transactions: Vec<TransactionSummary>,
    pub unread_alerts: Vec<Alert>,

    // Trends
    pub spending_this_month: Decimal,
    pub spending_last_month: Decimal,
    pub spending_change_pct: Decimal,
    pub top_spending_categories: Vec<CategorySpending>,

    // Cash Flow
    pub cash_flow_30d: Vec<CashFlowPoint>,

    // Debt Capacity
    pub debt_limit: Decimal,
    pub current_total_debt: Decimal,
    pub available_debt_capacity: Decimal,
}
```

### 6.2 Categorizer Service

Rule-based auto-categorization engine:

```
Input: transaction description / merchant name
  ↓
1. Check categorization_rules table (ordered by priority DESC)
2. For each rule, match against description using match_type
3. If match found → assign category, stop
4. If no match → assign "Uncategorized" (user can re-categorize manually)
5. Learn: when user manually categorizes, offer to create a new rule
```

The categorizer runs automatically on every new transaction (from statement import or manual entry).

### 6.3 Recurring Expense Detector

Background job that analyzes transaction history to find patterns:

```
Algorithm:
1. Group transactions by normalized merchant_name
2. For each merchant with 2+ transactions:
   a. Calculate time intervals between transactions
   b. Check if intervals cluster around known frequencies:
      - Weekly:    ~7 days   (±2)
      - Biweekly:  ~14 days  (±3)
      - Monthly:   ~30 days  (±5)
      - Quarterly: ~90 days  (±10)
      - Yearly:    ~365 days (±15)
   c. Calculate confidence score based on consistency
   d. If confidence > 0.6 → flag as recurring
3. For existing recurring expenses:
   a. Check for amount changes (subscription price change alert)
   b. Check for missing expected payments (cancellation?)
   c. Update next_expected date
```

### 6.4 Cash Flow Forecaster

Projects future balances based on current data:

```
Inputs:
- Current total balance across all accounts
- Known upcoming income (recurring credits)
- Known upcoming expenses (recurring debits)
- Scheduled debt payments (deadlines for debts I owe)
- Expected debt collections (deadlines for debts owed to me)
- Average daily spending from last 3 months

Output:
- Daily projected balance for next 30/60/90 days
- Danger zone markers where balance drops below safety threshold
- Confidence intervals (high confidence for known items, lower for projected)
```

### 6.5 Financial Health Score Calculator

Composite score (0–100) calculated from five sub-scores:

| Sub-Score | Weight | How It's Calculated |
|---|---|---|
| Savings Rate | 25% | `(actual_savings / income) × 100`. Target: 20%+ = 100 |
| Debt-to-Income Ratio | 25% | `1 - (total_debt_owed / monthly_income)`. Lower = better |
| Budget Adherence | 20% | `1 - (overspend / budget_limit)`. Under budget = 100 |
| Emergency Fund | 15% | `(total_balance / (3 × monthly_expenses))`. 3+ months = 100 |
| Debt Collection Rate | 15% | `(collected_this_month / total_receivable)`. Higher = better |

### 6.6 Smart Purchase Advisor

For each planned purchase, analyzes the current financial state and recommends:

```
Input: planned_purchase (item, cost, urgency, target_date)

Analysis:
1. Can you afford it from savings without dipping below emergency fund? → "Buy from savings"
2. Can you save for it by target_date from monthly surplus? → "Save ₹X/month for N months"
3. Is debt capacity available and urgency is high? → "Take debt of ₹X, repay in N months"
4. Is there a collection due that would cover it? → "Wait for ₹X collection from [person] due [date]"
5. Mixed strategy: "Save ₹X + take ₹Y debt"

Output: funding_strategy + funding_details JSON + recommended_purchase_date
```

### 6.7 Debt Optimizer

Helps manage debt strategically:

```
Features:
1. Net Position Calculator:
   - balance_after_settlements = total_balance - total_owed + total_receivable
   - Shows what you'll actually have after all debts settle

2. Debt Limit Enforcement:
   - Before taking new debt: current_debt + new_debt <= debt_limit
   - Warning at 80% of limit, block at 100%

3. Repayment Priority:
   - Sort debts by: deadline (nearest first) → interest_rate (highest first) → amount (smallest first)
   - Suggest optimal repayment order

4. Leverage Analysis:
   - If receivable > owed: "You're in a lending position"
   - If debt capacity available and a need exists: "Safe to take ₹X more debt"
   - Show cost of debt (interest) vs. benefit of early purchase
```

### 6.8 Alert Service

Checks conditions and generates alerts:

| Alert Type | Check Frequency | Condition |
|---|---|---|
| `debt_deadline` | Daily | Debt deadline within 7 days |
| `debt_overdue` | Daily | Debt past deadline |
| `collection_due` | Daily | Receivable debt past deadline |
| `budget_warning` | On each transaction | Spent > 80% of monthly budget |
| `budget_overrun` | On each transaction | Spent > 100% of monthly budget |
| `low_balance` | On each transaction | Any account balance < configurable threshold |
| `unusual_spending` | Daily | Category spending > 150% of 3-month average |
| `subscription_change` | On recurring detection | Recurring expense amount changed > 10% |
| `debt_limit_warning` | On debt creation | Total debt > 80% of limit |
| `recurring_new` | On recurring detection | New subscription detected |

### 6.9 Monthly Snapshot Generator

Runs on the 1st of each month (scheduled background job):

```
1. Calculate total income and expenses for the previous month
2. Calculate savings (income - expenses)
3. Capture all account balances
4. Break down expenses by category
5. Calculate net worth (total balance - total debt owed + total receivable)
6. Calculate health score
7. Calculate budget adherence percentage
8. Calculate savings rate
9. Identify top 10 expenses
10. Store as a monthly_snapshot record
```

### 6.10 Export Service

Generates downloadable reports:

- **CSV Export**: Transactions filtered by date range, account, category. Columns: Date, Description, Category, Amount, Account, Tags.
- **PDF Report**: Monthly financial summary formatted as a professional accountant's report. Includes: summary page, income/expense tables, category charts (as tables), debt position, budget adherence.

---

## 7. Bank Statement Pipeline

This is the core automated flow: **PDF/CSV/Excel → Database**.

### Pipeline Architecture

```
 ┌──────────────┐
 │  User Upload  │  POST /api/statements/upload
 │  (multipart)  │  { file, account_id }
 └──────┬───────┘
        ↓
 ┌──────────────┐
 │  File Store   │  Save to disk, compute SHA-256 hash
 │  + Dedup      │  Check hash against existing statements → reject if duplicate
 └──────┬───────┘
        ↓
 ┌──────────────┐
 │  Format       │  Detect file type: PDF, CSV, or Excel
 │  Detection    │  Route to appropriate extractor
 └──────┬───────┘
        ↓
 ┌──────────────────────────────────────────────┐
 │  Text Extraction                              │
 │                                                │
 │  PDF → pdf-extract / pdftotext (poppler)      │
 │  CSV → csv crate                               │
 │  Excel → calamine crate                        │
 └──────┬───────────────────────────────────────┘
        ↓
 ┌──────────────┐
 │  Bank Format  │  Auto-detect bank from header text
 │  Detection    │  Match against known bank signatures
 └──────┬───────┘
        ↓
 ┌──────────────────────────────────────────────┐
 │  Bank-Specific Parser                         │
 │                                                │
 │  Parse rows into Vec<RawTransaction>:          │
 │  { date, description, ref_no,                  │
 │    debit_amount, credit_amount, balance }       │
 │                                                │
 │  Each bank parser implements BankFormatParser   │
 │  trait. Falls back to GenericParser.             │
 └──────┬───────────────────────────────────────┘
        ↓
 ┌──────────────┐
 │  Auto-        │  Apply categorization_rules to each transaction
 │  Categorize   │  Match description against patterns
 └──────┬───────┘
        ↓
 ┌──────────────┐
 │  Dedup        │  Check for existing transactions with same
 │  Transactions │  (account_id, date, amount, ref_no) → skip duplicates
 └──────┬───────┘
        ↓
 ┌──────────────┐
 │  Persist      │  INSERT transactions into database
 │  to DB        │  Update account balance
 │               │  Update statement status → 'completed'
 └──────┬───────┘
        ↓
 ┌──────────────┐
 │  Post-Process │  • Run recurring expense detection on new data
 │               │  • Check budget alerts
 │               │  • Update cash flow forecast
 └──────────────┘
```

### Bank Format Parser Trait

```rust
pub struct RawTransaction {
    pub date: NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    pub debit_amount: Option<Decimal>,
    pub credit_amount: Option<Decimal>,
    pub balance: Option<Decimal>,
}

pub struct ParsedStatement {
    pub bank_name: String,
    pub account_number: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub transactions: Vec<RawTransaction>,
}

pub trait BankFormatParser: Send + Sync {
    /// Returns true if this parser can handle the given text
    fn can_parse(&self, raw_text: &str) -> bool;

    /// Parse extracted text into structured transactions
    fn parse(&self, raw_text: &str) -> Result<ParsedStatement, ParserError>;

    /// Bank name this parser handles
    fn bank_name(&self) -> &str;
}
```

### Supported Bank Formats (Initial)

| Bank | Detection Signature | Notes |
|---|---|---|
| HDFC Bank | "HDFC BANK" in header | Common format: Date, Narration, Chq/Ref No, Value Dt, Withdrawal, Deposit, Closing Balance |
| SBI | "STATE BANK OF INDIA" in header | Date, Description, Ref/Chq, Debit, Credit, Balance |
| ICICI | "ICICI Bank" in header | Similar to HDFC with slight variations |
| Axis Bank | "AXIS BANK" in header | Date, Particulars, Chq No, Debit, Credit, Balance |
| Kotak | "KOTAK MAHINDRA" in header | Date, Description, Debit, Credit, Balance |
| Generic | Fallback | Attempts to parse common CSV/tabular patterns |

New bank formats are added by implementing the `BankFormatParser` trait.

### Password-Protected PDFs

Some Indian bank statements are password-protected (usually DOB or account number). The upload API accepts an optional `password` field:

```
POST /api/statements/upload
Content-Type: multipart/form-data

Fields:
  - file: [binary]
  - account_id: UUID
  - password: (optional) PDF password
  - bank_hint: (optional) bank name hint for parser selection
```

---

## 8. API Design

All endpoints return a standardized response wrapper:

```json
{
    "success": true,
    "data": { ... },
    "error": null,
    "meta": {
        "page": 1,
        "per_page": 50,
        "total": 150
    }
}
```

### Endpoint Catalog

#### Dashboard
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/dashboard` | Full dashboard data |

#### Accounts
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/accounts` | List all accounts |
| `POST` | `/api/accounts` | Create account |
| `GET` | `/api/accounts/:id` | Get account details |
| `PUT` | `/api/accounts/:id` | Update account |
| `DELETE` | `/api/accounts/:id` | Soft-delete account |
| `GET` | `/api/accounts/:id/transactions` | Account transaction history |
| `GET` | `/api/accounts/:id/balance-history` | Balance over time |

#### Transactions
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/transactions` | List with filters (account, category, date range, amount range, source, tags) |
| `POST` | `/api/transactions` | Create manual transaction |
| `GET` | `/api/transactions/:id` | Get transaction |
| `PUT` | `/api/transactions/:id` | Update transaction (re-categorize, add notes/tags) |
| `DELETE` | `/api/transactions/:id` | Delete transaction |
| `POST` | `/api/transactions/:id/tags` | Add tags to transaction |
| `DELETE` | `/api/transactions/:id/tags/:tag_id` | Remove tag from transaction |

#### Statements (Bank Statement Import)
| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/api/statements/upload` | Upload and auto-parse statement |
| `GET` | `/api/statements` | List uploaded statements |
| `GET` | `/api/statements/:id` | Get statement details + parsed transactions |
| `POST` | `/api/statements/:id/reparse` | Re-parse a statement |
| `DELETE` | `/api/statements/:id` | Delete statement and its transactions |

#### Debts
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/debts` | List all debts (filterable by type, status) |
| `POST` | `/api/debts` | Create debt record |
| `GET` | `/api/debts/:id` | Get debt with payment history |
| `PUT` | `/api/debts/:id` | Update debt |
| `DELETE` | `/api/debts/:id` | Delete debt |
| `POST` | `/api/debts/:id/payments` | Record a payment |
| `GET` | `/api/debts/summary` | Debt summary: total owed, receivable, net, projected balance |
| `GET` | `/api/debts/limits` | Get current debt limit and usage |
| `POST` | `/api/debts/limits` | Set/update debt limit |

#### Budgets
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/budgets` | List all budgets |
| `POST` | `/api/budgets` | Create monthly budget |
| `GET` | `/api/budgets/current` | Current month's budget with real-time progress |
| `GET` | `/api/budgets/:id` | Get budget with category breakdown |
| `PUT` | `/api/budgets/:id` | Update budget |
| `DELETE` | `/api/budgets/:id` | Delete budget |

#### Planned Purchases
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/purchases` | List planned purchases |
| `POST` | `/api/purchases` | Create planned purchase |
| `GET` | `/api/purchases/:id` | Get purchase with funding advice |
| `PUT` | `/api/purchases/:id` | Update purchase |
| `DELETE` | `/api/purchases/:id` | Delete purchase |
| `GET` | `/api/purchases/:id/advice` | Get smart funding recommendation |

#### Categories
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/categories` | List all (tree structure) |
| `POST` | `/api/categories` | Create category |
| `PUT` | `/api/categories/:id` | Update category |
| `DELETE` | `/api/categories/:id` | Delete (reassign transactions to parent/uncategorized) |

#### Tags
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/tags` | List all tags with usage count |
| `POST` | `/api/tags` | Create tag |
| `PUT` | `/api/tags/:id` | Update tag |
| `DELETE` | `/api/tags/:id` | Delete tag |

#### Recurring Expenses
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/recurring` | List detected recurring expenses |
| `PUT` | `/api/recurring/:id` | Update (confirm/dismiss, change category) |
| `POST` | `/api/recurring/detect` | Manually trigger detection |

#### Alerts
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/alerts` | List alerts (filterable by read/unread, severity) |
| `PUT` | `/api/alerts/:id/read` | Mark alert as read |
| `PUT` | `/api/alerts/:id/dismiss` | Dismiss alert |
| `POST` | `/api/alerts/read-all` | Mark all as read |

#### Categorization Rules
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/rules` | List categorization rules |
| `POST` | `/api/rules` | Create rule |
| `PUT` | `/api/rules/:id` | Update rule |
| `DELETE` | `/api/rules/:id` | Delete rule |

#### Reports & Analytics
| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/reports/monthly/:month` | Monthly snapshot |
| `GET` | `/api/reports/health-score` | Current financial health score |
| `GET` | `/api/reports/health-score/history` | Health score over time |
| `GET` | `/api/reports/cash-flow` | Cash flow forecast (30/60/90 days) |
| `GET` | `/api/reports/trends` | Spending trends by category over time |
| `GET` | `/api/reports/income-vs-expenses` | Income vs expenses breakdown |
| `GET` | `/api/reports/export/csv` | Export transactions as CSV |
| `GET` | `/api/reports/export/pdf` | Export monthly report as PDF |

---

## 9. Docker & Deployment

### docker-compose.yml

```yaml
services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: accountant
      POSTGRES_PASSWORD: ${DB_PASSWORD:-accountant_dev}
      POSTGRES_DB: the_accountant
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U accountant"]
      interval: 5s
      timeout: 5s
      retries: 5

  app:
    build: .
    ports:
      - "${APP_PORT:-8080}:8080"
    environment:
      DATABASE_URL: postgresql://accountant:${DB_PASSWORD:-accountant_dev}@db:5432/the_accountant
      RUST_LOG: info,the_accountant=debug
      UPLOAD_DIR: /data/uploads
      APP_PORT: 8080
    volumes:
      - uploads:/data/uploads
    depends_on:
      db:
        condition: service_healthy

volumes:
  pgdata:
  uploads:
```

### Dockerfile (multi-stage build)

```dockerfile
# Stage 1: Build
FROM rust:1.82-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src/ src/
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    poppler-utils \  # pdftotext as fallback PDF parser
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/the-accountant .
COPY migrations/ migrations/

EXPOSE 8080
CMD ["./the-accountant"]
```

### .env.example

```env
DATABASE_URL=postgresql://accountant:accountant_dev@localhost:5432/the_accountant
APP_PORT=8080
RUST_LOG=info,the_accountant=debug
UPLOAD_DIR=./uploads
DANGER_ZONE_THRESHOLD=5000
DEFAULT_CURRENCY=INR
```

### Running with OrbStack / Podman

```bash
# OrbStack (drop-in Docker replacement on macOS)
docker compose up -d

# Podman
podman compose up -d

# Run migrations
sqlx database create
sqlx migrate run
```

---

## 10. Future: Native Apps

The REST API is designed to be consumed by any frontend. Planned native apps:

| Platform | Tech | Notes |
|---|---|---|
| macOS | SwiftUI | Desktop app, menu bar widget for quick glance |
| iPadOS | SwiftUI | Full dashboard with split view |
| iOS | SwiftUI | Mobile-first, quick expense entry, alerts |

All platforms share the same Rust backend API. The backend can run:
- **Locally** via Docker for personal use
- **On a VPS/cloud** for remote access across devices
- **Embedded** (future) — compile Rust backend as a library for on-device use

---

## Appendix A: Implementation Priority

### Phase 1: Foundation (MVP)
1. Project scaffolding (Cargo, Docker, database)
2. Database migrations (all tables)
3. Account CRUD
4. Category CRUD with seed data
5. Manual transaction CRUD
6. Basic dashboard (balances, totals)

### Phase 2: Statement Import
7. PDF text extraction
8. Bank format parser trait + generic parser
9. HDFC parser (most common, good reference implementation)
10. Statement upload API with full pipeline
11. Auto-categorization engine
12. CSV/Excel import support

### Phase 3: Debt Management
13. Debt CRUD with payments
14. Debt limits
15. Debt summary and net position
16. Debt deadline alerts

### Phase 4: Budgeting & Planning
17. Budget CRUD with category breakdown
18. Budget progress tracking
19. Planned purchases CRUD
20. Smart purchase advisor

### Phase 5: Intelligence
21. Recurring expense detector
22. Cash flow forecaster
23. Financial health score
24. Alert system (all types)
25. Monthly snapshot generator

### Phase 6: Analytics & Export
26. Spending trends and analytics
27. Tags system
28. CSV export
29. PDF report generation
30. Income vs expenses views

### Phase 7: Additional Bank Parsers
31. SBI parser
32. ICICI parser
33. Axis parser
34. Kotak parser

---

## Appendix B: Key Design Decisions

1. **`rust_decimal::Decimal` for all money** — Never `f64`. Financial applications demand exact arithmetic.
2. **UUID primary keys** — No auto-increment leaking record counts. Safe for API exposure.
3. **Soft deletes for accounts** — `is_active` flag, not hard delete, to preserve transaction history.
4. **SHA-256 file hashing** — Prevent duplicate statement imports.
5. **Transaction-level dedup** — Even within a new statement, check for duplicate transactions by (account, date, amount, ref_no).
6. **Pluggable bank parsers** — Trait-based design makes adding new bank formats trivial.
7. **Categorization rules in DB** — User can create custom rules without code changes.
8. **JSONB for flexible fields** — Monthly snapshots, purchase funding details, health score breakdowns stored as JSONB for schema flexibility.
9. **Background scheduler** — Long-running analysis (recurring detection, forecasting) runs as scheduled jobs, not on request.
10. **No authentication (Phase 1)** — Single-user personal app. Auth added when multi-device support is needed.
