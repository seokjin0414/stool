use std::process::Command;
use stool_core::config::Server;
use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::interactive;

pub fn connect(servers: &[Server]) -> Result<()> {
    let items: Vec<String> = servers
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {} ({}@{})", i + 1, s.name, s.user, s.ip))
        .collect();

    let selection = interactive::select_from_list("서버를 선택하세요", &items)?;
    let server = &servers[selection];

    println!("선택된 서버: {} ({})", server.name, server.ip);

    if let Some(key) = &server.key_path {
        println!("PEM key로 접속");
        let status = Command::new("ssh")
            .arg("-i")
            .arg(key)
            .arg(format!("{}@{}", server.user, server.ip))
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::SshConnectionFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::SshConnectionFailed));
        }
    } else if let Some(pass) = &server.password {
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
                user = server.user,
                ip = server.ip,
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
            .arg(format!("{}@{}", server.user, server.ip))
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::SshConnectionFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::SshConnectionFailed));
        }
    }

    Ok(())
}
