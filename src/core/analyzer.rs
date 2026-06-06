use anyhow::{Context, Result};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::models::{FFProbeOutput, SonarrContext, StreamInfo, StreamType};
use crate::utils::{SonarrMoveStatus, output_sonarr_move_status};

// MkvAnalyzer struct removed - migrated to ProcessingTask pattern
// See standalone functions below for the new implementation

/// Helper struct to group stream indices by type
struct StreamsByType {
    video: Vec<u32>,
    audio: Vec<u32>,
    subtitle: Vec<u32>,
    attachment: Vec<u32>,
}

/// Separates a list of stream indices into groups by stream type
fn separate_streams_by_type(all_streams: &[StreamInfo], indices_to_keep: &[u32]) -> StreamsByType {
    let mut result = StreamsByType {
        video: Vec::new(),
        audio: Vec::new(),
        subtitle: Vec::new(),
        attachment: Vec::new(),
    };

    for &index in indices_to_keep {
        if let Some(stream) = all_streams.iter().find(|s| s.index == index) {
            match stream.stream_type {
                StreamType::Video => result.video.push(index),
                StreamType::Audio => result.audio.push(index),
                StreamType::Subtitle => result.subtitle.push(index),
                StreamType::Attachment => result.attachment.push(index),
                _ => {}
            }
        }
    }

    result
}

fn parse_framerate(framerate_str: &str) -> Option<f64> {
    if framerate_str.contains('/') {
        let fraction_parts: Vec<&str> = framerate_str.split('/').collect();
        if fraction_parts.len() == 2 {
            if let (Ok(numerator), Ok(denominator)) = (
                fraction_parts[0].parse::<f64>(),
                fraction_parts[1].parse::<f64>(),
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
        if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
            if let Ok(seconds) = parts[2].parse::<f64>() {
                return Some(hours * 3600.0 + minutes * 60.0 + seconds);
            }
        }
    }
    None
}

// ===== New ProcessingTask-based functions =====

/// Analyze MKV file streams and return StreamInfo vector
/// This replaces MkvAnalyzer::analyze()
pub async fn analyze_mkv_streams(file_path: &std::path::Path) -> Result<Vec<StreamInfo>> {
    // Try to get ffprobe data first
    let ffprobe_data = get_ffprobe_data(file_path).await;

    // Try to get matroska data
    let matroska_data = get_matroska_data(file_path).await;

    // Combine the data sources
    extract_streams_from_data(ffprobe_data, matroska_data)
}

/// Process MKV streams using a ProcessingTask and global config/sonarr context
/// This replaces MkvAnalyzer::process_streams()
pub async fn process_mkv_streams(
    task: &crate::models::ProcessingTask,
    config: &Config,
    sonarr_context: Option<&SonarrContext>,
) -> Result<()> {
    // Determine streams to keep based on config
    let streams_to_keep = determine_streams_to_keep(&task.streams, config);

    // Check if we need to do any processing
    let all_stream_indices: Vec<u32> = task.streams.iter().map(|s| s.index).collect();
    let needs_processing =
        streams_to_keep.len() != all_stream_indices.len() || streams_to_keep != all_stream_indices;

    if !needs_processing {
        // No processing needed, just copy/hardlink
        let _output_path = task.generate_output_path()?;
        return handle_no_processing_needed_task(task, config, sonarr_context).await;
    }

    let output_path = task.generate_output_path()?;

    // Build and execute mkvmerge command
    let mut cmd = build_mkvmerge_command_for_task(task, &streams_to_keep, &output_path, config)?;

    // Check for dry-run mode before executing
    if config.processing.dry_run {
        println!(
            "🚧 Dry-run mode: Would execute mkvmerge to create: {}",
            output_path.display()
        );
        println!("🚧 Dry-run mode: Command: '{:?}'", cmd);
        println!("✅ Dry-run completed successfully!");
        return Ok(());
    }

    let output = cmd
        .output()
        .with_context(|| "Failed to execute mkvmerge command")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "mkvmerge failed with exit code {:?}, termination signal {:?}, stop signal {:?}:\n{}\n{}",
            output.status.code(),
            output.status.signal(),
            output.status.stopped_signal(),
            stdout,
            stderr
        ));
    }

    println!("✅ Successfully processed: {}", output_path.display());

    // Handle Sonarr communication
    if sonarr_context.is_some() {
        output_sonarr_move_status(SonarrMoveStatus::RenameRequested);
    }

    Ok(())
}

/// Handle no processing needed scenario for ProcessingTask
/// This replaces MkvAnalyzer::handle_no_processing_needed()
pub async fn handle_no_processing_needed_task(
    task: &crate::models::ProcessingTask,
    config: &Config,
    sonarr_context: Option<&SonarrContext>,
) -> Result<()> {
    let output_path = task.generate_output_path()?;

    if config.processing.dry_run {
        println!(
            "🔍 Dry run: Would copy {} to {}",
            task.source_file.display(),
            output_path.display()
        );
        return Ok(());
    }

    // Determine transfer mode from Sonarr context
    let transfer_mode = sonarr_context
        .and_then(|ctx| ctx.transfer_mode.as_deref())
        .unwrap_or("HardLinkOrCopy");

    match transfer_mode {
        "Move" => {
            match std::fs::rename(&task.source_file, &output_path) {
                Ok(()) => println!(
                    "📁 Moved: {} → {}",
                    task.source_file.display(),
                    output_path.display()
                ),
                Err(_) => {
                    // Cross-filesystem move: copy then delete
                    std::fs::copy(&task.source_file, &output_path).with_context(|| {
                        format!("Failed to copy file for cross-filesystem move")
                    })?;
                    std::fs::remove_file(&task.source_file)
                        .with_context(|| format!("Failed to remove source file after copy"))?;
                    println!(
                        "📁 Moved (cross-filesystem): {} → {}",
                        task.source_file.display(),
                        output_path.display()
                    );
                }
            }
        }
        "Copy" => {
            std::fs::copy(&task.source_file, &output_path)
                .with_context(|| format!("Failed to copy file"))?;
            println!(
                "📋 Copied: {} → {}",
                task.source_file.display(),
                output_path.display()
            );
        }
        "HardLink" => {
            std::fs::hard_link(&task.source_file, &output_path)
                .with_context(|| format!("Failed to create hard link"))?;
            println!(
                "🔗 Hard linked: {} → {}",
                task.source_file.display(),
                output_path.display()
            );
        }
        "HardLinkOrCopy" | _ => {
            // Default behavior: try hard link, fall back to copy
            match std::fs::hard_link(&task.source_file, &output_path) {
                Ok(()) => {
                    println!(
                        "🔗 Hard linked: {} → {}",
                        task.source_file.display(),
                        output_path.display()
                    );
                }
                Err(_) => {
                    std::fs::copy(&task.source_file, &output_path)
                        .with_context(|| format!("Failed to copy file after hard link failed"))?;
                    println!(
                        "📋 Copied (hard link failed): {} → {}",
                        task.source_file.display(),
                        output_path.display()
                    );
                }
            }
        }
    }

    // Handle Sonarr communication
    if sonarr_context.is_some() {
        output_sonarr_move_status(SonarrMoveStatus::MoveComplete);
    }

    Ok(())
}

// ===== Helper functions extracted from MkvAnalyzer =====

async fn get_ffprobe_data(file_path: &std::path::Path) -> Option<serde_json::Value> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            &file_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => match serde_json::from_slice(&output.stdout) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("Warning: Could not parse ffprobe output: {}", e);
                None
            }
        },
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

async fn get_matroska_data(file_path: &std::path::Path) -> Option<matroska::Matroska> {
    match std::fs::File::open(file_path) {
        Ok(file) => match matroska::Matroska::open(file) {
            Ok(mkv) => Some(mkv),
            Err(e) => {
                eprintln!("Warning: Could not parse with matroska crate: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Warning: Could not open file for matroska parsing: {}", e);
            None
        }
    }
}

fn extract_streams_from_data(
    ffprobe_data: Option<serde_json::Value>,
    _matroska_data: Option<matroska::Matroska>,
) -> Result<Vec<StreamInfo>> {
    let mut streams = Vec::new();

    // For now, focus on ffprobe data
    if let Some(data) = ffprobe_data {
        // Parse JSON into structured FFProbe output
        match serde_json::from_value::<FFProbeOutput>(data) {
            Ok(ffprobe_output) => {
                if let Some(stream_array) = ffprobe_output.streams {
                    for (index, stream) in stream_array.iter().enumerate() {
                        let stream_info =
                            create_stream_info_from_ffprobe_struct(index as u32, stream)?;
                        streams.push(stream_info);
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not parse ffprobe output with serde: {}", e);
                // Fallback to minimal stream info
                let stream_info = StreamInfo::new(0, StreamType::Unknown);
                streams.push(stream_info);
                return Ok(streams);
            }
        }
    } else {
        // Fallback: create minimal stream info
        eprintln!("Warning: No stream information available - using fallback");
        let stream_info = StreamInfo::new(0, StreamType::Unknown);
        streams.push(stream_info);
    }

    Ok(streams)
}

fn create_stream_info_from_ffprobe_struct(
    index: u32,
    stream: &crate::models::FFProbeStream,
) -> Result<StreamInfo> {
    let codec_type = stream.codec_type.as_deref().unwrap_or("unknown");
    let stream_type = match codec_type {
        "video" => StreamType::Video,
        "audio" => StreamType::Audio,
        "subtitle" => StreamType::Subtitle,
        "attachment" => StreamType::Attachment,
        _ => StreamType::Unknown,
    };

    let mut info = StreamInfo::new(index, stream_type);

    // Basic information
    info.codec = stream
        .codec_name
        .as_ref()
        .or(stream.codec_long_name.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Language and metadata from tags
    if let Some(tags) = &stream.tags {
        info.language = tags.language.clone();
        info.title = tags.title.clone();

        // Check for DURATION tag (format: "00:01:31.010000000")
        if let Some(duration_str) = &tags.duration {
            if let Some(duration_seconds) = parse_duration_tag(duration_str) {
                info.duration_seconds = Some(duration_seconds);
            }
        }

        // Check for NUMBER_OF_BYTES tag
        if let Some(bytes_str) = &tags.number_of_bytes {
            if let Ok(bytes) = bytes_str.parse::<u64>() {
                info.size_bytes = Some(bytes);
            }
        }
    }

    // Disposition (default/forced flags)
    if let Some(disposition) = &stream.disposition {
        info.default = disposition.default.unwrap_or(0) == 1;
        info.forced = disposition.forced.unwrap_or(0) == 1;
    }

    // Size and duration (from standard fields if tags didn't provide them)
    if let Some(bit_rate_str) = &stream.bit_rate {
        if let Ok(bit_rate) = bit_rate_str.parse::<u64>() {
            info.bitrate = Some(bit_rate);

            // Use standard duration field if we didn't get it from tags
            if info.duration_seconds.is_none() {
                if let Some(duration_str) = &stream.duration {
                    if let Ok(duration) = duration_str.parse::<f64>() {
                        info.duration_seconds = Some(duration);
                    }
                }
            }

            // Calculate size from bitrate and duration if we didn't get it from tags
            if info.size_bytes.is_none() {
                if let Some(duration) = info.duration_seconds {
                    info.size_bytes = Some((bit_rate * duration as u64) / 8);
                }
            }
        }
    }

    // Type-specific information
    match info.stream_type {
        StreamType::Video => {
            let width = stream.width.unwrap_or(0);
            let height = stream.height.unwrap_or(0);
            info.resolution = Some(format!("{}x{}", width, height));

            if let Some(fps_str) = &stream.r_frame_rate {
                info.framerate = parse_framerate(fps_str);
            }

            // Simple HDR detection
            info.hdr = Some(
                stream
                    .color_space
                    .as_ref()
                    .map(|color_space| color_space.to_lowercase().contains("bt2020"))
                    .unwrap_or(false),
            );
        }
        StreamType::Audio => {
            info.channels = stream.channels.map(|c| c as u32);
            info.sample_rate = stream
                .sample_rate
                .as_ref()
                .and_then(|sr| sr.parse::<u32>().ok());
        }
        StreamType::Subtitle => {
            info.subtitle_format = Some(info.codec.clone());
        }
        _ => {}
    }

    Ok(info)
}

fn create_stream_info_from_ffprobe(index: u32, stream: &serde_json::Value) -> Result<StreamInfo> {
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
        info.language = tags
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        info.title = tags
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

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
        info.default = disposition
            .get("default")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            == 1;
        info.forced = disposition
            .get("forced")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            == 1;
    }

    // Size and duration (from standard fields if tags didn't provide them)
    if let Some(bit_rate) = stream["bit_rate"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
    {
        info.bitrate = Some(bit_rate);

        // Use standard duration field if we didn't get it from tags
        if info.duration_seconds.is_none() {
            if let Some(duration) = stream["duration"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
            {
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
        }
        StreamType::Audio => {
            info.channels = stream["channels"].as_i64().map(|c| c as u32);
            info.sample_rate = stream["sample_rate"]
                .as_str()
                .and_then(|s| s.parse::<u32>().ok());
        }
        StreamType::Subtitle => {
            info.subtitle_format = stream["codec_name"].as_str().map(|s| s.to_string());
        }
        _ => {}
    }

    Ok(info)
}

fn determine_streams_to_keep(streams: &[StreamInfo], config: &Config) -> Vec<u32> {
    let mut streams_to_keep = Vec::new();

    for stream in streams {
        let should_keep = match stream.stream_type {
            StreamType::Video => {
                // Always keep video streams
                true
            }
            StreamType::Audio => {
                let lang = stream.effective_language();
                config.audio.keep_languages.iter().any(|l| l == lang)
            }
            StreamType::Subtitle => {
                let lang = stream.effective_language();
                // Check if any preference matches this subtitle
                config.subtitles.keep_languages.iter().any(|pref| {
                    pref.language == lang && pref.matches_title(stream.title.as_deref())
                })
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

fn build_mkvmerge_command_for_task(
    task: &crate::models::ProcessingTask,
    streams_to_keep: &[u32],
    output_path: &PathBuf,
    config: &Config,
) -> Result<Command> {
    let mut cmd = Command::new("mkvmerge");

    // Output file
    cmd.arg("-v").arg("-o").arg(output_path);

    // Separate streams by type
    let streams_by_type = separate_streams_by_type(&task.streams, streams_to_keep);
    let all_streams_by_type = separate_streams_by_type(
        &task.streams,
        &task.streams.iter().map(|s| s.index).collect::<Vec<_>>(),
    );

    add_track_selection_args(&mut cmd, &streams_by_type, &all_streams_by_type);
    add_default_track_flags(&mut cmd, task, &streams_by_type, config);

    // Input file
    cmd.arg(&task.source_file);

    Ok(cmd)
}

/// Add `--*-tracks` / `--no-*` selection args, but only for stream types where some
/// tracks are being dropped. When every track of a type is kept, mkvmerge's default
/// (include all) is left untouched.
fn add_track_selection_args(cmd: &mut Command, kept: &StreamsByType, all: &StreamsByType) {
    let selections = [
        (&kept.video, &all.video, "--video-tracks", "--no-video"),
        (&kept.audio, &all.audio, "--audio-tracks", "--no-audio"),
        (
            &kept.subtitle,
            &all.subtitle,
            "--subtitle-tracks",
            "--no-subtitles",
        ),
        (
            &kept.attachment,
            &all.attachment,
            "--attachments",
            "--no-attachments",
        ),
    ];

    for (kept_tracks, all_tracks, tracks_flag, no_flag) in selections {
        if kept_tracks.len() == all_tracks.len() {
            continue;
        }
        if kept_tracks.is_empty() {
            cmd.arg(no_flag);
        } else {
            cmd.arg(tracks_flag);
            cmd.arg(
                kept_tracks
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
    }
}

/// Set default-track and forced-display flags so that at most one audio and one
/// subtitle track are marked as default, based on language preferences.
fn add_default_track_flags(
    cmd: &mut Command,
    task: &crate::models::ProcessingTask,
    streams_by_type: &StreamsByType,
    config: &Config,
) {
    let default_audio = get_default_audio_track(&task.streams, &streams_by_type.audio, config);
    set_track_flags(cmd, &streams_by_type.audio, default_audio);

    let default_subtitle =
        get_default_subtitle_track(&task.streams, &streams_by_type.subtitle, config);
    set_track_flags(cmd, &streams_by_type.subtitle, default_subtitle);
}

/// Emit `--default-track-flag` (1 only for `default_track`) and clear the forced
/// display flag for every track in `tracks`.
fn set_track_flags(cmd: &mut Command, tracks: &[u32], default_track: Option<u32>) {
    for &track in tracks {
        let is_default = if Some(track) == default_track { 1 } else { 0 };
        cmd.arg("--default-track-flag")
            .arg(format!("{}:{}", track, is_default));
        cmd.arg("--forced-display-flag").arg(format!("{}:0", track));
    }
}

fn get_default_audio_track(
    streams: &[StreamInfo],
    audio_streams: &[u32],
    config: &Config,
) -> Option<u32> {
    // Find the first audio track that matches the highest priority language
    for preferred_lang in &config.audio.keep_languages {
        for &stream_index in audio_streams {
            if let Some(stream) = streams.iter().find(|s| s.index == stream_index) {
                let lang = stream.effective_language();
                if lang == preferred_lang {
                    return Some(stream_index);
                }
            }
        }
    }

    // If no language preference matches, return the first audio stream
    audio_streams.first().copied()
}

fn get_default_subtitle_track(
    streams: &[StreamInfo],
    subtitle_streams: &[u32],
    config: &Config,
) -> Option<u32> {
    // Find the first subtitle track that matches the highest priority preference
    for pref in &config.subtitles.keep_languages {
        for &stream_index in subtitle_streams {
            if let Some(stream) = streams.iter().find(|s| s.index == stream_index) {
                let lang = stream.effective_language();
                if lang == &pref.language && pref.matches_title(stream.title.as_deref()) {
                    return Some(stream_index);
                }
            }
        }
    }

    // No default subtitle - let all subtitle tracks be non-default
    None
}
