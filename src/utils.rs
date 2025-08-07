use crate::models::SonarrContext;
use anyhow::{Context, Result};
use std::path::Path;

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

/// Check if the file is a valid MKV file (returns bool, doesn't throw)
pub fn is_valid_mkv_file<P: AsRef<Path>>(file_path: P) -> bool {
    let path = file_path.as_ref();
    
    // Check if file exists and is a file
    if !path.exists() || !path.is_file() {
        return false;
    }
    
    // Check file extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if !["mkv", "mka", "mks"].contains(&ext_str.as_str()) {
            return false;
        }
    } else {
        return false;
    }
    
    // Check if file is readable
    std::fs::File::open(path).is_ok()
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

/// Validate that source and target paths are not nested within each other
/// This prevents scenarios like:
/// - Source: /movies/season1, Target: /movies/season1/processed
/// - Source: /movies/season1/episode1.mkv, Target: /movies
/// - Source: /movies, Target: /movies/processed
pub fn validate_source_target_paths(source_path: &Path, target_path: &Path) -> Result<()> {
    // Canonicalize source path to resolve symlinks and relative paths
    let source_canonical = source_path.canonicalize()
        .with_context(|| format!("Could not resolve source path: {}", source_path.display()))?;
    
    // For target path, handle the case where it might not exist yet
    let target_canonical = if target_path.exists() {
        target_path.canonicalize()
            .with_context(|| format!("Could not resolve target path: {}", target_path.display()))?
    } else {
        // If target doesn't exist, canonicalize its parent directory
        let parent = target_path.parent()
            .context("Target path has no parent directory")?;
        let parent_canonical = parent.canonicalize()
            .with_context(|| format!("Could not resolve target parent path: {}", parent.display()))?;
        parent_canonical.join(target_path.file_name().unwrap_or_default())
    };
    
    // Check if paths are exactly the same
    if source_canonical == target_canonical {
        anyhow::bail!(
            "Source and target paths cannot be the same.\nSource: {}\nTarget: {}", 
            source_path.display(), 
            target_path.display()
        );
    }
    
    // Check if target is nested within source
    if target_canonical.starts_with(&source_canonical) {
        anyhow::bail!(
            "Target path cannot be nested within the source path.\nSource: {}\nTarget: {}\nThis would cause the output to be processed as input in recursive mode.", 
            source_path.display(), 
            target_path.display()
        );
    }
    
    // Check if source is nested within target
    if source_canonical.starts_with(&target_canonical) {
        anyhow::bail!(
            "Source path cannot be nested within the target path.\nSource: {}\nTarget: {}\nThis would overwrite source files during processing.", 
            source_path.display(), 
            target_path.display()
        );
    }
    
    Ok(())
}

/// Collect Sonarr environment variables into a SonarrContext struct
/// Performs case-insensitive matching for environment variable names
pub fn collect_sonarr_environment() -> SonarrContext {
    let mut context = SonarrContext::default();
    
    // Collect all environment variables and filter for Sonarr ones
    let env_vars: std::collections::HashMap<String, String> = std::env::vars()
        .filter(|(key, _)| key.to_lowercase().starts_with("sonarr_"))
        .map(|(key, value)| (key.to_lowercase(), value))
        .collect();
    
    // Helper function to get environment variable value
    let get_env = |key: &str| -> Option<String> {
        env_vars.get(&format!("sonarr_{}", key)).map(|s| s.clone())
    };
    
    // File Paths
    context.source_path = get_env("sourcepath");
    context.destination_path = get_env("destinationpath");
    
    // Instance Information
    context.instance_name = get_env("instancename");
    context.application_url = get_env("applicationurl");
    context.transfer_mode = get_env("transfermode");
    
    // Series Metadata
    context.series_id = get_env("series_id");
    context.series_title = get_env("series_title");
    context.series_title_slug = get_env("series_titleslug");
    context.series_path = get_env("series_path");
    context.series_tvdb_id = get_env("series_tvdbid");
    context.series_tv_maze_id = get_env("series_tvmazeid");
    context.series_tmdb_id = get_env("series_tmdbid");
    context.series_imdb_id = get_env("series_imdbid");
    context.series_type = get_env("series_type");
    context.series_original_language = get_env("series_originallanguage");
    context.series_genres = get_env("series_genres");
    context.series_tags = get_env("series_tags");
    
    // Episode Information
    context.episode_file_episode_count = get_env("episodefile_episodecount");
    context.episode_file_episode_ids = get_env("episodefile_episodeids");
    context.episode_file_season_number = get_env("episodefile_seasonnumber");
    context.episode_file_episode_numbers = get_env("episodefile_episodenumbers");
    context.episode_file_episode_air_dates = get_env("episodefile_episodeairdates");
    context.episode_file_episode_air_dates_utc = get_env("episodefile_episodeairdatesutc");
    context.episode_file_episode_titles = get_env("episodefile_episodetitles");
    context.episode_file_episode_overviews = get_env("episodefile_episodeoverviews");
    
    // Quality and Media Information
    context.episode_file_quality = get_env("episodefile_quality");
    context.episode_file_quality_version = get_env("episodefile_qualityversion");
    context.episode_file_release_group = get_env("episodefile_releasegroup");
    context.episode_file_scene_name = get_env("episodefile_scenename");
    context.episode_file_media_info_audio_channels = get_env("episodefile_mediainfo_audiochannels");
    context.episode_file_media_info_audio_codec = get_env("episodefile_mediainfo_audiocodec");
    context.episode_file_media_info_audio_languages = get_env("episodefile_mediainfo_audiolanguages");
    context.episode_file_media_info_languages = get_env("episodefile_mediainfo_languages");
    context.episode_file_media_info_height = get_env("episodefile_mediainfo_height");
    context.episode_file_media_info_width = get_env("episodefile_mediainfo_width");
    context.episode_file_media_info_subtitles = get_env("episodefile_mediainfo_subtitles");
    context.episode_file_media_info_video_codec = get_env("episodefile_mediainfo_videocodec");
    context.episode_file_media_info_video_dynamic_range_type = get_env("episodefile_mediainfo_videodynamicrangetype");
    
    // Custom Formats
    context.episode_file_custom_format = get_env("episodefile_customformat");
    context.episode_file_custom_format_score = get_env("episodefile_customformatscore");
    
    // Download Information
    context.download_client = get_env("download_client");
    context.download_client_type = get_env("download_client_type");
    context.download_id = get_env("download_id");
    
    // Deleted Files (for upgrades)
    context.deleted_relative_paths = get_env("deletedrelativepaths");
    context.deleted_paths = get_env("deletedpaths");
    context.deleted_date_added = get_env("deleteddateadded");
    context.deleted_recycle_bin_paths = get_env("deletedrecyclebinpaths");
    
    if context.is_present() {
        println!("🎬 Detected Sonarr environment context");
        if let Some(ref series_title) = context.series_title {
            println!("📺 Processing for series: {}", series_title);
        }
        if let Some(ref season) = context.episode_file_season_number {
            if let Some(ref episode) = context.episode_file_episode_numbers {
                println!("📋 Episode: S{}E{}", season, episode);
            }
        }
    }
    
    context
}
