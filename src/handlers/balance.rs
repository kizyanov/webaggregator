use crate::models::Balance;
use crate::templates::BalanceTemplate;
use actix_web::{HttpResponse, Result, web};
use askama::Template;

use sqlx::PgPool;
use std::time::Instant;
pub async fn balances(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // balances

    // time start
    let start = Instant::now();

    let balances = sqlx::query_as::<_, Balance>("SELECT exchange, account_id, available, available_change, currency, hold_value, hold_change, relation_event, relation_event_id, event_time, total, symbol, order_id, trade_id, updated_at FROM balance ORDER BY updated_at DESC LIMIT 1000;")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let template = BalanceTemplate {
        balances: balances,
        elapsed_ms: start.elapsed().as_millis(),
    };
    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Error template render")),
    }
}
