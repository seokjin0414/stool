//! Command execution utilities.
//!
//! Provides helpers for executing external commands:
//! - SSH connection with multiple authentication methods
//! - SCP file transfer with authentication
//! - Generic command execution with status checking

use std::process::{Command, ExitStatus};
use stool_core::error::{Result, StoolError, StoolErrorType};

/// Checks command exit status and returns error if failed.
///
/// # Arguments
/// * `status` - Exit status from command execution
/// * `error_type` - Error type to return on failure
///
/// # Errors
/// Returns specified error type if status indicates failure
pub fn check_status(status: ExitStatus, error_type: StoolErrorType) -> Result<()> {
    if !status.success() {
        return Err(StoolError::new(error_type));
    }
    Ok(())
}

/// Executes a command with arguments and verifies success.
///
/// Runs command with inherited stdio for real-time output.
///
/// # Arguments
/// * `program` - Command name or path to execute
/// * `args` - Command-line arguments
/// * `error_type` - Error type to return on failure
///
/// # Errors
/// Returns specified error type if command fails to execute or exits with error
pub fn execute_command(program: &str, args: &[&str], error_type: StoolErrorType) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| StoolError::new(error_type).with_source(e))?;

    check_status(status, error_type)
}

/// Executes SSH connection with appropriate authentication.
///
/// Authentication priority:
/// 1. PEM key (`key_path`) - Uses `ssh -i`
/// 2. Password (`password`) - Uses expect script
/// 3. Default - Standard SSH connection
///
/// # Arguments
/// * `user` - Remote username
/// * `ip` - Remote server IP address
/// * `key_path` - Optional path to PEM key file
/// * `password` - Optional password for authentication
///
/// # Errors
/// Returns error if SSH connection fails or authentication is rejected
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

/// Executes SCP file transfer with appropriate authentication.
///
/// Authentication priority:
/// 1. PEM key (`key_path`) - Uses `scp -i`
/// 2. Password (`password`) - Uses expect script
/// 3. Default - Standard SCP connection
///
/// # Arguments
/// * `source` - Source file path (local or remote format: `user@ip:path`)
/// * `destination` - Destination path (local or remote format: `user@ip:path`)
/// * `key_path` - Optional path to PEM key file
/// * `password` - Optional password for authentication
///
/// # Errors
/// Returns error if file transfer fails or authentication is rejected
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
    use std::io::Write;
    use zeroize::Zeroize;

    let mut script = format!(
        "spawn ssh {}@{}\n\
expect {{\n\
\"yes/no\" {{\n\
send \"yes\\r\"\n\
exp_continue\n\
}}\n\
\"assword:\" {{\n\
send \"{}\\r\"\n\
}}\n\
}}\n\
interact\n",
        user, ip, password
    );

    let mut child = Command::new("expect")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            script.zeroize();
            StoolError::new(StoolErrorType::ExpectCommandFailed)
                .with_message(format!("Failed to spawn expect for ssh to {}@{}", user, ip))
                .with_source(e)
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        let write_result = stdin.write_all(script.as_bytes()).map_err(|e| {
            StoolError::new(StoolErrorType::ExpectCommandFailed)
                .with_message("Failed to write expect script to stdin")
                .with_source(e)
        });
        script.zeroize();
        write_result?;
    } else {
        script.zeroize();
    }

    let status = child.wait().map_err(|e| {
        StoolError::new(StoolErrorType::ExpectCommandFailed)
            .with_message(format!(
                "Failed to wait for expect process for ssh to {}@{}",
                user, ip
            ))
            .with_source(e)
    })?;

    check_status(status, StoolErrorType::SshConnectionFailed)
}

fn execute_expect_scp(source: &str, destination: &str, password: &str) -> Result<()> {
    use std::io::Write;
    use zeroize::Zeroize;

    let mut script = format!(
        "spawn scp {} {}\n\
expect {{\n\
\"yes/no\" {{\n\
send \"yes\\r\"\n\
exp_continue\n\
}}\n\
\"assword:\" {{\n\
send \"{}\\r\"\n\
}}\n\
}}\n\
expect eof\n",
        source, destination, password
    );

    let mut child = Command::new("expect")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            script.zeroize();
            StoolError::new(StoolErrorType::FileTransferFailed)
                .with_message(format!(
                    "Failed to spawn expect for scp from {} to {}",
                    source, destination
                ))
                .with_source(e)
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        let write_result = stdin.write_all(script.as_bytes()).map_err(|e| {
            StoolError::new(StoolErrorType::FileTransferFailed)
                .with_message("Failed to write expect script to stdin")
                .with_source(e)
        });
        script.zeroize();
        write_result?;
    } else {
        script.zeroize();
    }

    let status = child.wait().map_err(|e| {
        StoolError::new(StoolErrorType::FileTransferFailed)
            .with_message(format!(
                "Failed to wait for expect process for scp from {} to {}",
                source, destination
            ))
            .with_source(e)
    })?;

    check_status(status, StoolErrorType::FileTransferFailed)
}
