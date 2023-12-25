use crate::{
    api_key::{ApiKey, AuthorityLevel, BoardKeys, StringKey},
    database::{DatabasePool, Id, RequestError, RequestResult as Result, ScoreRecord},
    prelude::*,
};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use sqlx::{any::AnyRow, Row};
use tower_http::trace::TraceLayer;

pub async fn run(database_pool: DatabasePool) -> color_eyre::Result<()> {
    let addr = "0.0.0.0:3000";
    info!("Starting the server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("when binding a tcp listener")?;

    axum::serve(listener, app(Arc::new(database_pool))).await?;
    Ok(())
}

fn app(database_pool: Arc<DatabasePool>) -> Router {
    Router::new()
        .route("/", get(get_root))
        .route(
            "/board/:board_name",
            get(get_scores).post(submit_score).delete(delete_board),
        )
        .route("/board/create", post(create_board))
        .layer(TraceLayer::new_for_http())
        .with_state(database_pool)
}

async fn get_root() -> &'static str {
    "Hello, world"
}

/// Queries information about the board by name and returns its id
/// together with the authority level of the provided api key.
async fn check_board(
    Path(board_name): Path<String>,
    State(database): State<Arc<DatabasePool>>,
    api_key: Option<ApiKey>,
) -> Result<(Id, AuthorityLevel)> {
    let board_row = sqlx::query(
        "SELECT board_id, read_key, submit_key, admin_key FROM boards WHERE board_name = ?",
    )
    .bind(&board_name)
    .fetch_optional(&*database)
    .await?;

    let Some(row) = board_row else {
        return Err(RequestError::NoSuchBoard(board_name.clone()));
    };

    let board_id: i32 = row.try_get("board_id")?;
    let keys = BoardKeys {
        read: StringKey::new(row.try_get::<String, _>("read_key")?),
        submit: StringKey::new(row.try_get::<String, _>("submit_key")?),
        admin: StringKey::new(row.try_get::<String, _>("admin_key")?),
    };
    let authority = api_key.map_or(AuthorityLevel::Unauthorized, |key| {
        keys.check_authority(&key.0)
    });
    Ok((board_id, authority))
}

fn check_auth(auth: AuthorityLevel, required: AuthorityLevel) -> Result<()> {
    if let AuthorityLevel::Unauthorized = auth {
        Err(RequestError::Unathorized)
    } else if auth < required {
        Err(RequestError::Forbidden)
    } else {
        Ok(())
    }
}

fn validate_board_name(name: String) -> Result<String> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(RequestError::InvalidBoardName(name));
    }
    Ok(name)
}

async fn create_board(
    State(database): State<Arc<DatabasePool>>,
    Json(board_name): Json<String>,
) -> Result<Json<BoardKeys>> {
    // Validate the name
    let board_name = validate_board_name(board_name)?;

    // Check if a board with this name already exists
    let check = check_board(Path(board_name.clone()), State(database.clone()), None).await;
    if check.is_ok() {
        return Err(RequestError::BoardAlreadyExists(board_name));
    }

    // Generate keys
    let keys = BoardKeys::generate();

    // Create an entry
    sqlx::query(
        "
INSERT INTO boards (board_name, read_key, submit_key, admin_key)
VALUES (?, ?, ?, ?)
        ",
    )
    .bind(board_name)
    .bind(keys.read.inner())
    .bind(keys.submit.inner())
    .bind(keys.admin.inner())
    .execute(&*database)
    .await?;

    Ok(Json(keys))
}

async fn delete_board() -> Result<()> {
    Ok(())
}

async fn submit_score() {}

async fn get_scores(
    Path(board_name): Path<String>,
    State(database): State<Arc<DatabasePool>>,
    api_key: Option<ApiKey>,
) -> Result<Json<Vec<ScoreRecord>>> {
    let (board_id, auth) = check_board(Path(board_name), State(database.clone()), api_key).await?;
    check_auth(auth, AuthorityLevel::Read)?;

    // Fetch scores
    let scores = sqlx::query("SELECT player_id, score, extra_info FROM scores WHERE board_id = ?")
        .bind(board_id)
        .try_map(|row: AnyRow| {
            Ok(ScoreRecord {
                player_id: row.try_get("player_id")?,
                score: row.try_get("score")?,
                extra_info: row.try_get("extra_info").ok(),
            })
        })
        .fetch_all(&*database)
        .await?;

    Ok(Json(scores))
}
