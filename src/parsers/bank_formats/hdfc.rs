use crate::errors::AppError;
use crate::parsers::{BankFormatParser, ParsedStatement, RawTransaction};
use crate::parsers::bank_formats::{parse_amount, parse_date};
use regex::Regex;

pub struct HdfcParser;
impl BankFormatParser for HdfcParser {
    fn can_parse(&self, t: &str) -> bool { let u = t.to_uppercase(); u.contains("HDFC BANK") || u.contains("HDFC LTD") }
    fn parse(&self, raw: &str) -> Result<ParsedStatement, AppError> {
        let re = Regex::new(r"(\d{2}/\d{2}/\d{2,4})\s+(.+?)\s+(\S+)\s+\d{2}/\d{2}/\d{2,4}\s+([\d,.]+)?\s+([\d,.]+)?\s+([\d,.]+)").map_err(|e| AppError::ParseError(format!("{}", e)))?;
        let mut txns = Vec::new();
        for line in raw.lines() {
            if let Some(c) = re.captures(line.trim()) {
                if let Some(d) = parse_date(c.get(1).unwrap().as_str()) {
                    txns.push(RawTransaction { date: d, description: c.get(2).unwrap().as_str().trim().into(), reference_number: c.get(3).map(|m| m.as_str().into()),
                        debit_amount: c.get(4).and_then(|m| parse_amount(m.as_str())), credit_amount: c.get(5).and_then(|m| parse_amount(m.as_str())), balance: c.get(6).and_then(|m| parse_amount(m.as_str())) });
                }
            }
        }
        if txns.is_empty() { return Err(AppError::ParseError("No HDFC transactions found".into())); }
        Ok(ParsedStatement { bank_name: "HDFC Bank".into(), account_number: None, period_start: txns.first().map(|t|t.date), period_end: txns.last().map(|t|t.date), transactions: txns })
    }
    fn bank_name(&self) -> &str { "HDFC Bank" }
}
