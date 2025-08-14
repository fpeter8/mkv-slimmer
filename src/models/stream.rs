#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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