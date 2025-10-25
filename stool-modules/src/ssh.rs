//! SSH connection module.
//!
//! Handles SSH connections to servers with multiple authentication methods:
//! - PEM key authentication
//! - Password authentication (via expect)
//! - Default SSH key authentication

use stool_core::config::Server;
use stool_core::error::Result;
use stool_utils::{command, interactive};

/// Establishes SSH connection to a selected server.
///
/// Presents an interactive menu for server selection and handles
/// authentication using the configured method (key, password, or default).
///
/// # Arguments
/// * `servers` - List of available servers from configuration
///
/// # Errors
/// Returns error if connection fails or user input is invalid
pub fn connect(servers: &[Server]) -> Result<()> {
    let server_info = interactive::select_server(servers)?;

    let (user, ip, key_path, password) = match server_info {
        Some(info) => info,
        None => return Ok(()), // User cancelled
    };

    command::execute_ssh(&user, &ip, key_path.as_deref(), password.as_deref())
}
