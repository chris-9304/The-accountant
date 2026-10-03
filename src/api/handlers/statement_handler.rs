use axum::{extract::{State, Path, Multipart}, Json, http::StatusCode};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use rust_decimal::Decimal;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::statement::*;
use crate::repositories::{statement_repo, transaction_repo, account_repo};
use crate::parsers::statement_parser;
use crate::services::categorizer;

pub async fn upload_statement(State(state): State<AppState>, mut multipart: Multipart) -> Result<Json<ApiResponse<StatementUploadResponse>>, AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name = String::new();
    let mut account_id: Option<Uuid> = None;
    let mut password: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Validation(format!("Multipart error: {}", e)))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().unwrap_or("statement").to_string();
                file_bytes = Some(field.bytes().await.map_err(|e| AppError::Validation(format!("Read error: {}", e)))?.to_vec());
            }
            "account_id" => {
                let text = field.text().await.map_err(|e| AppError::Validation(format!("{}", e)))?;
                account_id = Some(text.parse::<Uuid>().map_err(|_| AppError::Validation("Invalid account_id".into()))?);
            }
            "password" => { password = Some(field.text().await.map_err(|e| AppError::Validation(format!("{}", e)))?); }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or_else(|| AppError::Validation("No file uploaded".into()))?;
    let account_id = account_id.ok_or_else(|| AppError::Validation("account_id required".into()))?;
    let _ = account_repo::find_by_id(&state.db, account_id).await?.ok_or_else(|| AppError::NotFound("Account not found".into()))?;

    let hash = hex::encode(Sha256::digest(&bytes));
    if statement_repo::find_by_hash(&state.db, &hash).await?.is_some() {
        return Err(AppError::Duplicate("Statement already uploaded".into()));
    }

    let ext = file_name.rsplit('.').next().unwrap_or("pdf").to_lowercase();
    let uid = Uuid::new_v4();
    let save_path = format!("{}/{}.{}", state.config.upload_dir, uid, ext);
    tokio::fs::write(&save_path, &bytes).await.map_err(|e| AppError::Internal(anyhow::anyhow!("Save failed: {}", e)))?;

    let stmt = statement_repo::create(&state.db, account_id, &file_name, &save_path, &ext, &hash).await?;

    match statement_parser::parse_statement(&save_path, &ext, password.as_deref()) {
        Ok(parsed) => {
            let mut total_credits = Decimal::ZERO;
            let mut total_debits = Decimal::ZERO;
            let mut inserted = 0i32;
            for raw in &parsed.transactions {
                let (amount, txn_type) = if let Some(credit) = raw.credit_amount { (credit, "credit") }
                    else if let Some(debit) = raw.debit_amount { (debit, "debit") } else { continue; };
                if transaction_repo::find_duplicate(&state.db, account_id, raw.date, amount, raw.reference_number.as_deref()).await? { continue; }
                let cat_id = categorizer::categorize_transaction(&state.db, &raw.description).await?;
                transaction_repo::create_from_statement(&state.db, account_id, stmt.id, cat_id, amount, txn_type, &raw.description, None, raw.reference_number.as_deref(), raw.date, raw.balance).await?;
                if txn_type == "credit" { total_credits += amount; } else { total_debits += amount; }
                inserted += 1;
            }
            if let Some(last) = parsed.transactions.last() { if let Some(bal) = last.balance { account_repo::update_balance(&state.db, account_id, bal).await?; } }
            statement_repo::update_parsed(&state.db, stmt.id, &parsed.bank_name, parsed.period_start, parsed.period_end, inserted, total_credits, total_debits).await?;
            Ok(Json(ApiResponse::success(StatementUploadResponse { statement_id: stmt.id, status: "completed".into(), total_transactions: inserted, total_credits, total_debits, message: format!("{} transactions imported from {}", inserted, parsed.bank_name) })))
        }
        Err(e) => {
            statement_repo::update_status(&state.db, stmt.id, "failed", Some(&e.to_string())).await?;
            Err(e)
        }
    }
}

pub async fn list_statements(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Statement>>>, AppError> {
    Ok(Json(ApiResponse::success(statement_repo::find_all(&state.db).await?)))
}

pub async fn get_statement(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<Statement>>, AppError> {
    let s = statement_repo::find_by_id(&state.db, id).await?.ok_or_else(|| AppError::NotFound("Statement not found".into()))?;
    Ok(Json(ApiResponse::success(s)))
}

pub async fn delete_statement(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    if !statement_repo::delete(&state.db, id).await? { return Err(AppError::NotFound("Statement not found".into())); }
    Ok(StatusCode::NO_CONTENT)
}
