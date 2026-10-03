pub mod hdfc;
pub mod sbi;
pub mod icici;
pub mod axis;
pub mod kotak;
pub mod generic;

use crate::errors::AppError;
use crate::parsers::{BankFormatParser, ParsedStatement};

pub fn detect_and_parse(raw_text: &str) -> Result<ParsedStatement, AppError> {
    let parsers: Vec<Box<dyn BankFormatParser>> = vec![
        Box::new(hdfc::HdfcParser), Box::new(sbi::SbiParser), Box::new(icici::IciciParser),
        Box::new(axis::AxisParser), Box::new(kotak::KotakParser), Box::new(generic::GenericParser),
    ];
    for p in &parsers { if p.can_parse(raw_text) { tracing::info!("Detected: {}", p.bank_name()); return p.parse(raw_text); } }
    Err(AppError::ParseError("Could not detect bank format".into()))
}

pub fn parse_generic_records(records: Vec<Vec<String>>, source: &str) -> Result<ParsedStatement, AppError> {
    generic::parse_from_records(records, source)
}

pub fn parse_amount(s: &str) -> Option<rust_decimal::Decimal> {
    let c = s.trim().replace(',', "").replace("Rs.", "").replace("Rs", "").replace("INR", "").replace('\u{20b9}', "").trim().to_string();
    if c.is_empty() || c == "-" { return None; }
    c.parse().ok()
}

pub fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    let s = s.trim();
    for fmt in &["%d/%m/%Y","%d-%m-%Y","%d/%m/%y","%d-%m-%y","%Y-%m-%d","%d %b %Y","%d-%b-%Y"] {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) { return Some(d); }
    }
    None
}
