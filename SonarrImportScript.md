# Sonarr Import Script Specification

This document provides a complete specification for creating custom import scripts that integrate with Sonarr's media file processing pipeline.

## Overview

Sonarr's Import Script feature allows external scripts to participate in the episode import process. These scripts can modify files, specify additional files to import, and control the import workflow. The script is executed after Sonarr has identified and parsed an episode file but before it's moved to its final location.

## Prerequisites

- Enable "Use Script Import" in Sonarr settings
- Set "Script Import Path" to your executable script
- Script must be executable and return exit code 0 for success

## Script Execution Flow

1. Sonarr downloads/finds a new episode file
2. Sonarr parses the file and matches it to series/episodes
3. **Your script is called** with source and destination paths as arguments
4. Script receives extensive metadata via environment variables
5. Script processes the file and outputs control commands
6. Sonarr processes script output and continues import based on the decision

## Command Line Arguments

Your script receives exactly 2 arguments:

```bash
script.sh "source_path" "destination_path"
```

- `$1` - Source file path (current location)
- `$2` - Destination file path (where Sonarr plans to move it)

## Environment Variables

### File Paths
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_SourcePath` | Current file location | `/downloads/Show.S01E01.mkv` |
| `Sonarr_DestinationPath` | Planned final location | `/tv/Show/Season 01/Show - S01E01 - Episode Title.mkv` |

### Instance Information
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_InstanceName` | Sonarr instance name | `Sonarr` |
| `Sonarr_ApplicationUrl` | Sonarr base URL | `http://localhost:8989` |
| `Sonarr_TransferMode` | Transfer method | `Move`, `Copy`, `HardLink` |

### Series Metadata
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_Series_Id` | Internal series ID | `123` |
| `Sonarr_Series_Title` | Series title | `Breaking Bad` |
| `Sonarr_Series_TitleSlug` | URL-safe title | `breaking-bad` |
| `Sonarr_Series_Path` | Series root folder | `/tv/Breaking Bad` |
| `Sonarr_Series_TvdbId` | TVDB ID | `81189` |
| `Sonarr_Series_TvMazeId` | TVMaze ID | `169` |
| `Sonarr_Series_TmdbId` | TMDB ID | `1396` |
| `Sonarr_Series_ImdbId` | IMDB ID | `tt0903747` |
| `Sonarr_Series_Type` | Series type | `Standard`, `Daily`, `Anime` |
| `Sonarr_Series_OriginalLanguage` | Original language (3-letter) | `eng` |
| `Sonarr_Series_Genres` | Pipe-separated genres | `Drama|Crime|Thriller` |
| `Sonarr_Series_Tags` | Pipe-separated tag labels | `HD|Favorite` |

### Episode Information
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_EpisodeFile_EpisodeCount` | Number of episodes in file | `1` |
| `Sonarr_EpisodeFile_EpisodeIds` | Comma-separated episode IDs | `456,457` |
| `Sonarr_EpisodeFile_SeasonNumber` | Season number | `1` |
| `Sonarr_EpisodeFile_EpisodeNumbers` | Comma-separated episode numbers | `1,2` |
| `Sonarr_EpisodeFile_EpisodeAirDates` | Comma-separated air dates | `2008-01-20,2008-01-27` |
| `Sonarr_EpisodeFile_EpisodeAirDatesUtc` | UTC air dates | `2008-01-21T02:00:00Z,2008-01-28T02:00:00Z` |
| `Sonarr_EpisodeFile_EpisodeTitles` | Pipe-separated episode titles | `Pilot|Cat's in the Bag...` |
| `Sonarr_EpisodeFile_EpisodeOverviews` | Pipe-separated episode summaries | `Walter White, a struggling...` |

### Quality and Media Information
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_EpisodeFile_Quality` | Quality profile name | `HDTV-720p` |
| `Sonarr_EpisodeFile_QualityVersion` | Quality revision | `1` |
| `Sonarr_EpisodeFile_ReleaseGroup` | Release group | `DIMENSION` |
| `Sonarr_EpisodeFile_SceneName` | Original scene name | `breaking.bad.s01e01.720p.hdtv.x264-dimension` |
| `Sonarr_EpisodeFile_MediaInfo_AudioChannels` | Audio channel count | `6` |
| `Sonarr_EpisodeFile_MediaInfo_AudioCodec` | Audio codec | `AC3` |
| `Sonarr_EpisodeFile_MediaInfo_AudioLanguages` | Audio languages | `English / Spanish` |
| `Sonarr_EpisodeFile_MediaInfo_Languages` | All languages | `English / Spanish` |
| `Sonarr_EpisodeFile_MediaInfo_Height` | Video height | `720` |
| `Sonarr_EpisodeFile_MediaInfo_Width` | Video width | `1280` |
| `Sonarr_EpisodeFile_MediaInfo_Subtitles` | Subtitle languages | `English / Spanish` |
| `Sonarr_EpisodeFile_MediaInfo_VideoCodec` | Video codec | `x264` |
| `Sonarr_EpisodeFile_MediaInfo_VideoDynamicRangeType` | HDR type | `SDR`, `HDR10`, `DV` |

### Custom Formats
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_EpisodeFile_CustomFormat` | Pipe-separated custom formats | `x264|HDTV` |
| `Sonarr_EpisodeFile_CustomFormatScore` | Total custom format score | `85` |

### Download Information
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_Download_Client` | Download client name | `SABnzbd` |
| `Sonarr_Download_Client_Type` | Download client type | `Sabnzbd` |
| `Sonarr_Download_Id` | Download ID | `SABnzbd_nzo_12345` |

### Deleted Files (for upgrades)
| Variable | Description | Example |
|----------|-------------|---------|
| `Sonarr_DeletedRelativePaths` | Pipe-separated relative paths | `Season 01/Old File.mkv` |
| `Sonarr_DeletedPaths` | Pipe-separated full paths | `/tv/Show/Season 01/Old File.mkv` |
| `Sonarr_DeletedDateAdded` | Pipe-separated add dates | `2023-01-15T10:30:00Z` |
| `Sonarr_DeletedRecycleBinPaths` | Pipe-separated recycle paths | `/recycle/Old File.mkv` |

## Script Output Commands

Your script communicates with Sonarr by writing specific commands to stdout. Each command must be on its own line.

### Media File Commands

#### `[MediaFile] /path/to/file`
Specifies the final media file to import. Only one media file can be specified.

**Requirements:**
- File must exist
- Must have a valid media file extension
- Path should be absolute

**Example:**
```bash
echo "[MediaFile] /processed/Show.S01E01.processed.mkv"
```

### Extra File Commands

#### `[ExtraFile] /path/to/extra`
Adds an extra file to be imported alongside the main media file. Can be used multiple times.

**Common extra files:**
- Subtitles (`.srt`, `.ass`, `.sup`)
- Artwork (`.jpg`, `.png`)
- NFO files (`.nfo`)
- Other metadata files

**Example:**
```bash
echo "[ExtraFile] /subtitles/Show.S01E01.en.srt"
echo "[ExtraFile] /artwork/Show.S01E01.thumb.jpg"
```

### Import Control Commands

#### `[PreventExtraImport]`
Prevents Sonarr from automatically detecting and importing extra files. Use when you want full control over which files are imported.

**Example:**
```bash
echo "[PreventExtraImport]"
```

### Move Status Commands

#### `[MoveStatus] MoveComplete`
Indicates the script has completed all file operations. Sonarr will proceed with normal import processing. This is the default behavior.

**Example:**
```bash
echo "[MoveStatus] MoveComplete"
```

#### `[MoveStatus] RenameRequested`
Requests that Sonarr rename the file according to its naming scheme. Use this when your script modifies the file but wants Sonarr to handle the final naming.

**Example:**
```bash
echo "[MoveStatus] RenameRequested"
```

#### `[MoveStatus] DeferMove`
Tells Sonarr to skip moving the file. Use this when your script will handle file placement or when you want to abort the import.

**Example:**
```bash
echo "[MoveStatus] DeferMove"
```

## Example Scripts

### Example 1: Basic Transcoding Script

This script transcodes video files to H.264 if they're not already in that codec.

```bash
#!/bin/bash
# Simple transcoding script

SOURCE_PATH="$1"
DEST_PATH="$2"

# Log the script execution
echo "Processing: $SOURCE_PATH -> $DEST_PATH" >&2

# Check if file is already H.264
if echo "$Sonarr_EpisodeFile_MediaInfo_VideoCodec" | grep -qi "x264\|h264"; then
    echo "File is already H.264, no transcoding needed" >&2
    # Use original file
    echo "[MediaFile] $SOURCE_PATH"
    echo "[MoveStatus] MoveComplete"
else
    echo "Transcoding $SOURCE_PATH to H.264" >&2
    
    # Create output filename
    OUTPUT_DIR="/tmp/transcoded"
    mkdir -p "$OUTPUT_DIR"
    BASENAME=$(basename "$SOURCE_PATH" .mkv)
    OUTPUT_FILE="$OUTPUT_DIR/${BASENAME}.h264.mkv"
    
    # Transcode using ffmpeg
    if ffmpeg -i "$SOURCE_PATH" -c:v libx264 -crf 23 -c:a copy "$OUTPUT_FILE" 2>&1; then
        echo "Transcoding successful" >&2
        # Tell Sonarr to use the transcoded file
        echo "[MediaFile] $OUTPUT_FILE"
        echo "[MoveStatus] RenameRequested"  # Ask Sonarr to rename it properly
    else
        echo "Transcoding failed, using original file" >&2
        echo "[MediaFile] $SOURCE_PATH"
        echo "[MoveStatus] MoveComplete"
    fi
fi
```

**What this script does:**
- Checks video codec from environment variable
- If not H.264, transcodes the file using ffmpeg
- Returns the transcoded file path with `[MediaFile]`
- Uses `[MoveStatus] RenameRequested` so Sonarr handles final naming

### Example 2: Subtitle Download Script

This script automatically downloads subtitles for episodes.

```bash
#!/bin/bash
# Subtitle download script

SOURCE_PATH="$1"
DEST_PATH="$2"

echo "Checking for subtitles for: $Sonarr_Series_Title S${Sonarr_EpisodeFile_SeasonNumber}E${Sonarr_EpisodeFile_EpisodeNumbers}" >&2

# Create temp directory for subtitles
SUBTITLE_DIR="/tmp/subtitles"
mkdir -p "$SUBTITLE_DIR"

# Download subtitles using subliminal (example)
SERIES_TITLE="$Sonarr_Series_Title"
SEASON="$Sonarr_EpisodeFile_SeasonNumber"
EPISODE="$Sonarr_EpisodeFile_EpisodeNumbers"

# Simulate subtitle download (replace with actual subtitle downloader)
SUBTITLE_FILE="$SUBTITLE_DIR/${SERIES_TITLE}.S${SEASON}E${EPISODE}.en.srt"

if command -v subliminal >/dev/null 2>&1; then
    echo "Downloading subtitles..." >&2
    if subliminal download -l en "$SOURCE_PATH" -s "$SUBTITLE_DIR" 2>&1; then
        echo "Subtitles downloaded successfully" >&2
        # Add the subtitle file as an extra
        echo "[ExtraFile] $SUBTITLE_FILE"
    else
        echo "Subtitle download failed" >&2
    fi
else
    echo "Subliminal not available, skipping subtitle download" >&2
fi

# Use original media file
echo "[MediaFile] $SOURCE_PATH"
echo "[MoveStatus] MoveComplete"
```

**What this script does:**
- Uses series/episode info from environment variables
- Downloads subtitles using an external tool
- Adds subtitle file with `[ExtraFile]`
- Proceeds with normal import using original file

### Example 3: Quality Control Script

This script rejects files that don't meet quality standards.

```bash
#!/bin/bash
# Quality control script

SOURCE_PATH="$1"
DEST_PATH="$2"

MIN_WIDTH=1280
MIN_HEIGHT=720
REQUIRED_CODEC="x264"

echo "Quality checking: $SOURCE_PATH" >&2

# Check video dimensions
WIDTH="$Sonarr_EpisodeFile_MediaInfo_Width"
HEIGHT="$Sonarr_EpisodeFile_MediaInfo_Height"
CODEC="$Sonarr_EpisodeFile_MediaInfo_VideoCodec"

echo "Video specs: ${WIDTH}x${HEIGHT}, codec: $CODEC" >&2

# Quality checks
if [ "$WIDTH" -lt "$MIN_WIDTH" ] || [ "$HEIGHT" -lt "$MIN_HEIGHT" ]; then
    echo "REJECTED: Resolution ${WIDTH}x${HEIGHT} below minimum ${MIN_WIDTH}x${MIN_HEIGHT}" >&2
    # Defer the move - this effectively rejects the file
    echo "[MoveStatus] DeferMove"
    exit 0
fi

if ! echo "$CODEC" | grep -qi "$REQUIRED_CODEC"; then
    echo "REJECTED: Codec '$CODEC' not acceptable (requires $REQUIRED_CODEC)" >&2
    echo "[MoveStatus] DeferMove"
    exit 0
fi

# File passes quality checks
echo "ACCEPTED: File meets quality standards" >&2
echo "[MediaFile] $SOURCE_PATH"
echo "[MoveStatus] MoveComplete"
```

**What this script does:**
- Checks video resolution and codec from environment variables
- Rejects files below quality thresholds using `[MoveStatus] DeferMove`
- Accepts qualifying files normally

### Example 4: Custom Organization Script

This script handles file organization with custom logic.

```bash
#!/bin/bash
# Custom organization script

SOURCE_PATH="$1"
DEST_PATH="$2"

# Custom folder structure based on quality
QUALITY="$Sonarr_EpisodeFile_Quality"
SERIES_PATH="$Sonarr_Series_Path"

echo "Organizing with custom structure for quality: $QUALITY" >&2

# Create quality-based subfolder
case "$QUALITY" in
    *"2160p"*|*"4K"*)
        QUALITY_FOLDER="4K"
        ;;
    *"1080p"*)
        QUALITY_FOLDER="1080p"
        ;;
    *"720p"*)
        QUALITY_FOLDER="720p"
        ;;
    *)
        QUALITY_FOLDER="SD"
        ;;
esac

# Build custom path
CUSTOM_DIR="$SERIES_PATH/Quality/$QUALITY_FOLDER"
mkdir -p "$CUSTOM_DIR"

# Generate filename
SEASON="$Sonarr_EpisodeFile_SeasonNumber"
EPISODE="$Sonarr_EpisodeFile_EpisodeNumbers"
TITLE="$Sonarr_EpisodeFile_EpisodeTitles"
EXT="${SOURCE_PATH##*.}"

CUSTOM_FILE="$CUSTOM_DIR/S${SEASON}E${EPISODE} - ${TITLE}.${EXT}"

# Copy file to custom location
echo "Moving to custom location: $CUSTOM_FILE" >&2
cp "$SOURCE_PATH" "$CUSTOM_FILE"

# Tell Sonarr about the new location
echo "[MediaFile] $CUSTOM_FILE"
echo "[MoveStatus] MoveComplete"

# Prevent automatic extra file detection since we're using custom structure
echo "[PreventExtraImport]"
```

**What this script does:**
- Creates custom folder structure based on video quality
- Generates custom filename using episode metadata
- Moves file to custom location and reports new path
- Uses `[PreventExtraImport]` to disable automatic extra file detection

## Error Handling

### Exit Codes
- **0**: Success - Sonarr will process script output
- **Non-zero**: Failure - Sonarr will throw ScriptImportException

### Error Examples
```bash
# Check for required tools
if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "ERROR: ffmpeg not found" >&2
    exit 1
fi

# Validate file exists
if [ ! -f "$SOURCE_PATH" ]; then
    echo "ERROR: Source file does not exist: $SOURCE_PATH" >&2
    exit 1
fi
```

## Testing Your Script

### Manual Testing
```bash
# Set environment variables manually
export Sonarr_Series_Title="Test Show"
export Sonarr_EpisodeFile_SeasonNumber="1"
export Sonarr_EpisodeFile_EpisodeNumbers="1"
# ... set other variables

# Run your script
./your_script.sh "/path/to/test/file.mkv" "/path/to/destination.mkv"
```

### Common Debugging
```bash
# Add debugging output to stderr (won't interfere with Sonarr)
echo "DEBUG: Processing $1" >&2
echo "DEBUG: Series: $Sonarr_Series_Title" >&2
echo "DEBUG: Quality: $Sonarr_EpisodeFile_Quality" >&2

# Log all environment variables
env | grep "^Sonarr_" >&2
```

## Best Practices

1. **Always validate inputs** - Check file existence and permissions
2. **Use stderr for logging** - stdout is parsed by Sonarr
3. **Handle failures gracefully** - Provide fallback behavior
4. **Test with various file types** - Different codecs, qualities, etc.
5. **Be mindful of performance** - Long-running scripts delay imports
6. **Clean up temporary files** - Remove any files you create
7. **Use absolute paths** - Relative paths may not work as expected

## Troubleshooting

### Common Issues

1. **Script not executing**: Check file permissions (`chmod +x script.sh`)
2. **No output processed**: Ensure exit code is 0
3. **Files not found**: Use absolute paths in output commands
4. **Import hanging**: Check for infinite loops or long-running processes
5. **Multiple media files error**: Only output one `[MediaFile]` command

### Logging
Enable debug logging in Sonarr to see script execution details and output parsing.