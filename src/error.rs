use std::fmt::{Display, Formatter};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::models::ErrorResponse;

#[derive(Debug)]
pub enum Error {
    UserAlreadyExists,
    InvalidToken,
    RedisError(redis::RedisError),
    JsonError(serde_json::Error),
    JwtError(jsonwebtoken::errors::Error),
    ValidationError(validator::ValidationError),
    SqliteError(rusqlite::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Error::SqliteError(value)
    }
}

impl From<validator::ValidationError> for Error {
    fn from(value: validator::ValidationError) -> Self {
        Error::ValidationError(value)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Error::JwtError(value)
    }
}

impl From<redis::RedisError> for Error {
    fn from(value: redis::RedisError) -> Self {
        Error::RedisError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::JsonError(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Error::UserAlreadyExists => (StatusCode::BAD_REQUEST, "User already exists"),
            Error::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            Error::RedisError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Redis database failure"),
            Error::JsonError(_) => (StatusCode::BAD_REQUEST, "Invalid json data"),
            Error::JwtError(_) => (StatusCode::BAD_REQUEST, "Jwt Error"),
            Error::ValidationError(_) => (StatusCode::BAD_REQUEST, "Failed to validate request"),
            Error::SqliteError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Bad sql request"),
        };

        let response = ErrorResponse {
            response: status.as_u16(),
            error: self.to_string(),
            message: Some(message.to_string()),
        };

        (status, Json(response)).into_response()
    }
}
