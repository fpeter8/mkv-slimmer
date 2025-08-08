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