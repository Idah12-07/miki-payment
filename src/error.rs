//! API error type and its HTTP mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    NotFound(String),

    #[error("internal server error")]
    Internal(String),

    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Internal(_) | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn client_message(&self) -> String {
        match self {
            // Never forward SQL or internal details to the client.
            Self::Internal(_) | Self::Database(_) => "internal server error".to_string(),
            other => other.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match &self {
            Self::Database(err) => tracing::error!(error = %err, "database query failed"),
            Self::Internal(detail) => tracing::error!(detail = %detail, "internal error"),
            _ => {}
        }

        let body = json!({
            "error": self.client_message(),
            "code": self.status().as_u16(),
        });

        (self.status(), Json(body)).into_response()
    }
}
