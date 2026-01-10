mod cli;
mod config;
mod core;
mod display;
mod error;
mod models;
mod utils;

use anyhow::{Context, Result};
use std::path::Path;

use cli::{ProcessingSettings, TargetType, prepare_processing_settings, print_configuration_info};
use core::analyzer::analyze_mkv_streams;
use core::{BatchProcessor, handle_non_mkv_file, process_task};
use models::{ProcessingTask, StreamInfo};
use utils::{is_valid_mkv_file, validate_source_target_paths};

#[tokio::main]
async fn main() -> Result<()> {
    // Get processed settings from CLI
    let settings = prepare_processing_settings().await?;

    if settings.input_is_file {
        // Process single file
        process_single_file(settings).await?;
    } else {
        // Process directory
        process_directory(settings).await?;
    }

    Ok(())
}

async fn process_single_file(settings: ProcessingSettings) -> Result<()> {
    // Handle different target types to determine output location
    let (target_directory, output_filename) = match settings.target_type {
        TargetType::File => {
            // File → File: use parent directory and extract filename
            let parent_dir = settings
                .target_path
                .parent()
                .context("Could not determine parent directory from target file path")?;
            let filename = settings
                .target_path
                .file_name()
                .context("Could not extract filename from target path")?
                .to_string_lossy()
                .to_string();
            (parent_dir, Some(filename))
        }
        TargetType::Directory => {
            // File → Directory: current behavior
            (settings.target_path.as_path(), None)
        }
    };

    // Validate source and target paths are not nested within each other
    let source_dir = settings
        .input_path
        .parent()
        .context("Could not determine source directory")?;
    validate_source_target_paths(source_dir, target_directory)
        .context("Source and target path validation failed")?;

    // Display processing info
    println!("📁 Analyzing: {}", settings.input_path.display());
    match settings.target_type {
        TargetType::File => {
            println!("📄 Target file: {}", settings.target_path.display());
        }
        TargetType::Directory => {
            println!("📂 Target directory: {}", settings.target_path.display());
        }
    }
    print_configuration_info(&settings.config);

    // Check if file is valid MKV - if not, handle immediately
    if !is_valid_mkv_file(&settings.input_path) {
        println!(
            "⚠️  File is not a valid MKV file: {}",
            settings.input_path.display()
        );
        println!("🔄 Falling back to copying original file (no processing needed)");

        handle_non_mkv_file(
            &settings.input_path,
            &target_directory.to_path_buf(),
            output_filename,
            &settings.config,
            settings.sonarr_context.as_ref(),
        )
        .await?;

        return Ok(());
    }

    // Create ProcessingTask with stream analysis
    let task = create_processing_task(
        settings.input_path,
        target_directory.to_path_buf(),
        output_filename,
    )
    .await?;

    // Process the task
    process_task(
        task,
        &settings.config,
        settings.sonarr_context.as_ref(),
        true,
    )
    .await
}

async fn process_directory(settings: ProcessingSettings) -> Result<()> {
    // Validate source and target paths are not nested within each other
    validate_source_target_paths(&settings.input_path, &settings.target_path)
        .context("Source and target path validation failed")?;

    println!("📁 Source directory: {}", settings.input_path.display());
    println!("📂 Target directory: {}", settings.target_path.display());
    print_configuration_info(&settings.config);

    let batch_processor = BatchProcessor::new(
        settings.input_path,
        settings.target_path,
        settings.recursive,
        settings.filter_pattern,
        settings.config,
        settings.sonarr_context,
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

/// Create a ProcessingTask by analyzing the MKV file streams
async fn create_processing_task(
    source_file: std::path::PathBuf,
    target_location: std::path::PathBuf,
    output_filename: Option<String>,
) -> Result<ProcessingTask> {
    // Analyze streams using the new analyzer functions
    let streams = analyze_mkv_streams_local(&source_file)
        .await
        .with_context(|| format!("Failed to analyze MKV streams: {}", source_file.display()))?;

    Ok(ProcessingTask::new(
        source_file,
        target_location,
        streams,
        output_filename,
    ))
}

/// Analyze MKV file streams and return StreamInfo vector
async fn analyze_mkv_streams_local(file_path: &Path) -> Result<Vec<StreamInfo>> {
    analyze_mkv_streams(file_path).await
}
