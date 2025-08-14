/// Represents the different types of streams found in MKV files
///
/// MKV files can contain multiple stream types, each serving different purposes:
/// - Video streams contain the visual content
/// - Audio streams contain sound tracks in different languages
/// - Subtitle streams provide text overlays in different languages
/// - Attachment streams contain fonts, cover art, or other embedded files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamType {
    /// Video stream containing visual content
    Video,
    /// Audio stream containing audio tracks (music, dialogue, etc.)
    Audio,
    /// Subtitle stream containing text overlays
    Subtitle,
    /// Attachment stream containing fonts, cover art, or other embedded files
    Attachment,
    /// Unknown or unsupported stream type
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

/// Contains detailed information about a single stream within an MKV file
///
/// This struct aggregates metadata from multiple sources (ffprobe, matroska parser)
/// to provide comprehensive information about each stream, including codec details,
/// language information, and stream-specific properties.
///
/// # Examples
///
/// ```rust
/// use mkv_slimmer::models::{StreamInfo, StreamType};
///
/// let video_stream = StreamInfo::new(0, StreamType::Video);
/// assert_eq!(video_stream.index, 0);
/// assert_eq!(video_stream.stream_type, StreamType::Video);
/// ```
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// Zero-based index of the stream within the MKV file
    pub index: u32,
    /// The type of stream (video, audio, subtitle, attachment, unknown)
    pub stream_type: StreamType,
    /// Codec name (e.g., "h264", "aac", "subrip")
    pub codec: String,
    /// Language code if available (e.g., "eng", "jpn", "fre")
    pub language: Option<String>,
    /// Human-readable title or description of the stream
    pub title: Option<String>,
    /// Whether this stream is marked as default for its type
    pub default: bool,
    /// Whether this stream is marked as forced (for subtitles)
    pub forced: bool,
    /// Size of the stream in bytes, if calculable
    pub size_bytes: Option<u64>,
    /// Duration of the stream in seconds
    pub duration_seconds: Option<f64>,
    
    // Video-specific fields
    /// Video resolution as a string (e.g., "1920x1080")
    pub resolution: Option<String>,
    /// Frame rate in frames per second
    pub framerate: Option<f64>,
    /// Whether the video uses HDR color space
    pub hdr: Option<bool>,
    
    // Audio-specific fields
    /// Number of audio channels
    pub channels: Option<u32>,
    /// Audio sample rate in Hz
    pub sample_rate: Option<u32>,
    /// Audio bitrate in bits per second
    pub bitrate: Option<u64>,
    
    // Subtitle-specific fields
    /// Subtitle format (e.g., "subrip", "ass", "vobsub")
    pub subtitle_format: Option<String>,
}

impl StreamInfo {
    /// Creates a new StreamInfo with minimal information
    ///
    /// All optional fields are initialized to their default values.
    /// This is typically used as a starting point before populating
    /// additional metadata from ffprobe or matroska parsing.
    ///
    /// # Arguments
    /// * `index` - Zero-based index of the stream within the MKV file
    /// * `stream_type` - The type of stream being created
    ///
    /// # Examples
    /// ```rust
    /// use mkv_slimmer::models::{StreamInfo, StreamType};
    ///
    /// let audio_stream = StreamInfo::new(1, StreamType::Audio);
    /// assert_eq!(audio_stream.index, 1);
    /// assert_eq!(audio_stream.codec, "unknown");
    /// ```
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
    
    /// Converts the stream size from bytes to megabytes
    ///
    /// Returns `None` if the size is not available for this stream.
    ///
    /// # Examples
    /// ```rust
    /// use mkv_slimmer::models::{StreamInfo, StreamType};
    ///
    /// let mut stream = StreamInfo::new(0, StreamType::Video);
    /// stream.size_bytes = Some(1048576); // 1 MB in bytes
    /// assert_eq!(stream.size_mb(), Some(1.0));
    /// ```
    pub fn size_mb(&self) -> Option<f64> {
        self.size_bytes.map(|bytes| bytes as f64 / (1024.0 * 1024.0))
    }
}