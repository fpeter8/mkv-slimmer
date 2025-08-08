use tabled::Tabled;

#[derive(Tabled)]
pub struct VideoStreamRow {
    #[tabled(rename = "#")]
    pub index: String,
    #[tabled(rename = "Codec")]
    pub codec: String,
    #[tabled(rename = "Resolution")]
    pub resolution: String,
    #[tabled(rename = "FPS")]
    pub fps: String,
    #[tabled(rename = "HDR")]
    pub hdr: String,
    #[tabled(rename = "Size")]
    pub size: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

#[derive(Tabled)]
pub struct AudioStreamRow {
    #[tabled(rename = "#")]
    pub index: String,
    #[tabled(rename = "Codec")]
    pub codec: String,
    #[tabled(rename = "Language")]
    pub language: String,
    #[tabled(rename = "Channels")]
    pub channels: String,
    #[tabled(rename = "Sample Rate")]
    pub sample_rate: String,
    #[tabled(rename = "Size")]
    pub size: String,
    #[tabled(rename = "Default")]
    pub default: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

#[derive(Tabled)]
pub struct SubtitleStreamRow {
    #[tabled(rename = "#")]
    pub index: String,
    #[tabled(rename = "Format")]
    pub format: String,
    #[tabled(rename = "Language")]
    pub language: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Default")]
    pub default: String,
    #[tabled(rename = "Forced")]
    pub forced: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

#[derive(Tabled)]
pub struct AttachmentStreamRow {
    #[tabled(rename = "#")]
    pub index: String,
    #[tabled(rename = "Type")]
    pub attachment_type: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Size")]
    pub size: String,
}