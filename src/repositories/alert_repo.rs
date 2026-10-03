use sqlx::PgPool;
use uuid::Uuid;
use crate::models::alert::*;

pub async fn find_all(pool: &PgPool, f: &AlertFilters) -> Result<(Vec<Alert>, i64), sqlx::Error> {
    let page = f.page.unwrap_or(1).max(1);
    let per_page = f.per_page.unwrap_or(50);
    let offset = ((page - 1) * per_page) as i64;
    let rows: Vec<Alert> = sqlx::query_as::<_, Alert>(
        "SELECT * FROM alerts WHERE ($1::text IS NULL OR severity = $1) AND ($2::bool IS NULL OR is_read = $2) AND ($3::text IS NULL OR alert_type = $3) \
         ORDER BY triggered_at DESC LIMIT $4 OFFSET $5"
    ).bind(&f.severity).bind(f.is_read).bind(&f.alert_type).bind(per_page as i64).bind(offset).fetch_all(pool).await?;
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM alerts WHERE ($1::text IS NULL OR severity = $1) AND ($2::bool IS NULL OR is_read = $2) AND ($3::text IS NULL OR alert_type = $3)"
    ).bind(&f.severity).bind(f.is_read).bind(&f.alert_type).fetch_one(pool).await?;
    Ok((rows, count.0))
}

pub async fn create(pool: &PgPool, alert_type: &str, title: &str, message: &str, severity: &str, ref_type: Option<&str>, ref_id: Option<Uuid>) -> Result<Alert, sqlx::Error> {
    sqlx::query_as::<_, Alert>(
        "INSERT INTO alerts (alert_type, title, message, severity, reference_type, reference_id) VALUES ($1,$2,$3,$4,$5,$6) RETURNING *"
    ).bind(alert_type).bind(title).bind(message).bind(severity).bind(ref_type).bind(ref_id).fetch_one(pool).await
}

pub async fn mark_read(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE alerts SET is_read = true WHERE id = $1").bind(id).execute(pool).await?; Ok(())
}

pub async fn mark_dismissed(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE alerts SET is_dismissed = true WHERE id = $1").bind(id).execute(pool).await?; Ok(())
}

pub async fn mark_all_read(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE alerts SET is_read = true WHERE is_read = false").execute(pool).await?; Ok(())
}

pub async fn count_unread(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alerts WHERE is_read = false AND is_dismissed = false").fetch_one(pool).await?;
    Ok(row.0)
}
