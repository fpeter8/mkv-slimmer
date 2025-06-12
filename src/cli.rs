use anyhow::{Context, Result};
use clap::{Arg, Command, ArgAction};
use std::path::PathBuf;

use crate::config::Config;
use crate::analyzer::MkvAnalyzer;
use crate::utils::{check_dependencies, validate_mkv_file, validate_stream_removal, validate_source_target_paths};
use crate::batch::BatchProcessor;

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
            Arg::new("target_directory")
                .help("Directory where the modified MKV will be created")
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
        .expect("input_path argument is required but was not provided by clap");
    let target_directory = matches.get_one::<PathBuf>("target_directory")
        .expect("target_directory argument is required but was not provided by clap");
    let config_path = matches.get_one::<PathBuf>("config")
        .expect("config argument has a default value but was not provided by clap");
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

    // Validate target directory exists
    if !target_directory.exists() {
        anyhow::bail!("Target directory does not exist: {}", target_directory.display());
    }
    if !target_directory.is_dir() {
        anyhow::bail!("Target path is not a directory: {}", target_directory.display());
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

    // Check if input is a file or directory and route accordingly
    if input_path.is_file() {
        // Single file mode
        process_single_file(input_path, target_directory, config).await
    } else if input_path.is_dir() {
        // Batch mode
        process_directory(input_path, target_directory, recursive, filter_pattern, config).await
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
) -> Result<()> {
    // Create analyzer and process
    let mut analyzer = MkvAnalyzer::new(mkv_file.clone(), target_directory.clone(), config);
    
    analyzer.analyze().await
        .with_context(|| format!("Failed to analyze MKV file: {}", mkv_file.display()))?;
    
    // Only display streams in interactive mode (not in batch mode)
    if display_streams {
        analyzer.display_streams()
            .context("Failed to display stream information")?;
    }
    
    // Validate stream removal before processing
    validate_stream_removal(&analyzer.streams, &analyzer.config)
        .context("Stream validation failed")?;

    if display_streams {
        println!("\n🎬 Processing streams...");
    }
    analyzer.process_streams().await
        .context("Failed to process streams")?;

    Ok(())
}

async fn process_single_file(
    mkv_file: &PathBuf,
    target_directory: &PathBuf,
    config: Config,
) -> Result<()> {
    // Validate MKV file
    validate_mkv_file(mkv_file)
        .with_context(|| format!("Invalid MKV file: {}", mkv_file.display()))?;
    
    // Validate source and target paths are not nested within each other
    let source_dir = mkv_file.parent()
        .context("Could not determine source directory")?;
    validate_source_target_paths(source_dir, target_directory)
        .context("Source and target path validation failed")?;

    println!("📁 Analyzing: {}", mkv_file.display());
    println!("📂 Target directory: {}", target_directory.display());
    print_configuration_info(&config);

    analyze_and_process_mkv_file(mkv_file, target_directory, config, true).await
}

async fn process_directory(
    input_dir: &PathBuf,
    target_directory: &PathBuf,
    recursive: bool,
    filter_pattern: Option<String>,
    config: Config,
) -> Result<()> {
    // Validate source and target paths are not nested within each other
    validate_source_target_paths(input_dir, target_directory)
        .context("Source and target path validation failed")?;

    print_configuration_info(&config);

    // Create batch processor and run
    let batch_processor = BatchProcessor::new(
        input_dir.clone(),
        target_directory.clone(),
        recursive,
        filter_pattern,
        config,
    );

    let result = batch_processor.process().await?;
    result.print_summary();

    Ok(())
}
