use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::models::account::*;
use crate::repositories::account_repo;

pub async fn list_accounts(pool: &PgPool) -> Result<Vec<Account>, AppError> {
    Ok(account_repo::find_all(pool).await?)
}

pub async fn get_account(pool: &PgPool, id: Uuid) -> Result<Account, AppError> {
    account_repo::find_by_id(pool, id).await?.ok_or_else(|| AppError::NotFound(format!("Account {} not found", id)))
}

pub async fn create_account(pool: &PgPool, req: CreateAccountRequest) -> Result<Account, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("Account name cannot be empty".into()));
    }
    let valid = ["bank", "upi_lite", "cash"];
    if !valid.contains(&req.account_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid account type '{}'. Must be: bank, upi_lite, cash", req.account_type)));
    }
    Ok(account_repo::create(pool, &req).await?)
}

pub async fn update_account(pool: &PgPool, id: Uuid, req: UpdateAccountRequest) -> Result<Account, AppError> {
    if let Some(ref name) = req.name { if name.trim().is_empty() { return Err(AppError::Validation("Name cannot be empty".into())); } }
    account_repo::update(pool, id, &req).await?.ok_or_else(|| AppError::NotFound(format!("Account {} not found", id)))
}

pub async fn delete_account(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    if !account_repo::delete(pool, id).await? { return Err(AppError::NotFound(format!("Account {} not found", id))); }
    Ok(())
}
