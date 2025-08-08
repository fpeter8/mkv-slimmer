use std::path::PathBuf;
use anyhow::{Context, Result};

use super::stream::StreamInfo;

/// Lightweight struct holding file-specific processing information
/// Global info (config, sonarr context) is passed separately to processing functions
#[derive(Debug, Clone)]
pub struct ProcessingTask {
    pub source_file: PathBuf,
    pub target_location: PathBuf,
    pub streams: Vec<StreamInfo>,
    pub output_filename: Option<String>,
}

impl ProcessingTask {
    pub fn new(
        source_file: PathBuf,
        target_location: PathBuf,
        streams: Vec<StreamInfo>,
        output_filename: Option<String>,
    ) -> Self {
        Self {
            source_file,
            target_location,
            streams,
            output_filename,
        }
    }

    /// Generate the full output path for this processing task
    pub fn generate_output_path(&self) -> Result<PathBuf> {
        let output_path = match &self.output_filename {
            Some(filename) => self.target_location.join(filename),
            None => {
                let original_name = self.source_file
                    .file_name()
                    .context("Could not extract filename from source path")?
                    .to_string_lossy();
                self.target_location.join(original_name.as_ref())
            }
        };
        
        Ok(output_path)
    }

    /// Get the source filename for display purposes
    pub fn source_filename(&self) -> String {
        self.source_file
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| self.source_file.display().to_string())
    }
}