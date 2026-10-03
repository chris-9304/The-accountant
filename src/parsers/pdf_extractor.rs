use crate::errors::AppError;
use std::process::Command;

pub fn extract_text(file_path: &str, password: Option<&str>) -> Result<String, AppError> {
    let mut cmd = Command::new("pdftotext");
    cmd.arg("-layout");
    if let Some(pw) = password { cmd.arg("-upw").arg(pw); }
    cmd.arg(file_path).arg("-");
    let output = cmd.output().map_err(|e| AppError::ParseError(format!("pdftotext failed. Is poppler-utils installed? {}", e)))?;
    if !output.status.success() {
        let mut retry = Command::new("pdftotext");
        if let Some(pw) = password { retry.arg("-upw").arg(pw); }
        retry.arg(file_path).arg("-");
        let r2 = retry.output().map_err(|e| AppError::ParseError(format!("pdftotext retry failed: {}", e)))?;
        if !r2.status.success() { return Err(AppError::ParseError(format!("pdftotext failed: {}", String::from_utf8_lossy(&output.stderr)))); }
        return Ok(String::from_utf8_lossy(&r2.stdout).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
