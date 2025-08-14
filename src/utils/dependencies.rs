use anyhow::Result;
use crate::error::dependency_error;

/// Check for required external dependencies
pub fn check_dependencies() -> Result<Vec<String>> {
    let mut missing = Vec::new();
    
    // Check for ffprobe (optional but recommended)
    if which::which("ffprobe").is_err() {
        missing.push("ffprobe".to_string());
    }
    
    // Check for mkvmerge (required for actual modifications)
    if which::which("mkvmerge").is_err() {
        return Err(dependency_error(
            "mkvmerge",
            "Install MKVToolNix from https://mkvtoolnix.download/ or use your package manager (apt install mkvtoolnix, brew install mkvtoolnix, etc.)"
        ));
    }
    
    Ok(missing)
}