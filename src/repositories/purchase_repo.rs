use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::models::planned_purchase::*;

pub async fn find_all(pool: &PgPool, status: Option<&str>) -> Result<Vec<PlannedPurchase>, sqlx::Error> {
    sqlx::query_as::<_, PlannedPurchase>(
        "SELECT * FROM planned_purchases WHERE ($1::text IS NULL OR status = $1) ORDER BY urgency DESC, target_date ASC NULLS LAST"
    ).bind(status).fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<PlannedPurchase>, sqlx::Error> {
    sqlx::query_as::<_, PlannedPurchase>("SELECT * FROM planned_purchases WHERE id = $1").bind(id).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreatePurchaseRequest) -> Result<PlannedPurchase, sqlx::Error> {
    let urgency = req.urgency.as_deref().unwrap_or("normal");
    sqlx::query_as::<_, PlannedPurchase>(
        "INSERT INTO planned_purchases (item_name, estimated_cost, urgency, target_date, category_id, notes) VALUES ($1,$2,$3,$4,$5,$6) RETURNING *"
    ).bind(&req.item_name).bind(req.estimated_cost).bind(urgency).bind(req.target_date).bind(req.category_id).bind(&req.notes)
     .fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdatePurchaseRequest) -> Result<Option<PlannedPurchase>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(c) = current else { return Ok(None) };
    let name = req.item_name.as_deref().unwrap_or(&c.item_name);
    let cost = req.estimated_cost.unwrap_or(c.estimated_cost);
    let urgency = req.urgency.as_deref().unwrap_or(&c.urgency);
    let target = req.target_date.or(c.target_date);
    let status = req.status.as_deref().unwrap_or(&c.status);
    let notes = req.notes.as_ref().or(c.notes.as_ref());
    sqlx::query_as::<_, PlannedPurchase>(
        "UPDATE planned_purchases SET item_name=$1,estimated_cost=$2,urgency=$3,target_date=$4,status=$5,notes=$6,updated_at=NOW() WHERE id=$7 RETURNING *"
    ).bind(name).bind(cost).bind(urgency).bind(target).bind(status).bind(notes).bind(id).fetch_optional(pool).await
}

pub async fn update_advice(pool: &PgPool, id: Uuid, recommended_date: Option<NaiveDate>, strategy: &str, details: serde_json::Value) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE planned_purchases SET recommended_purchase_date=$1, funding_strategy=$2, funding_details=$3, updated_at=NOW() WHERE id=$4")
        .bind(recommended_date).bind(strategy).bind(details).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM planned_purchases WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
