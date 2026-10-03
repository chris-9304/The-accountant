use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub app_port: u16,
    pub upload_dir: String,
    pub danger_zone_threshold: rust_decimal::Decimal,
    pub default_currency: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://accountant:accountant_dev@localhost:5432/the_accountant".to_string()),
            app_port: env::var("APP_PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080),
            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
            danger_zone_threshold: env::var("DANGER_ZONE_THRESHOLD")
                .unwrap_or_else(|_| "5000".to_string()).parse().unwrap_or_else(|_| rust_decimal::Decimal::new(5000, 0)),
            default_currency: env::var("DEFAULT_CURRENCY").unwrap_or_else(|_| "INR".to_string()),
        })
    }
}
