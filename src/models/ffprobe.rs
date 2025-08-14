use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct FFProbeOutput {
    pub streams: Option<Vec<FFProbeStream>>,
}

#[derive(Deserialize)]
pub struct FFProbeStream {
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    pub codec_long_name: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub r_frame_rate: Option<String>,
    pub color_space: Option<String>,
    pub channels: Option<i64>,
    pub sample_rate: Option<String>,
    pub bit_rate: Option<String>,
    pub duration: Option<String>,
    pub tags: Option<FFProbeTags>,
    pub disposition: Option<FFProbeDisposition>,
}

#[derive(Deserialize)]
pub struct FFProbeTags {
    pub language: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "DURATION")]
    pub duration: Option<String>,
    #[serde(rename = "NUMBER_OF_BYTES")]
    pub number_of_bytes: Option<String>,
    // Allow any other tags to be present without failing
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct FFProbeDisposition {
    pub default: Option<i64>,
    pub forced: Option<i64>,
    // Allow any other disposition fields to be present without failing
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}