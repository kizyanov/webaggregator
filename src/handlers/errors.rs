use crate::models::Error;
use crate::templates::ErrorsTemplate;
use actix_web::{HttpResponse, Result, web};
use askama::Template;

use sqlx::PgPool;
use std::time::Instant;
pub async fn errors(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // errors

    // time start
    let start = Instant::now();

    let errors = sqlx::query_as::<_, Error>(
        "SELECT exchange, msg, updated_at FROM errors ORDER BY updated_at DESC LIMIT 1000;",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let template = ErrorsTemplate {
        errors: errors,
        elapsed_ms: start.elapsed().as_millis(),
    };
    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Error template render")),
    }
}
