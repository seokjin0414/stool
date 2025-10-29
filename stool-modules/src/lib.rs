//! Feature modules for stool CLI operations.
//!
//! This crate contains all major functionality modules:
//! - SSH connection management
//! - System update operations (Homebrew, Rust)
//! - Filesystem operations (find, count)
//! - File transfer via SCP
//! - AWS CLI wrapper

pub mod aws;
pub mod filesystem;
pub mod ssh;
pub mod transfer;
pub mod update;
