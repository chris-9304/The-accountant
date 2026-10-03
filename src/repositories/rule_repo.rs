use sqlx::PgPool;
use uuid::Uuid;
use crate::models::categorization_rule::*;

pub async fn find_all(pool: &PgPool) -> Result<Vec<CategorizationRule>, sqlx::Error> {
    sqlx::query_as::<_, CategorizationRule>("SELECT * FROM categorization_rules ORDER BY priority DESC").fetch_all(pool).await
}

pub async fn find_all_active(pool: &PgPool) -> Result<Vec<CategorizationRule>, sqlx::Error> {
    sqlx::query_as::<_, CategorizationRule>("SELECT * FROM categorization_rules WHERE is_active = true ORDER BY priority DESC").fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CategorizationRule>, sqlx::Error> {
    sqlx::query_as::<_, CategorizationRule>("SELECT * FROM categorization_rules WHERE id = $1").bind(id).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreateRuleRequest) -> Result<CategorizationRule, sqlx::Error> {
    let priority = req.priority.unwrap_or(0);
    let match_type = req.match_type.as_deref().unwrap_or("contains");
    sqlx::query_as::<_, CategorizationRule>(
        "INSERT INTO categorization_rules (pattern, category_id, priority, match_type) VALUES ($1,$2,$3,$4) RETURNING *"
    ).bind(&req.pattern).bind(req.category_id).bind(priority).bind(match_type).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateRuleRequest) -> Result<Option<CategorizationRule>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(c) = current else { return Ok(None) };
    let pattern = req.pattern.as_deref().unwrap_or(&c.pattern);
    let category_id = req.category_id.unwrap_or(c.category_id);
    let priority = req.priority.unwrap_or(c.priority);
    let match_type = req.match_type.as_deref().unwrap_or(&c.match_type);
    let is_active = req.is_active.unwrap_or(c.is_active);
    sqlx::query_as::<_, CategorizationRule>(
        "UPDATE categorization_rules SET pattern=$1, category_id=$2, priority=$3, match_type=$4, is_active=$5 WHERE id=$6 RETURNING *"
    ).bind(pattern).bind(category_id).bind(priority).bind(match_type).bind(is_active).bind(id).fetch_optional(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM categorization_rules WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
