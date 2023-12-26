pub use nertboard_core::ScoreEntry;
use reqwest::Client;

pub struct Nertboard {
    url: String,
    api_key: Option<String>,
    client: Client,
}

impl Nertboard {
    pub fn new(url: String, api_key: Option<String>) -> Self {
        Self {
            url,
            api_key,
            client: Client::new(),
        }
    }

    pub async fn fetch_scores(&self) -> reqwest::Result<Vec<ScoreEntry>> {
        let mut req = self.client.get(&self.url);
        if let Some(key) = &self.api_key {
            req = req.header("api_key", key);
        }

        let response = req.send().await?;
        response.json().await
    }

    pub async fn submit_score(&self, entry: &ScoreEntry) -> reqwest::Result<Vec<ScoreEntry>> {
        let mut req = self.client.post(&self.url);
        if let Some(key) = &self.api_key {
            req = req.header("api_key", key);
        }

        let req = req.json(entry);

        let response = req.send().await?;
        response.json().await
    }
}
