- Try to use expect instead of unwrap and explain the broken assumption in the error message

## Stream Processing Implementation

- Stream removal functionality has been implemented using mkvmerge
- Smart optimization detects when no processing is needed and uses hardlinking/copying instead
- Proper default flag management ensures only one stream per type is marked as default
- Comprehensive error handling with helpful messages for common failure scenarios

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
