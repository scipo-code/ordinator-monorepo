pub mod v1;

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(ToSchema, Debug, Error)]
pub enum AppError
{
    Anyhow(String),
}

impl std::fmt::Display for AppError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let AppError::Anyhow(value) = self;
        write!(f, "{value}")
    }
}

impl IntoResponse for AppError
{
    fn into_response(self) -> axum::response::Response
    {
        match self {
            AppError::Anyhow(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error" : error.to_string()})),
            )
                .into_response(),
        }
    }
}
