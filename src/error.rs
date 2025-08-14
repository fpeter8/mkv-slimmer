/// Custom error types and user-friendly error handling for mkv-slimmer
///
/// This module provides structured error types and consistent formatting
/// to improve user experience when operations fail.

use anyhow::Result;
use std::path::Path;

/// Creates a user-friendly file validation error message
pub fn file_validation_error(path: &Path, reason: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "❌ File validation failed\n   File: {}\n   Issue: {}",
        path.display(),
        reason
    )
}

/// Creates a user-friendly directory operation error message
pub fn directory_error(path: &Path, operation: &str, reason: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "❌ Directory operation failed\n   Path: {}\n   Operation: {}\n   Issue: {}",
        path.display(),
        operation,
        reason
    )
}

/// Creates a user-friendly configuration error message
pub fn config_error(context: &str, reason: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "❌ Configuration error\n   Context: {}\n   Issue: {}",
        context,
        reason
    )
}

/// Creates a user-friendly processing error message
pub fn processing_error(file: &Path, stage: &str, reason: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "❌ Processing failed\n   File: {}\n   Stage: {}\n   Issue: {}",
        file.display(),
        stage,
        reason
    )
}

/// Creates a user-friendly dependency error message
pub fn dependency_error(tool: &str, suggestion: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "❌ Missing dependency: {}\n   Suggestion: {}",
        tool,
        suggestion
    )
}

/// Creates a user-friendly path validation error message for dangerous operations
pub fn path_safety_error(source: &Path, target: &Path, issue: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "❌ Unsafe path configuration detected\n   Source: {}\n   Target: {}\n   Issue: {}\n   💡 Choose different source and target directories to avoid conflicts",
        source.display(),
        target.display(),
        issue
    )
}

/// Provides suggestions based on common error scenarios
pub fn suggest_solution(error: &str) -> Option<&'static str> {
    if error.contains("Permission denied") {
        Some("💡 Try running with appropriate permissions or check file ownership")
    } else if error.contains("No space left") {
        Some("💡 Free up disk space or choose a different target directory")
    } else if error.contains("not found") || error.contains("No such file") {
        Some("💡 Check that the file path is correct and the file exists")
    } else if error.contains("mkvmerge") {
        Some("💡 Make sure mkvtoolnix is installed and mkvmerge is in your PATH")
    } else if error.contains("ffprobe") {
        Some("💡 Install ffmpeg to get detailed stream information")
    } else {
        None
    }
}

/// Wraps an anyhow error with additional user-friendly context and suggestions
pub fn enhance_error(result: Result<()>, operation: &str) -> Result<()> {
    result.map_err(|err| {
        let error_msg = err.to_string();
        let mut enhanced_msg = format!("❌ {}\n   Details: {}", operation, error_msg);
        
        if let Some(suggestion) = suggest_solution(&error_msg) {
            enhanced_msg.push_str(&format!("\n   {}", suggestion));
        }
        
        anyhow::anyhow!(enhanced_msg)
    })
}