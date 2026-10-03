use crate::errors::AppError;
use crate::parsers::{BankFormatParser, ParsedStatement, RawTransaction};
use crate::parsers::bank_formats::{parse_amount, parse_date};
use regex::Regex;

pub struct IciciParser;
impl BankFormatParser for IciciParser {
    fn can_parse(&self, t: &str) -> bool { t.to_uppercase().contains("ICICI") }
    fn parse(&self, raw: &str) -> Result<ParsedStatement, AppError> {
        let re = Regex::new(r"(\d{2}[/-]\d{2}[/-]\d{2,4})\s+(.+?)\s+([\d,.]+)?\s+([\d,.]+)?\s+([\d,.]+)").map_err(|e| AppError::ParseError(format!("{}", e)))?;
        let mut txns = Vec::new();
        for line in raw.lines() { if let Some(c) = re.captures(line.trim()) { if let Some(d) = parse_date(c.get(1).unwrap().as_str()) {
            txns.push(RawTransaction { date: d, description: c.get(2).unwrap().as_str().trim().into(), reference_number: None, debit_amount: c.get(3).and_then(|m| parse_amount(m.as_str())), credit_amount: c.get(4).and_then(|m| parse_amount(m.as_str())), balance: c.get(5).and_then(|m| parse_amount(m.as_str())) });
        }}}
        if txns.is_empty() { return Err(AppError::ParseError("No ICICI transactions".into())); }
        Ok(ParsedStatement { bank_name: "ICICI".into(), account_number: None, period_start: txns.first().map(|t|t.date), period_end: txns.last().map(|t|t.date), transactions: txns })
    }
    fn bank_name(&self) -> &str { "ICICI" }
}
