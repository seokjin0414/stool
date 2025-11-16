//! Docker operations module.
//!
//! Handles Docker image building, tagging, and pushing to ECR:
//! - Build Docker images with standardized options
//! - Tag images for ECR with version management
//! - Push images to AWS ECR registries
//! - Version increment (major, middle, minor)

use std::process::Command;
use stool_core::config::EcrRegistry;
use stool_core::error::{Result, StoolError, StoolErrorType};
use stool_utils::interactive;

/// Default Docker build options for multi-platform support.
const DEFAULT_BUILD_OPTIONS: &[&str] = &[
    "--platform",
    "linux/arm64",
    "--provenance=false",
    "--sbom=false",
];

/// Default Docker image tag.
const DEFAULT_TAG: &str = "latest";

/// Version increment type.
#[derive(Debug)]
pub enum VersionIncrement {
    Latest,
    Major,
    Middle,
    Minor,
}

/// Semantic version representation.
#[derive(Debug, Clone)]
struct Version {
    major: u32,
    middle: u32,
    minor: u32,
}

impl Version {
    /// Parses version string in format "x.y.z".
    fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        Some(Version {
            major: parts[0].parse().ok()?,
            middle: parts[1].parse().ok()?,
            minor: parts[2].parse().ok()?,
        })
    }

    /// Increments version based on type.
    fn increment(&self, inc_type: VersionIncrement) -> String {
        match inc_type {
            VersionIncrement::Latest => DEFAULT_TAG.to_string(),
            VersionIncrement::Major => format!("{}.0.0", self.major + 1),
            VersionIncrement::Middle => format!("{}.{}.0", self.major, self.middle + 1),
            VersionIncrement::Minor => {
                format!("{}.{}.{}", self.major, self.middle, self.minor + 1)
            }
        }
    }
}

/// Selects or inputs image name from ECR registry configuration.
fn select_image_name(registry: &EcrRegistry) -> Result<String> {
    if registry.images.is_empty() {
        interactive::input_text("Image name:")
    } else {
        let mut image_items: Vec<String> = registry
            .images
            .iter()
            .enumerate()
            .map(|(i, img)| format!("{}. {}", i + 1, img))
            .collect();
        image_items.push(format!("{}. Manual input", image_items.len() + 1));

        let image_idx = interactive::select_from_list("Select image:", &image_items)?;

        if image_idx < registry.images.len() {
            Ok(registry.images[image_idx].clone())
        } else {
            interactive::input_text("Image name:")
        }
    }
}

/// Builds Docker image with default options.
fn build_image(image_name: &str) -> Result<()> {
    println!("Building Docker image: {}:latest", image_name);
    let image_tag = format!("{}:latest", image_name);
    let mut build_args = vec!["build"];
    build_args.extend(DEFAULT_BUILD_OPTIONS);
    build_args.push("-t");
    build_args.push(&image_tag);
    build_args.push(".");

    execute_docker(&build_args)?;
    println!("Build completed successfully");
    Ok(())
}

/// Builds Docker image only.
///
/// # Arguments
/// * `registries` - List of available ECR registries from configuration
///
/// # Errors
/// Returns error if Docker build fails or user input is invalid
pub fn build_only(registries: &[EcrRegistry]) -> Result<()> {
    if registries.is_empty() {
        return Err(StoolError::new(StoolErrorType::ConfigLoadFailed)
            .with_message("No ECR registries configured"));
    }

    // Select ECR registry
    let registry_items: Vec<String> = registries
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "{}. {} ({}.dkr.ecr.{}.amazonaws.com)",
                i + 1,
                r.name,
                r.account_id,
                r.region
            )
        })
        .collect();

    let registry_idx = interactive::select_from_list("Select ECR registry:", &registry_items)?;
    let registry = &registries[registry_idx];

    // Select or input image name
    let image_name = select_image_name(registry)?;

    // Build Docker image
    build_image(&image_name)
}

/// Builds, tags, and pushes Docker image to ECR.
///
/// # Arguments
/// * `registries` - List of available ECR registries from configuration
///
/// # Errors
/// Returns error if Docker commands fail or user input is invalid
pub fn push_to_ecr(registries: &[EcrRegistry]) -> Result<()> {
    if registries.is_empty() {
        return Err(StoolError::new(StoolErrorType::ConfigLoadFailed)
            .with_message("No ECR registries configured"));
    }

    // Select ECR registry
    let registry_items: Vec<String> = registries
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "{}. {} ({}.dkr.ecr.{}.amazonaws.com)",
                i + 1,
                r.name,
                r.account_id,
                r.region
            )
        })
        .collect();

    let registry_idx = interactive::select_from_list("Select ECR registry:", &registry_items)?;
    let registry = &registries[registry_idx];

    // Select or input image name
    let image_name = select_image_name(registry)?;

    // Build Docker image
    build_image(&image_name)?;

    // Get current version from ECR
    let current_version = get_latest_ecr_version(registry, &image_name)?;

    // Select version increment type
    let version_items = if let Some(ref ver) = current_version {
        let v = Version::parse(ver).unwrap_or(Version {
            major: 0,
            middle: 1,
            minor: 0,
        });
        vec![
            format!("1. latest"),
            format!("2. major ({})", v.increment(VersionIncrement::Major)),
            format!("3. middle ({})", v.increment(VersionIncrement::Middle)),
            format!("4. minor ({})", v.increment(VersionIncrement::Minor)),
        ]
    } else {
        vec![
            "1. latest".to_string(),
            "2. major (1.0.0)".to_string(),
            "3. middle (0.1.0)".to_string(),
            "4. minor (0.0.1)".to_string(),
        ]
    };

    let version_idx = interactive::select_from_list("Select version type:", &version_items)?;

    let new_version = if let Some(ref ver) = current_version {
        let v = Version::parse(ver).unwrap_or(Version {
            major: 0,
            middle: 1,
            minor: 0,
        });
        match version_idx {
            0 => v.increment(VersionIncrement::Latest),
            1 => v.increment(VersionIncrement::Major),
            2 => v.increment(VersionIncrement::Middle),
            3 => v.increment(VersionIncrement::Minor),
            _ => return Err(StoolError::new(StoolErrorType::InvalidInput)),
        }
    } else {
        match version_idx {
            0 => "latest".to_string(),
            1 => "1.0.0".to_string(),
            2 => "0.1.0".to_string(),
            3 => "0.0.1".to_string(),
            _ => return Err(StoolError::new(StoolErrorType::InvalidInput)),
        }
    };

    let ecr_url = format!(
        "{}.dkr.ecr.{}.amazonaws.com",
        registry.account_id, registry.region
    );

    // Tag for latest
    println!("Tagging image: {}:{}", image_name, DEFAULT_TAG);
    execute_docker(&[
        "tag",
        &format!("{}:latest", image_name),
        &format!("{}/{}:latest", ecr_url, image_name),
    ])?;

    // Tag for version
    if new_version != "latest" {
        println!("Tagging image: {}:{}", image_name, new_version);
        execute_docker(&[
            "tag",
            &format!("{}:latest", image_name),
            &format!("{}/{}:{}", ecr_url, image_name, new_version),
        ])?;
    }

    // Push latest
    println!("Pushing {}/{}:latest", ecr_url, image_name);
    execute_docker(&["push", &format!("{}/{}:latest", ecr_url, image_name)])?;

    // Push version
    if new_version != "latest" {
        println!("Pushing {}/{}:{}", ecr_url, image_name, new_version);
        execute_docker(&[
            "push",
            &format!("{}/{}:{}", ecr_url, image_name, new_version),
        ])?;
    }

    println!("Push completed successfully");
    Ok(())
}

/// Executes Docker command with error handling.
fn execute_docker(args: &[&str]) -> Result<()> {
    let status = Command::new("docker").args(args).status().map_err(|e| {
        StoolError::new(StoolErrorType::CommandExecutionFailed)
            .with_message(format!("Failed to execute docker {}", args.join(" ")))
            .with_source(e)
    })?;

    if !status.success() {
        return Err(StoolError::new(StoolErrorType::CommandExecutionFailed)
            .with_message(format!("Docker command failed: docker {}", args.join(" "))));
    }

    Ok(())
}

/// Retrieves latest version tag from ECR repository.
fn get_latest_ecr_version(registry: &EcrRegistry, image_name: &str) -> Result<Option<String>> {
    let output = Command::new("aws")
        .args([
            "ecr",
            "describe-images",
            "--repository-name",
            image_name,
            "--region",
            &registry.region,
            "--query",
            "sort_by(imageDetails,& imagePushedAt)[-1].imageTags[0]",
            "--output",
            "text",
        ])
        .output()
        .map_err(|e| {
            StoolError::new(StoolErrorType::CommandExecutionFailed)
                .with_message("Failed to execute aws ecr describe-images")
                .with_source(e)
        })?;

    if !output.status.success() {
        // Repository might not exist or be empty
        return Ok(None);
    }

    let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if version_str.is_empty() || version_str == "None" {
        Ok(None)
    } else {
        Ok(Some(version_str))
    }
}
