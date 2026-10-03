use sqlx::PgPool;
use chrono::NaiveDate;
use crate::models::cash_flow::*;

pub async fn find_range(pool: &PgPool, from: NaiveDate, to: NaiveDate) -> Result<Vec<CashFlowForecast>, sqlx::Error> {
    sqlx::query_as::<_, CashFlowForecast>(
        "SELECT * FROM cash_flow_forecasts WHERE forecast_date >= $1 AND forecast_date <= $2 ORDER BY forecast_date"
    ).bind(from).bind(to).fetch_all(pool).await
}

pub async fn delete_future(pool: &PgPool, from: NaiveDate) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM cash_flow_forecasts WHERE forecast_date >= $1").bind(from).execute(pool).await?;
    Ok(())
}

pub async fn create_batch(pool: &PgPool, forecasts: &[CashFlowForecast]) -> Result<(), sqlx::Error> {
    for f in forecasts {
        sqlx::query(
            "INSERT INTO cash_flow_forecasts (id, forecast_date, projected_income, projected_expenses, projected_debt_payments, projected_debt_collections, projected_balance, is_danger_zone, danger_threshold, confidence) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
        ).bind(f.id).bind(f.forecast_date).bind(f.projected_income).bind(f.projected_expenses)
         .bind(f.projected_debt_payments).bind(f.projected_debt_collections).bind(f.projected_balance)
         .bind(f.is_danger_zone).bind(f.danger_threshold).bind(f.confidence)
         .execute(pool).await?;
    }
    Ok(())
}
