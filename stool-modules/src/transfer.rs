use std::process::Command;
use stool_core::config::Server;
use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::interactive;

#[derive(Debug)]
pub enum TransferMode {
    Upload,
    Download,
}

pub fn transfer(servers: &[Server]) -> Result<()> {
    // Select transfer mode
    let mode_items: Vec<String> = vec![
        "Upload (local -> remote)".to_string(),
        "Download (remote -> local)".to_string(),
        "Cancel".to_string(),
    ];
    let mode_selection = interactive::select_from_list("Transfer mode:", &mode_items)?;
    let mode = match mode_selection {
        0 => TransferMode::Upload,
        1 => TransferMode::Download,
        2 => return Ok(()),
        _ => return Err(StoolError::new(StoolErrorType::InvalidInput)),
    };

    // Select server or manual input
    let mut server_items: Vec<String> = servers
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {} ({}@{})", i + 1, s.name, s.user, s.ip))
        .collect();
    server_items.push("Manual input".to_string());
    server_items.push("Cancel".to_string());

    let selection = interactive::select_from_list("Select server:", &server_items)?;

    if selection == server_items.len() - 1 {
        return Ok(());
    }

    let (user, ip, key_path, password) = if selection < servers.len() {
        let server = &servers[selection];
        (
            server.user.clone(),
            server.ip.clone(),
            server.key_path.clone(),
            server.password.clone(),
        )
    } else {
        // Manual input
        let user_input = interactive::input_text("Enter username:")?;
        let ip_input = interactive::input_text("Enter IP address:")?;
        (user_input, ip_input, None, None)
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

    execute_scp(
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

    execute_scp(
        &format!("{}@{}:{}", user, ip, remote_path),
        &local_path,
        key_path,
        password,
    )
}

fn execute_scp(
    source: &str,
    destination: &str,
    key_path: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    if let Some(key) = key_path {
        println!("Using key authentication");
        let status = Command::new("scp")
            .arg("-i")
            .arg(key)
            .arg(source)
            .arg(destination)
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::FileTransferFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::FileTransferFailed));
        }
    } else if let Some(pass) = password {
        println!("Using password authentication");
        let status = Command::new("expect")
            .arg("-c")
            .arg(format!(
                r#"
                spawn scp {source} {destination}
                expect {{
                    "yes/no" {{
                        send "yes\r"
                        exp_continue
                    }}
                    "password:" {{
                        send "{pass}\r"
                    }}
                }}
                expect eof
                "#,
                source = source,
                destination = destination,
                pass = pass
            ))
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::FileTransferFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::FileTransferFailed));
        }
    } else {
        println!("Using default authentication");
        let status = Command::new("scp")
            .arg(source)
            .arg(destination)
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::FileTransferFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::FileTransferFailed));
        }
    }

    println!("Transfer completed successfully");
    Ok(())
}
