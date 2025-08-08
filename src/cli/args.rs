use clap::{Arg, Command, ArgAction};
use std::path::PathBuf;

pub fn create_app() -> Command {
    Command::new("mkv-slimmer")
        .version("0.1.0")
        .about("Analyze and remove unnecessary streams from MKV files")
        .arg(
            Arg::new("input_path")
                .help("Path to the MKV file or directory to process")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("target_path")
                .help("Path where the modified MKV will be created (can be a file or directory)")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("audio_languages")
                .short('a')
                .long("audio-languages")
                .help("Languages to keep for audio tracks (can be specified multiple times)")
                .action(ArgAction::Append)
                .value_name("LANG")
        )
        .arg(
            Arg::new("subtitle_languages")
                .short('s')
                .long("subtitle-languages")
                .help("Languages to keep for subtitle tracks (can be specified multiple times)")
                .action(ArgAction::Append)
                .value_name("LANG")
        )
        .arg(
            Arg::new("dry_run")
                .short('n')
                .long("dry-run")
                .help("Show what would be removed without modifying")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Alternative config file path (optional, uses defaults if not found)")
                .default_value("settings.yaml")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Process directories recursively (only applies when input is a directory)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("filter")
                .short('f')
                .long("filter")
                .help("Glob pattern to filter files (applies to filename in non-recursive mode, relative path in recursive mode)")
                .value_name("PATTERN")
        )
}

pub struct CliArgs {
    pub input_path: PathBuf,
    pub target_path: PathBuf,
    pub config_path: PathBuf,
    pub dry_run: bool,
    pub recursive: bool,
    pub filter_pattern: Option<String>,
    pub audio_languages: Option<Vec<String>>,
    pub subtitle_languages: Option<Vec<String>>,
}

impl CliArgs {
    pub fn parse() -> anyhow::Result<Self> {
        let matches = create_app().get_matches();

        let input_path = matches.get_one::<PathBuf>("input_path")
            .ok_or_else(|| anyhow::anyhow!("Required input_path argument missing - clap configuration error"))?;
        let target_path = matches.get_one::<PathBuf>("target_path")
            .ok_or_else(|| anyhow::anyhow!("Required target_path argument missing - clap configuration error"))?;
        let config_path = matches.get_one::<PathBuf>("config")
            .ok_or_else(|| anyhow::anyhow!("Config argument with default value missing - clap configuration error"))?;
        let dry_run = matches.get_flag("dry_run");
        let recursive = matches.get_flag("recursive");
        let filter_pattern = matches.get_one::<String>("filter").map(|s| s.clone());
        
        let audio_languages: Option<Vec<String>> = matches
            .get_many::<String>("audio_languages")
            .map(|values| values.cloned().collect());
        
        let subtitle_languages: Option<Vec<String>> = matches
            .get_many::<String>("subtitle_languages")
            .map(|values| values.cloned().collect());

        Ok(CliArgs {
            input_path: input_path.clone(),
            target_path: target_path.clone(),
            config_path: config_path.clone(),
            dry_run,
            recursive,
            filter_pattern,
            audio_languages,
            subtitle_languages,
        })
    }
}