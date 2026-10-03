use sqlx::PgPool;
use uuid::Uuid;
use regex::Regex;
use crate::errors::AppError;
use crate::repositories::rule_repo;

pub async fn categorize_transaction(pool: &PgPool, description: &str) -> Result<Option<Uuid>, AppError> {
    let rules = rule_repo::find_all_active(pool).await?;
    let lower = description.to_lowercase();
    for rule in rules {
        let matched = match rule.match_type.as_str() {
            "exact" => lower == rule.pattern.to_lowercase(),
            "contains" => lower.contains(&rule.pattern.to_lowercase()),
            "starts_with" => lower.starts_with(&rule.pattern.to_lowercase()),
            "regex" => Regex::new(&rule.pattern).map(|re| re.is_match(description)).unwrap_or(false),
            _ => false,
        };
        if matched { return Ok(Some(rule.category_id)); }
    }
    Ok(None)
}
