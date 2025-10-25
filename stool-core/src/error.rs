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
            Self::SshConnectionFailed => write!(f, "SSH 접속 실패"),
            Self::SshAuthenticationFailed => write!(f, "SSH 인증 실패"),
            Self::ServerNotFound => write!(f, "서버를 찾을 수 없음"),
            Self::ExpectCommandFailed => write!(f, "expect 명령어 실행 실패"),

            Self::FileNotFound => write!(f, "파일을 찾을 수 없음"),
            Self::SearchPatternInvalid => write!(f, "잘못된 검색 패턴"),

            Self::FileTransferFailed => write!(f, "파일 전송 실패"),
            Self::ScpCommandFailed => write!(f, "scp 명령어 실행 실패"),
            Self::SftpCommandFailed => write!(f, "sftp 명령어 실행 실패"),
            Self::SourceFileNotFound => write!(f, "원본 파일을 찾을 수 없음"),

            Self::ConfigLoadFailed => write!(f, "설정 파일 로드 실패"),
            Self::ConfigParseError => write!(f, "설정 파일 파싱 오류"),
            Self::YamlParseError => write!(f, "YAML 파싱 오류"),

            Self::BrewUpdateFailed => write!(f, "brew 업데이트 실패"),
            Self::RustupUpdateFailed => write!(f, "rustup 업데이트 실패"),

            Self::DockerCommandFailed => write!(f, "도커 명령어 실행 실패"),
            Self::DockerNotInstalled => write!(f, "도커가 설치되어 있지 않음"),

            Self::AwsCommandFailed => write!(f, "AWS 명령어 실행 실패"),
            Self::AwsCliNotInstalled => write!(f, "AWS CLI가 설치되어 있지 않음"),

            Self::CommandExecutionFailed => write!(f, "명령어 실행 실패"),
            Self::InvalidInput => write!(f, "잘못된 입력"),
            Self::PermissionDenied => write!(f, "권한 거부"),
            Self::IoError => write!(f, "I/O 오류"),
            Self::Cancelled => write!(f, "작업 취소됨"),
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
