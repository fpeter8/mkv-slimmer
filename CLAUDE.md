- Try to use expect instead of unwrap and explain the broken assumption in the error message

## Stream Processing Implementation

- Stream removal functionality has been implemented using mkvmerge
- Smart optimization detects when no processing is needed and uses hardlinking/copying instead
- Proper default flag management ensures only one stream per type is marked as default
- Comprehensive error handling with helpful messages for common failure scenarios
- Forced subtitles no longer automatically preserved - they follow same language/title rules

## Title-Based Subtitle Selection

- Subtitle preferences support both language-only and language+title formats
- Format: "language" or "language, title prefix" (e.g., "eng, Dialogue")
- Title matching is case-insensitive prefix matching
- Titles can contain commas - only the first comma separates language from title
- Configuration parsing uses split_once(',') to handle complex titles
- Backward compatible with existing language-only configurations
- Output display shows "title match" indicator when subtitle is kept due to title

## Batch Processing

- Supports both single file and directory processing modes
- Automatic detection of input type (file vs directory)
- Non-recursive directory processing by default
- Optional recursive mode with --recursive flag maintains directory structure
- Glob pattern filtering with --filter flag for selective file processing
- Comprehensive path validation prevents nested source/target scenarios
- Progress reporting and error collection for batch operations
- BatchProcessor handles file discovery, filtering, and sequential processing

## Path Validation System

- validate_source_target_paths() function prevents dangerous directory relationships
- Detects same directory scenarios (source == target)
- Prevents target nested in source (e.g., /movies → /movies/output)
- Prevents source nested in target (e.g., /movies/season1 → /movies)
- Uses canonical paths to resolve symlinks and relative paths
- Protects against infinite loops in recursive batch processing

## Code Architecture

### Duplication Elimination
- Extracted print_configuration_info() helper for consistent config display
- Created shared analyze_and_process_mkv_file() for core MKV processing logic
- BatchProcessor reuses CLI processing functions instead of duplicating logic
- display_streams parameter controls output verbosity (interactive vs batch mode)

### Module Structure
- src/cli.rs: CLI argument parsing and high-level processing coordination
- src/batch.rs: Batch processing with file discovery and filtering
- src/analyzer.rs: Core MKV analysis and stream processing logic
- src/config.rs: Configuration management with SubtitlePreference support
- src/utils.rs: Shared utilities including validation, dependency checks, and Sonarr environment collection
- src/output.rs: Stream display formatting with title matching indicators
- src/models.rs: Data structures including StreamInfo and SonarrContext

## Target Path Flexibility

- CLI argument changed from target_directory to target_path
- Supports both file and directory targets for single file processing
- Target type detection using file extensions and path characteristics
- Input/output validation for supported combinations:
  - File → File: Uses provided path instead of input filename
  - File → Directory: Appends input filename to output directory (original behavior)
  - Directory → Directory: Batch processing (original behavior)
  - Directory → File: Rejected with clear error message
- Enhanced path validation handles non-existent target files

## Sonarr Integration

### Environment Variable Collection
- SonarrContext struct stores all Sonarr environment variables as raw strings
- Case-insensitive parsing handles discrepancies between documentation and implementation
- Comprehensive collection of all variables from Sonarr specification:
  - File paths (source_path, destination_path)
  - Instance info (instance_name, application_url, transfer_mode)
  - Series metadata (id, title, path, external IDs, type, language, genres, tags)
  - Episode info (count, IDs, season/episode numbers, air dates, titles, overviews)
  - Media info (quality, codecs, dimensions, languages, subtitles)
  - Custom formats and download information
  - Deleted files information for upgrades

### Transfer Mode Support
- Respects Sonarr_TransferMode preference when available
- Supported modes:
  - **Move**: Uses std::fs::rename() with cross-filesystem fallback (copy+delete)
  - **Copy**: Uses std::fs::copy()
  - **HardLink**: Uses std::fs::hard_link() (fails if not possible)
  - **HardLinkOrCopy**: Default behavior (hard link with copy fallback)
  - **Unknown modes**: Falls back to default with warning
- Cross-filesystem move handling prevents failures on different storage devices

### Sonarr Communication
- Outputs proper Sonarr import script commands to stdout:
  - **[MoveStatus] MoveComplete**: When no processing needed (file unchanged)
  - **[MoveStatus] RenameRequested**: When file modified (streams changed)
- No Sonarr output during dry-run mode (file not actually created)
- Integration throughout analyzer workflow:
  - handle_no_processing_needed() → MoveComplete
  - process_streams() → RenameRequested

### Integration Architecture
- SonarrContext added to MkvAnalyzer struct
- Passed through all processing paths (CLI, batch, analyzer)
- Environment collection at CLI setup stage (after config validation)
- Optional integration - works normally without Sonarr environment
