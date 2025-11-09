//! Error types and result handling for stool.
//!
//! This module provides a unified error handling system using [`StoolError`]
//! and [`StoolErrorType`] enum for all stool operations.

use std::fmt;

/// Error types for all stool operations.
///
/// Each variant represents a specific category of error that can occur
/// during stool execution. Implements Copy for easy error type reuse.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoolErrorType {
    // SSH related
    SshConnectionFailed,
    SshAuthenticationFailed,
    ServerNotFound,
    ExpectCommandFailed,

    // File search related
    FileNotFound,
    SearchPatternInvalid,

    // File transfer related
    FileTransferFailed,
    ScpCommandFailed,
    SftpCommandFailed,
    SourceFileNotFound,

    // Config related
    ConfigLoadFailed,
    ConfigParseError,
    YamlParseError,

    // Update related
    BrewUpdateFailed,
    RustupUpdateFailed,

    // Docker related
    DockerCommandFailed,
    DockerNotInstalled,

    // AWS related
    AwsCommandFailed,
    AwsCliNotInstalled,

    // General
    CommandExecutionFailed,
    InvalidInput,
    PermissionDenied,
    IoError,
    Cancelled,
}

impl fmt::Display for StoolErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SshConnectionFailed => write!(f, "SSH connection failed"),
            Self::SshAuthenticationFailed => write!(f, "SSH authentication failed"),
            Self::ServerNotFound => write!(f, "Server not found"),
            Self::ExpectCommandFailed => write!(f, "expect command failed"),

            Self::FileNotFound => write!(f, "File not found"),
            Self::SearchPatternInvalid => write!(f, "Invalid search pattern"),

            Self::FileTransferFailed => write!(f, "File transfer failed"),
            Self::ScpCommandFailed => write!(f, "scp command failed"),
            Self::SftpCommandFailed => write!(f, "sftp command failed"),
            Self::SourceFileNotFound => write!(f, "Source file not found"),

            Self::ConfigLoadFailed => write!(f, "Config load failed"),
            Self::ConfigParseError => write!(f, "Config parse error"),
            Self::YamlParseError => write!(f, "YAML parse error"),

            Self::BrewUpdateFailed => write!(f, "brew update failed"),
            Self::RustupUpdateFailed => write!(f, "rustup update failed"),

            Self::DockerCommandFailed => write!(f, "Docker command failed"),
            Self::DockerNotInstalled => write!(f, "Docker not installed"),

            Self::AwsCommandFailed => write!(f, "AWS command failed"),
            Self::AwsCliNotInstalled => write!(f, "AWS CLI not installed"),

            Self::CommandExecutionFailed => write!(f, "Command execution failed"),
            Self::InvalidInput => write!(f, "Invalid input"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::IoError => write!(f, "I/O error"),
            Self::Cancelled => write!(f, "Operation cancelled"),
        }
    }
}

/// Error type with additional context.
///
/// Wraps a [`StoolErrorType`] with optional message and source error
/// for detailed error reporting.
#[derive(Debug)]
pub struct StoolError {
    pub error_type: StoolErrorType,
    pub message: Option<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl StoolError {
    /// Creates a new error with the given type.
    pub fn new(error_type: StoolErrorType) -> Self {
        Self {
            error_type,
            message: None,
            source: None,
        }
    }

    /// Adds a descriptive message to the error.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Adds a source error for error chain tracking.
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

impl fmt::Display for StoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_type)?;
        if let Some(msg) = &self.message {
            write!(f, ": {}", msg)?;
        }
        Ok(())
    }
}

impl std::error::Error for StoolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for StoolError {
    fn from(err: std::io::Error) -> Self {
        StoolError::new(StoolErrorType::IoError).with_source(err)
    }
}

/// Result type alias for stool operations.
///
/// Uses [`StoolError`] as the error type for all stool functions.
pub type Result<T> = std::result::Result<T, StoolError>;
