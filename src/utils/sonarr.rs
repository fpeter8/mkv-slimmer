use crate::models::SonarrContext;

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