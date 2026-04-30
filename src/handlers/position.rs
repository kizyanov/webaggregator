use crate::models::{PositionAsset, PositionDebt, PositionRatio};
use crate::templates::{PositinRatioTemplate, PositionAssetTemplate, PositionDebtTemplate};
use actix_web::{HttpResponse, Result, web};
use askama::Template;

use sqlx::PgPool;
use std::time::Instant;
pub async fn positionasset(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // positionasset

    // time start
    let start = Instant::now();

    let position_asset = sqlx::query_as::<_, PositionAsset>(
        "SELECT exchange, asset_symbol, asset_total, asset_available, asset_hold, updated_at FROM positionasset ORDER BY updated_at DESC LIMIT 1000;",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let template = PositionAssetTemplate {
        position_asset: position_asset,
        elapsed_ms: start.elapsed().as_millis(),
    };
    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Error template render")),
    }
}
pub async fn positiondebt(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // positiondebt

    // time start
    let start = Instant::now();

    let position_debt = sqlx::query_as::<_, PositionDebt>(
        "SELECT exchange, debt_symbol, debt_value, updated_at FROM positiondebt ORDER BY updated_at DESC LIMIT 1000;",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let template = PositionDebtTemplate {
        position_debt: position_debt,
        elapsed_ms: start.elapsed().as_millis(),
    };
    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Error template render")),
    }
}
pub async fn positionratio(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // positionratio

    // time start
    let start = Instant::now();

    let position_ratio = sqlx::query_as::<_, PositionRatio>(
        "SELECT exchange, debt_ratio, total_asset, margin_coefficient_total_asset, total_debt, updated_at FROM positionratio ORDER BY updated_at DESC LIMIT 1000;",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let template = PositinRatioTemplate {
        position_ratio: position_ratio,
        elapsed_ms: start.elapsed().as_millis(),
    };
    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Error template render")),
    }
}
