use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Template error: {0}")]
    Template(#[from] askama::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            AppError::Database(_) => {
                actix_web::HttpResponse::InternalServerError().body("Database error")
            }
            AppError::Template(_) => {
                actix_web::HttpResponse::InternalServerError().body("Template render error")
            }
            _ => actix_web::HttpResponse::InternalServerError().body("Internal server error"),
        }
    }
}
