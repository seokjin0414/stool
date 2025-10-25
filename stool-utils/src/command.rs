use std::process::{Command, ExitStatus};
use stool_core::error::{Result, StoolError, StoolErrorType};

/// Check command exit status and return error if failed
pub fn check_status(status: ExitStatus, error_type: StoolErrorType) -> Result<()> {
    if !status.success() {
        return Err(StoolError::new(error_type));
    }
    Ok(())
}

/// Execute a command with arguments and check status
pub fn execute_command(program: &str, args: &[&str], error_type: StoolErrorType) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| StoolError::new(error_type).with_source(e))?;

    check_status(status, error_type)
}

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
            .map_err(|e| {
                StoolError::new(StoolErrorType::SshConnectionFailed)
                    .with_message(format!("Failed to execute ssh command to {}@{}", user, ip))
                    .with_source(e)
            })?;

        check_status(status, StoolErrorType::SshConnectionFailed)?;
    } else if let Some(pass) = password {
        println!("Connecting with password authentication");
        execute_expect_ssh(user, ip, pass)?;
    } else {
        println!("Connecting with default SSH authentication");
        let status = Command::new("ssh")
            .arg(format!("{}@{}", user, ip))
            .status()
            .map_err(|e| {
                StoolError::new(StoolErrorType::SshConnectionFailed)
                    .with_message(format!("Failed to execute ssh command to {}@{}", user, ip))
                    .with_source(e)
            })?;

        check_status(status, StoolErrorType::SshConnectionFailed)?;
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
            .map_err(|e| {
                StoolError::new(StoolErrorType::FileTransferFailed)
                    .with_message(format!(
                        "Failed to execute scp from {} to {}",
                        source, destination
                    ))
                    .with_source(e)
            })?;

        check_status(status, StoolErrorType::FileTransferFailed)?;
    } else if let Some(pass) = password {
        println!("Transferring with password authentication");
        execute_expect_scp(source, destination, pass)?;
    } else {
        println!("Transferring with default SSH authentication");
        let status = Command::new("scp")
            .arg(source)
            .arg(destination)
            .status()
            .map_err(|e| {
                StoolError::new(StoolErrorType::FileTransferFailed)
                    .with_message(format!(
                        "Failed to execute scp from {} to {}",
                        source, destination
                    ))
                    .with_source(e)
            })?;

        check_status(status, StoolErrorType::FileTransferFailed)?;
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
        .map_err(|e| {
            StoolError::new(StoolErrorType::ExpectCommandFailed)
                .with_message(format!(
                    "Failed to execute expect for ssh to {}@{}",
                    user, ip
                ))
                .with_source(e)
        })?;

    check_status(status, StoolErrorType::SshConnectionFailed)
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
        .map_err(|e| {
            StoolError::new(StoolErrorType::FileTransferFailed)
                .with_message(format!(
                    "Failed to execute expect for scp from {} to {}",
                    source, destination
                ))
                .with_source(e)
        })?;

    check_status(status, StoolErrorType::FileTransferFailed)
}
