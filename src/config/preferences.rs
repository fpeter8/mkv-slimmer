use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::error::config_error;

#[derive(Debug, Clone, PartialEq)]
pub struct SubtitlePreference {
    pub language: String,
    pub title_prefix: Option<String>,
}

impl SubtitlePreference {
    /// Parse a subtitle preference from a string.
    /// Format: "language" or "language, title prefix"
    pub fn parse(s: &str) -> Result<Self> {
        if let Some((lang, title)) = s.split_once(',') {
            let language = lang.trim().to_string();
            let title_prefix = title.trim().to_string();
            
            if language.is_empty() {
                return Err(config_error(
                    "Subtitle language preference", 
                    &format!("Language code cannot be empty in preference '{}'. Use format 'language' or 'language, title prefix'", s)
                ));
            }
            
            // Empty title prefix is valid but treated as None
            let title_prefix = if title_prefix.is_empty() {
                None
            } else {
                Some(title_prefix)
            };
            
            Ok(Self { language, title_prefix })
        } else {
            let language = s.trim().to_string();
            if language.is_empty() {
                return Err(config_error(
                    "Subtitle language preference", 
                    &format!("Language code cannot be empty in preference '{}'. Use format 'language' or 'language, title prefix'", s)
                ));
            }
            Ok(Self { language, title_prefix: None })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub keep_languages: Vec<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            keep_languages: vec!["eng".to_string(), "jpn".to_string(), "und".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleConfig {
    #[serde(serialize_with = "serialize_preferences", deserialize_with = "deserialize_preferences")]
    pub keep_languages: Vec<SubtitlePreference>,
}

// Custom serialization to maintain backward compatibility
fn serialize_preferences<S>(prefs: &Vec<SubtitlePreference>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(prefs.len()))?;
    for pref in prefs {
        if let Some(title) = &pref.title_prefix {
            seq.serialize_element(&format!("{}, {}", pref.language, title))?;
        } else {
            seq.serialize_element(&pref.language)?;
        }
    }
    seq.end()
}

// Custom deserialization to parse preferences
fn deserialize_preferences<'de, D>(deserializer: D) -> Result<Vec<SubtitlePreference>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    strings
        .into_iter()
        .map(|s| SubtitlePreference::parse(&s).map_err(serde::de::Error::custom))
        .collect()
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            keep_languages: vec![
                SubtitlePreference { language: "eng".to_string(), title_prefix: None },
                SubtitlePreference { language: "spa".to_string(), title_prefix: None },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub dry_run: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self { dry_run: false }
    }
}