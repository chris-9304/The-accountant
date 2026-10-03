use crate::errors::AppError;
use std::fs;

pub fn extract_records(file_path: &str) -> Result<Vec<Vec<String>>, AppError> {
    let content = fs::read_to_string(file_path).map_err(|e| AppError::ParseError(format!("Failed to read CSV: {}", e)))?;
    let first = content.lines().next().unwrap_or("");
    let delim = if first.contains('\t') { b'\t' } else if first.contains(';') { b';' } else { b',' };
    let mut rdr = csv::ReaderBuilder::new().delimiter(delim).flexible(true).has_headers(false).from_reader(content.as_bytes());
    let mut records = Vec::new();
    for result in rdr.records() {
        if let Ok(rec) = result {
            let row: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
            if !row.iter().all(|s| s.is_empty()) { records.push(row); }
        }
    }
    if records.is_empty() { return Err(AppError::ParseError("No records in CSV".into())); }
    Ok(records)
}
