use crate::errors::AppError;
use crate::parsers::{ParsedStatement, pdf_extractor, csv_extractor, excel_extractor, bank_formats};

pub fn parse_statement(file_path: &str, file_format: &str, password: Option<&str>) -> Result<ParsedStatement, AppError> {
    match file_format {
        "pdf" => {
            let text = pdf_extractor::extract_text(file_path, password)?;
            if text.trim().is_empty() { return Err(AppError::ParseError("PDF produced no text".into())); }
            bank_formats::detect_and_parse(&text)
        }
        "csv" => {
            let records = csv_extractor::extract_records(file_path)?;
            let text: String = records.iter().map(|r| r.join("  ")).collect::<Vec<_>>().join("\n");
            bank_formats::detect_and_parse(&text).or_else(|_| bank_formats::parse_generic_records(records, "CSV"))
        }
        "xlsx" | "xls" => {
            let records = excel_extractor::extract_records(file_path)?;
            let text: String = records.iter().map(|r| r.join("  ")).collect::<Vec<_>>().join("\n");
            bank_formats::detect_and_parse(&text).or_else(|_| bank_formats::parse_generic_records(records, "Excel"))
        }
        _ => Err(AppError::ParseError(format!("Unsupported format: {}", file_format))),
    }
}
