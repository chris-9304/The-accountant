# Claude Code Instructions — The Accountant

> Personal Finance Management System · Rust Backend · PostgreSQL · Docker

Read `architecture.md` in this same directory for the full system architecture, database schema, API design, and implementation priorities.

---

## Project Overview

The Accountant is a personal finance management application with a Rust (Axum) backend and PostgreSQL database, deployed via Docker. It automatically ingests bank statements (PDF/CSV/Excel), categorizes expenses, tracks debt, manages budgets, forecasts cash flow, and provides financial health insights.

**This is a single-user personal finance app. No authentication is needed in Phase 1.**

---

## Tech Stack (Non-Negotiable)

- **Language**: Rust (latest stable edition 2024)
- **Framework**: Axum 0.7+
- **Database**: PostgreSQL 16+ via SQLx 0.8+ (compile-time checked queries)
- **Money type**: `rust_decimal::Decimal` — **NEVER use f32/f64 for monetary values**
- **IDs**: `uuid::Uuid` (v4) for all primary keys
- **Dates**: `chrono::NaiveDate` for dates, `chrono::DateTime<Utc>` for timestamps
- **Serialization**: serde + serde_json
- **PDF parsing**: `pdf-extract` + `lopdf` (with `pdftotext` from poppler-utils as Docker fallback)
- **CSV**: `csv` crate
- **Excel**: `calamine` crate
- **Error handling**: `thiserror` for library errors, `anyhow` for application errors
- **Logging**: `tracing` + `tracing-subscriber`
- **Background jobs**: `tokio-cron-scheduler`

---

## Architecture Rules

### Layered Architecture (Strict)

```
API Layer (handlers) → Service Layer (business logic) → Repository Layer (SQL) → PostgreSQL
                                                      → Parser Layer (PDF/CSV)  → File System
```

1. **Handlers** call **services** only. Never write SQL in a handler.
2. **Services** call **repositories** and **parsers**. All business rules live here.
3. **Repositories** execute SQL queries via SQLx. Return domain models. No business logic.
4. **Parsers** convert raw file data to structs. No database access.

### Code Organization

```
src/
├── main.rs              → Entry point
├── lib.rs               → Re-exports
├── config.rs            → Env config
├── errors.rs            → Error types
├── app_state.rs         → AppState (PgPool + config)
├── models/              → Domain structs + API DTOs
├── repositories/        → Database queries (SQLx)
├── services/            → Business logic
├── parsers/             → Statement parsing
│   └── bank_formats/    → Per-bank parsers
├── api/
│   ├── router.rs        → Route definitions
│   ├── middleware.rs     → Logging, error handling
│   ├── responses.rs     → Standardized response wrapper
│   └── handlers/        → HTTP handler functions
└── scheduler/           → Background jobs
```

### Naming Conventions

- Files: `snake_case.rs`
- Structs/Enums: `PascalCase`
- Functions: `snake_case`
- Database tables: `snake_case` (plural: `accounts`, `transactions`)
- Database columns: `snake_case`
- API endpoints: `kebab-case` for multi-word paths (e.g., `/health-score`)
- Repository functions: `find_*`, `find_all_*`, `create_*`, `update_*`, `delete_*`
- Service functions: descriptive verbs (e.g., `calculate_health_score`, `detect_recurring`)

---

## Database

### Migrations

Use `sqlx-cli` for migrations. Each migration is a numbered `.sql` file in `migrations/`.

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Create migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

Migration files should be ordered and prefixed:
```
migrations/
├── 001_create_accounts.sql
├── 002_create_categories.sql
├── ...
```

### Key Schema Rules

1. All tables have `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`
2. All tables have `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
3. Mutable tables also have `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
4. All monetary columns: `DECIMAL(15, 2)` — 15 digits, 2 decimal places
5. Use `REFERENCES` with appropriate `ON DELETE` behavior
6. Add indexes on frequently filtered/sorted columns
7. Use `JSONB` for flexible/variable-structure data (snapshots, funding details)

### Full Schema

Refer to **Section 5: Database Schema** in `architecture.md` for complete table definitions.

---

## API Standards

### Response Format

Every API response uses this wrapper:

```rust
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub meta: Option<PaginationMeta>,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}
```

### Error Handling

Map all errors to appropriate HTTP status codes:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate: {0}")]
    Duplicate(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Duplicate(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::ParseError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        // Return ApiResponse with success: false
    }
}
```

### Pagination

List endpoints accept query params:
```
?page=1&per_page=50&sort_by=transaction_date&sort_order=desc
```

### Filtering

Transaction list supports these filters:
```
?account_id=UUID
&category_id=UUID
&date_from=2026-01-01
&date_to=2026-01-31
&amount_min=100
&amount_max=5000
&transaction_type=debit
&source=statement
&tag=trip-goa
&search=swiggy
```

---

## Core Implementation Details

### Bank Statement Pipeline

The most critical flow. When a user uploads a statement:

1. **Save file** to `UPLOAD_DIR`, compute SHA-256 hash
2. **Check for duplicates** — if hash exists in `statements` table, reject
3. **Create statement record** with status `processing`
4. **Extract text** from file (PDF/CSV/Excel)
5. **Detect bank format** — iterate through registered `BankFormatParser` implementations, call `can_parse()`
6. **Parse transactions** — call matched parser's `parse()` method
7. **Auto-categorize** each transaction using `categorization_rules`
8. **Deduplicate transactions** — check for existing (account_id, transaction_date, amount, reference_number)
9. **Insert transactions** into database
10. **Update account balance** based on latest transaction's `balance_after`
11. **Update statement** — set status `completed`, transaction count, totals
12. **Post-process** — trigger recurring detection, budget alerts

```rust
// parsers/mod.rs
pub trait BankFormatParser: Send + Sync {
    fn can_parse(&self, raw_text: &str) -> bool;
    fn parse(&self, raw_text: &str) -> Result<ParsedStatement, ParserError>;
    fn bank_name(&self) -> &str;
}

// parsers/bank_formats/mod.rs
pub fn detect_and_parse(raw_text: &str) -> Result<ParsedStatement, ParserError> {
    let parsers: Vec<Box<dyn BankFormatParser>> = vec![
        Box::new(HdfcParser),
        Box::new(SbiParser),
        Box::new(IciciParser),
        Box::new(AxisParser),
        Box::new(KotakParser),
        Box::new(GenericParser), // always last — fallback
    ];

    for parser in &parsers {
        if parser.can_parse(raw_text) {
            return parser.parse(raw_text);
        }
    }

    Err(ParserError::UnsupportedFormat)
}
```

### Auto-Categorization Engine

```rust
// services/categorizer.rs
pub async fn categorize_transaction(
    pool: &PgPool,
    description: &str,
) -> Result<Option<Uuid>, AppError> {
    let rules = rule_repo::find_all_active(pool).await?;
    // Rules are ordered by priority DESC

    for rule in rules {
        let matched = match rule.match_type.as_str() {
            "exact" => description.eq_ignore_ascii_case(&rule.pattern),
            "contains" => description.to_lowercase().contains(&rule.pattern.to_lowercase()),
            "starts_with" => description.to_lowercase().starts_with(&rule.pattern.to_lowercase()),
            "regex" => Regex::new(&rule.pattern)?.is_match(description),
            _ => false,
        };

        if matched {
            return Ok(Some(rule.category_id));
        }
    }

    Ok(None) // Uncategorized
}
```

### Financial Health Score

```rust
pub struct HealthScoreInput {
    pub monthly_income: Decimal,
    pub monthly_expenses: Decimal,
    pub total_savings: Decimal,       // total balance across all accounts
    pub total_debt_owed: Decimal,
    pub total_debt_receivable: Decimal,
    pub collected_this_month: Decimal,
    pub budget_limit: Decimal,
    pub actual_spending: Decimal,
}

pub fn calculate_health_score(input: &HealthScoreInput) -> HealthScore {
    let savings_rate = if input.monthly_income > Decimal::ZERO {
        ((input.monthly_income - input.monthly_expenses) / input.monthly_income * dec!(100))
            .min(dec!(100))
            .max(dec!(0))
    } else {
        Decimal::ZERO
    };

    // savings_rate_score: 20%+ savings = 100, linear scale
    let savings_rate_score = (savings_rate / dec!(20) * dec!(100)).min(dec!(100));

    // debt_ratio_score: 0 debt = 100, debt >= income = 0
    let debt_ratio_score = if input.monthly_income > Decimal::ZERO {
        (dec!(1) - input.total_debt_owed / input.monthly_income) * dec!(100)
    } else {
        dec!(100)
    }.max(dec!(0)).min(dec!(100));

    // budget_adherence_score: under budget = 100, over by 50%+ = 0
    let budget_adherence_score = if input.budget_limit > Decimal::ZERO {
        let ratio = input.actual_spending / input.budget_limit;
        if ratio <= dec!(1) { dec!(100) }
        else { ((dec!(1.5) - ratio) / dec!(0.5) * dec!(100)).max(dec!(0)) }
    } else {
        dec!(50) // no budget set = neutral
    };

    // emergency_fund_score: 3+ months expenses = 100
    let months_covered = if input.monthly_expenses > Decimal::ZERO {
        input.total_savings / input.monthly_expenses
    } else {
        dec!(6) // no expenses = perfect
    };
    let emergency_fund_score = (months_covered / dec!(3) * dec!(100)).min(dec!(100));

    // debt_collection_score
    let debt_collection_score = if input.total_debt_receivable > Decimal::ZERO {
        (input.collected_this_month / input.total_debt_receivable * dec!(100)).min(dec!(100))
    } else {
        dec!(100) // no debt to collect = perfect
    };

    // Weighted average
    let overall = (savings_rate_score * dec!(25)
        + debt_ratio_score * dec!(25)
        + budget_adherence_score * dec!(20)
        + emergency_fund_score * dec!(15)
        + debt_collection_score * dec!(15)) / dec!(100);

    HealthScore {
        overall_score: overall.to_u32().unwrap_or(0),
        savings_rate_score: savings_rate_score.to_u32().unwrap_or(0),
        debt_ratio_score: debt_ratio_score.to_u32().unwrap_or(0),
        budget_adherence_score: budget_adherence_score.to_u32().unwrap_or(0),
        emergency_fund_score: emergency_fund_score.to_u32().unwrap_or(0),
        debt_collection_score: debt_collection_score.to_u32().unwrap_or(0),
    }
}
```

---

## Background Scheduler

Configure scheduled jobs in `main.rs` using `tokio-cron-scheduler`:

| Job | Schedule | Description |
|---|---|---|
| Recurring Detector | Daily at 2 AM | Scan transactions for recurring patterns |
| Alert Checker | Every hour | Check debt deadlines, budget status, balances |
| Snapshot Generator | 1st of month, 3 AM | Generate previous month's financial snapshot |
| Forecast Refresh | Weekly (Sunday 3 AM) | Regenerate 90-day cash flow forecast |
| Debt Status Updater | Daily at midnight | Mark overdue debts, update statuses |

---

## Implementation Order

Follow this exact order. Each phase should be fully tested before moving on.

### Phase 1: Foundation
1. `cargo init` with proper `Cargo.toml` (all dependencies listed in architecture.md)
2. `docker-compose.yml` + `Dockerfile` (from architecture.md)
3. `.env.example` and `config.rs`
4. `app_state.rs` with `PgPool`
5. `errors.rs` with `AppError` enum implementing `IntoResponse`
6. `api/responses.rs` with `ApiResponse<T>` wrapper
7. Database migrations for ALL tables (see architecture.md Section 5)
8. Seed migration for default categories and sample categorization rules
9. `main.rs` — init DB pool, run migrations, start Axum server

### Phase 2: Core CRUD
10. Models for: Account, Category, Transaction, Tag
11. Repositories for: accounts, categories, transactions, tags
12. Services for: accounts, transactions (including manual entry)
13. Handlers + routes for: accounts, categories, transactions, tags
14. Dashboard handler (basic: account balances, totals)

### Phase 3: Statement Import
15. PDF text extractor (`parsers/pdf_extractor.rs`)
16. CSV extractor (`parsers/csv_extractor.rs`)
17. Excel extractor (`parsers/excel_extractor.rs`)
18. `BankFormatParser` trait
19. Generic parser (fallback)
20. HDFC parser (reference implementation)
21. Statement upload handler with full pipeline
22. Auto-categorization engine
23. Transaction deduplication

### Phase 4: Debt System
24. Models + Repo + Service + Handlers for debts
25. Debt payments
26. Debt limits
27. Debt summary (net position, projected balance after settlements)

### Phase 5: Budget & Planning
28. Budget CRUD with category allocations
29. Budget progress tracking (actual vs. limit per category)
30. Planned purchases CRUD
31. Smart purchase advisor service

### Phase 6: Intelligence
32. Recurring expense detector (background job)
33. Cash flow forecaster
34. Financial health score calculator
35. Alert system (all alert types)
36. Monthly snapshot generator
37. Scheduler setup in `main.rs`

### Phase 7: Analytics & Export
38. Spending trends endpoint
39. Income vs. expenses breakdown
40. CSV export
41. PDF report generation
42. Enhanced dashboard with all data

### Phase 8: Additional Parsers
43. SBI, ICICI, Axis, Kotak bank parsers

---

## Testing

### Unit Tests
- All services should have unit tests
- All parsers should have unit tests with fixture files
- Place sample bank statement PDFs/CSVs in `tests/fixtures/`

### Integration Tests
- API endpoint tests using `axum::test` utilities
- Use a test database (create/drop per test suite)
- Test the full statement upload → parse → categorize → store pipeline

### Test Commands
```bash
# Run all tests
cargo test

# Run specific module tests
cargo test services::
cargo test parsers::
cargo test api::

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## Docker Commands

```bash
# Build and start everything
docker compose up -d --build

# View logs
docker compose logs -f app

# Run migrations manually
docker compose exec app sqlx migrate run

# Stop
docker compose down

# Reset database
docker compose down -v && docker compose up -d
```

---

## Important Reminders

1. **NEVER use floating point for money.** Always `rust_decimal::Decimal`.
2. **All SQL queries must be compile-time checked** via `sqlx::query!` or `sqlx::query_as!`. Set `DATABASE_URL` in `.env` for offline checking, or use `sqlx prepare` to generate query metadata.
3. **Handle errors properly.** No `.unwrap()` in production code. Use `?` operator and `AppError`.
4. **Log important operations** using `tracing::info!`, `tracing::warn!`, `tracing::error!`.
5. **Validate all inputs** in the service layer before database operations.
6. **Use database transactions** for multi-table operations (e.g., creating debt + first payment, importing statement + transactions).
7. **Keep handlers thin.** Extract body, call service, return response. That's it.
8. **The statement parsing pipeline is the most complex part.** Build it incrementally: first get PDF→text working, then text→structured data, then auto-categorize, then persist.
