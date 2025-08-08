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

### Circular Dependency Resolution

The original codebase had a critical circular dependency issue:
- `batch.rs` imported `analyze_and_process_mkv_file()` from `cli.rs`
- `cli.rs` imported `BatchProcessor` from `batch.rs`

This was resolved by introducing the **processor pattern**:

**src/core/processor.rs**: Shared Processing Logic
- Contains `analyze_and_process_mkv_file()` function extracted from CLI
- Acts as the single source of truth for MKV processing workflow
- Imported by both CLI and batch processing modules
- Eliminates the circular dependency completely

**Benefits of Processor Pattern:**
- ✅ **Single Responsibility**: One function handles the core processing workflow
- ✅ **Code Reuse**: Both CLI and batch processing use the same logic
- ✅ **Consistency**: Identical behavior across single-file and batch modes
- ✅ **Testability**: Core processing logic can be tested independently
- ✅ **Maintainability**: Changes to processing logic only need to be made in one place

### Duplication Elimination
- Extracted print_configuration_info() helper for consistent config display
- Created shared analyze_and_process_mkv_file() in core/processor.rs for unified processing
- BatchProcessor reuses core processing functions instead of duplicating logic
- display_streams parameter controls output verbosity (interactive vs batch mode)

### Module Structure

The codebase now uses a clean hierarchical module structure with one-way dependencies:

```
src/
├── main.rs                    # Entry point only
├── cli/                       # CLI layer
│   ├── args.rs                # Argument parsing and CliArgs struct
│   ├── commands.rs            # Command processing and business logic coordination
│   └── mod.rs                 # Module exports
├── core/                      # Business logic layer  
│   ├── analyzer.rs            # Core MKV analysis and stream processing logic
│   ├── batch.rs               # Batch processing with file discovery and filtering
│   ├── processor.rs           # Shared processing logic (eliminates circular dependencies)
│   └── mod.rs                 # Module exports
├── config/                    # Configuration layer
│   ├── settings.rs            # Config struct and YAML loading
│   ├── preferences.rs         # SubtitlePreference and audio/subtitle config structs
│   └── mod.rs                 # Module exports
├── models/                    # Data structures
│   ├── stream.rs              # StreamInfo and StreamType
│   ├── sonarr.rs              # SonarrContext
│   └── mod.rs                 # Module exports
├── display/                   # Output formatting
│   ├── formatter.rs           # StreamDisplayer and display logic
│   ├── tables.rs              # Table row structs (VideoStreamRow, AudioStreamRow, etc.)
│   └── mod.rs                 # Module exports
└── utils/                     # Utilities
    ├── dependencies.rs        # Dependency checking (mkvmerge, ffprobe)
    ├── validation.rs          # MKV file validation and path validation
    ├── format.rs              # Size formatting utilities
    ├── sonarr.rs              # Sonarr environment collection
    └── mod.rs                 # Module exports
```

**Dependency Flow:**
```
main.rs → cli/ → core/ → {config/, models/, display/, utils/}
```

**Key Architectural Improvements:**
- ✅ Eliminated circular dependencies (batch.rs ↔ cli.rs)
- ✅ One-way dependency hierarchy with clear layers
- ✅ Logical functionality grouping by semantics
- ✅ Better testability with focused modules
- ✅ Clean separation of concerns

## CLI Architecture Refactoring

The CLI layer has been completely restructured for better maintainability and separation of concerns:

### CLI Module Structure
- **src/cli/args.rs**: Argument parsing and validation
  - `create_app()` function builds the clap Command
  - `CliArgs` struct with `parse()` method for clean argument extraction
  - All argument definitions centralized in one location

- **src/cli/commands.rs**: Business logic coordination  
  - `run_cli()` main entry point function
  - `determine_target_type()` for file vs directory detection
  - Path validation and dependency checking
  - Routing logic for single file vs batch processing
  - `print_configuration_info()` helper for consistent config display

- **src/cli/mod.rs**: Clean module exports
  - Exposes only `run` function to main.rs
  - Internal CLI structure hidden from other modules

### Key Improvements
- **Separation of Concerns**: Argument parsing completely separated from business logic
- **Better Testability**: Each component can be tested independently  
- **Reduced Complexity**: Large CLI function split into focused, single-responsibility functions
- **Consistent Error Handling**: Centralized error processing and user-friendly messages
- **Clean Dependencies**: CLI layer cleanly imports from core layer without circular dependencies

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
