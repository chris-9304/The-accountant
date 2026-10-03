use sqlx::PgPool;
use uuid::Uuid;
use crate::models::category::*;

pub async fn find_all(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY is_income DESC, name")
        .fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
        .bind(id).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreateCategoryRequest) -> Result<Category, sqlx::Error> {
    let is_income = req.is_income.unwrap_or(false);
    sqlx::query_as::<_, Category>(
        "INSERT INTO categories (name, parent_id, icon, color, is_income) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    ).bind(&req.name).bind(req.parent_id).bind(&req.icon).bind(&req.color).bind(is_income)
     .fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateCategoryRequest) -> Result<Option<Category>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(current) = current else { return Ok(None) };
    let name = req.name.as_deref().unwrap_or(&current.name);
    let icon = req.icon.as_ref().or(current.icon.as_ref());
    let color = req.color.as_ref().or(current.color.as_ref());
    sqlx::query_as::<_, Category>(
        "UPDATE categories SET name = $1, icon = $2, color = $3 WHERE id = $4 RETURNING *"
    ).bind(name).bind(icon).bind(color).bind(id).fetch_optional(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM categories WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
