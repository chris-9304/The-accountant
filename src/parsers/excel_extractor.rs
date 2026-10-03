use crate::errors::AppError;
use calamine::{Reader, open_workbook, Xlsx, Data};

pub fn extract_records(file_path: &str) -> Result<Vec<Vec<String>>, AppError> {
    let mut wb: Xlsx<_> = open_workbook(file_path).map_err(|e| AppError::ParseError(format!("Failed to open Excel: {}", e)))?;
    let sheets = wb.sheet_names().to_vec();
    if sheets.is_empty() { return Err(AppError::ParseError("No sheets".into())); }
    let range = wb.worksheet_range(&sheets[0]).map_err(|e| AppError::ParseError(format!("Sheet error: {}", e)))?;
    let mut records = Vec::new();
    for row in range.rows() {
        let r: Vec<String> = row.iter().map(|c| match c { Data::Empty => String::new(), Data::String(s) => s.clone(), Data::Float(f) => format!("{:.2}", f), Data::Int(i) => i.to_string(), Data::Bool(b) => b.to_string(), _ => String::new() }).collect();
        if !r.iter().all(|s| s.is_empty()) { records.push(r); }
    }
    if records.is_empty() { return Err(AppError::ParseError("No records in Excel".into())); }
    Ok(records)
}
