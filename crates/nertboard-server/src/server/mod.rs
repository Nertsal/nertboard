use crate::{database::DatabasePool, prelude::*};

use axum::{extract::Path, routing::get, Extension, Router};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub async fn run(database_pool: DatabasePool) -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Starting the server on {}", listener.local_addr().unwrap());

    axum::serve(listener, app(database_pool)).await?;
    Ok(())
}

fn app(database_pool: DatabasePool) -> Router {
    Router::new()
        .route("/", get(get_root))
        .route("/board/:board_id", get(get_board))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(database_pool))
}

async fn get_root() -> &'static str {
    "Hello, world"
}

async fn get_board(Path(board_id): Path<Uuid>) {}
