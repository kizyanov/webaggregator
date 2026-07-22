mod api {
    pub mod models;
    pub mod templates;
    pub mod tools;
}
mod handlers;
use crate::api::tools::get_env;
use crate::handlers::{
    balance::balances,
    bots::bots,
    currency::currencies,
    errors::errors,
    events::{events, msgevent, msgsend},
    index::index,
    orders::eventorders,
    pg::pg,
    position::{positionasset, positiondebt, positionratio},
    symbol::{symbols, tradeable},
    system::{favicon, serve_css},
    ticker::tickers,
};

use actix_web::{App, HttpServer, middleware, web};
use dotenvy::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::info;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .init();
}

async fn create_db_pool() -> Result<PgPool, String> {
    let database_url: String = get_env("DATABASE_URL")?;

    Ok(PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to PostgreSQL:{e}"))?)
}

fn routes(cfg: &mut web::ServiceConfig) {
    use web::get;
    cfg.route("/", get().to(index))
        .route("/pg", get().to(pg))
        .route("/events", get().to(events))
        .route("/errors", get().to(errors))
        .route("/balance", get().to(balances))
        .route("/eventorder", get().to(eventorders))
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
        .route("/static/style.css", get().to(serve_css))
        .route("/favicon.png", get().to(favicon));
}

const SERVER_ADDR: &str = "0.0.0.0:8080";

#[actix_web::main]
async fn main() -> Result<(), String> {
    init_tracing();
    dotenv().ok();

    let pool = create_db_pool().await?;
    info!("Database connected");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Compress::default())
            .configure(routes)
    })
    .bind(SERVER_ADDR)
    .map_err(|e| format!("Failed to bind server to {SERVER_ADDR}:{e}"))?;

    info!("Server running on http://0.0.0.0:8080");

    server
        .run()
        .await
        .map_err(|e| format!("Server crashed:{e}"))?;

    Ok(())
}
