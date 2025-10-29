//! AWS CLI wrapper module.
//!
//! Provides simplified access to AWS CLI commands,
//! starting with aws configure.

use std::process::Command;
use stool_core::error::{Result, StoolError, StoolErrorType};

/// Run aws configure interactively.
///
/// Executes `aws configure` command and allows user to input
/// AWS credentials and configuration manually.
/// Pressing Enter without input keeps existing values.
pub fn configure() -> Result<()> {
    // Check if AWS CLI is installed
    let check = Command::new("which")
        .arg("aws")
        .output()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCliNotInstalled).with_source(e))?;

    if !check.status.success() {
        return Err(StoolError::new(StoolErrorType::AwsCliNotInstalled)
            .with_message("AWS CLI is not installed. Install it via: brew install awscli"));
    }

    // Run aws configure interactively
    let status = Command::new("aws")
        .arg("configure")
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCommandFailed).with_source(e))?;

    if !status.success() {
        return Err(
            StoolError::new(StoolErrorType::AwsCommandFailed).with_message("aws configure failed")
        );
    }

    Ok(())
}
