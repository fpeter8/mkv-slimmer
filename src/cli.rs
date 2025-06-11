use anyhow::{Context, Result};
use clap::{Arg, Command, ArgAction};
use std::path::PathBuf;

use crate::config::Config;
use crate::analyzer::MkvAnalyzer;
use crate::utils::{check_dependencies, validate_mkv_file, validate_stream_removal};

pub async fn run() -> Result<()> {
    let matches = Command::new("mkv-slimmer")
        .version("0.1.0")
        .about("Analyze and remove unnecessary streams from MKV files")
        .arg(
            Arg::new("mkv_file")
                .help("Path to the MKV file to analyze")
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
        .get_matches();

    let mkv_file = matches.get_one::<PathBuf>("mkv_file")
        .expect("mkv_file argument is required but was not provided by clap");
    let target_directory = matches.get_one::<PathBuf>("target_directory")
        .expect("target_directory argument is required but was not provided by clap");
    let config_path = matches.get_one::<PathBuf>("config")
        .expect("config argument has a default value but was not provided by clap");
    let dry_run = matches.get_flag("dry_run");
    
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

    // Validate MKV file
    validate_mkv_file(mkv_file)
        .with_context(|| format!("Invalid MKV file: {}", mkv_file.display()))?;
    
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

    println!("📁 Analyzing: {}", mkv_file.display());
    println!("📂 Target directory: {}", target_directory.display());
    println!("🎵 Audio languages (ordered by preference): {}", config.audio.keep_languages.join(", "));
    println!("📄 Subtitle languages (ordered by preference): {}", config.subtitles.keep_languages.join(", "));
    
    println!();

    // Analyze MKV file
    let mut analyzer = MkvAnalyzer::new(mkv_file.clone(), target_directory.clone(), config);
    analyzer.analyze().await
        .context("Failed to analyze MKV file")?;
    
    analyzer.display_streams()
        .context("Failed to display stream information")?;
    
    // Validate stream removal before processing
    validate_stream_removal(&analyzer.streams, &analyzer.config)
        .context("Stream validation failed")?;

    println!("\n🎬 Processing streams...");
    analyzer.process_streams().await
        .context("Failed to process streams")?;

    Ok(())
}
