# MKV Slimmer 🦀

A fast, safe Rust tool to analyze and remove unnecessary streams from MKV files based on language preferences.

## Features

- 🚀 **Fast & Safe** - Written in Rust for performance and memory safety
- 📊 **Detailed Analysis** - Display comprehensive stream information with beautiful tables
- 🌍 **Language Filtering** - Filter audio and subtitle tracks by language codes
- ⚙️ **Flexible Configuration** - Three-layer configuration system (CLI > YAML > interactive prompts)
- 🔍 **Dry-run Mode** - Preview changes without modifying files
- 🎨 **Rich Output** - Colored terminal output with emojis and formatted tables
- 🎯 **Default Track Management** - Set default tracks based on language preferences

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

# Set English as default audio and subtitle
cargo run -- movie.mkv -d eng -t eng

# Dry run with custom config
cargo run -- movie.mkv -n -c custom-settings.yaml

# Keep Spanish and Japanese audio, set Spanish as default
cargo run -- movie.mkv -a spa -a jpn -d spa -n
```

## Configuration

The tool uses a three-layer configuration system:

1. **CLI parameters** (highest priority) - Override any other settings
2. **settings.yaml** file (medium priority) - Default configuration file  
3. **Interactive prompts** (fallback) - For missing required values

### Example `settings.yaml`:
```yaml
# Languages to keep
audio:
  keep_languages:
    - eng    # English
    - jpn    # Japanese
    - und    # Undefined
  default_language: eng

subtitles:
  keep_languages:
    - eng    # English
    - spa    # Spanish
  default_language: eng
  forced_only: false

# Stream preferences
video:
  keep_all: true

# Output settings
output:
  suffix: "_slimmed"
  overwrite: false

# Processing options
processing:
  dry_run: false
```

## CLI Options

- `<MKV_FILE>` - Path to the MKV file to analyze (required)
- `-a, --audio-languages <LANG>` - Languages to keep for audio tracks (can be specified multiple times)
- `-s, --subtitle-languages <LANG>` - Languages to keep for subtitle tracks (can be specified multiple times)
- `-n, --dry-run` - Show what would be removed without modifying
- `-c, --config <FILE>` - Alternative config file path (default: settings.yaml)
- `-d, --default-audio-language <LANG>` - Set default audio track by language
- `-t, --default-subtitle-language <LANG>` - Set default subtitle track by language
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

## Example Output

```
🎬 Video Streams:
╭───┬───────┬────────────┬───────┬─────┬──────┬────────╮
│ # │ Codec │ Resolution │ FPS   │ HDR │ Size │ Status │
├───┼───────┼────────────┼───────┼─────┼──────┼────────┤
│ 0 │ h264  │ 1920x1080  │ 23.98 │ No  │ 2.1G │ KEEP   │
╰───┴───────┴────────────┴───────┴─────┴──────┴────────╯

🎵 Audio Streams:
╭───┬───────┬──────────┬──────────┬─────────────┬──────┬─────────┬────────╮
│ # │ Codec │ Language │ Channels │ Sample Rate │ Size │ Default │ Status │
├───┼───────┼──────────┼──────────┼─────────────┼──────┼─────────┼────────┤
│ 1 │ ac3   │ eng      │ 6        │ 48000 Hz    │ 645M │ Yes     │ KEEP   │
│ 2 │ aac   │ jpn      │ 2        │ 48000 Hz    │ 156M │ No      │ KEEP   │
│ 3 │ ac3   │ spa      │ 6        │ 48000 Hz    │ 645M │ No      │ REMOVE │
╰───┴───────┴──────────┴──────────┴─────────────┴──────┴─────────┴────────╯
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