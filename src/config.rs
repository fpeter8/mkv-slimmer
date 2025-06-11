use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use dialoguer::MultiSelect;

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
    pub keep_languages: Vec<String>,
    pub forced_only: bool,
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            keep_languages: vec!["eng".to_string(), "spa".to_string()],
            forced_only: false,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub suffix: String,
    pub overwrite: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            suffix: "_slimmed".to_string(),
            overwrite: false,
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
    pub output: OutputConfig,
    pub processing: ProcessingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: OutputConfig::default(),
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
            self.subtitles.keep_languages = langs;
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
                .map(|i| languages[i].to_string())
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

