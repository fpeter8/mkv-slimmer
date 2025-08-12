use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use dialoguer::MultiSelect;

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
                anyhow::bail!("Language cannot be empty in subtitle preference: '{}'", s);
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
                anyhow::bail!("Language cannot be empty in subtitle preference: '{}'", s);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub audio: AudioConfig,
    pub subtitles: SubtitleConfig,
    pub processing: ProcessingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            processing: ProcessingConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from YAML file.
    /// Returns default configuration if file doesn't exist (no error).
    /// Only fails if file exists but cannot be read or parsed.
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Gracefully handle missing config file by using defaults
        if !path.exists() {
            eprintln!("Missing config file: {}", path.display());
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    pub fn merge_cli_args(
        &mut self,
        audio_languages: Option<Vec<String>>,
        subtitle_languages: Option<Vec<String>>,
        dry_run: bool,
    ) {
        // Audio languages
        if let Some(langs) = audio_languages {
            self.audio.keep_languages = langs;
        }
        
        // Subtitle languages
        if let Some(langs) = subtitle_languages {
            self.subtitles.keep_languages = langs
                .into_iter()
                .map(|s| SubtitlePreference::parse(&s))
                .collect::<Result<Vec<_>>>()
                .unwrap_or_else(|e| {
                    eprintln!("Error parsing subtitle language: {}", e);
                    std::process::exit(1);
                });
        }
        
        // Processing options
        if dry_run {
            self.processing.dry_run = true;
        }
        
        // Validate configuration after CLI merge
        if let Err(e) = self.validate() {
            eprintln!("Error: Configuration validation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    pub fn prompt_missing_values(&mut self) -> Result<()> {
        // Check if we're running in a TTY
        if !atty::is(atty::Stream::Stdin) {
            return Ok(());
        }
        
        // Prompt for audio languages if empty
        if self.audio.keep_languages.is_empty() {
            println!("No audio languages specified. Select languages to keep:");
            let languages = vec!["eng", "jpn", "spa", "fre", "ger", "ita", "und"];
            let selections = MultiSelect::new()
                .with_prompt("Audio languages to keep")
                .items(&languages)
                .interact()?;
            
            self.audio.keep_languages = selections
                .into_iter()
                .map(|i| languages[i].to_string())
                .collect();
        }
        
        // Prompt for subtitle languages if empty
        if self.subtitles.keep_languages.is_empty() {
            println!("No subtitle languages specified. Select languages to keep:");
            let languages = vec!["eng", "jpn", "spa", "fre", "ger", "ita", "und"];
            let selections = MultiSelect::new()
                .with_prompt("Subtitle languages to keep")
                .items(&languages)
                .interact()?;
            
            self.subtitles.keep_languages = selections
                .into_iter()
                .map(|i| SubtitlePreference {
                    language: languages[i].to_string(),
                    title_prefix: None,
                })
                .collect();
        }
        
        Ok(())
    }
    
    /// Validate configuration - currently no specific validations needed
    pub fn validate(&self) -> Result<()> {
        // No specific validation needed since default languages are removed
        // and video/attachment streams are always kept
        Ok(())
    }
}
