#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamType {
    Video,
    Audio,
    Subtitle,
    Attachment,
    Unknown,
}

impl std::fmt::Display for StreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamType::Video => write!(f, "Video"),
            StreamType::Audio => write!(f, "Audio"),
            StreamType::Subtitle => write!(f, "Subtitle"),
            StreamType::Attachment => write!(f, "Attachment"),
            StreamType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub index: u32,
    pub stream_type: StreamType,
    pub codec: String,
    pub language: Option<String>,
    pub title: Option<String>,
    pub default: bool,
    pub forced: bool,
    pub size_bytes: Option<u64>,
    pub duration_seconds: Option<f64>,
    
    // Video-specific
    pub resolution: Option<String>,
    pub framerate: Option<f64>,
    pub hdr: Option<bool>,
    
    // Audio-specific
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bitrate: Option<u64>,
    
    // Subtitle-specific
    pub subtitle_format: Option<String>,
}

impl StreamInfo {
    pub fn new(index: u32, stream_type: StreamType) -> Self {
        Self {
            index,
            stream_type,
            codec: "unknown".to_string(),
            language: None,
            title: None,
            default: false,
            forced: false,
            size_bytes: None,
            duration_seconds: None,
            resolution: None,
            framerate: None,
            hdr: None,
            channels: None,
            sample_rate: None,
            bitrate: None,
            subtitle_format: None,
        }
    }
    
    pub fn size_mb(&self) -> Option<f64> {
        self.size_bytes.map(|bytes| bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Sonarr environment context containing all variables passed by Sonarr
/// All fields are stored as raw strings to avoid parsing complications
#[derive(Debug, Clone, Default)]
pub struct SonarrContext {
    // File Paths
    pub source_path: Option<String>,
    pub destination_path: Option<String>,
    
    // Instance Information
    pub instance_name: Option<String>,
    pub application_url: Option<String>,
    pub transfer_mode: Option<String>,
    
    // Series Metadata
    pub series_id: Option<String>,
    pub series_title: Option<String>,
    pub series_title_slug: Option<String>,
    pub series_path: Option<String>,
    pub series_tvdb_id: Option<String>,
    pub series_tv_maze_id: Option<String>,
    pub series_tmdb_id: Option<String>,
    pub series_imdb_id: Option<String>,
    pub series_type: Option<String>,
    pub series_original_language: Option<String>,
    pub series_genres: Option<String>,
    pub series_tags: Option<String>,
    
    // Episode Information
    pub episode_file_episode_count: Option<String>,
    pub episode_file_episode_ids: Option<String>,
    pub episode_file_season_number: Option<String>,
    pub episode_file_episode_numbers: Option<String>,
    pub episode_file_episode_air_dates: Option<String>,
    pub episode_file_episode_air_dates_utc: Option<String>,
    pub episode_file_episode_titles: Option<String>,
    pub episode_file_episode_overviews: Option<String>,
    
    // Quality and Media Information
    pub episode_file_quality: Option<String>,
    pub episode_file_quality_version: Option<String>,
    pub episode_file_release_group: Option<String>,
    pub episode_file_scene_name: Option<String>,
    pub episode_file_media_info_audio_channels: Option<String>,
    pub episode_file_media_info_audio_codec: Option<String>,
    pub episode_file_media_info_audio_languages: Option<String>,
    pub episode_file_media_info_languages: Option<String>,
    pub episode_file_media_info_height: Option<String>,
    pub episode_file_media_info_width: Option<String>,
    pub episode_file_media_info_subtitles: Option<String>,
    pub episode_file_media_info_video_codec: Option<String>,
    pub episode_file_media_info_video_dynamic_range_type: Option<String>,
    
    // Custom Formats
    pub episode_file_custom_format: Option<String>,
    pub episode_file_custom_format_score: Option<String>,
    
    // Download Information
    pub download_client: Option<String>,
    pub download_client_type: Option<String>,
    pub download_id: Option<String>,
    
    // Deleted Files (for upgrades)
    pub deleted_relative_paths: Option<String>,
    pub deleted_paths: Option<String>,
    pub deleted_date_added: Option<String>,
    pub deleted_recycle_bin_paths: Option<String>,
}

impl SonarrContext {
    /// Check if any Sonarr environment variables were found
    pub fn is_present(&self) -> bool {
        self.source_path.is_some() || 
        self.instance_name.is_some() || 
        self.series_id.is_some()
    }
}
