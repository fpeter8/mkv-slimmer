# MKV Slimmer 🦀

A fast, safe Rust tool to analyze and remove unnecessary streams from MKV files based on language preferences.

## Features

- 🚀 **Fast & Safe** - Written in Rust for performance and memory safety
- 📊 **Detailed Analysis** - Display comprehensive stream information with beautiful tables
- 🌍 **Language Filtering** - Filter audio and subtitle tracks by language codes (ordered by preference)
- ⚙️ **Simplified Configuration** - Easy YAML configuration with language preferences
- 🔍 **Dry-run Mode** - Preview changes without modifying files
- 🎨 **Rich Output** - Colored terminal output with emojis and formatted tables
- 🎯 **Smart Default Selection** - First available language from preference list becomes default
- 🛡️ **Stream Protection** - Prevents removal of all audio streams, warns about subtitle removal
- 📎 **Attachment Preservation** - All video and attachment streams are always kept

## Installation

### Prerequisites
- Rust (1.70 or later)
- ffprobe (from FFmpeg) - for detailed stream information
- mkvmerge (from MKVToolNix) - for file modifications

### Build from source
```bash
# Clone the repository
git clone <repository-url>
cd mkv-slimmer

# Build with Cargo
cargo build --release

# Or run directly
cargo run -- --help
```

## Usage

### Basic usage
```bash
# Analyze MKV file with default settings
cargo run -- movie.mkv

# Or using the compiled binary
./target/release/mkv-slimmer movie.mkv
```

### Advanced usage
```bash
# Keep only English and Japanese audio, English subtitles
cargo run -- movie.mkv -a eng -a jpn -s eng

# Dry run with custom config
cargo run -- movie.mkv -n -c custom-settings.yaml

# Keep Spanish and Japanese audio (Spanish will be default as it's listed first)
cargo run -- movie.mkv -a spa -a jpn -n
```

## Configuration

The tool uses a simple configuration system:

1. **CLI parameters** (highest priority) - Override configuration settings
2. **settings.yaml** file (default) - Main configuration file
3. **Interactive prompts** (fallback) - For missing required values when running in a TTY

### Example `settings.yaml`:
```yaml
# Languages to keep (ordered by preference - first available becomes default)
audio:
  keep_languages:
    - jpn    # Japanese (first preference)
    - und    # Undefined (fallback)

subtitles:
  keep_languages:
    - hun    # Hungarian (first preference)
    - und    # Undefined (second preference)
    - eng    # English (third preference)
    - jpn    # Japanese (fourth preference)
  forced_only: false     # Keep all matching languages, not just forced

# Note: Video and attachment streams are always kept

# Output preferences  
output:
  suffix: "_slimmed"
  overwrite: false

# Processing options
processing:
  dry_run: false
```

### Language Preference System

- **Ordered Lists**: Languages in `keep_languages` are ordered by preference
- **First Available Wins**: The first language from the list that exists in the video becomes the default
- **Single Default**: Only one stream per type is marked as default (the first found)
- **Automatic Fallback**: If the first preference doesn't exist, it tries the next one

## CLI Options

- `<MKV_FILE>` - Path to the MKV file to analyze (required)
- `-a, --audio-languages <LANG>` - Languages to keep for audio tracks (ordered by preference, can be specified multiple times)
- `-s, --subtitle-languages <LANG>` - Languages to keep for subtitle tracks (ordered by preference, can be specified multiple times)
- `-n, --dry-run` - Show what would be removed without modifying
- `-c, --config <FILE>` - Alternative config file path (default: settings.yaml)
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Dependencies

### Runtime Dependencies
- **ffprobe** (from FFmpeg) - For detailed stream information
- **mkvmerge** (from MKVToolNix) - For file modifications

### Rust Crates
- `clap` - Command-line argument parsing
- `serde` / `serde_yaml` - Configuration management
- `tabled` - Beautiful table formatting
- `colored` - Terminal colors and styling
- `anyhow` - Error handling
- `dialoguer` - Interactive prompts
- `matroska` - MKV parsing (backup to ffprobe)
- `tokio` - Async runtime

## Stream Protection & Validation

The tool includes intelligent stream protection:

- **Audio Protection**: Fails with an error if all audio streams would be removed
- **Subtitle Warning**: Shows a warning if all subtitle streams would be removed (but continues)
- **Video/Attachment Preservation**: All video and attachment streams are always kept

### Error Example:
```
Error: All audio streams would be removed. Audio languages to keep: [fre, ger], but available languages are: [jpn, eng]
```

## Example Output

```
📁 Analyzing: movie.mkv
🎵 Audio languages (ordered by preference): eng, jpn
📄 Subtitle languages (ordered by preference): eng, spa

🎬 Video Streams:
╭───┬───────┬────────────┬───────┬─────┬──────┬────────╮
│ # │ Codec │ Resolution │ FPS   │ HDR │ Size │ Status │
├───┼───────┼────────────┼───────┼─────┼──────┼────────┤
│ 0 │ h264  │ 1920x1080  │ 23.98 │ No  │ 2.1G │ KEEP   │
╰───┴───────┴────────────┴───────┴─────┴──────┴────────╯

🎵 Audio Streams:
╭───┬───────┬──────────┬──────────┬─────────────┬──────┬─────────┬────────────────╮
│ # │ Codec │ Language │ Channels │ Sample Rate │ Size │ Default │ Status         │
├───┼───────┼──────────┼──────────┼─────────────┼──────┼─────────┼────────────────┤
│ 1 │ ac3   │ eng      │ 6        │ 48000 Hz    │ 645M │ Yes     │ KEEP (default) │
│ 2 │ aac   │ jpn      │ 2        │ 48000 Hz    │ 156M │ No      │ KEEP           │
│ 3 │ ac3   │ spa      │ 6        │ 48000 Hz    │ 645M │ No      │ REMOVE         │
╰───┴───────┴──────────┴──────────┴─────────────┴──────┴─────────┴────────────────╯

📄 Subtitle Streams:
╭───┬────────┬──────────┬───────────────┬─────────┬────────┬────────────────╮
│ # │ Format │ Language │ Title         │ Default │ Forced │ Status         │
├───┼────────┼──────────┼───────────────┼─────────┼────────┼────────────────┤
│ 3 │ srt    │ eng      │               │ Yes     │ No     │ KEEP (default) │
│ 4 │ srt    │ spa      │ Signs & Songs │ No      │ No     │ KEEP           │
│ 5 │ ass    │ fre      │ Dialogue      │ No      │ No     │ REMOVE         │
╰───┴────────┴──────────┴───────────────┴─────────┴────────┴────────────────╯

📎 Attachments:
Attachment Summary:
  TrueType Font files: 8
  PNG Image files: 2
  Unknown File files: 3

📊 Summary:
Original size: 3.2 GB
After processing: 2.1 GB  
Space savings: 1.1 GB (34.4%)
Streams to remove: 2
```

## Development Status

- ✅ **Stream Analysis** - Complete with detailed metadata extraction
- ✅ **Language Filtering** - Full support for audio/subtitle filtering  
- ✅ **Configuration System** - Three-layer config with validation
- ✅ **Beautiful Output** - Formatted tables with colors and emojis
- 🚧 **Stream Removal** - Planned for future releases
- 🚧 **Batch Processing** - Multiple files support
- 🚧 **GUI Interface** - Desktop application

## Performance

The Rust implementation provides significant performance improvements over the previous Python version:
- 🚀 **Faster startup** - No interpreter overhead
- 💾 **Lower memory usage** - Efficient memory management
- 🛡️ **Memory safety** - Zero-cost abstractions without runtime panics
- ⚡ **Concurrent processing** - Built-in async support for future features