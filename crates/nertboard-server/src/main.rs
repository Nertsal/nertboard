mod api_key;
mod database;
mod prelude;
mod server;
mod setup;

use self::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    setup::setup()?;

    let database_url =
        dotenv::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");
    let database_pool = setup::connect_database(&database_url)
        .await
        .context(format!("when connecting to the database: {}", database_url))?;

    server::run(database_pool).await.context("server error")
}
