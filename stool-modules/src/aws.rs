//! AWS CLI wrapper module.
//!
//! Provides simplified access to AWS CLI commands,
//! including aws configure and ECR login.

use std::process::Command;
use stool_core::config::EcrRegistry;
use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::interactive;

/// Run aws configure interactively.
///
/// Executes `aws configure` command and allows user to input
/// AWS credentials and configuration manually.
/// Pressing Enter without input keeps existing values.
pub fn configure() -> Result<()> {
    // Check if AWS CLI is installed
    check_aws_cli()?;

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

/// Login to AWS ECR registry.
///
/// Executes ECR login command:
/// `aws ecr get-login-password --region {region} | docker login --username AWS --password-stdin {account_id}.dkr.ecr.{region}.amazonaws.com`
///
/// Provides interactive menu to select from configured registries or manual input.
///
/// # Arguments
/// * `registries` - List of ECR registries from configuration
///
/// # Errors
/// Returns error if AWS CLI or Docker is not installed, or login fails
pub fn ecr_login(registries: &[EcrRegistry]) -> Result<()> {
    // Check if AWS CLI is installed
    check_aws_cli()?;

    // Check if Docker is installed
    check_docker()?;

    // Select registry or manual input
    let registry_info = select_ecr_registry(registries)?;

    let (account_id, region) = match registry_info {
        Some(info) => info,
        None => return Ok(()), // User cancelled
    };

    // Execute ECR login
    execute_ecr_login(&account_id, &region)?;

    println!(
        "Successfully logged in to ECR registry: {}.dkr.ecr.{}.amazonaws.com",
        account_id, region
    );
    Ok(())
}

/// Check if AWS CLI is installed.
fn check_aws_cli() -> Result<()> {
    let check = Command::new("which")
        .arg("aws")
        .output()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCliNotInstalled).with_source(e))?;

    if !check.status.success() {
        return Err(StoolError::new(StoolErrorType::AwsCliNotInstalled)
            .with_message("AWS CLI is not installed. Install it via: brew install awscli"));
    }

    Ok(())
}

/// Check if Docker is installed.
fn check_docker() -> Result<()> {
    let check = Command::new("which")
        .arg("docker")
        .output()
        .map_err(|e| StoolError::new(StoolErrorType::DockerNotInstalled).with_source(e))?;

    if !check.status.success() {
        return Err(StoolError::new(StoolErrorType::DockerNotInstalled)
            .with_message("Docker is not installed. Install it via: brew install docker"));
    }

    Ok(())
}

/// Select ECR registry from list or manual input.
///
/// Returns (account_id, region) or None if cancelled.
fn select_ecr_registry(registries: &[EcrRegistry]) -> Result<Option<(String, String)>> {
    let mut items: Vec<String> = registries
        .iter()
        .enumerate()
        .map(|(i, reg)| {
            format!(
                "{}. {} ({}.dkr.ecr.{}.amazonaws.com)",
                i + 1,
                reg.name,
                reg.account_id,
                reg.region
            )
        })
        .collect();

    items.push(format!("{}. Manual input", items.len() + 1));
    items.push(format!("{}. Cancel", items.len() + 1));

    let selection = interactive::select_from_list("Select ECR registry:", &items)?;

    if selection == items.len() - 1 {
        // Cancel
        return Ok(None);
    } else if selection == items.len() - 2 {
        // Manual input
        let account_id = interactive::input_text("AWS Account ID:")?;
        let region = interactive::input_text("AWS Region (e.g., ap-northeast-2):")?;
        return Ok(Some((account_id, region)));
    } else if selection < registries.len() {
        // Selected from list
        let reg = &registries[selection];
        return Ok(Some((reg.account_id.clone(), reg.region.clone())));
    }

    Err(StoolError::new(StoolErrorType::InvalidInput))
}

/// Execute ECR login command.
fn execute_ecr_login(account_id: &str, region: &str) -> Result<()> {
    let registry_url = format!("{}.dkr.ecr.{}.amazonaws.com", account_id, region);

    // Get ECR login password
    let password_output = Command::new("aws")
        .args(["ecr", "get-login-password", "--region", region])
        .output()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCommandFailed).with_source(e))?;

    if !password_output.status.success() {
        return Err(StoolError::new(StoolErrorType::AwsCommandFailed)
            .with_message("Failed to get ECR login password"));
    }

    let password = String::from_utf8_lossy(&password_output.stdout)
        .trim()
        .to_string();

    // Docker login
    let mut docker_login = Command::new("docker")
        .args([
            "login",
            "--username",
            "AWS",
            "--password-stdin",
            &registry_url,
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| StoolError::new(StoolErrorType::DockerCommandFailed).with_source(e))?;

    if let Some(mut stdin) = docker_login.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(password.as_bytes())
            .map_err(|e| StoolError::new(StoolErrorType::DockerCommandFailed).with_source(e))?;
    }

    let status = docker_login
        .wait()
        .map_err(|e| StoolError::new(StoolErrorType::DockerCommandFailed).with_source(e))?;

    if !status.success() {
        return Err(StoolError::new(StoolErrorType::DockerCommandFailed)
            .with_message("Docker login failed"));
    }

    Ok(())
}
