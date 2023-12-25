use axum::http::{request::Parts, StatusCode};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StringKey(Box<str>);

#[derive(Serialize, Deserialize)]
pub struct BoardKeys {
    pub read: StringKey,
    pub submit: StringKey,
    pub admin: StringKey,
}

impl StringKey {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn generate(length: usize) -> Self {
        let rng = rand::thread_rng();
        let key: String = rng
            .sample_iter(rand::distributions::Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        Self(key.into())
    }
}

impl BoardKeys {
    pub fn generate() -> Self {
        Self {
            read: StringKey::generate(10),
            submit: StringKey::generate(10),
            admin: StringKey::generate(20),
        }
    }
}

pub struct ApiKey(pub String);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for ApiKey {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.get("api-key") {
            None => Err((StatusCode::BAD_REQUEST, "api key missing")),
            Some(key) => match key.to_str() {
                Ok(key) => Ok(Self(key.to_string())),
                Err(_) => Err((StatusCode::BAD_REQUEST, "api key is invalid")),
            },
        }
    }
}
