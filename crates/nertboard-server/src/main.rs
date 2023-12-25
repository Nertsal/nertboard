mod api_key;
mod database;

use database::DatabasePool;

use axum::{extract::Path, routing::get, Extension, Router};
use color_eyre::{eyre::Context, Result};
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    setup()?;

    let database_url =
        dotenv::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");
    let database_pool = connect_database(&database_url)
        .await
        .context(format!("when connecting to the database: {}", database_url))?;

    run(database_pool).await.context("server error")
}

fn setup() -> Result<()> {
    // Panic handler
    color_eyre::install()?;

    // Setup logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(std::io::stdout),
        )
        .init();

    // Load .env
    dotenv::dotenv().ok(); // Error if file does not exist: ignore

    // SQL drivers
    sqlx::any::install_default_drivers();

    Ok(())
}

async fn connect_database(url: &str) -> Result<DatabasePool> {
    tracing::info!("Connecting to database {}", url);
    let pool = DatabasePool::connect(url).await?;
    Ok(pool)
}

async fn run(database_pool: DatabasePool) -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Starting the server on {}", listener.local_addr().unwrap());

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
