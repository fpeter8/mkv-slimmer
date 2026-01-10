# Code Deduplication Plan for mkv-slimmer

**Date:** 2026-01-09
**Status:** ✅ **MOSTLY COMPLETED** (2026-01-09)
**Objective:** Eliminate duplicate code and functionality, reduce codebase size by ~1000 lines while maintaining all functionality.

## Completion Status

✅ **COMPLETED (2026-01-09):**
- **DUP-01:** Removed MkvAnalyzer struct (~960 lines)
- **DUP-02:** Removed legacy wrapper function (~40 lines)
- **DUP-03 to DUP-11, DUP-16:** Automatically resolved by DUP-01 completion
- **DUP-12:** Extracted title matching to SubtitlePreference method (~42 lines)
- **DUP-13:** Extracted stream type filtering pattern (~55 lines)

**Total Lines Removed: ~1,097 lines**

⏳ **OPTIONAL (Not Yet Completed):**
- **DUP-14:** Refactor long command builder function (improves readability)
- **DUP-15:** Centralize Sonarr status output (minor cleanup)

## Results Achieved

✅ **Successfully migrated from dual architecture to single ProcessingTask pattern**
- Removed entire MkvAnalyzer struct and all duplicate methods
- Single-file and batch processing now use identical code paths
- analyzer.rs reduced from 1704 lines to ~730 lines (57% reduction)

✅ **Code quality improvements:**
- Eliminated circular dependencies
- Created reusable helper functions (separate_streams_by_type, matches_title)
- Improved maintainability - bug fixes only need to be applied once
- Better code organization and separation of concerns

✅ **All tests passing** - cargo test and cargo fmt completed successfully

---

## Completed Work Summary

All major duplications have been eliminated. The following sections document what was completed:

### Phase 1: Legacy Architecture Removal (COMPLETED)

**DUP-01: MkvAnalyzer Struct Removal** (~960 lines removed)
- Deleted entire `MkvAnalyzer` struct and implementation
- Migrated all functionality to standalone ProcessingTask-based functions
- File: `src/core/analyzer.rs` reduced from 1704 to ~730 lines

**DUP-02: Legacy Wrapper Function** (~40 lines removed)
- Removed `analyze_and_process_mkv_file()` from processor.rs
- Updated batch.rs to use ProcessingTask directly
- Unified single-file and batch processing code paths

**DUP-03 through DUP-11, DUP-16:** Automatically resolved by DUP-01 completion
- All MkvAnalyzer methods had duplicate standalone versions
- Removing the struct eliminated all these duplicates

### Phase 2: Pattern Extraction (COMPLETED)

**DUP-12: Title Prefix Matching** (~42 lines removed)
- Added `matches_title()` method to `SubtitlePreference` in config/preferences.rs
- Replaced 6 duplicate occurrences across analyzer.rs and formatter.rs
- Single source of truth for subtitle title matching logic

**DUP-13: Stream Type Filtering** (~55 lines removed)
- Created `StreamsByType` struct and `separate_streams_by_type()` function
- Replaced 10+ duplicate filtering patterns in analyzer.rs
- Significantly cleaned up `build_mkvmerge_command_for_task()`

---

## Optional Future Improvements

These improvements are non-critical and can be implemented if desired to further improve code quality:

### DUP-14: Refactor Long Command Builder Function

**Location:** `src/core/analyzer.rs::build_mkvmerge_command_for_task()`
**Size:** ~160 lines
**Priority:** Optional (improves readability, not critical)

**Current State:**
The function handles multiple responsibilities:
1. Separate streams by type - ✅ Already extracted via DUP-13
2. Determine if filtering is needed (10 lines)
3. Build track selection arguments (80 lines)
4. Build default flag arguments (40 lines)
5. Build forced flag arguments (10 lines)
6. Assemble final command

**Proposed Improvement:**
Break into smaller, focused helper functions:
- `is_filtering_needed()` - Determine if we're filtering vs keeping all streams
- `add_track_selection_args()` - Add --audio-tracks, --subtitle-tracks, etc.
- `add_default_track_flags()` - Set default track flags for audio/subtitles
- `add_forced_display_flags()` - Set forced flags for subtitles

**Benefits:**
- Each function has single responsibility
- Easier to test individual components
- Better code readability

---

### DUP-15: Centralize Sonarr Status Output

**Location:** `src/core/analyzer.rs`
**Occurrences:** 2 locations
**Priority:** Optional (minor cleanup)

**Current State:**
Direct println! calls for Sonarr status:
- Line ~1084: `println!("[MoveStatus] RenameRequested");`
- Line ~1151: `println!("[MoveStatus] MoveComplete");`

**Proposed Improvement:**
Create Sonarr output utilities in `src/utils/sonarr.rs`:

```rust
pub enum SonarrMoveStatus {
    /// File was not modified, can be moved/hardlinked as-is
    MoveComplete,
    /// File was modified (streams changed), needs rename from Sonarr
    RenameRequested,
}

pub fn output_sonarr_move_status(status: SonarrMoveStatus) {
    match status {
        SonarrMoveStatus::MoveComplete => {
            println!("[MoveStatus] MoveComplete");
        }
        SonarrMoveStatus::RenameRequested => {
            println!("[MoveStatus] RenameRequested");
        }
    }
}
```

**Benefits:**
- Centralized Sonarr communication
- Clear enum makes intent obvious
- Easy to extend with new statuses if needed
- Better documentation

---

## Architecture Documentation

### Module Structure After Deduplication

```
src/
├── core/
│   ├── analyzer.rs      # ~730 lines (was 1704)
│   ├── processor.rs     # Shared processing logic
│   └── batch.rs         # Batch processing
├── config/
│   ├── settings.rs
│   └── preferences.rs   # Added matches_title() method
├── models/
│   ├── stream.rs
│   ├── sonarr.rs
│   └── task.rs          # ProcessingTask struct
└── display/
    └── formatter.rs     # Uses matches_title() method
```

### Key Improvements:
- **Single Architecture:** ProcessingTask-based only
- **No Circular Dependencies:** Clean one-way dependencies
- **Reusable Helpers:** matches_title(), separate_streams_by_type()
- **Unified Processing:** Same code for single-file and batch modes

---

## Testing Notes

All completed work has been verified with:
- ✅ `cargo check` - No compilation errors
- ✅ `cargo build` - Successful build
- ✅ `cargo test` - All tests passing
- ✅ `cargo fmt` - Code formatted

Manual testing recommended for:
- Single file processing
- Directory batch processing
- Stream filtering logic
- Default track selection
- Sonarr integration

---

## References

- Original plan: `/home/fpeter/.claude/plans/jiggly-conjuring-honey.md`
- Total lines removed: **~1,097 lines**
- Primary file affected: `src/core/analyzer.rs` (57% reduction)
- Files modified: 6 files across core/, config/, and display/ modules
