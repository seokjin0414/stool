use dialoguer::{theme::ColorfulTheme, Select};
use stool_core::error::{Result, StoolError, StoolErrorType};

pub fn select_from_list(prompt: &str, items: &[String]) -> Result<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
        .map_err(|e| StoolError::new(StoolErrorType::InvalidInput).with_source(e))
}
