use super::*;

use axum::{
    body::Body,
    http::{request::Builder, Request, Response, StatusCode},
};
use color_eyre::Result;
use http_body_util::BodyExt;
use nertboard_core::Player;
use serde::{de::DeserializeOwned, Serialize};
use tower::{util::ServiceExt, Service};

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
async fn test_e2e() -> Result<()> {
    let mut app = test_app().await?.into_service();

    // Create board
    let response = app
        .ready()
        .await?
        .call(request_json(Request::post("/board/create"), &"test-table")?)
        .await?;

    println!("{:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    let keys: BoardKeys = response_json(response).await?;

    // Create player
    let response = app
        .ready()
        .await?
        .call(request_json(Request::post("/player/create"), &"nertsal")?)
        .await?;

    println!("{:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    let player: Player = response_json(response).await?;

    // Submit scores
    let scores = vec![
        nertboard_core::ScoreEntry {
            player: "nertsal".to_string(),
            score: 10,
            extra_info: None,
        },
        nertboard_core::ScoreEntry {
            player: "nert".to_string(), // Change name
            score: 5,
            extra_info: Some("very cool".to_string()),
        },
    ];

    // First score
    let response = app
        .ready()
        .await?
        .call(request_json(
            Request::post(format!("/board/test-table?player_id={}", player.id))
                .header("api-key", keys.submit.inner())
                .header("player-key", &player.key),
            &scores[0],
        )?)
        .await?;

    println!("{:?}", response);
    assert_eq!(response.status(), StatusCode::OK);

    // Second score
    let response = app
        .ready()
        .await?
        .call(request_json(
            Request::post(format!("/board/test-table?player_id={}", player.id))
                .header("api-key", keys.submit.inner())
                .header("player-key", &player.key),
            &scores[1],
        )?)
        .await?;

    println!("{:?}", response);
    assert_eq!(response.status(), StatusCode::OK);

    // Retrieve scores
    let response = app
        .ready()
        .await?
        .call(
            Request::get("/board/test-table")
                .header("api-key", keys.read.inner())
                .body(Body::empty())?,
        )
        .await?;

    println!("{:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    let returned_scores: Vec<nertboard_core::ScoreEntry> = response_json(response).await?;
    // Update name
    let new_scores: Vec<_> = scores
        .into_iter()
        .map(|entry| nertboard_core::ScoreEntry {
            player: "nert".to_string(),
            ..entry
        })
        .collect();
    assert_eq!(returned_scores, new_scores);

    Ok(())
}
