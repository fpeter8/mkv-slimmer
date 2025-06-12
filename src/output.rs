use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use tabled::{Table, Tabled, settings::Style};

use crate::config::Config;
use crate::models::{StreamInfo, StreamType};
use crate::utils::format_size;

#[derive(Tabled)]
struct VideoStreamRow {
    #[tabled(rename = "#")]
    index: String,
    #[tabled(rename = "Codec")]
    codec: String,
    #[tabled(rename = "Resolution")]
    resolution: String,
    #[tabled(rename = "FPS")]
    fps: String,
    #[tabled(rename = "HDR")]
    hdr: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Status")]
    status: String,
}

#[derive(Tabled)]
struct AudioStreamRow {
    #[tabled(rename = "#")]
    index: String,
    #[tabled(rename = "Codec")]
    codec: String,
    #[tabled(rename = "Language")]
    language: String,
    #[tabled(rename = "Channels")]
    channels: String,
    #[tabled(rename = "Sample Rate")]
    sample_rate: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Default")]
    default: String,
    #[tabled(rename = "Status")]
    status: String,
}

#[derive(Tabled)]
struct SubtitleStreamRow {
    #[tabled(rename = "#")]
    index: String,
    #[tabled(rename = "Format")]
    format: String,
    #[tabled(rename = "Language")]
    language: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Default")]
    default: String,
    #[tabled(rename = "Forced")]
    forced: String,
    #[tabled(rename = "Status")]
    status: String,
}

#[derive(Tabled)]
struct AttachmentStreamRow {
    #[tabled(rename = "#")]
    index: String,
    #[tabled(rename = "Type")]
    attachment_type: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Size")]
    size: String,
}

pub struct StreamDisplayer<'a> {
    streams: &'a [StreamInfo],
    config: &'a Config,
    grouped_streams: HashMap<StreamType, Vec<&'a StreamInfo>>,
}

impl<'a> StreamDisplayer<'a> {
    pub fn new(streams: &'a [StreamInfo], config: &'a Config) -> Self {
        let mut grouped_streams = HashMap::new();
        
        for stream in streams {
            grouped_streams
                .entry(stream.stream_type.clone())
                .or_insert_with(Vec::new)
                .push(stream);
        }
        
        Self {
            streams,
            config,
            grouped_streams,
        }
    }
    
    /// Find the preferred default audio stream (returns stream index)
    /// Uses the first language from keep_languages that exists in the streams
    fn get_preferred_default_audio_stream(&self) -> Option<u32> {
        let audio_streams = self.grouped_streams.get(&StreamType::Audio)?;
        
        for keep_lang in &self.config.audio.keep_languages {
            for stream in audio_streams {
                if let Some(ref lang) = stream.language {
                    if lang == keep_lang {
                        return Some(stream.index);
                    }
                }
            }
        }
        None
    }
    
    /// Find the preferred default subtitle stream (returns stream index)
    /// Uses the first preference from keep_languages that exists in the streams
    fn get_preferred_default_subtitle_stream(&self) -> Option<u32> {
        let subtitle_streams = self.grouped_streams.get(&StreamType::Subtitle)?;
        
        for pref in &self.config.subtitles.keep_languages {
            for stream in subtitle_streams {
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
                            return Some(stream.index);
                        }
                    }
                }
            }
        }
        None
    }
    
    
    pub fn display(&self) -> Result<()> {
        // Display video streams
        if let Some(streams) = self.grouped_streams.get(&StreamType::Video) {
            self.display_video_streams(streams)?;
        }
        
        // Display audio streams
        if let Some(streams) = self.grouped_streams.get(&StreamType::Audio) {
            self.display_audio_streams(streams)?;
        }
        
        // Display subtitle streams
        if let Some(streams) = self.grouped_streams.get(&StreamType::Subtitle) {
            self.display_subtitle_streams(streams)?;
        }
        
        // Display attachments
        if let Some(streams) = self.grouped_streams.get(&StreamType::Attachment) {
            self.display_attachment_streams(streams)?;
        }
        
        // Display summary
        self.display_summary()?;
        
        Ok(())
    }
    
    fn display_video_streams(&self, streams: &[&StreamInfo]) -> Result<()> {
        println!("\n{}", "🎬 Video Streams:".bold().cyan());
        
        let rows: Vec<VideoStreamRow> = streams
            .iter()
            .map(|stream| VideoStreamRow {
                index: stream.index.to_string(),
                codec: stream.codec.clone(),
                resolution: stream.resolution.clone().unwrap_or_else(|| "?".to_string()),
                fps: stream.framerate
                    .map(|f| format!("{:.2}", f))
                    .unwrap_or_else(|| "?".to_string()),
                hdr: stream.hdr
                    .map(|h| if h { "Yes" } else { "No" }.to_string())
                    .unwrap_or_else(|| "No".to_string()),
                size: stream.size_mb()
                    .map(|s| format!("{:.1} MB", s))
                    .unwrap_or_else(|| "?".to_string()),
                status: self.get_stream_status(stream),
            })
            .collect();
        
        let table = Table::new(rows)
            .with(Style::rounded())
            .to_string();
        
        println!("{}", table);
        Ok(())
    }
    
    fn display_audio_streams(&self, streams: &[&StreamInfo]) -> Result<()> {
        println!("\n{}", "🎵 Audio Streams:".bold().cyan());
        
        let rows: Vec<AudioStreamRow> = streams
            .iter()
            .map(|stream| AudioStreamRow {
                index: stream.index.to_string(),
                codec: stream.codec.clone(),
                language: self.format_language(&stream.language),
                channels: stream.channels
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".to_string()),
                sample_rate: stream.sample_rate
                    .map(|sr| format!("{} Hz", sr))
                    .unwrap_or_else(|| "?".to_string()),
                size: stream.size_mb()
                    .map(|s| format!("{:.1} MB", s))
                    .unwrap_or_else(|| "?".to_string()),
                default: if stream.default { "Yes" } else { "No" }.to_string(),
                status: self.get_stream_status(stream),
            })
            .collect();
        
        let table = Table::new(rows)
            .with(Style::rounded())
            .to_string();
        
        println!("{}", table);
        Ok(())
    }
    
    fn display_subtitle_streams(&self, streams: &[&StreamInfo]) -> Result<()> {
        println!("\n{}", "📄 Subtitle Streams:".bold().cyan());
        
        let rows: Vec<SubtitleStreamRow> = streams
            .iter()
            .map(|stream| SubtitleStreamRow {
                index: stream.index.to_string(),
                format: stream.subtitle_format.clone()
                    .or_else(|| Some(stream.codec.clone()))
                    .unwrap_or_else(|| "unknown".to_string()),
                language: self.format_language(&stream.language),
                title: stream.title.clone().unwrap_or_else(|| "".to_string()),
                default: if stream.default { "Yes" } else { "No" }.to_string(),
                forced: if stream.forced { "Yes" } else { "No" }.to_string(),
                status: self.get_stream_status(stream),
            })
            .collect();
        
        let table = Table::new(rows)
            .with(Style::rounded())
            .to_string();
        
        println!("{}", table);
        Ok(())
    }
    
    fn display_attachment_streams(&self, streams: &[&StreamInfo]) -> Result<()> {
        println!("\n{}", "📎 Attachments:".bold().cyan());
        
        // Group attachments by type for cleaner display
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for stream in streams {
            let attachment_type = self.get_attachment_type(&stream.codec);
            *type_counts.entry(attachment_type).or_insert(0) += 1;
        }
        
        // If we have many of the same type, show a summary
        if streams.len() > 10 && type_counts.len() < streams.len() {
            println!("Attachment Summary:");
            for (attachment_type, count) in type_counts {
                println!("  {} files: {}", attachment_type, count);
            }
            println!("\nFirst few attachments:");
            
            let limited_streams: Vec<_> = streams.iter().take(5).collect();
            let rows: Vec<AttachmentStreamRow> = limited_streams
                .iter()
                .map(|stream| AttachmentStreamRow {
                    index: stream.index.to_string(),
                    attachment_type: self.get_attachment_type(&stream.codec),
                    title: stream.title.clone().unwrap_or_else(|| "".to_string()),
                    size: stream.size_mb()
                        .map(|s| format!("{:.1} MB", s))
                        .unwrap_or_else(|| "?".to_string()),
                })
                .collect();
            
            let table = Table::new(rows)
                .with(Style::rounded())
                .to_string();
            
            println!("{}", table);
            if streams.len() > 5 {
                println!("... and {} more attachments", streams.len() - 5);
            }
        } else {
            let rows: Vec<AttachmentStreamRow> = streams
                .iter()
                .map(|stream| AttachmentStreamRow {
                    index: stream.index.to_string(),
                    attachment_type: self.get_attachment_type(&stream.codec),
                    title: stream.title.clone().unwrap_or_else(|| "".to_string()),
                    size: stream.size_mb()
                        .map(|s| format!("{:.1} MB", s))
                        .unwrap_or_else(|| "?".to_string()),
                })
                .collect();
            
            let table = Table::new(rows)
                .with(Style::rounded())
                .to_string();
            
            println!("{}", table);
        }
        Ok(())
    }
    
    fn get_attachment_type(&self, codec: &str) -> String {
        match codec.to_lowercase().as_str() {
            "ttf" => "TrueType Font".to_string(),
            "otf" => "OpenType Font".to_string(),
            "woff" | "woff2" => "Web Font".to_string(),
            "jpg" | "jpeg" => "JPEG Image".to_string(),
            "png" => "PNG Image".to_string(),
            "gif" => "GIF Image".to_string(),
            "webp" => "WebP Image".to_string(),
            "pdf" => "PDF Document".to_string(),
            "txt" => "Text File".to_string(),
            _ => if codec == "unknown" { "Unknown File".to_string() } else { codec.to_uppercase() },
        }
    }
    
    fn get_stream_status(&self, stream: &StreamInfo) -> String {
        match stream.stream_type {
            StreamType::Video => {
                // Always keep all video streams
                "KEEP".green().to_string()
            }
            StreamType::Audio => {
                if let Some(ref lang) = stream.language {
                    if self.config.audio.keep_languages.contains(lang) {
                        let preferred_default_index = self.get_preferred_default_audio_stream();
                        if preferred_default_index == Some(stream.index) {
                            "KEEP (default)".yellow().to_string()
                        } else {
                            "KEEP".green().to_string()
                        }
                    } else {
                        "REMOVE".red().to_string()
                    }
                } else {
                    // Unknown language - remove unless it's explicitly kept somehow
                    "REMOVE".red().to_string()
                }
            }
            StreamType::Subtitle => {
                if let Some(ref lang) = stream.language {
                    // Check if any preference matches this subtitle
                    let matches_preference = self.config.subtitles.keep_languages.iter().any(|pref| {
                        pref.language == *lang && 
                        match (&pref.title_prefix, &stream.title) {
                            (Some(prefix), Some(title)) => {
                                // Case-insensitive prefix matching
                                title.to_lowercase().starts_with(&prefix.to_lowercase())
                            }
                            (Some(_), None) => false, // Title required but not present
                            (None, _) => true, // No title requirement
                        }
                    });
                    
                    if matches_preference {
                        let mut status_parts = Vec::new();
                        
                        let preferred_default_index = self.get_preferred_default_subtitle_stream();
                        if preferred_default_index == Some(stream.index) {
                            status_parts.push("default");
                        }
                        
                        // Add title match indicator if there was a specific title requirement
                        if let Some(ref title) = stream.title {
                            if self.config.subtitles.keep_languages.iter().any(|pref| 
                                pref.language == *lang && 
                                pref.title_prefix.as_ref().map(|p| title.to_lowercase().starts_with(&p.to_lowercase())).unwrap_or(false)
                            ) {
                                status_parts.push("title match");
                            }
                        }
                        
                        if !status_parts.is_empty() {
                            format!("KEEP ({})", status_parts.join(", ")).yellow().to_string()
                        } else {
                            "KEEP".green().to_string()
                        }
                    } else {
                        "REMOVE".red().to_string()
                    }
                } else {
                    "REMOVE".red().to_string()
                }
            }
            StreamType::Attachment => {
                // Always keep all attachment streams
                "KEEP".green().to_string()
            }
            _ => "UNKNOWN".dimmed().to_string(),
        }
    }
    
    fn format_language(&self, language: &Option<String>) -> String {
        language.clone().unwrap_or_else(|| "none".dimmed().to_string())
    }
    
    fn display_summary(&self) -> Result<()> {
        println!("\n{}", "📊 Summary:".bold());
        
        let total_size: u64 = self.streams.iter()
            .filter_map(|s| s.size_bytes)
            .sum();
        
        let mut keep_size = 0u64;
        let mut remove_count = 0;
        
        for stream in self.streams {
            let status = self.get_stream_status(stream);
            if !status.contains("REMOVE") {
                keep_size += stream.size_bytes.unwrap_or(0);
            } else {
                remove_count += 1;
            }
        }
        
        if total_size > 0 {
            let savings = total_size - keep_size;
            let savings_pct = (savings as f64 / total_size as f64) * 100.0;
            
            println!("Original size: {}", format_size(total_size));
            println!("After processing: {}", format_size(keep_size));
            println!("Space savings: {} ({:.1}%)", format_size(savings), savings_pct);
            println!("Streams to remove: {}", remove_count);
        } else {
            println!("Unable to calculate size information");
        }
        
        Ok(())
    }
}
