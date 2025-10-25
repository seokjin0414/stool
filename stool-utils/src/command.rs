use std::process::Command;
use stool_core::error::{Result, StoolError, StoolErrorType};

/// Execute SSH command with authentication
pub fn execute_ssh(
    user: &str,
    ip: &str,
    key_path: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    if let Some(key) = key_path {
        println!("Connecting with PEM key authentication");
        let status = Command::new("ssh")
            .arg("-i")
            .arg(key)
            .arg(format!("{}@{}", user, ip))
            .status()
            .map_err(|e| StoolError::new(StoolErrorType::SshConnectionFailed).with_source(e))?;

        if !status.success() {
            return Err(StoolError::new(StoolErrorType::SshConnectionFailed));
        }
    } else if let Some(pass) = password {
        println!("Connecting with password authentication");
        execute_expect_ssh(user, ip, pass)?;
    } else {
        println!("Connecting with default SSH authentication");
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

/// Execute SCP command with authentication
pub fn execute_scp(
    source: &str,
    destination: &str,
    key_path: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    if let Some(key) = key_path {
        println!("Transferring with PEM key authentication");
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
        println!("Transferring with password authentication");
        execute_expect_scp(source, destination, pass)?;
    } else {
        println!("Transferring with default SSH authentication");
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

fn execute_expect_ssh(user: &str, ip: &str, password: &str) -> Result<()> {
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
            pass = password
        ))
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::ExpectCommandFailed).with_source(e))?;

    if !status.success() {
        return Err(StoolError::new(StoolErrorType::SshConnectionFailed));
    }

    Ok(())
}

fn execute_expect_scp(source: &str, destination: &str, password: &str) -> Result<()> {
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
            pass = password
        ))
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::FileTransferFailed).with_source(e))?;

    if !status.success() {
        return Err(StoolError::new(StoolErrorType::FileTransferFailed));
    }

    Ok(())
}
