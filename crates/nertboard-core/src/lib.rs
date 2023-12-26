use serde::{Deserialize, Serialize};

pub type Score = i32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreEntry {
    pub player: String,
    pub score: Score,
    pub extra_info: Option<String>,
}
