//! Feature modules for stool CLI operations.
//!
//! This crate contains all major functionality modules:
//! - SSH connection management
//! - System update operations (Homebrew, Rust)
//! - Filesystem operations (find, count)
//! - File transfer via SCP
//! - Docker operations (build, tag, push to ECR)
//! - AWS CLI wrapper

pub mod aws;
pub mod docker;
pub mod filesystem;
pub mod ssh;
pub mod transfer;
pub mod update;
