use anyhow::{Context, Result};
use clap::{Arg, Command, ArgAction};
use std::path::PathBuf;

use crate::config::Config;
use crate::analyzer::MkvAnalyzer;
use crate::utils::{check_dependencies, validate_mkv_file, validate_source_target_paths, collect_sonarr_environment};
use crate::batch::BatchProcessor;

#[derive(Debug, Clone, PartialEq)]
enum TargetType {
    File,
    Directory,
}

/// Determine if target_path represents a file or directory
fn determine_target_type(target_path: &PathBuf) -> TargetType {
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

pub async fn run() -> Result<()> {
    let matches = Command::new("mkv-slimmer")
        .version("0.1.0")
        .about("Analyze and remove unnecessary streams from MKV files")
        .arg(
            Arg::new("input_path")
                .help("Path to the MKV file or directory to process")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("target_path")
                .help("Path where the modified MKV will be created (can be a file or directory)")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("audio_languages")
                .short('a')
                .long("audio-languages")
                .help("Languages to keep for audio tracks (can be specified multiple times)")
                .action(ArgAction::Append)
                .value_name("LANG")
        )
        .arg(
            Arg::new("subtitle_languages")
                .short('s')
                .long("subtitle-languages")
                .help("Languages to keep for subtitle tracks (can be specified multiple times)")
                .action(ArgAction::Append)
                .value_name("LANG")
        )
        .arg(
            Arg::new("dry_run")
                .short('n')
                .long("dry-run")
                .help("Show what would be removed without modifying")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Alternative config file path (optional, uses defaults if not found)")
                .default_value("settings.yaml")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Process directories recursively (only applies when input is a directory)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("filter")
                .short('f')
                .long("filter")
                .help("Glob pattern to filter files (applies to filename in non-recursive mode, relative path in recursive mode)")
                .value_name("PATTERN")
        )
        .get_matches();

    let input_path = matches.get_one::<PathBuf>("input_path")
        .ok_or_else(|| anyhow::anyhow!("Required input_path argument missing - clap configuration error"))?;
    let target_path = matches.get_one::<PathBuf>("target_path")
        .ok_or_else(|| anyhow::anyhow!("Required target_path argument missing - clap configuration error"))?;
    let config_path = matches.get_one::<PathBuf>("config")
        .ok_or_else(|| anyhow::anyhow!("Config argument with default value missing - clap configuration error"))?;
    let dry_run = matches.get_flag("dry_run");
    let recursive = matches.get_flag("recursive");
    let filter_pattern = matches.get_one::<String>("filter").map(|s| s.clone());
    
    let audio_languages: Option<Vec<String>> = matches
        .get_many::<String>("audio_languages")
        .map(|values| values.cloned().collect());
    
    let subtitle_languages: Option<Vec<String>> = matches
        .get_many::<String>("subtitle_languages")
        .map(|values| values.cloned().collect());
    

    // Check dependencies
    let missing_deps = check_dependencies()?;
    if !missing_deps.is_empty() {
        eprintln!("Warning: Missing optional dependencies: {}", missing_deps.join(", "));
        eprintln!("Some features may be limited. Install ffmpeg for full functionality.\n");
    }

    // Determine target type and validate combinations
    let target_type = determine_target_type(target_path);
    let input_is_file = input_path.is_file();
    let input_is_dir = input_path.is_dir();

    // Validate input/output combinations
    match (input_is_file, input_is_dir, &target_type) {
        (true, false, TargetType::File) => {
            // File → File: Valid
            // Ensure target directory exists if target doesn't exist
            if !target_path.exists() {
                if let Some(parent) = target_path.parent() {
                    if !parent.exists() {
                        anyhow::bail!("Target directory does not exist: {}", parent.display());
                    }
                    if !parent.is_dir() {
                        anyhow::bail!("Target parent path is not a directory: {}", parent.display());
                    }
                }
            }
        }
        (true, false, TargetType::Directory) => {
            // File → Directory: Valid (current behavior)
            if !target_path.exists() {
                anyhow::bail!("Target directory does not exist: {}", target_path.display());
            }
            if !target_path.is_dir() {
                anyhow::bail!("Target path is not a directory: {}", target_path.display());
            }
        }
        (false, true, TargetType::Directory) => {
            // Directory → Directory: Valid (current behavior)
            if !target_path.exists() {
                anyhow::bail!("Target directory does not exist: {}", target_path.display());
            }
            if !target_path.is_dir() {
                anyhow::bail!("Target path is not a directory: {}", target_path.display());
            }
        }
        (false, true, TargetType::File) => {
            // Directory → File: Invalid
            anyhow::bail!("Cannot specify a file as target when input is a directory. Use a directory as target instead.");
        }
        _ => {
            // Input path doesn't exist or is neither file nor directory
            anyhow::bail!("Input path does not exist or is neither a file nor a directory: {}", input_path.display());
        }
    }

    // Load configuration (uses defaults if file doesn't exist)
    let mut config = Config::from_yaml(config_path)?;

    // Merge CLI arguments
    config.merge_cli_args(
        audio_languages,
        subtitle_languages,
        dry_run,
    );

    // Prompt for missing values
    config.prompt_missing_values()
        .context("Failed to prompt for missing configuration values")?;

    // Validate required configuration values
    if config.audio.keep_languages.is_empty() {
        anyhow::bail!("At least one audio language must be specified");
    }
    if config.subtitles.keep_languages.is_empty() {
        anyhow::bail!("At least one subtitle language must be specified");
    }

    // Display configuration
    if config.processing.dry_run {
        println!("⚠️  Running in dry-run mode - no files will be modified\n");
    }
    
    // Collect Sonarr environment variables if present
    let sonarr_context = collect_sonarr_environment();

    // Check if input is a file or directory and route accordingly
    if input_path.is_file() {
        // Single file mode
        process_single_file(input_path, target_path, &target_type, config, Some(sonarr_context)).await
    } else if input_path.is_dir() {
        // Batch mode
        process_directory(input_path, target_path, recursive, filter_pattern, config, Some(sonarr_context)).await
    } else {
        anyhow::bail!("Input path does not exist or is neither a file nor a directory: {}", input_path.display());
    }
}

fn print_configuration_info(config: &Config) {
    println!("🎵 Audio languages (ordered by preference): {}", config.audio.keep_languages.join(", "));
    println!("📄 Subtitle languages (ordered by preference): {}", 
        config.subtitles.keep_languages
            .iter()
            .map(|pref| {
                if let Some(title) = &pref.title_prefix {
                    format!("{}, {}", pref.language, title)
                } else {
                    pref.language.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
}

pub async fn analyze_and_process_mkv_file(
    mkv_file: &PathBuf,
    target_directory: &PathBuf,
    config: Config,
    display_streams: bool,
    output_filename: Option<String>,
    sonarr_context: Option<crate::models::SonarrContext>,
) -> Result<()> {
    // Create analyzer and process
    let mut analyzer = MkvAnalyzer::new(mkv_file.clone(), target_directory.clone(), config, output_filename, sonarr_context);
    
    analyzer.analyze().await
        .with_context(|| format!("Failed to analyze MKV file: {}", mkv_file.display()))?;
    
    // Only display streams in interactive mode (not in batch mode)
    if display_streams {
        analyzer.display_streams()
            .context("Failed to display stream information")?;
    }

    if display_streams {
        println!("\n🎬 Processing streams...");
    }
    analyzer.process_streams().await
        .context("Failed to process streams")?;

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
    target_path: &PathBuf,
    recursive: bool,
    filter_pattern: Option<String>,
    config: Config,
    sonarr_context: Option<crate::models::SonarrContext>,
) -> Result<()> {
    // Ensure target is a directory for batch processing
    if !target_path.is_dir() {
        anyhow::bail!("Target must be a directory when processing a directory of files");
    }

    // Validate source and target paths are not nested within each other
    validate_source_target_paths(input_dir, target_path)
        .context("Source and target path validation failed")?;

    print_configuration_info(&config);

    // Create batch processor and run
    let batch_processor = BatchProcessor::new(
        input_dir.clone(),
        target_path.clone(),
        recursive,
        filter_pattern,
        config,
        sonarr_context,
    );

    let result = batch_processor.process().await?;
    result.print_summary();

    Ok(())
}
