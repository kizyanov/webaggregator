mod api {
    pub mod models;
    pub mod templates;
}
mod core {
    pub mod app_state;
    pub mod error;
}
mod config;
mod handlers;
mod repositories;
mod services;

use crate::config::AppConfig;
use crate::core::app_state::AppState;

use crate::handlers::{
    balance::balances,
    balance_clear::clear_balances,
    bots::bots,
    candle::{candle_series, candles, candles_meta, latest_candles},
    currency::currencies,
    error_clear::clear_errors,
    errors::errors,
    events::{events, msgevent, msgsend},
    index::index,
    orders::eventorders,
    pg::pg,
    position::{positionasset, positiondebt, positionratio},
    sendorder::sendorders,
    stoporders::stoporders,
    symbol::{symbols, tradeable},
    system::{favicon, serve_css},
    ticker::tickers,
    trade_history::trade_history,
};
use actix_web::{App, HttpServer, middleware, web};
use anyhow::{Context, Result};
use dotenvy::dotenv;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

fn init_tracing(config: &config::LoggingConfig) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.level));

    let builder = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(true);

    match config.format.as_str() {
        "json" => builder.json().init(),
        "text" => builder.init(),
        _ => builder.init(),
    }
}

async fn create_db_pool(config: &config::DatabaseConfig) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(&config.url)
        .await
        .context("Failed to connect to PostgreSQL")
}

fn routes(cfg: &mut web::ServiceConfig) {
    use web::{delete, get};
    cfg.route("/", get().to(index))
        .route("/pg", get().to(pg))
        .route("/events", get().to(events))
        .route("/errors", get().to(errors))
        .route("/errors/clear", delete().to(clear_errors))
        .route("/balance", get().to(balances))
        .route("/balance/clear", delete().to(clear_balances))
        .route("/eventorder", get().to(eventorders))
        .route("/sendorders", get().to(sendorders))
        .route("/stoporders", get().to(stoporders))
        .route("/trade_history/{symbol}", get().to(trade_history))
        .route("/positiondebt", get().to(positiondebt))
        .route("/msgevent", get().to(msgevent))
        .route("/msgsend", get().to(msgsend))
        .route("/positionasset", get().to(positionasset))
        .route("/positionratio", get().to(positionratio))
        .route("/tradeable", get().to(tradeable))
        .route("/tickers", get().to(tickers))
        .route("/currencies", get().to(currencies))
        .route("/symbols", get().to(symbols))
        .route("/bots", get().to(bots))
        .route("/api/candles", get().to(candles))
        .route("/api/candles/latest", get().to(latest_candles))
        .route("/api/candles/meta", get().to(candles_meta))
        .route("/api/candles/series/{symbol}", get().to(candle_series))
        .route("/static/style.css", get().to(serve_css))
        .route("/favicon.png", get().to(favicon));
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let config = AppConfig::from_env()?;

    init_tracing(&config.logging);

    let pool = create_db_pool(&config.database).await?;
    info!("Database connected");

    let app_state = AppState::new(pool);

    let server_addr = config.server_addr();
    let workers = config.server.workers;

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Compress::default())
            .configure(routes)
    })
    .bind(&server_addr)
    .with_context(|| format!("Failed to bind server to {}", server_addr))?
    .workers(workers);

    info!("Server running on http://{}", server_addr);
    info!("Workers: {}", workers);

    server.run().await.context("Server crashed")?;

    Ok(())
}
