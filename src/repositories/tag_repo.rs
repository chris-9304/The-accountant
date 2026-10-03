use sqlx::PgPool;
use uuid::Uuid;
use crate::models::tag::*;

pub async fn find_all(pool: &PgPool) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY name").fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = $1").bind(id).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreateTagRequest) -> Result<Tag, sqlx::Error> {
    sqlx::query_as::<_, Tag>("INSERT INTO tags (name, color) VALUES ($1, $2) RETURNING *")
        .bind(&req.name).bind(&req.color).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateTagRequest) -> Result<Option<Tag>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(current) = current else { return Ok(None) };
    let name = req.name.as_deref().unwrap_or(&current.name);
    let color = req.color.as_ref().or(current.color.as_ref());
    sqlx::query_as::<_, Tag>("UPDATE tags SET name = $1, color = $2 WHERE id = $3 RETURNING *")
        .bind(name).bind(color).bind(id).fetch_optional(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM tags WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
