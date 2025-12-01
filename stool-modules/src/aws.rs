//! AWS CLI wrapper module.
//!
//! Provides simplified access to AWS CLI commands,
//! including aws configure and ECR login.

use std::process::Command;
use stool_core::config::{EcrRegistry, SsoConfig};
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

/// Configure AWS SSO interactively.
///
/// Selects SSO config, auto-fills text inputs, manual selection for account/role.
pub fn sso_configure(configs: &[SsoConfig]) -> Result<()> {
    check_aws_cli()?;

    let selected = select_sso_config(configs)?;

    let cfg = match selected {
        Some(c) => c,
        None => return Ok(()), // User cancelled
    };

    // expect script:
    // - Auto: SSO session name, start URL, region, scopes
    // - Wait/interact for browser auth, account/role selection
    // - Auto: Default region, output format, profile name
    let script = format!(
        r#"
        set timeout -1
        spawn aws configure sso
        expect "SSO session name"
        send "{sso_session_name}\r"
        expect "SSO start URL"
        send "{start_url}\r"
        expect "SSO region"
        send "{sso_region}\r"
        expect "SSO registration scopes"
        send "\r"
        interact -o "Default client Region" return
        send "{region}\r"
        expect "CLI default output format"
        send "{output_format}\r"
        expect "Profile name"
        send "{profile_name}\r"
        expect eof
        "#,
        sso_session_name = cfg.sso_session_name,
        start_url = cfg.start_url,
        sso_region = cfg.region,
        region = cfg.region,
        output_format = cfg.output_format,
        profile_name = cfg.profile_name
    );

    let status = Command::new("expect")
        .arg("-c")
        .arg(&script)
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCommandFailed).with_source(e))?;

    if !status.success() {
        return Err(StoolError::new(StoolErrorType::AwsCommandFailed)
            .with_message("aws configure sso failed"));
    }

    println!(
        "SSO profile '{}' configured successfully.",
        cfg.profile_name
    );
    Ok(())
}

/// Select SSO config from list or manual input.
fn select_sso_config(configs: &[SsoConfig]) -> Result<Option<SsoConfig>> {
    let mut items: Vec<String> = configs
        .iter()
        .enumerate()
        .map(|(i, cfg)| format!("{}. {}", i + 1, cfg.profile_name))
        .collect();

    items.push(format!("{}. Manual input", items.len() + 1));
    items.push(format!("{}. Cancel", items.len() + 1));

    let selection = interactive::select_from_list("Select SSO config:", &items)?;

    if selection == items.len() - 1 {
        return Ok(None);
    } else if selection == items.len() - 2 {
        let profile_name = interactive::input_text("Profile name:")?;
        let sso_session_name = interactive::input_text("SSO session name:")?;
        let start_url = interactive::input_text("SSO start URL:")?;
        let region = interactive::input_text("Region:")?;
        return Ok(Some(SsoConfig {
            profile_name,
            sso_session_name,
            start_url,
            region,
            output_format: "json".to_string(),
        }));
    } else if selection < configs.len() {
        return Ok(Some(configs[selection].clone()));
    }

    Err(StoolError::new(StoolErrorType::InvalidInput))
}

/// Login to AWS SSO or refresh token.
///
/// Executes `aws sso login` command to authenticate via browser.
/// Selects profile from YAML config (using name as profile) or manual input.
pub fn sso_login(configs: &[SsoConfig]) -> Result<()> {
    check_aws_cli()?;

    let profile_name = select_sso_profile(configs)?;

    let profile = match profile_name {
        Some(p) => p,
        None => return Ok(()), // User cancelled
    };

    let status = Command::new("aws")
        .args(["sso", "login", "--profile", &profile])
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCommandFailed).with_source(e))?;

    if !status.success() {
        return Err(
            StoolError::new(StoolErrorType::AwsCommandFailed).with_message("aws sso login failed")
        );
    }

    Ok(())
}

/// Select SSO profile from config or manual input.
fn select_sso_profile(configs: &[SsoConfig]) -> Result<Option<String>> {
    let mut items: Vec<String> = configs
        .iter()
        .enumerate()
        .map(|(i, cfg)| format!("{}. {}", i + 1, cfg.profile_name))
        .collect();

    items.push(format!("{}. Manual input", items.len() + 1));
    items.push(format!("{}. Cancel", items.len() + 1));

    let selection = interactive::select_from_list("Select SSO profile:", &items)?;

    if selection == items.len() - 1 {
        return Ok(None);
    } else if selection == items.len() - 2 {
        let profile = interactive::input_text("Profile name:")?;
        return Ok(Some(profile));
    } else if selection < configs.len() {
        return Ok(Some(configs[selection].profile_name.clone()));
    }

    Err(StoolError::new(StoolErrorType::InvalidInput))
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
    use std::io::Write;
    use zeroize::Zeroize;

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

    let mut password = String::from_utf8_lossy(&password_output.stdout)
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
        .map_err(|e| {
            password.zeroize();
            StoolError::new(StoolErrorType::DockerCommandFailed).with_source(e)
        })?;

    if let Some(mut stdin) = docker_login.stdin.take() {
        let write_result = stdin
            .write_all(password.as_bytes())
            .map_err(|e| StoolError::new(StoolErrorType::DockerCommandFailed).with_source(e));
        password.zeroize();
        write_result?;
        drop(stdin);
    } else {
        password.zeroize();
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
