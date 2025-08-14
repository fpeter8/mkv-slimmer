use anyhow::{Context, Result};
use colored::*;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::{file_validation_error, config_error};
use crate::models::SonarrContext;
use crate::utils::{check_dependencies, collect_sonarr_environment};

use super::args::CliArgs;

#[derive(Debug, Clone, PartialEq)]
pub enum TargetType {
    File,
    Directory,
}

/// Processed CLI settings ready for main processing
#[derive(Debug, Clone)]
pub struct ProcessingSettings {
    pub input_path: PathBuf,
    pub target_path: PathBuf,
    pub target_type: TargetType,
    pub recursive: bool,
    pub filter_pattern: Option<String>,
    pub config: Config,
    pub sonarr_context: Option<SonarrContext>,
    pub input_is_file: bool,
    pub input_is_dir: bool,
}

/// Determine if target_path represents a file or directory
pub fn determine_target_type(target_path: &PathBuf) -> TargetType {
    if target_path.exists() {
        // If it exists, check what it actually is
        if target_path.is_file() {
            TargetType::File
        } else {
            TargetType::Directory
        }
    } else {
        // If it doesn't exist, infer from the path characteristics
        // Check if it has an extension (common indicator of a file)
        if target_path.extension().is_some() {
            TargetType::File
        } else if target_path.to_string_lossy().ends_with(std::path::MAIN_SEPARATOR) {
            // Ends with path separator, likely a directory
            TargetType::Directory
        } else {
            // Ambiguous case - treat as directory for backward compatibility
            TargetType::Directory
        }
    }
}

/// Parse CLI arguments, validate settings, and prepare configuration
/// Returns ProcessingSettings ready for main processing orchestration
pub async fn prepare_processing_settings() -> Result<ProcessingSettings> {
    let args = CliArgs::parse()?;

    // Check dependencies
    let missing_deps = check_dependencies()?;
    if !missing_deps.is_empty() {
        eprintln!("Warning: Missing optional dependencies: {}", missing_deps.join(", "));
        eprintln!("Some features may be limited. Install ffmpeg for full functionality.\n");
    }

    // Determine target type and validate combinations
    let target_type = determine_target_type(&args.target_path);
    let input_is_file = args.input_path.is_file();
    let input_is_dir = args.input_path.is_dir();

    // Validate input/output combinations
    match (input_is_file, input_is_dir, &target_type) {
        (true, false, TargetType::File) => {
            // File → File: Valid
            // Ensure target directory exists if target doesn't exist
            if !args.target_path.exists() {
                if let Some(parent) = args.target_path.parent() {
                    if !parent.exists() {
                        anyhow::bail!(
                            "Target directory does not exist: {}\nPlease create the directory first or specify a different target path.",
                            parent.display()
                        );
                    }
                }
            }
        }
        (true, false, TargetType::Directory) => {
            // File → Directory: Valid (original behavior)
        }
        (false, true, TargetType::Directory) => {
            // Directory → Directory: Valid
        }
        (false, true, TargetType::File) => {
            // Directory → File: Invalid
            anyhow::bail!(
                "Cannot process directory to single file.\nInput: {} (directory)\nTarget: {} (file)\n\nUse a target directory instead.",
                args.input_path.display(),
                args.target_path.display()
            );
        }
        (false, false, _) => {
            // Input doesn't exist
            return Err(file_validation_error(&args.input_path, "Input path does not exist. Check that the file or directory is accessible."));
        }
        (true, true, _) => {
            // This shouldn't happen - a path can't be both file and directory
            unreachable!("Path cannot be both file and directory");
        }
    }

    // Load configuration
    let mut config = Config::from_yaml(&args.config_path)
        .with_context(|| format!("Failed to load configuration from: {}", args.config_path.display()))?;
    
    // Merge CLI arguments with config
    config.merge_cli_args(args.audio_languages, args.subtitle_languages, args.dry_run)
        .context("Failed to merge CLI arguments with configuration")?;
    
    // Prompt for missing values if running interactively
    config.prompt_missing_values()
        .context("Failed to prompt for missing configuration values")?;

    // Collect Sonarr environment if available
    let sonarr_context = collect_sonarr_environment();
    let sonarr_context_opt = if sonarr_context.is_present() {
        Some(sonarr_context)
    } else {
        None
    };

    Ok(ProcessingSettings {
        input_path: args.input_path,
        target_path: args.target_path,
        target_type,
        recursive: args.recursive,
        filter_pattern: args.filter_pattern,
        config,
        sonarr_context: sonarr_context_opt,
        input_is_file,
        input_is_dir,
    })
}


pub fn print_configuration_info(config: &Config) {
    println!("\n⚙️  Configuration:");
    println!("🎵 Audio languages: {:?}", config.audio.keep_languages);
    println!("📄 Subtitle languages: {:?}", config.subtitles.keep_languages);
    if config.processing.dry_run {
        println!("🔍 Mode: Dry run (no files will be modified)");
    } else {
        println!("💾 Mode: Live processing");
    }
    println!(
        "ℹ️  Note: Video streams and attachments are always kept\n{}",
        "     Forced subtitles are not automatically preserved".dimmed()
    );
    println!();
}