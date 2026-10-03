pub mod pdf_extractor;
pub mod csv_extractor;
pub mod excel_extractor;
pub mod statement_parser;
pub mod bank_formats;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    pub date: NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    pub debit_amount: Option<Decimal>,
    pub credit_amount: Option<Decimal>,
    pub balance: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct ParsedStatement {
    pub bank_name: String,
    pub account_number: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub transactions: Vec<RawTransaction>,
}

pub trait BankFormatParser: Send + Sync {
    fn can_parse(&self, raw_text: &str) -> bool;
    fn parse(&self, raw_text: &str) -> Result<ParsedStatement, crate::errors::AppError>;
    fn bank_name(&self) -> &str;
}
