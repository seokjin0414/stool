use crate::error::{Result, StoolError, StoolErrorType};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub user: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub servers: Vec<Server>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            StoolError::new(StoolErrorType::ConfigLoadFailed)
                .with_message(format!("Failed to read config file: {}", path))
                .with_source(e)
        })?;

        serde_yaml::from_str(&content).map_err(|e| {
            StoolError::new(StoolErrorType::YamlParseError)
                .with_message("Failed to parse YAML config")
                .with_source(e)
        })
    }
}
