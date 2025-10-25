use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::command;

pub fn update_brew() -> Result<()> {
    println!("Updating Homebrew");

    command::execute_command("brew", &["update"], StoolErrorType::BrewUpdateFailed)?;
    command::execute_command("brew", &["upgrade"], StoolErrorType::BrewUpdateFailed)?;

    println!("Homebrew updated successfully");
    Ok(())
}

pub fn update_rustup() -> Result<()> {
    println!("Updating Rust toolchain");

    command::execute_command("rustup", &["update"], StoolErrorType::RustupUpdateFailed)?;

    println!("Rust toolchain updated successfully");
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
