//! File transfer module.
//!
//! Handles SCP-based file transfers between local and remote systems:
//! - Upload files to remote servers
//! - Download files from remote servers
//! - Supports multiple authentication methods (key, password, default)

use stool_core::config::Server;
use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::{command, interactive};

/// Transfer mode selection.
#[derive(Debug)]
pub enum TransferMode {
    Upload,
    Download,
}

/// Initiates file transfer between local and remote systems.
///
/// Presents interactive menus for:
/// 1. Transfer mode selection (upload/download)
/// 2. Server selection from config or manual input
/// 3. File path inputs
///
/// Uses SCP with authentication based on server configuration.
///
/// # Arguments
/// * `servers` - List of available servers from configuration
///
/// # Errors
/// Returns error if transfer fails or user input is invalid
pub fn transfer(servers: &[Server]) -> Result<()> {
    // Select transfer mode
    let mode_items: Vec<String> = vec![
        "1. Upload (local -> remote)".to_string(),
        "2. Download (remote -> local)".to_string(),
        "3. Cancel".to_string(),
    ];
    let mode_selection = interactive::select_from_list("Transfer mode:", &mode_items)?;
    let mode = match mode_selection {
        0 => TransferMode::Upload,
        1 => TransferMode::Download,
        2 => return Ok(()),
        _ => return Err(StoolError::new(StoolErrorType::InvalidInput)),
    };

    // Select server or manual input
    let server_info = interactive::select_server(servers)?;

    let (user, ip, key_path, password) = match server_info {
        Some(info) => info,
        None => return Ok(()), // User cancelled
    };

    match mode {
        TransferMode::Upload => {
            execute_upload(&user, &ip, key_path.as_deref(), password.as_deref())?
        }
        TransferMode::Download => {
            execute_download(&user, &ip, key_path.as_deref(), password.as_deref())?
        }
    }

    Ok(())
}

fn execute_upload(
    user: &str,
    ip: &str,
    key_path: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    let local_path = interactive::input_text("Local file path:")?;
    let remote_path_input = interactive::input_text("Remote path (default: ~/): ")?;
    let remote_path = if remote_path_input.trim().is_empty() {
        "~/".to_string()
    } else {
        remote_path_input
    };

    command::execute_scp(
        &local_path,
        &format!("{}@{}:{}", user, ip, remote_path),
        key_path,
        password,
    )
}

fn execute_download(
    user: &str,
    ip: &str,
    key_path: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    let remote_path = interactive::input_text("Remote file path:")?;
    let local_path_input = interactive::input_text("Local path (default: ~/Downloads/): ")?;
    let local_path = if local_path_input.trim().is_empty() {
        "~/Downloads/".to_string()
    } else {
        local_path_input
    };

    command::execute_scp(
        &format!("{}@{}:{}", user, ip, remote_path),
        &local_path,
        key_path,
        password,
    )
}
