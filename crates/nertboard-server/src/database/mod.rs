use axum::http::StatusCode;

pub type DatabasePool = sqlx::AnyPool;

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;

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
}

impl RequestError {
    fn status(&self) -> StatusCode {
        match self {
            RequestError::Unathorized => StatusCode::UNAUTHORIZED,
            RequestError::Forbidden => StatusCode::FORBIDDEN,
            RequestError::InvalidBoardName(_) => StatusCode::BAD_REQUEST,
            RequestError::BoardAlreadyExists(_) => StatusCode::CONFLICT,
            RequestError::NoSuchBoard(_) => StatusCode::NOT_FOUND,
        }
    }
}
