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
- src/utils.rs: Shared utilities including validation and dependency checks
- src/output.rs: Stream display formatting with title matching indicators
