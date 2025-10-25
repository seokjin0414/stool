use std::process::Command;
use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::command;

pub fn update_brew() -> Result<()> {
    println!("Updating Homebrew...");

    let status = Command::new("brew")
        .arg("update")
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::BrewUpdateFailed).with_source(e))?;

    command::check_status(status, StoolErrorType::BrewUpdateFailed)?;

    let status = Command::new("brew")
        .arg("upgrade")
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::BrewUpdateFailed).with_source(e))?;

    command::check_status(status, StoolErrorType::BrewUpdateFailed)?;

    println!("Homebrew update completed");
    Ok(())
}

pub fn update_rustup() -> Result<()> {
    println!("Updating Rust toolchain...");

    let status = Command::new("rustup")
        .arg("update")
        .status()
        .map_err(|e| StoolError::new(StoolErrorType::RustupUpdateFailed).with_source(e))?;

    command::check_status(status, StoolErrorType::RustupUpdateFailed)?;

    println!("Rust toolchain update completed");
    Ok(())
}

pub fn update_all() -> Result<()> {
    let mut errors = Vec::new();

    if let Err(e) = update_brew() {
        eprintln!("Brew update failed: {}", e);
        errors.push("brew");
    }

    if let Err(e) = update_rustup() {
        eprintln!("Rustup update failed: {}", e);
        errors.push("rustup");
    }

    if errors.is_empty() {
        println!("\nAll updates completed successfully");
        Ok(())
    } else {
        Err(StoolError::new(StoolErrorType::CommandExecutionFailed)
            .with_message(format!("Failed updates: {}", errors.join(", "))))
    }
}
