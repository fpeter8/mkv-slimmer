use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;

use crate::config::Config;
use crate::utils::is_valid_mkv_file;
use super::processor::analyze_and_process_mkv_file;
use crate::models::SonarrContext;

pub struct BatchProcessor {
    input_path: PathBuf,
    target_directory: PathBuf,
    recursive: bool,
    filter_pattern: Option<String>,
    config: Config,
    sonarr_context: Option<SonarrContext>,
}

pub struct BatchResult {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: HashMap<PathBuf, String>,
}

impl BatchProcessor {
    pub fn new(
        input_path: PathBuf,
        target_directory: PathBuf,
        recursive: bool,
        filter_pattern: Option<String>,
        config: Config,
        sonarr_context: Option<SonarrContext>,
    ) -> Self {
        Self {
            input_path,
            target_directory,
            recursive,
            filter_pattern,
            config,
            sonarr_context,
        }
    }

    pub async fn process(&self) -> Result<BatchResult> {
        println!("🎬 Starting batch processing...");
        println!("📁 Source: {}", self.input_path.display());
        println!("📂 Target: {}", self.target_directory.display());
        if self.recursive {
            println!("🔄 Mode: Recursive");
        } else {
            println!("📑 Mode: Non-recursive");
        }
        if let Some(filter) = &self.filter_pattern {
            println!("🔍 Filter: {}", filter);
        }
        println!();

        let mkv_files = self.collect_mkv_files()?;
        
        if mkv_files.is_empty() {
            println!("⚠️  No MKV files found matching criteria");
            return Ok(BatchResult {
                total_files: 0,
                successful: 0,
                failed: 0,
                errors: HashMap::new(),
            });
        }

        println!("📊 Found {} MKV file(s) to process\n", mkv_files.len());

        let mut successful = 0;
        let mut failed = 0;
        let mut errors = HashMap::new();

        for (index, file_path) in mkv_files.iter().enumerate() {
            println!("🎯 Processing file {} of {}: {}", 
                index + 1, mkv_files.len(), file_path.display());
            
            match self.process_single_file(file_path).await {
                Ok(()) => {
                    successful += 1;
                    println!("✅ Successfully processed: {}\n", file_path.display());
                }
                Err(e) => {
                    failed += 1;
                    let error_msg = format!("{:#}", e);
                    errors.insert(file_path.clone(), error_msg.clone());
                    println!("❌ Failed to process: {} - {}\n", file_path.display(), error_msg);
                }
            }
        }

        Ok(BatchResult {
            total_files: mkv_files.len(),
            successful,
            failed,
            errors,
        })
    }

    fn collect_mkv_files(&self) -> Result<Vec<PathBuf>> {
        let mut mkv_files = Vec::new();

        if self.recursive {
            self.collect_recursive(&self.input_path, &mut mkv_files)?;
        } else {
            self.collect_non_recursive(&self.input_path, &mut mkv_files)?;
        }

        // Apply filter if specified
        if let Some(filter) = &self.filter_pattern {
            mkv_files = self.apply_filter(mkv_files, filter)?;
        }

        // Sort for consistent processing order
        mkv_files.sort();

        Ok(mkv_files)
    }

    fn collect_non_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && is_valid_mkv_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    fn collect_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && is_valid_mkv_file(&path) {
                files.push(path);
            } else if path.is_dir() {
                self.collect_recursive(&path, files)?;
            }
        }

        Ok(())
    }

    fn apply_filter(&self, files: Vec<PathBuf>, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut filtered_files = Vec::new();

        for file in files {
            let match_path = if self.recursive {
                // For recursive mode, match against relative path from input directory
                file.strip_prefix(&self.input_path)
                    .with_context(|| format!("Failed to strip prefix from {}", file.display()))?
            } else {
                // For non-recursive mode, match against filename only
                file.file_name()
                    .context("Failed to get filename")?
                    .as_ref()
            };

            let match_str = match_path.to_string_lossy();
            
            // Use glob pattern matching
            if glob::Pattern::new(pattern)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?
                .matches(&match_str)
            {
                filtered_files.push(file);
            }
        }

        Ok(filtered_files)
    }

    async fn process_single_file(&self, file_path: &Path) -> Result<()> {
        // Calculate target path
        let target_path = self.calculate_target_path(file_path)?;
        
        // Ensure target directory exists and get it for processing
        let target_directory = target_path.parent()
            .context("Target path has no parent directory - cannot determine where to place output file")?;
            
        fs::create_dir_all(target_directory).await
            .with_context(|| format!("Failed to create target directory: {}", target_directory.display()))?;

        // Use shared processing function (without stream display for batch mode)
        analyze_and_process_mkv_file(
            &file_path.to_path_buf(),
            &target_directory.to_path_buf(),
            self.config.clone(),
            false,
            None,
            self.sonarr_context.clone(),
        ).await
    }

    fn calculate_target_path(&self, source_file: &Path) -> Result<PathBuf> {
        let filename = source_file.file_name()
            .context("Failed to get filename")?;

        if self.recursive {
            // Preserve directory structure
            let relative_path = source_file.strip_prefix(&self.input_path)
                .with_context(|| format!("Failed to strip prefix from {}", source_file.display()))?;
            
            // Validate no path traversal components
            for component in relative_path.components() {
                if matches!(component, std::path::Component::ParentDir) {
                    anyhow::bail!("Path traversal attempt detected in: {}", relative_path.display());
                }
            }
            
            Ok(self.target_directory.join(relative_path))
        } else {
            // Simple filename in target directory
            Ok(self.target_directory.join(filename))
        }
    }
}

impl BatchResult {
    pub fn print_summary(&self) {
        println!("📊 Batch Processing Summary:");
        println!("   Total files: {}", self.total_files);
        println!("   Successful: {}", self.successful);
        println!("   Failed: {}", self.failed);
        
        if !self.errors.is_empty() {
            println!("\n❌ Failed files:");
            for (file, error) in &self.errors {
                println!("   {}: {}", file.display(), error);
            }
        }
        
        if self.successful == self.total_files {
            println!("\n🎉 All files processed successfully!");
        } else if self.successful > 0 {
            println!("\n⚠️  Batch completed with some failures");
        } else {
            println!("\n💥 Batch processing failed completely");
        }
    }
}
