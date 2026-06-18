use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub file_path: String,
    pub output_path: Option<String>,
    pub status: String,
    pub message: String,
    pub completed_at: String,
}

#[derive(Clone, Debug)]
pub struct HistoryStore {
    path: PathBuf,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("无法读写历史记录：{0}")]
    Io(#[from] std::io::Error),
    #[error("历史记录格式无效：{0}")]
    Json(#[from] serde_json::Error),
}

impl HistoryStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Vec<HistoryEntry>, HistoryError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, entries: &[HistoryEntry]) -> Result<(), HistoryError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(entries)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}
