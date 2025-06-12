# Batch Mode Implementation Plan

## Overview
Add batch processing capability to mkv-slimmer, allowing it to process multiple MKV files in a directory with optional recursive traversal and glob filtering.

## Requirements
1. First parameter can be either:
   - A file (existing behavior)
   - A folder (new batch mode)
2. In folder mode:
   - List all supported files (.mkv, .mka, .mks) non-recursively by default
   - Use `validate_mkv_file()` as a filter instead of throwing errors
   - Apply the same configuration to each file
3. Optional recursive flag (`-r`, `--recursive`):
   - Walk directories recursively
   - Maintain subfolder structure in target directory
4. Optional filter parameter (`-f`, `--filter`):
   - Use glob patterns to filter files
   - Match against filenames in non-recursive mode
   - Match against relative paths in recursive mode

## Implementation Steps

### 1. Update CLI Arguments
**File: `src/cli.rs`**
- Change `mkv_file` argument to `input_path` that accepts both files and directories
- Add `--recursive` flag (short: `-r`)
- Add `--filter` parameter (short: `-f`) that accepts a glob pattern
- Update help text to reflect new functionality

### 2. Create Batch Processing Module
**New file: `src/batch.rs`**
- Create a new module for batch processing logic
- Add to `main.rs` module declarations

Key functions:
```rust
pub struct BatchProcessor {
    input_path: PathBuf,
    target_directory: PathBuf,
    recursive: bool,
    filter_pattern: Option<String>,
    config: Config,
}

impl BatchProcessor {
    pub fn new(...) -> Self
    pub async fn process(&self) -> Result<()>
    fn collect_mkv_files(&self) -> Result<Vec<PathBuf>>
    fn matches_filter(&self, file_path: &Path) -> bool
    async fn process_single_file(&self, file_path: PathBuf) -> Result<()>
    fn calculate_target_path(&self, source_file: &Path) -> Result<PathBuf>
}
```

### 3. Refactor validate_mkv_file
**File: `src/utils.rs`**
- Split into two functions:
  - `validate_mkv_file()` - keeps existing behavior (throws errors)
  - `is_valid_mkv_file()` - returns bool, used for filtering in batch mode

### 4. Update Main CLI Logic
**File: `src/cli.rs`**
- Detect if input is file or directory
- If file: use existing single-file logic
- If directory: create BatchProcessor and run batch mode
- Move single-file processing logic to a separate function for reuse

### 5. Directory Structure Preservation
**File: `src/batch.rs`**
- In recursive mode, calculate relative path from source to file
- Create matching subdirectories in target directory
- Example: `source/sub/file.mkv` → `target/sub/file.mkv`

### 6. Progress Reporting
**File: `src/batch.rs`**
- Show overall progress (e.g., "Processing file 3 of 10")
- Clear separation between files in output
- Summary at the end showing success/failure counts

### 7. Error Handling
- Continue processing other files if one fails
- Collect all errors and report at the end
- Option to stop on first error (future enhancement)

## File Changes Summary

### Modified Files:
1. **src/cli.rs**
   - Update argument parsing
   - Add routing logic for file vs directory
   - Extract single-file processing to separate function

2. **src/utils.rs**
   - Add `is_valid_mkv_file()` function
   - Keep `validate_mkv_file()` for backward compatibility

3. **src/main.rs**
   - Add `mod batch;` declaration

### New Files:
1. **src/batch.rs**
   - Complete batch processing implementation
   - File collection with filtering
   - Recursive directory walking
   - Target path calculation

## CLI Examples

```bash
# Process single file (existing behavior)
mkv-slimmer movie.mkv /output/dir

# Process all MKV files in a directory
mkv-slimmer /movies/dir /output/dir

# Process recursively
mkv-slimmer /movies/dir /output/dir --recursive

# Process with filter (non-recursive)
mkv-slimmer /movies/dir /output/dir --filter "*.mkv"

# Process with filter (recursive, match full relative path)
mkv-slimmer /movies/dir /output/dir --recursive --filter "series/**/*.mkv"

# Combine with existing options
mkv-slimmer /movies/dir /output/dir -r -f "*.mkv" -a eng -a jpn -s eng
```

## Testing Strategy

1. **Unit Tests**
   - Test file filtering logic
   - Test glob pattern matching
   - Test target path calculation
   - Test recursive file collection

2. **Integration Tests**
   - Test batch processing with mock files
   - Test error handling (mix of valid/invalid files)
   - Test directory structure preservation

3. **Manual Testing**
   - Test with real MKV files
   - Test with nested directory structures
   - Test with various glob patterns
   - Test error scenarios (permissions, disk space, etc.)

## Future Enhancements (Not in initial implementation)
- Parallel processing of multiple files
- Option to stop on first error
- Detailed progress bar for each file
- Resume capability for interrupted batch jobs
- Exclude patterns in addition to include filters