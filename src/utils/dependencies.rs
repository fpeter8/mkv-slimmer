use anyhow::Result;

/// Check for required external dependencies
pub fn check_dependencies() -> Result<Vec<String>> {
    let mut missing = Vec::new();
    
    // Check for ffprobe (optional but recommended)
    if which::which("ffprobe").is_err() {
        missing.push("ffprobe".to_string());
    }
    
    // Check for mkvmerge (required for actual modifications)
    if which::which("mkvmerge").is_err() {
        return Err(anyhow::anyhow!(
            "mkvmerge is not available. Please install MKVToolNix to process MKV files.\n\
            Visit: https://mkvtoolnix.download/"
        ));
    }
    
    Ok(missing)
}