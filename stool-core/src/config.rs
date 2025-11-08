//! Configuration management for server connections.
//!
//! This module handles loading and parsing YAML configuration files
//! containing server connection details (SSH, SCP).

use crate::error::{Result, StoolError, StoolErrorType};
use serde::{Deserialize, Serialize};
use std::fs;

/// Server connection configuration.
///
/// Represents a single server with authentication details.
/// Supports multiple authentication methods via optional fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub user: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

/// ECR registry configuration.
///
/// Represents AWS ECR registry with account and region details.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EcrRegistry {
    pub name: String,
    pub account_id: String,
    pub region: String,
}

/// Configuration container for server list and ECR registries.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub servers: Vec<Server>,
    #[serde(default)]
    pub ecr_registries: Vec<EcrRegistry>,
}

impl Config {
    /// Loads configuration from an external YAML file.
    ///
    /// # Arguments
    /// * `path` - Path to the YAML configuration file
    ///
    /// # Errors
    /// Returns error if file cannot be read or parsed as YAML
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

    /// Loads configuration embedded at build time.
    ///
    /// Uses config.yaml from project root, embedded via `include_str!`.
    ///
    /// # Errors
    /// Returns error if embedded YAML cannot be parsed
    pub fn load_embedded() -> Result<Self> {
        let content = include_str!("../../config.yaml");
        serde_yaml::from_str(content).map_err(|e| {
            StoolError::new(StoolErrorType::YamlParseError)
                .with_message("Failed to parse embedded YAML config")
                .with_source(e)
        })
    }
}
