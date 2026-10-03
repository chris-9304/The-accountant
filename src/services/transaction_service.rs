use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::models::transaction::*;
use crate::repositories::{transaction_repo, account_repo};
use crate::services::categorizer;

pub async fn list_transactions(pool: &PgPool, filters: TransactionFilters) -> Result<(Vec<Transaction>, i64), AppError> {
    Ok(transaction_repo::find_all(pool, &filters).await?)
}

pub async fn get_transaction(pool: &PgPool, id: Uuid) -> Result<Transaction, AppError> {
    transaction_repo::find_by_id(pool, id).await?.ok_or_else(|| AppError::NotFound(format!("Transaction {} not found", id)))
}

pub async fn create_transaction(pool: &PgPool, mut req: CreateTransactionRequest) -> Result<Transaction, AppError> {
    let account = account_repo::find_by_id(pool, req.account_id).await?.ok_or_else(|| AppError::NotFound(format!("Account {} not found", req.account_id)))?;
    if !account.is_active { return Err(AppError::Validation("Cannot add to inactive account".into())); }
    if req.category_id.is_none() {
        if let Some(desc) = &req.description { req.category_id = categorizer::categorize_transaction(pool, desc).await?; }
        if req.category_id.is_none() { if let Some(m) = &req.merchant_name { req.category_id = categorizer::categorize_transaction(pool, m).await?; } }
    }
    let tag_ids = req.tag_ids.clone();
    let txn = transaction_repo::create_from_request(pool, &req).await?;
    if let Some(tags) = tag_ids { if !tags.is_empty() { transaction_repo::set_tags(pool, txn.id, &tags).await?; } }
    let change = if req.transaction_type == "credit" { req.amount } else { -req.amount };
    account_repo::update_balance(pool, req.account_id, account.balance + change).await?;
    Ok(txn)
}

pub async fn update_transaction(pool: &PgPool, id: Uuid, req: UpdateTransactionRequest) -> Result<Transaction, AppError> {
    let tag_ids = req.tag_ids.clone();
    let txn = transaction_repo::update(pool, id, &req).await?.ok_or_else(|| AppError::NotFound(format!("Transaction {} not found", id)))?;
    if let Some(tags) = tag_ids { transaction_repo::set_tags(pool, id, &tags).await?; }
    Ok(txn)
}

pub async fn delete_transaction(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let txn = transaction_repo::find_by_id(pool, id).await?.ok_or_else(|| AppError::NotFound(format!("Transaction {} not found", id)))?;
    if let Some(account) = account_repo::find_by_id(pool, txn.account_id).await? {
        let change = if txn.transaction_type == "credit" { -txn.amount } else { txn.amount };
        account_repo::update_balance(pool, txn.account_id, account.balance + change).await?;
    }
    transaction_repo::delete(pool, id).await?;
    Ok(())
}

pub async fn add_tag(pool: &PgPool, txn_id: Uuid, tag_id: Uuid) -> Result<(), AppError> {
    let _ = get_transaction(pool, txn_id).await?;
    Ok(transaction_repo::add_tag(pool, txn_id, tag_id).await?)
}

pub async fn remove_tag(pool: &PgPool, txn_id: Uuid, tag_id: Uuid) -> Result<(), AppError> {
    Ok(transaction_repo::remove_tag(pool, txn_id, tag_id).await?)
}
