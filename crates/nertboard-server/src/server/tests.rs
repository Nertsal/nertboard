use super::*;

use axum::{
    body::Body,
    http::{request::Builder, Request, Response, StatusCode},
};
use color_eyre::Result;
use http_body_util::BodyExt;
use serde::{de::DeserializeOwned, Serialize};
use tower::util::ServiceExt;

async fn test_database() -> Result<DatabasePool> {
    crate::setup::setup().context("when setting up the environment")?;

    let pool = sqlx::any::AnyPoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .context("when connecting to the in-memory database")?;

    crate::database::init_database(&pool)
        .await
        .context("when initializing the test database")?;

    Ok(pool)
}

async fn test_app() -> Result<Router> {
    Ok(app(Arc::new(
        test_database()
            .await
            .context("when setting up a test database")?,
    )))
}

fn request_json<T: Serialize>(request: Builder, body: &T) -> Result<Request<Body>> {
    let request = request
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(Body::from(
            serde_json::to_vec(body).context("when serializing request body as json")?,
        ))
        .context("when constructing a request")?;
    Ok(request)
}

async fn response_json<T: DeserializeOwned>(response: Response<Body>) -> Result<T> {
    let body = response
        .into_body()
        .collect()
        .await
        .context("when collecting response body")?
        .to_bytes();
    let body = serde_json::from_slice(&body).context("when decoding json")?;
    Ok(body)
}

#[tokio::test]
async fn test() -> Result<()> {
    let app = test_app().await?;

    let response = app
        .oneshot(request_json(Request::post("/board/create"), &"test-table")?)
        .await?;

    println!("{:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    let _: BoardKeys = response_json(response).await?;

    Ok(())
}
