use dialoguer::{theme::ColorfulTheme, Input, Select};
use stool_core::config::Server;
use stool_core::error::{Result, StoolError, StoolErrorType};

pub const MENU_MANUAL_INPUT: &str = "Manual input";
pub const MENU_CANCEL: &str = "Cancel";

/// Server info tuple: (user, ip, key_path, password)
pub type ServerInfo = (String, String, Option<String>, Option<String>);

pub fn select_from_list(prompt: &str, items: &[String]) -> Result<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))
}

pub fn input_text(prompt: &str) -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))
}

/// Select server from config list or manual input
/// Returns ServerInfo or None if cancelled
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
