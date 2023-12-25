mod init;

pub use self::init::init_database;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

pub type DatabasePool = sqlx::AnyPool; // TODO: behind a trait?

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;

pub type Id = i32;
pub type Score = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub player_id: Id,
    pub score: Score,
    pub extra_info: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("unathorized request")]
    Unathorized,
    #[error("unathorized request, not enough rights")]
    Forbidden,
    #[error("invalid board name: {0}")]
    InvalidBoardName(String),
    #[error("a board called {0} already exists")]
    BoardAlreadyExists(String),
    #[error("a board called {0} not found")]
    NoSuchBoard(String),
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
}

impl RequestError {
    fn status(&self) -> StatusCode {
        match self {
            RequestError::Unathorized => StatusCode::UNAUTHORIZED,
            RequestError::Forbidden => StatusCode::FORBIDDEN,
            RequestError::InvalidBoardName(_) => StatusCode::BAD_REQUEST,
            RequestError::BoardAlreadyExists(_) => StatusCode::CONFLICT,
            RequestError::NoSuchBoard(_) => StatusCode::NOT_FOUND,
            RequestError::Sql(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        let body = format!("{}", self);
        (self.status(), body).into_response()
    }
}
