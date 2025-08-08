use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::Config;
use crate::models::SonarrContext;
use super::analyzer::MkvAnalyzer;

/// Shared function for analyzing and processing MKV files
/// Used by both CLI and batch processing to avoid code duplication
pub async fn analyze_and_process_mkv_file(
    mkv_file: &PathBuf,
    target_directory: &PathBuf,
    config: Config,
    display_streams: bool,
    output_filename: Option<String>,
    sonarr_context: Option<SonarrContext>,
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