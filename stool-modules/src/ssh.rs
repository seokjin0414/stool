use std::process::Command;
use stool_core::config::Server;
use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::interactive;

pub fn connect(servers: &[Server]) -> Result<()> {
    let mut items: Vec<String> = servers
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {} ({}@{})", i + 1, s.name, s.user, s.ip))
        .collect();
    items.push("Manual input".to_string());
    items.push("Cancel".to_string());

    let selection = interactive::select_from_list("Select server:", &items)?;

    if selection == items.len() - 1 {
        return Err(StoolError::new(StoolErrorType::Cancelled));
    }

    let (user, ip, key_path, password) = if selection < servers.len() {
        let server = &servers[selection];
        println!("선택된 서버: {} ({})", server.name, server.ip);
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
        println!("연결: {}@{}", user_input, ip_input);
        (user_input, ip_input, None, None)
    };

    if let Some(key) = &key_path {
        println!("PEM key로 접속");
        let status = Command::new("ssh")
            .arg("-i")
            .arg(key)
            .arg(format!("{}@{}", user, ip))
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::SshConnectionFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::SshConnectionFailed));
        }
    } else if let Some(pass) = &password {
        println!("PW init...");
        let status = Command::new("expect")
            .arg("-c")
            .arg(format!(
                r#"
                spawn ssh {user}@{ip}
                expect {{
                    "yes/no" {{
                        send "yes\r"
                        exp_continue
                    }}
                    "password:" {{
                        send "{pass}\r"
                    }}
                }}
                interact
                "#,
                user = user,
                ip = ip,
                pass = pass
            ))
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::ExpectCommandFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::SshConnectionFailed));
        }
    } else {
        println!("바로 접속");
        let status = Command::new("ssh")
            .arg(format!("{}@{}", user, ip))
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::SshConnectionFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::SshConnectionFailed));
        }
    }

    Ok(())
}
