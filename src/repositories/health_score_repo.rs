use sqlx::PgPool;
use crate::models::health_score::*;

pub async fn find_latest(pool: &PgPool) -> Result<Option<HealthScore>, sqlx::Error> {
    sqlx::query_as::<_, HealthScore>("SELECT * FROM financial_health_scores ORDER BY calculated_at DESC LIMIT 1").fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, s: &HealthScore) -> Result<HealthScore, sqlx::Error> {
    sqlx::query_as::<_, HealthScore>(
        "INSERT INTO financial_health_scores (id, overall_score, savings_rate_score, debt_ratio_score, budget_adherence_score, emergency_fund_score, debt_collection_score, details) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *"
    ).bind(s.id).bind(s.overall_score).bind(s.savings_rate_score).bind(s.debt_ratio_score)
     .bind(s.budget_adherence_score).bind(s.emergency_fund_score).bind(s.debt_collection_score).bind(&s.details)
     .fetch_one(pool).await
}

pub async fn find_history(pool: &PgPool, limit: i64) -> Result<Vec<HealthScore>, sqlx::Error> {
    sqlx::query_as::<_, HealthScore>("SELECT * FROM financial_health_scores ORDER BY calculated_at DESC LIMIT $1").bind(limit).fetch_all(pool).await
}
