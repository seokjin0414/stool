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

    // Write SSO profile directly to ~/.aws/config
    // This bypasses AWS CLI's interactive UI which requires CPR support.
    write_sso_config(&cfg)?;

    println!("SSO profile '{}' configured.", cfg.profile_name);

    // Run sso login
    let status = Command::new("aws")
        .args(["sso", "login", "--profile", &cfg.profile_name])
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCommandFailed).with_source(e))?;

    if !status.success() {
        return Err(
            StoolError::new(StoolErrorType::AwsCommandFailed).with_message("aws sso login failed")
        );
    }

    Ok(())
}

/// Write SSO profile to ~/.aws/config.
///
/// Creates or updates the AWS config file with SSO profile and session.
fn write_sso_config(cfg: &SsoConfig) -> Result<()> {
    use std::fs::{self, OpenOptions};
    use std::io::{Read, Write};
    use std::path::PathBuf;

    let home = std::env::var("HOME").map_err(|_| {
        StoolError::new(StoolErrorType::AwsCommandFailed).with_message("HOME not set")
    })?;
    let aws_dir = PathBuf::from(&home).join(".aws");
    let config_path = aws_dir.join("config");

    // Create ~/.aws directory if not exists
    if !aws_dir.exists() {
        fs::create_dir_all(&aws_dir).map_err(|e| {
            StoolError::new(StoolErrorType::AwsCommandFailed)
                .with_message("Failed to create ~/.aws directory")
                .with_source(e)
        })?;
    }

    // Read existing config
    let mut existing_content = String::new();
    if config_path.exists() {
        let mut file = fs::File::open(&config_path).map_err(|e| {
            StoolError::new(StoolErrorType::AwsCommandFailed)
                .with_message("Failed to read ~/.aws/config")
                .with_source(e)
        })?;
        file.read_to_string(&mut existing_content).map_err(|e| {
            StoolError::new(StoolErrorType::AwsCommandFailed)
                .with_message("Failed to read ~/.aws/config")
                .with_source(e)
        })?;
    }

    // Check if profile/session already exists
    let profile_header = format!("[profile {}]", cfg.profile_name);
    let session_header = format!("[sso-session {}]", cfg.sso_session_name);

    if existing_content.contains(&profile_header) {
        println!("Profile '{}' already exists. Skipping.", cfg.profile_name);
        return Ok(());
    }

    // Build new config sections
    let profile_section = format!(
        "\n{}\nsso_session = {}\nsso_account_id = {}\nsso_role_name = {}\nregion = {}\noutput = {}\n",
        profile_header,
        cfg.sso_session_name,
        cfg.sso_account_id,
        cfg.sso_role_name,
        cfg.region,
        cfg.output_format
    );

    let session_section = if !existing_content.contains(&session_header) {
        format!(
            "\n{}\nsso_start_url = {}\nsso_region = {}\nsso_registration_scopes = sso:account:access\n",
            session_header, cfg.start_url, cfg.region
        )
    } else {
        String::new()
    };

    // Append to config file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .map_err(|e| {
            StoolError::new(StoolErrorType::AwsCommandFailed)
                .with_message("Failed to open ~/.aws/config for writing")
                .with_source(e)
        })?;

    file.write_all(profile_section.as_bytes()).map_err(|e| {
        StoolError::new(StoolErrorType::AwsCommandFailed)
            .with_message("Failed to write profile to ~/.aws/config")
            .with_source(e)
    })?;

    if !session_section.is_empty() {
        file.write_all(session_section.as_bytes()).map_err(|e| {
            StoolError::new(StoolErrorType::AwsCommandFailed)
                .with_message("Failed to write session to ~/.aws/config")
                .with_source(e)
        })?;
    }

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
        let sso_account_id = interactive::input_text("AWS Account ID:")?;
        let sso_role_name = interactive::input_text("IAM Role name:")?;
        return Ok(Some(SsoConfig {
            profile_name,
            sso_session_name,
            start_url,
            region,
            sso_account_id,
            sso_role_name,
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
/// If sso_profile is set, checks SSO session and logs in if needed.
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

    let (account_id, region, sso_profile) = match registry_info {
        Some(info) => info,
        None => return Ok(()), // User cancelled
    };

    // If SSO profile is set, check and login if needed
    if let Some(ref profile) = sso_profile {
        ensure_sso_login(profile)?;
    }

    // Execute ECR login
    execute_ecr_login(&account_id, &region, sso_profile.as_deref())?;

    println!(
        "Successfully logged in to ECR registry: {}.dkr.ecr.{}.amazonaws.com",
        account_id, region
    );
    Ok(())
}

/// Check SSO session validity and login if expired.
fn ensure_sso_login(profile: &str) -> Result<()> {
    // Check if SSO session is valid
    let check = Command::new("aws")
        .args(["sts", "get-caller-identity", "--profile", profile])
        .output()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCommandFailed).with_source(e))?;

    if check.status.success() {
        return Ok(());
    }

    // SSO session expired, login
    println!("SSO session expired. Logging in...");
    let status = Command::new("aws")
        .args(["sso", "login", "--profile", profile])
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::AwsCommandFailed).with_source(e))?;

    if !status.success() {
        return Err(
            StoolError::new(StoolErrorType::AwsCommandFailed).with_message("SSO login failed")
        );
    }

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
/// Returns (account_id, region, sso_profile) or None if cancelled.
fn select_ecr_registry(
    registries: &[EcrRegistry],
) -> Result<Option<(String, String, Option<String>)>> {
    let mut items: Vec<String> = registries
        .iter()
        .enumerate()
        .map(|(i, reg)| {
            let sso_marker = if reg.sso_profile.is_some() {
                " [SSO]"
            } else {
                ""
            };
            format!(
                "{}. {} ({}.dkr.ecr.{}.amazonaws.com){}",
                i + 1,
                reg.name,
                reg.account_id,
                reg.region,
                sso_marker
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
        // Manual input (no SSO)
        let account_id = interactive::input_text("AWS Account ID:")?;
        let region = interactive::input_text("AWS Region (e.g., ap-northeast-2):")?;
        return Ok(Some((account_id, region, None)));
    } else if selection < registries.len() {
        // Selected from list
        let reg = &registries[selection];
        return Ok(Some((
            reg.account_id.clone(),
            reg.region.clone(),
            reg.sso_profile.clone(),
        )));
    }

    Err(StoolError::new(StoolErrorType::InvalidInput))
}

/// Execute ECR login command.
fn execute_ecr_login(account_id: &str, region: &str, profile: Option<&str>) -> Result<()> {
    use std::io::Write;
    use zeroize::Zeroize;

    let registry_url = format!("{}.dkr.ecr.{}.amazonaws.com", account_id, region);

    // Get ECR login password
    let mut cmd = Command::new("aws");
    cmd.args(["ecr", "get-login-password", "--region", region]);
    if let Some(p) = profile {
        cmd.args(["--profile", p]);
    }
    let password_output = cmd
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
