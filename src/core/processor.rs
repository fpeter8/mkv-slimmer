use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::models::{SonarrContext, ProcessingTask};
use crate::display::StreamDisplayer;
use super::analyzer::{analyze_mkv_streams, process_mkv_streams, handle_no_processing_needed_task};

/// Processes a single MKV file using a ProcessingTask with configuration
///
/// This is the main processing function that handles stream analysis, filtering,
/// and mkvmerge execution. It replaces the original analyze_and_process_mkv_file
/// function and provides a clean interface for both CLI and batch processing.
///
/// # Arguments
/// * `task` - Pre-analyzed processing task containing file info and streams
/// * `config` - Configuration for stream filtering and processing behavior  
/// * `sonarr_context` - Optional Sonarr context for automated processing
/// * `display_streams` - Whether to show stream information (for interactive mode)
///
/// # Returns
/// `Ok(())` if processing completed successfully, `Err` with context on failure
///
/// # Examples
/// ```rust
/// use mkv_slimmer::core::{process_task, ProcessingTask};
/// use mkv_slimmer::config::Config;
/// use std::path::PathBuf;
///
/// # tokio_test::block_on(async {
/// let task = ProcessingTask::new(
///     PathBuf::from("input.mkv"), 
///     PathBuf::from("output.mkv")
/// );
/// let config = Config::default();
/// 
/// let result = process_task(task, &config, None, true).await;
/// # });
/// ```
pub async fn process_task(
    task: ProcessingTask,
    config: &Config,
    sonarr_context: Option<&SonarrContext>,
    display_streams: bool,
) -> Result<()> {
    // Display streams in interactive mode (not in batch mode)
    if display_streams {
        let displayer = StreamDisplayer::new(&task.streams, config);
        displayer.display()
            .context("Failed to display stream information")?;
        println!("\n🎬 Processing streams...");
    }

    // Process the streams using the task
    process_mkv_streams(&task, config, sonarr_context).await
        .with_context(|| format!("Failed to process streams for: {}", task.source_file.display()))?;

    Ok(())
}

/// Handle non-MKV files by copying/hardlinking immediately
/// This handles files that don't need stream processing
pub async fn handle_non_mkv_file(
    source_file: &Path,
    target_directory: &Path,
    output_filename: Option<String>,
    config: &Config,
    sonarr_context: Option<&SonarrContext>,
) -> Result<()> {
    // Create a minimal task for file operations
    let task = ProcessingTask::new(
        source_file.to_path_buf(),
        target_directory.to_path_buf(),
        Vec::new(), // No streams for non-MKV files
        output_filename,
    );

    handle_no_processing_needed_task(&task, config, sonarr_context).await
        .with_context(|| format!("Failed to copy non-MKV file: {}", source_file.display()))?;

    Ok(())
}

/// Legacy function for backward compatibility with batch processing
/// TODO: Remove this once batch.rs is updated to use ProcessingTask
pub async fn analyze_and_process_mkv_file(
    mkv_file: &std::path::PathBuf,
    target_directory: &std::path::PathBuf,
    config: Config,
    display_streams: bool,
    output_filename: Option<String>,
    sonarr_context: Option<SonarrContext>,
) -> Result<()> {
    use crate::utils::is_valid_mkv_file;
    
    // Check if file is a valid MKV - if not, fall back to copy/hardlink
    if !is_valid_mkv_file(mkv_file) {
        println!("⚠️  File is not a valid MKV file: {}", mkv_file.display());
        println!("🔄 Falling back to copying original file (no processing needed)");
        
        return handle_non_mkv_file(
            mkv_file,
            target_directory,
            output_filename,
            &config,
            sonarr_context.as_ref(),
        ).await;
    }
    
    // Analyze streams and create task
    let streams = analyze_mkv_streams(mkv_file).await
        .with_context(|| format!("Failed to analyze MKV streams: {}", mkv_file.display()))?;
    
    let task = ProcessingTask::new(
        mkv_file.clone(),
        target_directory.clone(),
        streams,
        output_filename,
    );
    
    // Process the task
    process_task(task, &config, sonarr_context.as_ref(), display_streams).await
}