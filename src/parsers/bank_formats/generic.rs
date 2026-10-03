use crate::errors::AppError;
use crate::parsers::{BankFormatParser, ParsedStatement, RawTransaction};
use crate::parsers::bank_formats::{parse_amount, parse_date};
use regex::Regex;

pub struct GenericParser;
impl BankFormatParser for GenericParser {
    fn can_parse(&self, _: &str) -> bool { true }
    fn parse(&self, raw: &str) -> Result<ParsedStatement, AppError> {
        let re = Regex::new(r"^\s*(\d{2}[/-]\d{2}[/-]\d{2,4})\s+(.+?)\s+([\d,.]+)?\s+([\d,.]+)?\s*([\d,.]+)?\s*$").map_err(|e| AppError::ParseError(format!("{}", e)))?;
        let mut txns = Vec::new();
        for line in raw.lines() {
            let l = line.trim();
            if l.is_empty() || l.len() < 10 { continue; }
            let u = l.to_uppercase();
            if u.contains("DATE") && (u.contains("DESCRIPTION") || u.contains("NARRATION")) { continue; }
            if u.contains("OPENING BALANCE") || u.contains("CLOSING BALANCE") || u.contains("STATEMENT") { continue; }
            if let Some(c) = re.captures(l) {
                if let Some(d) = parse_date(c.get(1).unwrap().as_str()) {
                    let (db, cr, bal) = match (c.get(3).and_then(|m|parse_amount(m.as_str())), c.get(4).and_then(|m|parse_amount(m.as_str())), c.get(5).and_then(|m|parse_amount(m.as_str()))) {
                        (a, b, c) => (a, b, c),
                    };
                    txns.push(RawTransaction { date: d, description: c.get(2).unwrap().as_str().trim().into(), reference_number: None, debit_amount: db, credit_amount: cr, balance: bal });
                }
            }
        }
        if txns.is_empty() { return Err(AppError::ParseError("No transactions parsed".into())); }
        Ok(ParsedStatement { bank_name: "Unknown".into(), account_number: None, period_start: txns.first().map(|t|t.date), period_end: txns.last().map(|t|t.date), transactions: txns })
    }
    fn bank_name(&self) -> &str { "Generic" }
}

pub fn parse_from_records(records: Vec<Vec<String>>, source: &str) -> Result<ParsedStatement, AppError> {
    if records.is_empty() { return Err(AppError::ParseError("No records".into())); }
    let h = &records[0];
    let mut dc=None; let mut desc=None; let mut dbc=None; let mut crc=None; let mut bc=None;
    for (i, col) in h.iter().enumerate() {
        let u = col.to_uppercase();
        if u.contains("DATE") && !u.contains("VALUE") && dc.is_none() { dc = Some(i); }
        else if (u.contains("DESCRIPTION") || u.contains("NARRATION")) && desc.is_none() { desc = Some(i); }
        else if (u.contains("DEBIT") || u.contains("WITHDRAWAL")) && dbc.is_none() { dbc = Some(i); }
        else if (u.contains("CREDIT") || u.contains("DEPOSIT")) && crc.is_none() { crc = Some(i); }
        else if u.contains("BALANCE") && bc.is_none() { bc = Some(i); }
    }
    let dc = dc.ok_or_else(|| AppError::ParseError("No Date column".into()))?;
    let mut txns = Vec::new();
    for row in records.iter().skip(1) {
        if row.len() <= dc { continue; }
        let Some(d) = parse_date(&row[dc]) else { continue };
        txns.push(RawTransaction { date: d, description: desc.and_then(|i| row.get(i)).cloned().unwrap_or_default(), reference_number: None,
            debit_amount: dbc.and_then(|i| row.get(i)).and_then(|s| parse_amount(s)), credit_amount: crc.and_then(|i| row.get(i)).and_then(|s| parse_amount(s)), balance: bc.and_then(|i| row.get(i)).and_then(|s| parse_amount(s)) });
    }
    if txns.is_empty() { return Err(AppError::ParseError(format!("No transactions in {} data", source))); }
    Ok(ParsedStatement { bank_name: source.into(), account_number: None, period_start: txns.first().map(|t|t.date), period_end: txns.last().map(|t|t.date), transactions: txns })
}
