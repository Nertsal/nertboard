use axum::{extract::Path, routing::get, Router};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(get_root))
        .route("/board/:board_id", get(get_board));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap()
}

async fn get_root() {
    println!("Hello, world");
}

async fn root() {
    println!("Hello, world");
}

async fn get_board(Path(board_id): Path<Uuid>) {}
