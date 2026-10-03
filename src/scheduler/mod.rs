use sqlx::PgPool;
use tokio_cron_scheduler::{JobScheduler, Job};
use chrono::Datelike;
use crate::errors::AppError;

pub async fn start_scheduler(pool: PgPool) -> Result<(), AppError> {
    let sched = JobScheduler::new().await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Scheduler init failed: {}", e)))?;

    // Daily at 2 AM - Recurring expense detection
    let p1 = pool.clone();
    sched.add(Job::new_async("0 0 2 * * *", move |_uuid, _l| {
        let pool = p1.clone();
        Box::pin(async move {
            tracing::info!("Running recurring expense detection...");
            if let Err(e) = crate::services::recurring_detector::detect_recurring(&pool).await {
                tracing::error!("Recurring detection failed: {:?}", e);
            }
        })
    }).map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?)
    .await.map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    // Every hour - Alert checking
    let p2 = pool.clone();
    sched.add(Job::new_async("0 0 * * * *", move |_uuid, _l| {
        let pool = p2.clone();
        Box::pin(async move {
            tracing::info!("Checking alerts...");
            if let Err(e) = crate::services::alert_service::check_and_generate_alerts(&pool).await {
                tracing::error!("Alert check failed: {:?}", e);
            }
        })
    }).map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?)
    .await.map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    // 1st of month at 3 AM - Snapshot generation
    let p3 = pool.clone();
    sched.add(Job::new_async("0 0 3 1 * *", move |_uuid, _l| {
        let pool = p3.clone();
        Box::pin(async move {
            tracing::info!("Generating monthly snapshot...");
            let today = chrono::Utc::now().date_naive();
            let prev = if today.month() == 1 {
                chrono::NaiveDate::from_ymd_opt(today.year() - 1, 12, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(today.year(), today.month() - 1, 1)
            };
            if let Some(month) = prev {
                if let Err(e) = crate::services::snapshot_service::generate_snapshot(&pool, month).await {
                    tracing::error!("Snapshot generation failed: {:?}", e);
                }
            }
        })
    }).map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?)
    .await.map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    // Weekly Sunday 3 AM - Cash flow forecast
    let p4 = pool.clone();
    sched.add(Job::new_async("0 0 3 * * 0", move |_uuid, _l| {
        let pool = p4.clone();
        Box::pin(async move {
            tracing::info!("Generating cash flow forecast...");
            if let Err(e) = crate::services::cash_flow_forecast::generate_forecast(&pool, 90).await {
                tracing::error!("Forecast generation failed: {:?}", e);
            }
        })
    }).map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?)
    .await.map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    sched.start().await.map_err(|e| AppError::Internal(anyhow::anyhow!("Scheduler start failed: {}", e)))?;
    tracing::info!("Background scheduler started with 4 jobs");
    Ok(())
}
