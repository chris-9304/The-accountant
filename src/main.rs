use the_accountant::{
    config::AppConfig,
    app_state::AppState,
    api::router::create_router,
    scheduler,
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env()?;
    tracing::info!("Starting The Accountant on port {}", config.app_port);

    std::fs::create_dir_all(&config.upload_dir)?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    tracing::info!("Database migrations complete");

    let scheduler_pool = pool.clone();
    tokio::spawn(async move {
        if let Err(e) = scheduler::start_scheduler(scheduler_pool).await {
            tracing::error!("Scheduler error: {:?}", e);
        }
    });

    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
