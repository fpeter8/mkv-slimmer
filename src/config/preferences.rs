use crate::error::config_error;
use anyhow::Result;
use serde::{Deserialize, Serialize};

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
                    &format!(
                        "Language code cannot be empty in preference '{}'. Use format 'language' or 'language, title prefix'",
                        s
                    ),
                ));
            }

            // Empty title prefix is valid but treated as None
            let title_prefix = if title_prefix.is_empty() {
                None
            } else {
                Some(title_prefix)
            };

            Ok(Self {
                language,
                title_prefix,
            })
        } else {
            let language = s.trim().to_string();
            if language.is_empty() {
                return Err(config_error(
                    "Subtitle language preference",
                    &format!(
                        "Language code cannot be empty in preference '{}'. Use format 'language' or 'language, title prefix'",
                        s
                    ),
                ));
            }
            Ok(Self {
                language,
                title_prefix: None,
            })
        }
    }

    /// Returns true if the given title matches this preference's title prefix requirement.
    ///
    /// Matching rules:
    /// - If no title prefix is specified: always matches (returns true)
    /// - If title prefix is specified but stream has no title: no match (returns false)
    /// - If both exist: case-insensitive prefix matching
    ///
    /// # Examples
    /// ```
    /// use mkv_slimmer::config::SubtitlePreference;
    ///
    /// let pref = SubtitlePreference { language: "eng".to_string(), title_prefix: None };
    /// assert!(pref.matches_title(Some("Any title")));
    /// assert!(pref.matches_title(None));
    ///
    /// let pref = SubtitlePreference {
    ///     language: "eng".to_string(),
    ///     title_prefix: Some("Dialogue".to_string())
    /// };
    /// assert!(pref.matches_title(Some("Dialogue - Main")));
    /// assert!(pref.matches_title(Some("dialogue for hearing")));
    /// assert!(!pref.matches_title(Some("Signs")));
    /// assert!(!pref.matches_title(None));
    /// ```
    pub fn matches_title(&self, stream_title: Option<&str>) -> bool {
        match (&self.title_prefix, stream_title) {
            (Some(prefix), Some(title)) => {
                // Case-insensitive prefix matching
                title.to_lowercase().starts_with(&prefix.to_lowercase())
            }
            (Some(_), None) => false, // Title required but not present
            (None, _) => true,        // No title requirement
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
    #[serde(
        serialize_with = "serialize_preferences",
        deserialize_with = "deserialize_preferences"
    )]
    pub keep_languages: Vec<SubtitlePreference>,
}

// Custom serialization to maintain backward compatibility
fn serialize_preferences<S>(
    prefs: &Vec<SubtitlePreference>,
    serializer: S,
) -> Result<S::Ok, S::Error>
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
                SubtitlePreference {
                    language: "eng".to_string(),
                    title_prefix: None,
                },
                SubtitlePreference {
                    language: "spa".to_string(),
                    title_prefix: None,
                },
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
