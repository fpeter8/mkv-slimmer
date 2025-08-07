use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::models::{StreamInfo, StreamType, SonarrContext};
use crate::output::StreamDisplayer;

pub struct MkvAnalyzer {
    pub file_path: PathBuf,
    pub target_directory: PathBuf,
    pub config: Config,
    pub streams: Vec<StreamInfo>,
    pub output_filename: Option<String>,
    pub sonarr_context: Option<SonarrContext>,
}

impl MkvAnalyzer {
    pub fn new(file_path: PathBuf, target_directory: PathBuf, config: Config, output_filename: Option<String>, sonarr_context: Option<SonarrContext>) -> Self {
        Self {
            file_path,
            target_directory,
            config,
            streams: Vec::new(),
            output_filename,
            sonarr_context,
        }
    }
    
    pub async fn analyze(&mut self) -> Result<()> {
        // Try to get ffprobe data first
        let ffprobe_data = self.get_ffprobe_data().await;
        
        // Try to get matroska data
        let matroska_data = self.get_matroska_data().await;
        
        // Combine the data sources
        self.extract_streams(ffprobe_data, matroska_data)?;
        
        Ok(())
    }
    
    async fn get_ffprobe_data(&self) -> Option<serde_json::Value> {
        let output = Command::new("ffprobe")
            .args([
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
                &self.file_path.to_string_lossy(),
            ])
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                match serde_json::from_slice(&output.stdout) {
                    Ok(data) => Some(data),
                    Err(e) => {
                        eprintln!("Warning: Could not parse ffprobe output: {}", e);
                        None
                    }
                }
            }
            Ok(_) => {
                eprintln!("Warning: ffprobe failed, using limited stream information");
                None
            }
            Err(_) => {
                eprintln!("Warning: ffprobe not available, using limited stream information");
                None
            }
        }
    }
    
    async fn get_matroska_data(&self) -> Option<matroska::Matroska> {
        match std::fs::File::open(&self.file_path) {
            Ok(file) => {
                match matroska::Matroska::open(file) {
                    Ok(mkv) => Some(mkv),
                    Err(e) => {
                        eprintln!("Warning: Could not parse with matroska crate: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not open file for matroska parsing: {}", e);
                None
            }
        }
    }
    
    fn extract_streams(
        &mut self,
        ffprobe_data: Option<serde_json::Value>,
        _matroska_data: Option<matroska::Matroska>,
    ) -> Result<()> {
        // For now, focus on ffprobe data
        if let Some(data) = ffprobe_data {
            if let Some(streams) = data["streams"].as_array() {
                for (index, stream) in streams.iter().enumerate() {
                    let stream_info = self.create_stream_info_from_ffprobe(index as u32, stream)?;
                    self.streams.push(stream_info);
                }
            }
        } else {
            // Fallback: create minimal stream info
            eprintln!("Warning: No stream information available - using fallback");
            let stream_info = StreamInfo::new(0, StreamType::Unknown);
            self.streams.push(stream_info);
        }
        
        Ok(())
    }
    
    fn create_stream_info_from_ffprobe(
        &self,
        index: u32,
        stream: &serde_json::Value,
    ) -> Result<StreamInfo> {
        let codec_type = stream["codec_type"].as_str().unwrap_or("unknown");
        let stream_type = match codec_type {
            "video" => StreamType::Video,
            "audio" => StreamType::Audio,
            "subtitle" => StreamType::Subtitle,
            "attachment" => StreamType::Attachment,
            _ => StreamType::Unknown,
        };
        
        let mut info = StreamInfo::new(index, stream_type);
        
        // Basic information
        info.codec = stream["codec_name"]
            .as_str()
            .or_else(|| stream["codec_long_name"].as_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Language and metadata
        if let Some(tags) = stream["tags"].as_object() {
            info.language = tags.get("language").and_then(|v| v.as_str()).map(|s| s.to_string());
            info.title = tags.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            // Check for DURATION tag (format: "00:01:31.010000000")
            if let Some(duration_str) = tags.get("DURATION").and_then(|v| v.as_str()) {
                if let Some(duration_seconds) = parse_duration_tag(duration_str) {
                    info.duration_seconds = Some(duration_seconds);
                }
            }
            
            // Check for NUMBER_OF_BYTES tag
            if let Some(bytes_str) = tags.get("NUMBER_OF_BYTES").and_then(|v| v.as_str()) {
                if let Ok(bytes) = bytes_str.parse::<u64>() {
                    info.size_bytes = Some(bytes);
                }
            }
        }
        
        // Disposition (default/forced flags)
        if let Some(disposition) = stream["disposition"].as_object() {
            info.default = disposition.get("default").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
            info.forced = disposition.get("forced").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
        }
        
        // Size and duration (from standard fields if tags didn't provide them)
        if let Some(bit_rate) = stream["bit_rate"].as_str().and_then(|s| s.parse::<u64>().ok()) {
            info.bitrate = Some(bit_rate);
            
            // Use standard duration field if we didn't get it from tags
            if info.duration_seconds.is_none() {
                if let Some(duration) = stream["duration"].as_str().and_then(|s| s.parse::<f64>().ok()) {
                    info.duration_seconds = Some(duration);
                }
            }
            
            // Calculate size from bitrate and duration if we didn't get it from tags
            if info.size_bytes.is_none() {
                if let Some(duration) = info.duration_seconds {
                    info.size_bytes = Some((bit_rate * duration as u64) / 8);
                }
            }
        }
        
        // Type-specific information
        match info.stream_type {
            StreamType::Video => {
                info.resolution = Some(format!(
                    "{}x{}",
                    stream["width"].as_i64().unwrap_or(0),
                    stream["height"].as_i64().unwrap_or(0)
                ));
                
                if let Some(fps_str) = stream["r_frame_rate"].as_str() {
                    info.framerate = parse_framerate(fps_str);
                }
                
                // Simple HDR detection
                info.hdr = Some(
                    stream["color_space"]
                        .as_str()
                        .map(|color_space| color_space.to_lowercase().contains("bt2020"))
                        .unwrap_or(false)
                );
            }
            StreamType::Audio => {
                info.channels = stream["channels"].as_i64().map(|channel_count| channel_count as u32);
                info.sample_rate = stream["sample_rate"]
                    .as_str()
                    .and_then(|sample_rate_str| sample_rate_str.parse::<u32>().ok());
            }
            StreamType::Subtitle => {
                info.subtitle_format = Some(info.codec.clone());
            }
            _ => {}
        }
        
        Ok(info)
    }
    
    pub fn display_streams(&self) -> Result<()> {
        let displayer = StreamDisplayer::new(&self.streams, &self.config);
        displayer.display()
    }
    
    pub async fn process_streams(&self) -> Result<()> {
        // Determine which streams to keep
        let streams_to_keep = self.get_streams_to_keep();
        
        if streams_to_keep.is_empty() {
            return Err(anyhow::anyhow!("No streams would be kept - refusing to process"));
        }
        
        println!("🎯 Keeping {} stream(s): {}", 
            streams_to_keep.len(),
            streams_to_keep.iter()
                .map(|&i| format!("#{}", i))
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        // Generate output filename
        let output_path = self.generate_output_path()?;
        
        // Check if mkvmerge processing is necessary
        if !self.is_mkvmerge_necessary(&streams_to_keep) {
            return self.handle_no_processing_needed(&output_path).await;
        }
                
        // Build mkvmerge command
        let mut cmd = self.build_mkvmerge_command(&streams_to_keep, &output_path)?;
        
        // Check for dry-run mode before executing
        if self.config.processing.dry_run {
            println!("🚧 Dry-run mode: Would execute mkvmerge to create: {}", output_path.display());
            println!("✅ Dry-run completed successfully!");
            return Ok(());
        }
        
        // Execute the command
        println!("🔄 Running mkvmerge to create: {}", output_path.display());
        
        let output = cmd.output()
            .with_context(|| format!("Failed to execute mkvmerge. Command: {:?}", cmd))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Try to provide helpful error messages for common issues
            let error_msg = if stderr.contains("No space left on device") {
                "Insufficient disk space to create output file".to_string()
            } else if stderr.contains("Permission denied") {
                "Permission denied - check file and directory permissions".to_string()
            } else if stderr.contains("No such file or directory") {
                "Input file not found or became unavailable during processing".to_string()
            } else {
                format!("mkvmerge failed with exit code {}", output.status.code().unwrap_or(-1))
            };
            
            return Err(anyhow::anyhow!(
                "{}\n\nStderr: {}\nStdout: {}",
                error_msg,
                stderr,
                stdout
            ));
        }
        
        // Show success message with file info
        let original_size = std::fs::metadata(&self.file_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let new_size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let size_reduction = if original_size > new_size {
            original_size - new_size
        } else {
            0
        };
        
        println!("📁 Output file: {}", output_path.display());
        println!("📊 Original size: {}", crate::utils::format_size(original_size));
        println!("📊 New size: {}", crate::utils::format_size(new_size));
        if size_reduction > 0 {
            println!("💾 Space saved: {} ({:.1}%)", 
                crate::utils::format_size(size_reduction),
                (size_reduction as f64 / original_size as f64) * 100.0
            );
        }
        
        println!("✅ Stream processing completed successfully!");
        
        // Notify Sonarr if context is present - file was modified so request rename
        if self.sonarr_context.as_ref().map(|ctx| ctx.is_present()).unwrap_or(false) {
            println!("[MoveStatus] RenameRequested");
        }
        
        Ok(())
    }
    
    fn is_mkvmerge_necessary(&self, streams_to_keep: &[u32]) -> bool {
        // Check if any streams are being removed
        let all_stream_count = self.streams.len();
        if streams_to_keep.len() != all_stream_count {
            return true; // Some streams are being removed
        }
        
        // Check if default flags need to be changed
        if self.needs_default_flag_changes(streams_to_keep) {
            return true;
        }
        
        false // No processing needed
    }
    
    fn needs_default_flag_changes(&self, streams_to_keep: &[u32]) -> bool {
        // Get audio and subtitle streams to check
        let audio_streams: Vec<u32> = streams_to_keep.iter()
            .filter(|&&index| {
                self.streams.iter()
                    .find(|s| s.index == index)
                    .map(|s| s.stream_type == StreamType::Audio)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        
        let subtitle_streams: Vec<u32> = streams_to_keep.iter()
            .filter(|&&index| {
                self.streams.iter()
                    .find(|s| s.index == index)
                    .map(|s| s.stream_type == StreamType::Subtitle)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        
        // Check ALL audio streams for correct default flag state
        let desired_default_audio = self.get_default_audio_track(&audio_streams);
        for &audio_index in &audio_streams {
            if let Some(stream) = self.streams.iter().find(|s| s.index == audio_index) {
                let should_be_default = Some(audio_index) == desired_default_audio;
                if stream.default != should_be_default {
                    return true; // This stream's default flag needs to change
                }
            }
        }
        
        // Check ALL subtitle streams for correct default flag state
        let desired_default_subtitle = self.get_default_subtitle_track(&subtitle_streams);
        for &subtitle_index in &subtitle_streams {
            if let Some(stream) = self.streams.iter().find(|s| s.index == subtitle_index) {
                let should_be_default = Some(subtitle_index) == desired_default_subtitle;
                if stream.default != should_be_default {
                    return true; // This stream's default flag needs to change
                }
            }
        }
        
        false
    }
    
    async fn handle_no_processing_needed(&self, output_path: &PathBuf) -> Result<()> {
        if self.config.processing.dry_run {
            println!("🚧 Dry-run mode: No processing needed - would link/copy file to: {}", output_path.display());
            println!("✅ Dry-run completed successfully!");
            return Ok(());
        }
        
        println!("✨ No stream processing needed - transferring file instead");
        
        // Use Sonarr transfer mode if available, otherwise default behavior
        match self.sonarr_context.as_ref().and_then(|ctx| ctx.transfer_mode.as_deref()) {
            Some("Move") => {
                // Try rename first, fallback to copy+delete for cross-filesystem moves
                match std::fs::rename(&self.file_path, output_path) {
                    Ok(()) => {
                        println!("📦 Moved to: {}", output_path.display());
                        Ok(())
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
                        std::fs::copy(&self.file_path, output_path)
                            .with_context(|| format!("Failed to copy file from {} to {}", 
                                self.file_path.display(), output_path.display()))?;
                        std::fs::remove_file(&self.file_path)
                            .with_context(|| format!("Failed to remove source file: {}", self.file_path.display()))?;
                        println!("📦 Moved to: {} (via copy+delete)", output_path.display());
                        Ok(())
                    }
                    Err(e) => Err(e).with_context(|| format!("Failed to move file from {} to {}", 
                        self.file_path.display(), output_path.display())),
                }
            }
            Some("Copy") => {
                std::fs::copy(&self.file_path, output_path)
                    .with_context(|| format!("Failed to copy file from {} to {}", 
                        self.file_path.display(), output_path.display()))?;
                println!("📋 Copied to: {}", output_path.display());
                Ok(())
            }
            Some("HardLink") => {
                std::fs::hard_link(&self.file_path, output_path)
                    .with_context(|| format!("Failed to hard link file from {} to {}", 
                        self.file_path.display(), output_path.display()))?;
                println!("🔗 Hard linked to: {}", output_path.display());
                Ok(())
            }
            Some("HardLinkOrCopy") | None => {
                // Default behavior: try hard link first, then copy
                match std::fs::hard_link(&self.file_path, output_path) {
                    Ok(()) => {
                        println!("🔗 Hard linked to: {}", output_path.display());
                        Ok(())
                    }
                    Err(_) => {
                        std::fs::copy(&self.file_path, output_path)
                            .with_context(|| format!("Failed to copy file from {} to {}", 
                                self.file_path.display(), output_path.display()))?;
                        println!("📋 Copied to: {}", output_path.display());
                        Ok(())
                    }
                }
            }
            Some(unknown_mode) => {
                eprintln!("⚠️  Unknown Sonarr transfer mode '{}', using default behavior", unknown_mode);
                // Default behavior: try hard link first, then copy
                match std::fs::hard_link(&self.file_path, output_path) {
                    Ok(()) => {
                        println!("🔗 Hard linked to: {}", output_path.display());
                        Ok(())
                    }
                    Err(_) => {
                        std::fs::copy(&self.file_path, output_path)
                            .with_context(|| format!("Failed to copy file from {} to {}", 
                                self.file_path.display(), output_path.display()))?;
                        println!("📋 Copied to: {}", output_path.display());
                        Ok(())
                    }
                }
            }
        }?;
        
        // Show file info
        let file_size = std::fs::metadata(output_path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        println!("📁 Output file: {}", output_path.display());
        println!("📊 File size: {}", crate::utils::format_size(file_size));
        println!("💾 Space saved: 0 B (0.0%) - no processing required");
        println!("✅ Stream processing completed successfully!");
        
        // Notify Sonarr if context is present
        if self.sonarr_context.as_ref().map(|ctx| ctx.is_present()).unwrap_or(false) {
            println!("[MoveStatus] MoveComplete");
        }
        
        Ok(())
    }
    
    fn get_streams_to_keep(&self) -> Vec<u32> {
        let mut streams_to_keep = Vec::new();
        
        for stream in &self.streams {
            let should_keep = match stream.stream_type {
                StreamType::Video => {
                    // Always keep video streams
                    true
                }
                StreamType::Audio => {
                    if let Some(ref lang) = stream.language {
                        self.config.audio.keep_languages.contains(lang)
                    } else {
                        // Keep audio streams without language if no other audio would be kept
                        {
                            let mut has_matching_audio = false;
                            for stream in &self.streams {
                                if stream.stream_type == StreamType::Audio {
                                    if let Some(ref lang) = stream.language {
                                        if self.config.audio.keep_languages.contains(lang) {
                                            has_matching_audio = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            !has_matching_audio
                        }
                    }
                }
                StreamType::Subtitle => {
                    if let Some(ref lang) = stream.language {
                        // Check if any preference matches this subtitle
                        self.config.subtitles.keep_languages.iter().any(|pref| {
                            pref.language == *lang && 
                            match (&pref.title_prefix, &stream.title) {
                                (Some(prefix), Some(title)) => {
                                    // Case-insensitive prefix matching
                                    title.to_lowercase().starts_with(&prefix.to_lowercase())
                                }
                                (Some(_), None) => false, // Title required but not present
                                (None, _) => true, // No title requirement
                            }
                        })
                    } else {
                        false // No language
                    }
                }
                StreamType::Attachment => {
                    // Usually keep attachments (fonts, etc.)
                    true
                }
                StreamType::Unknown => {
                    // Keep unknown streams to be safe
                    true
                }
            };
            
            if should_keep {
                streams_to_keep.push(stream.index);
            }
        }
        
        streams_to_keep
    }
    
    fn generate_output_path(&self) -> Result<PathBuf> {
        let output_filename = if let Some(ref custom_filename) = self.output_filename {
            // Use the provided filename
            custom_filename.clone()
        } else {
            // Generate filename from input file
            let file_stem = self.file_path.file_stem()
                .context("Could not get file stem from input path")?
                .to_string_lossy();
            let extension = self.file_path.extension()
                .context("Could not get file extension from input path")?
                .to_string_lossy();
            
            format!("{}.{}", file_stem, extension)
        };
        
        let output_path = self.target_directory.join(output_filename);
        
        // Verify we can write to the target directory
        if let Err(e) = std::fs::File::create(&output_path).and_then(|f| {
            std::fs::remove_file(&output_path)?;
            Ok(f)
        }) {
            return Err(anyhow::anyhow!(
                "Cannot write to target directory {}: {}",
                self.target_directory.display(),
                e
            ));
        }
        
        Ok(output_path)
    }
    
    fn build_mkvmerge_command(&self, streams_to_keep: &[u32], output_path: &PathBuf) -> Result<Command> {
        let mut cmd = Command::new("mkvmerge");
        
        // Output file
        cmd.arg("-o").arg(output_path);
        
        // Separate streams by type
        let video_streams: Vec<u32> = streams_to_keep.iter()
            .filter(|&&index| {
                self.streams.iter()
                    .find(|s| s.index == index)
                    .map(|s| s.stream_type == StreamType::Video)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        
        let audio_streams: Vec<u32> = streams_to_keep.iter()
            .filter(|&&index| {
                self.streams.iter()
                    .find(|s| s.index == index)
                    .map(|s| s.stream_type == StreamType::Audio)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        
        let subtitle_streams: Vec<u32> = streams_to_keep.iter()
            .filter(|&&index| {
                self.streams.iter()
                    .find(|s| s.index == index)
                    .map(|s| s.stream_type == StreamType::Subtitle)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        
        let attachment_streams: Vec<u32> = streams_to_keep.iter()
            .filter(|&&index| {
                self.streams.iter()
                    .find(|s| s.index == index)
                    .map(|s| s.stream_type == StreamType::Attachment)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        
        // Check if we need to filter streams (only specify if we're removing some)
        let all_video_streams: Vec<u32> = self.streams.iter()
            .filter(|s| s.stream_type == StreamType::Video)
            .map(|s| s.index)
            .collect();
        let all_audio_streams: Vec<u32> = self.streams.iter()
            .filter(|s| s.stream_type == StreamType::Audio)
            .map(|s| s.index)
            .collect();
        let all_subtitle_streams: Vec<u32> = self.streams.iter()
            .filter(|s| s.stream_type == StreamType::Subtitle)
            .map(|s| s.index)
            .collect();
        let all_attachment_streams: Vec<u32> = self.streams.iter()
            .filter(|s| s.stream_type == StreamType::Attachment)
            .map(|s| s.index)
            .collect();
        
        // Only specify track selection if we're filtering out some tracks
        // If all tracks of a type are being kept, omit the flag to let mkvmerge copy all
        
        if video_streams.len() != all_video_streams.len() {
            if video_streams.is_empty() {
                cmd.arg("--no-video");
            } else {
                cmd.arg("--video-tracks");
                cmd.arg(video_streams.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","));
            }
        }
        
        if audio_streams.len() != all_audio_streams.len() {
            if audio_streams.is_empty() {
                cmd.arg("--no-audio");
            } else {
                cmd.arg("--audio-tracks");
                cmd.arg(audio_streams.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","));
            }
        }
        
        if subtitle_streams.len() != all_subtitle_streams.len() {
            if subtitle_streams.is_empty() {
                cmd.arg("--no-subtitles");
            } else {
                cmd.arg("--subtitle-tracks");
                cmd.arg(subtitle_streams.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","));
            }
        }
        
        if attachment_streams.len() != all_attachment_streams.len() {
            if attachment_streams.is_empty() {
                cmd.arg("--no-attachments");
            } else {
                cmd.arg("--attachments");
                cmd.arg(attachment_streams.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","));
            }
        }
        
        // Set default flags based on language preferences
        
        // Default audio track - first matching language in preference order
        if let Some(default_audio) = self.get_default_audio_track(&audio_streams) {
            cmd.arg("--default-track-flag").arg(format!("{}:1", default_audio));
            
            // Set all other audio tracks to non-default
            for &track in &audio_streams {
                if track != default_audio {
                    cmd.arg("--default-track-flag").arg(format!("{}:0", track));
                }
            }
        }
        
        // Default subtitle track - first matching language in preference order
        if let Some(default_subtitle) = self.get_default_subtitle_track(&subtitle_streams) {
            cmd.arg("--default-track-flag").arg(format!("{}:1", default_subtitle));
            
            // Set all other subtitle tracks to non-default
            for &track in &subtitle_streams {
                if track != default_subtitle {
                    cmd.arg("--default-track-flag").arg(format!("{}:0", track));
                }
            }
        } else {
            // If no subtitle should be default, make sure all are set to non-default
            for &track in &subtitle_streams {
                cmd.arg("--default-track-flag").arg(format!("{}:0", track));
            }
        }
        
        // Input file
        cmd.arg(&self.file_path);
        
        Ok(cmd)
    }
    
    fn get_default_audio_track(&self, audio_streams: &[u32]) -> Option<u32> {
        // Find the first audio track that matches the highest priority language
        for preferred_lang in &self.config.audio.keep_languages {
            for &stream_index in audio_streams {
                if let Some(stream) = self.streams.iter().find(|s| s.index == stream_index) {
                    if let Some(ref lang) = stream.language {
                        if lang == preferred_lang {
                            return Some(stream_index);
                        }
                    }
                }
            }
        }
        
        // If no language preference matches, return the first audio stream
        audio_streams.first().copied()
    }
    
    fn get_default_subtitle_track(&self, subtitle_streams: &[u32]) -> Option<u32> {
        // Find the first subtitle track that matches the highest priority preference
        for pref in &self.config.subtitles.keep_languages {
            for &stream_index in subtitle_streams {
                if let Some(stream) = self.streams.iter().find(|s| s.index == stream_index) {
                    if let Some(ref lang) = stream.language {
                        if lang == &pref.language {
                            // Check if title matches if required
                            let title_matches = match (&pref.title_prefix, &stream.title) {
                                (Some(prefix), Some(title)) => {
                                    // Case-insensitive prefix matching
                                    title.to_lowercase().starts_with(&prefix.to_lowercase())
                                }
                                (Some(_), None) => false, // Title required but not present
                                (None, _) => true, // No title requirement
                            };
                            
                            if title_matches {
                                return Some(stream_index);
                            }
                        }
                    }
                }
            }
        }
        
        // No default subtitle - let all subtitle tracks be non-default
        None
    }
}

fn parse_framerate(framerate_str: &str) -> Option<f64> {
    if framerate_str.contains('/') {
        let fraction_parts: Vec<&str> = framerate_str.split('/').collect();
        if fraction_parts.len() == 2 {
            if let (Ok(numerator), Ok(denominator)) = (
                fraction_parts[0].parse::<f64>(), 
                fraction_parts[1].parse::<f64>()
            ) {
                if denominator != 0.0 {
                    return Some(numerator / denominator);
                }
            }
        }
    } else if let Ok(framerate_value) = framerate_str.parse::<f64>() {
        return Some(framerate_value);
    }
    None
}

fn parse_duration_tag(duration_str: &str) -> Option<f64> {
    // Parse duration in format "00:01:31.010000000"
    let parts: Vec<&str> = duration_str.split(':').collect();
    if parts.len() == 3 {
        if let (Ok(hours), Ok(minutes)) = (
            parts[0].parse::<f64>(),
            parts[1].parse::<f64>()
        ) {
            if let Ok(seconds) = parts[2].parse::<f64>() {
                return Some(hours * 3600.0 + minutes * 60.0 + seconds);
            }
        }
    }
    None
}
