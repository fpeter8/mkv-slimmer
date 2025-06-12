# MKV Slimmer 🦀

A fast, safe Rust tool to analyze and remove unnecessary streams from MKV files based on language preferences.

## Features

- 🚀 **Fast & Safe** - Written in Rust for performance and memory safety
- 📊 **Detailed Analysis** - Display comprehensive stream information with beautiful tables
- 🌍 **Language Filtering** - Filter audio and subtitle tracks by language codes (ordered by preference)
- 🏷️ **Title-Based Selection** - Advanced subtitle filtering by both language and title prefix matching
- ✂️ **Stream Removal** - Remove unwanted streams using mkvmerge with proper error handling
- ⚡ **Smart Optimization** - Automatically detects when processing is unnecessary and uses hardlinking/copying instead
- 🎯 **Default Flag Management** - Properly sets default flags based on language preferences (only one default per type)
- 📁 **Batch Processing** - Process entire directories with optional recursive traversal and glob filtering
- 🔍 **Path Validation** - Comprehensive validation prevents nested source/target scenarios
- ⚙️ **Simplified Configuration** - Easy YAML configuration with language preferences
- 🔍 **Dry-run Mode** - Preview changes without modifying files
- 🎨 **Rich Output** - Colored terminal output with emojis and formatted tables
- 🛡️ **Stream Protection** - Prevents removal of all audio streams, warns about subtitle removal
- 📎 **Attachment Preservation** - All video and attachment streams are always kept

## Installation

### Prerequisites
- Rust (1.70 or later)
- ffprobe (from FFmpeg) - for detailed stream information
- mkvmerge (from MKVToolNix) - **required** for stream removal and modifications

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
# Process MKV file and output to specified directory
cargo run -- movie.mkv /path/to/output/directory

# Or using the compiled binary
./target/release/mkv-slimmer movie.mkv /path/to/output/directory
```

### Advanced usage
```bash
# Keep only English and Japanese audio, English subtitles
cargo run -- movie.mkv /output/dir -a eng -a jpn -s eng

# Dry run with custom config (preview changes without modifying)
cargo run -- movie.mkv /output/dir -n -c custom-settings.yaml

# Keep Spanish and Japanese audio (Spanish will be default as it's listed first)
cargo run -- movie.mkv /output/dir -a spa -a jpn

# Smart optimization: if all streams are kept and defaults are correct, 
# the tool will hardlink/copy instead of using mkvmerge
cargo run -- movie.mkv /output/dir -a eng -a jpn -a spa -s eng -s jpn
```

### Batch Processing
```bash
# Process all MKV files in a directory
cargo run -- /movies/folder /output/dir

# Process recursively (maintains directory structure)
cargo run -- /movies/folder /output/dir --recursive

# Filter files with glob patterns
cargo run -- /movies/folder /output/dir --filter "*.mkv"
cargo run -- /movies/folder /output/dir -r -f "series/**/*.mkv"

# Combine with other options
cargo run -- /movies/folder /output/dir -r -f "*.mkv" -a eng -a jpn -s eng -n
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
  # Subtitle preferences can be:
  # - Language only: "eng"
  # - Language with title prefix: "eng, Dialogue"
  keep_languages:
    - hun    # Hungarian (first preference)
    - und    # Undefined (second preference)
    - "eng, Dialogue"  # English with title starting with "Dialogue"
    - eng    # English (any title)
    - jpn    # Japanese (fourth preference)

# Note: Video and attachment streams are always kept

# Processing options
processing:
  dry_run: false
```

### Language Preference System

- **Ordered Lists**: Languages in `keep_languages` are ordered by preference
- **First Available Wins**: The first language from the list that exists in the video becomes the default
- **Single Default**: Only one stream per type is marked as default (the first found)
- **Automatic Fallback**: If the first preference doesn't exist, it tries the next one

### Title-Based Subtitle Selection

Subtitles can be selected based on both language and title prefix:

- **Language only**: `"eng"` - Matches any English subtitle
- **Language with title**: `"eng, Dialogue"` - Matches English subtitles with titles starting with "Dialogue"
- **Case-insensitive**: Title matching is case-insensitive
- **Prefix matching**: Only the beginning of the title needs to match
- **Commas in titles**: Titles can contain commas - only the first comma separates language from title

Examples:
- `"eng, Full Subtitles"` matches "Full Subtitles - Complete Translation"
- `"jpn, Signs, Songs & Lyrics"` matches "Signs, Songs & Lyrics (Karaoke)"
- `"eng, Dialogue"` does NOT match "Signs & Songs"

## CLI Options

- `<INPUT_PATH>` - Path to the MKV file or directory to process (required)
- `<TARGET_DIRECTORY>` - Directory where the modified MKV files will be created (required)
- `-a, --audio-languages <LANG>` - Languages to keep for audio tracks (ordered by preference, can be specified multiple times)
- `-s, --subtitle-languages <LANG>` - Languages to keep for subtitle tracks (ordered by preference, can be specified multiple times, supports "lang" or "lang, title prefix" format)
- `-r, --recursive` - Process directories recursively (maintains subdirectory structure)
- `-f, --filter <PATTERN>` - Glob pattern to filter files (filename in non-recursive mode, relative path in recursive mode)
- `-n, --dry-run` - Show what would be removed without modifying
- `-c, --config <FILE>` - Alternative config file path (default: settings.yaml)
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Dependencies

### Runtime Dependencies
- **ffprobe** (from FFmpeg) - For detailed stream information
- **mkvmerge** (from MKVToolNix) - **Required** for stream removal and default flag modifications

### Rust Crates
- `clap` - Command-line argument parsing
- `serde` / `serde_yaml` - Configuration management
- `tabled` - Beautiful table formatting
- `colored` - Terminal colors and styling
- `anyhow` - Error handling
- `dialoguer` - Interactive prompts
- `matroska` - MKV parsing (backup to ffprobe)
- `tokio` - Async runtime
- `glob` - Pattern matching for file filtering

## Protection & Validation

The tool includes comprehensive safety measures:

### Stream Protection
- **Audio Protection**: Fails with an error if all audio streams would be removed
- **Subtitle Warning**: Shows a warning if all subtitle streams would be removed (but continues)
- **Video/Attachment Preservation**: All video and attachment streams are always kept

### Path Validation
- **Nested Directory Prevention**: Prevents dangerous source/target relationships
- **Same Directory Detection**: Blocks processing when source and target are identical
- **Infinite Loop Protection**: Stops recursive processing from including its own output

### Validation Examples:
```bash
# Stream protection
Error: All audio streams would be removed. Audio languages to keep: [fre, ger], but available languages are: [jpn, eng]

# Path validation
Error: Target directory cannot be nested within the source path.
Source: /movies/season1
Target: /movies/season1/processed
This would cause the output to be processed as input in recursive mode.
```

## Example Output

### Stream Removal Example:
```
📁 Analyzing: movie.mkv
📂 Target directory: /output/directory
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

📊 Summary:
Original size: 3.2 GB
After processing: 2.1 GB  
Space savings: 1.1 GB (34.4%)
Streams to remove: 1

🎬 Processing streams...
🎯 Keeping 2 stream(s): #0, #1, #2
🔄 Running mkvmerge to create: /output/directory/movie.mkv
📁 Output file: /output/directory/movie.mkv
📊 Original size: 3.2 GB
📊 New size: 2.1 GB
💾 Space saved: 1.1 GB (34.4%)
✅ Stream processing completed successfully!
```

### Smart Optimization Example:
```
📁 Analyzing: movie.mkv
📂 Target directory: /output/directory  
🎵 Audio languages (ordered by preference): eng, jpn, spa
📄 Subtitle languages (ordered by preference): eng, spa

🎬 Processing streams...
🎯 Keeping 4 stream(s): #0, #1, #2, #3
✨ No stream processing needed - linking/copying file instead
🔗 Hard linked to: /output/directory/movie.mkv
📁 Output file: /output/directory/movie.mkv
📊 File size: 3.2 GB
💾 Space saved: 0 B (0.0%) - no processing required
✅ Stream processing completed successfully!
```

## Development Status

- ✅ **Stream Analysis** - Complete with detailed metadata extraction
- ✅ **Language Filtering** - Full support for audio/subtitle filtering  
- ✅ **Configuration System** - Three-layer config with validation
- ✅ **Beautiful Output** - Formatted tables with colors and emojis
- ✅ **Stream Removal** - Complete with mkvmerge integration and error handling
- ✅ **Smart Optimization** - Automatic detection and hardlinking/copying when no processing needed
- ✅ **Default Flag Management** - Proper setting of default flags based on language preferences
- 🚧 **Batch Processing** - Multiple files support
- 🚧 **GUI Interface** - Desktop application

## Performance

The Rust implementation provides significant performance improvements:
- 🚀 **Faster startup** - No interpreter overhead
- 💾 **Lower memory usage** - Efficient memory management
- 🛡️ **Memory safety** - Zero-cost abstractions without runtime panics
- ⚡ **Smart optimization** - Automatic hardlinking/copying when no processing needed (instant operation)
- 🔧 **Efficient mkvmerge usage** - Only processes when necessary, with proper stream selection and default flag management
