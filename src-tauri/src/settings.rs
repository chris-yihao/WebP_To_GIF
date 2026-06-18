use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub output_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct SettingsStore {
    path: PathBuf,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("无法读写设置：{0}")]
    Io(#[from] std::io::Error),
    #[error("设置格式无效：{0}")]
    Json(#[from] serde_json::Error),
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load_or_default(&self, default_output_dir: PathBuf) -> Result<Settings, SettingsError> {
        if !self.path.exists() {
            return Ok(Settings {
                output_dir: default_output_dir,
            });
        }

        let content = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, serde_json::to_string_pretty(settings)?)?;
        Ok(())
    }
}
