use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::models::{StreamInfo, StreamType};
use crate::output::StreamDisplayer;

pub struct MkvAnalyzer {
    pub file_path: PathBuf,
    pub config: Config,
    pub streams: Vec<StreamInfo>,
}

impl MkvAnalyzer {
    pub fn new(file_path: PathBuf, config: Config) -> Self {
        Self {
            file_path,
            config,
            streams: Vec::new(),
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
        }
        
        // Disposition (default/forced flags)
        if let Some(disposition) = stream["disposition"].as_object() {
            info.default = disposition.get("default").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
            info.forced = disposition.get("forced").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
        }
        
        // Size and duration
        if let Some(bit_rate) = stream["bit_rate"].as_str().and_then(|s| s.parse::<u64>().ok()) {
            if let Some(duration) = stream["duration"].as_str().and_then(|s| s.parse::<f64>().ok()) {
                info.size_bytes = Some((bit_rate * duration as u64) / 8);
                info.duration_seconds = Some(duration);
                info.bitrate = Some(bit_rate);
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