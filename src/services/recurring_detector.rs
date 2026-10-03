use sqlx::PgPool;
use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::errors::AppError;
use crate::models::recurring_expense::RecurringExpense;
use crate::models::transaction::TransactionFilters;
use crate::repositories::{transaction_repo, recurring_repo};

pub async fn detect_recurring(pool: &PgPool) -> Result<Vec<RecurringExpense>, AppError> {
    let filters = TransactionFilters { transaction_type: Some("debit".into()), per_page: Some(10000), ..Default::default() };
    let (txns, _) = transaction_repo::find_all(pool, &filters).await?;
    let mut groups: HashMap<String, Vec<(chrono::NaiveDate, Decimal)>> = HashMap::new();
    for t in &txns {
        let m = t.merchant_name.as_deref().or(t.description.as_deref()).unwrap_or("").to_lowercase().trim().to_string();
        if !m.is_empty() { groups.entry(m).or_default().push((t.transaction_date, t.amount)); }
    }
    let mut detected = Vec::new();
    for (merchant, mut entries) in groups {
        if entries.len() < 3 { continue; }
        entries.sort_by_key(|(d,_)| *d);
        let intervals: Vec<i64> = entries.windows(2).filter_map(|w| { let d = (w[1].0 - w[0].0).num_days(); if d > 0 { Some(d) } else { None } }).collect();
        if intervals.is_empty() { continue; }
        let avg: f64 = intervals.iter().sum::<i64>() as f64 / intervals.len() as f64;
        let (freq, exp) = if (5.0..=9.0).contains(&avg) { ("weekly",7.0) } else if (12.0..=17.0).contains(&avg) { ("biweekly",14.0) }
            else if (25.0..=35.0).contains(&avg) { ("monthly",30.0) } else if (80.0..=100.0).contains(&avg) { ("quarterly",90.0) }
            else if (350.0..=380.0).contains(&avg) { ("yearly",365.0) } else { continue; };
        let var: f64 = intervals.iter().map(|&i| (i as f64 - exp).powi(2)).sum::<f64>() / intervals.len() as f64;
        let conf = (1.0 - (var.sqrt() / exp)).max(0.0).min(1.0);
        if conf < 0.5 { continue; }
        let amts: Vec<Decimal> = entries.iter().map(|(_,a)| *a).collect();
        let avg_a = amts.iter().copied().sum::<Decimal>() / Decimal::new(amts.len() as i64,0);
        let min_a = amts.iter().copied().min().unwrap_or(Decimal::ZERO);
        let max_a = amts.iter().copied().max().unwrap_or(Decimal::ZERO);
        let last = entries.last().map(|(d,_)| *d);
        let next = last.map(|d| d + chrono::Duration::days(exp as i64));
        let cd = Decimal::new((conf * 100.0) as i64, 2);
        if let Some(existing) = recurring_repo::find_by_merchant(pool, &merchant).await? {
            recurring_repo::update_detection(pool, existing.id, avg_a, min_a, max_a, entries.len() as i32, last.unwrap_or_default(), next.unwrap_or_default(), cd).await?;
        } else {
            let ne = RecurringExpense { id: uuid::Uuid::new_v4(), merchant_name: merchant.clone(), category_id: None, average_amount: avg_a, min_amount: Some(min_a), max_amount: Some(max_a), frequency: freq.into(), last_seen: last, next_expected: next, account_id: None, occurrence_count: entries.len() as i32, is_active: true, confidence: cd, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            recurring_repo::create(pool, &ne).await?;
        }
        if let Some(r) = recurring_repo::find_by_merchant(pool, &merchant).await? { detected.push(r); }
    }
    tracing::info!("Detected {} recurring expenses", detected.len());
    Ok(detected)
}
