//! Interactive user input utilities.
//!
//! Provides functions for interactive CLI operations:
//! - Server selection menus
//! - Text input prompts
//! - List selection dialogs
//! - File path input with tab completion

use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Config, Editor};
use rustyline::{Context, Helper};
use stool_core::config::Server;
use stool_core::error::{Result, StoolError, StoolErrorType};

/// Menu option for manual server input.
pub const MENU_MANUAL_INPUT: &str = "Manual input";

/// Menu option for canceling operation.
pub const MENU_CANCEL: &str = "Cancel";

/// Server information tuple: (user, ip, key_path, password).
pub type ServerInfo = (String, String, Option<String>, Option<String>);

/// Helper for rustyline with file path completion support.
struct PathHelper(FilenameCompleter);

impl Helper for PathHelper {}

impl Completer for PathHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.0.complete(line, pos, ctx)
    }
}

impl Hinter for PathHelper {
    type Hint = String;
}

impl Highlighter for PathHelper {}

impl Validator for PathHelper {}

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

/// Prompts user for text input with empty value allowed.
///
/// # Arguments
/// * `prompt` - Message displayed before input field
///
/// # Returns
/// User-entered text string (may be empty)
///
/// # Errors
/// Returns error if user interaction fails
pub fn input_text_optional(prompt: &str) -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))
}

/// Prompts user for password input with masked display.
///
/// # Arguments
/// * `prompt` - Message displayed before input field
///
/// # Returns
/// User-entered password string (may be empty)
///
/// # Errors
/// Returns error if user interaction fails
pub fn input_password(prompt: &str) -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .allow_empty_password(true)
        .interact()
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))
}

/// Prompts user for file path input with tab completion.
///
/// Provides interactive file path input with:
/// - Tab key for file/directory completion
/// - Support for relative and absolute paths
/// - Tilde expansion for home directory
///
/// # Arguments
/// * `prompt` - Message displayed before input field
///
/// # Returns
/// User-entered file path string
///
/// # Errors
/// Returns error if user interaction fails or input is cancelled
pub fn input_path(prompt: &str) -> Result<String> {
    let config = Config::builder().auto_add_history(true).build();
    let helper = PathHelper(FilenameCompleter::new());
    let mut editor = Editor::with_config(config)
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))?;
    editor.set_helper(Some(helper));

    let readline = editor.readline(&format!("{} ", prompt));
    match readline {
        Ok(line) => Ok(line.trim().to_string()),
        Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
            Err(StoolError::new(StoolErrorType::Cancelled))
        }
        Err(e) => Err(StoolError::new(StoolErrorType::InvalidInput).with_source(e)),
    }
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

    let (user, ip, key_path, mut password) = if selection < servers.len() {
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

    // Prompt for password if not in config and no key path
    if password.is_none() && key_path.is_none() {
        let pass = input_password("Enter password (leave empty for default SSH auth):")?;
        if !pass.is_empty() {
            password = Some(pass);
        }
    }

    Ok(Some((user, ip, key_path, password)))
}
