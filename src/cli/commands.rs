use anyhow::{Context, Result};
use colored::*;
use std::path::PathBuf;

use crate::config::Config;
use crate::core::{BatchProcessor, analyze_and_process_mkv_file};
use crate::utils::{check_dependencies, validate_mkv_file, validate_source_target_paths, collect_sonarr_environment};

use super::args::CliArgs;

#[derive(Debug, Clone, PartialEq)]
pub enum TargetType {
    File,
    Directory,
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

pub async fn run_cli() -> Result<()> {
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
            anyhow::bail!("Input path does not exist: {}", args.input_path.display());
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
    config.merge_cli_args(args.audio_languages, args.subtitle_languages, args.dry_run);
    
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

    if input_is_file {
        // Process single file
        process_single_file(&args.input_path, &args.target_path, &target_type, config, sonarr_context_opt).await?;
    } else {
        // Process directory
        process_directory(&args.input_path, &args.target_path, args.recursive, args.filter_pattern, config, sonarr_context_opt).await?;
    }

    Ok(())
}

async fn process_single_file(
    mkv_file: &PathBuf,
    target_path: &PathBuf,
    target_type: &TargetType,
    config: Config,
    sonarr_context: Option<crate::models::SonarrContext>,
) -> Result<()> {
    // Validate MKV file
    validate_mkv_file(mkv_file)
        .with_context(|| format!("Invalid MKV file: {}", mkv_file.display()))?;
    
    // Handle different target types
    let (target_directory, output_filename) = match target_type {
        TargetType::File => {
            // File → File: use parent directory and extract filename
            let parent_dir = target_path.parent()
                .context("Could not determine parent directory from target file path")?;
            let filename = target_path.file_name()
                .context("Could not extract filename from target path")?
                .to_string_lossy()
                .to_string();
            (parent_dir, Some(filename))
        }
        TargetType::Directory => {
            // File → Directory: current behavior
            (target_path.as_path(), None)
        }
    };
    
    // Validate source and target paths are not nested within each other
    let source_dir = mkv_file.parent()
        .context("Could not determine source directory")?;
    validate_source_target_paths(source_dir, target_directory)
        .context("Source and target path validation failed")?;

    println!("📁 Analyzing: {}", mkv_file.display());
    match target_type {
        TargetType::File => {
            println!("📄 Target file: {}", target_path.display());
        }
        TargetType::Directory => {
            println!("📂 Target directory: {}", target_path.display());
        }
    }
    print_configuration_info(&config);

    analyze_and_process_mkv_file(mkv_file, &target_directory.to_path_buf(), config, true, output_filename, sonarr_context).await
}

async fn process_directory(
    input_dir: &PathBuf,
    target_directory: &PathBuf,
    recursive: bool,
    filter_pattern: Option<String>,
    config: Config,
    sonarr_context: Option<crate::models::SonarrContext>,
) -> Result<()> {
    // Validate source and target paths are not nested within each other
    validate_source_target_paths(input_dir, target_directory)
        .context("Source and target path validation failed")?;

    println!("📁 Source directory: {}", input_dir.display());
    println!("📂 Target directory: {}", target_directory.display());
    print_configuration_info(&config);

    let batch_processor = BatchProcessor::new(
        input_dir.clone(),
        target_directory.clone(),
        recursive,
        filter_pattern,
        config,
        sonarr_context,
    );

    let result = batch_processor.process().await?;

    println!("\n🎯 Batch Processing Results:");
    println!("📊 Total files processed: {}", result.total_files);
    println!("✅ Successful: {}", result.successful);
    if result.failed > 0 {
        println!("❌ Failed: {}", result.failed);
        println!("\nErrors encountered:");
        for (file, error) in &result.errors {
            println!("  {} - {}", file.display(), error);
        }
    }

    Ok(())
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