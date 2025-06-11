use anyhow::{Context, Result};
use std::path::Path;
use crate::config::Config;
use crate::models::{StreamInfo, StreamType};

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

/// Validate that the file is a valid MKV file
pub fn validate_mkv_file<P: AsRef<Path>>(file_path: P) -> Result<()> {
    let path = file_path.as_ref();
    
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }
    
    if !path.is_file() {
        anyhow::bail!("Not a file: {}", path.display());
    }
    
    // Check file extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if !["mkv", "mka", "mks"].contains(&ext_str.as_str()) {
            anyhow::bail!("Not an MKV file: {}", path.display());
        }
    } else {
        anyhow::bail!("File has no extension: {}", path.display());
    }
    
    // Check file is readable
    std::fs::File::open(path)
        .with_context(|| format!("Cannot read file: {}", path.display()))?;
    
    // Check for EBML header (MKV signature)
    let mut file = std::fs::File::open(path)?;
    let mut header = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut header)
        .with_context(|| format!("Cannot read MKV header from: {}", path.display()))?;
    
    if header != [0x1a, 0x45, 0xdf, 0xa3] {
        anyhow::bail!("Invalid MKV file format: {}", path.display());
    }
    
    Ok(())
}

/// Format size in human-readable format
pub fn format_size(size_bytes: u64) -> String {
    const SIZE_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_value = size_bytes as f64;
    let mut current_unit_index = 0;
    
    while size_value >= 1024.0 && current_unit_index < SIZE_UNITS.len() - 1 {
        size_value /= 1024.0;
        current_unit_index += 1;
    }
    
    format!("{:.1} {}", size_value, SIZE_UNITS[current_unit_index])
}

/// Validate that we're not removing all streams of critical types
pub fn validate_stream_removal(streams: &[StreamInfo], config: &Config) -> Result<()> {
    // Group streams by type
    let audio_streams: Vec<&StreamInfo> = streams
        .iter()
        .filter(|s| s.stream_type == StreamType::Audio)
        .collect();
    
    let subtitle_streams: Vec<&StreamInfo> = streams
        .iter()
        .filter(|s| s.stream_type == StreamType::Subtitle)
        .collect();
    
    // Check audio streams - fail if all would be removed
    if !audio_streams.is_empty() {
        let keep_count = audio_streams.iter()
            .filter(|stream| {
                if let Some(ref lang) = stream.language {
                    config.audio.keep_languages.contains(lang)
                } else {
                    false
                }
            })
            .count();
        
        if keep_count == 0 {
            return Err(anyhow::anyhow!(
                "All audio streams would be removed. Audio languages to keep: [{}], but available languages are: [{}]",
                config.audio.keep_languages.join(", "),
                audio_streams.iter()
                    .filter_map(|s| s.language.as_ref().map(|lang| lang.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    
    // Check subtitle streams - warn if all would be removed
    if !subtitle_streams.is_empty() {
        let keep_count = subtitle_streams.iter()
            .filter(|stream| {
                if let Some(ref lang) = stream.language {
                    config.subtitles.keep_languages.contains(lang)
                } else if stream.forced {
                    true // Keep forced subtitles even without language
                } else {
                    false
                }
            })
            .count();
        
        if keep_count == 0 {
            eprintln!("⚠️  Warning: All subtitle streams would be removed. Subtitle languages to keep: [{}], but available languages are: [{}]",
                config.subtitles.keep_languages.join(", "),
                subtitle_streams.iter()
                    .filter_map(|s| s.language.as_ref().map(|lang| lang.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    
    Ok(())
}
