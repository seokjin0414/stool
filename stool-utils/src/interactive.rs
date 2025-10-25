//! Interactive user input utilities.
//!
//! Provides functions for interactive CLI operations:
//! - Server selection menus
//! - Text input prompts
//! - List selection dialogs

use dialoguer::{theme::ColorfulTheme, Input, Select};
use stool_core::config::Server;
use stool_core::error::{Result, StoolError, StoolErrorType};

/// Menu option for manual server input.
pub const MENU_MANUAL_INPUT: &str = "Manual input";

/// Menu option for canceling operation.
pub const MENU_CANCEL: &str = "Cancel";

/// Server information tuple: (user, ip, key_path, password).
pub type ServerInfo = (String, String, Option<String>, Option<String>);

/// Displays an interactive selection menu.
///
/// # Arguments
/// * `prompt` - Message displayed above the menu
/// * `items` - List of options to choose from
///
/// # Returns
/// Index of the selected item
///
/// # Errors
/// Returns error if user interaction fails
pub fn select_from_list(prompt: &str, items: &[String]) -> Result<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))
}

/// Prompts user for text input.
///
/// # Arguments
/// * `prompt` - Message displayed before input field
///
/// # Returns
/// User-entered text string
///
/// # Errors
/// Returns error if user interaction fails
pub fn input_text(prompt: &str) -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))
}

/// Presents server selection menu with cancel option.
///
/// Allows user to:
/// - Select from configured servers
/// - Enter server details manually
/// - Cancel the operation
///
/// # Arguments
/// * `servers` - List of available servers from configuration
///
/// # Returns
/// - `Some(ServerInfo)` if server selected or manual input provided
/// - `None` if user cancelled
///
/// # Errors
/// Returns error if user interaction fails
pub fn select_server(servers: &[Server]) -> Result<Option<ServerInfo>> {
    let mut items: Vec<String> = servers
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {} ({}@{})", i + 1, s.name, s.user, s.ip))
        .collect();
    items.push(MENU_MANUAL_INPUT.to_string());
    items.push(MENU_CANCEL.to_string());

    let selection = select_from_list("Select server:", &items)?;

    if selection == items.len() - 1 {
        // Cancel selected
        return Ok(None);
    }

    let (user, ip, key_path, password) = if selection < servers.len() {
        let server = &servers[selection];
        println!("Selected server: {} ({})", server.name, server.ip);
        (
            server.user.clone(),
            server.ip.clone(),
            server.key_path.clone(),
            server.password.clone(),
        )
    } else {
        // Manual input
        let user_input = input_text("Enter username:")?;
        let ip_input = input_text("Enter IP address:")?;
        println!("Target: {}@{}", user_input, ip_input);
        (user_input, ip_input, None, None)
    };

    Ok(Some((user, ip, key_path, password)))
}
