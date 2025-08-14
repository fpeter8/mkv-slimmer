use anyhow::{Context, Result};
use std::path::Path;

/// Checks if a file is a valid MKV file without throwing errors
///
/// Performs basic validation including existence, file type, and extension checks.
/// This is a non-throwing version suitable for filtering file lists.
///
/// # Arguments
/// * `file_path` - Path to the file to validate
///
/// # Returns
/// `true` if the file appears to be a valid MKV file, `false` otherwise
///
/// # Examples
/// ```rust
/// use mkv_slimmer::utils::is_valid_mkv_file;
/// use std::path::Path;
///
/// assert_eq!(is_valid_mkv_file("movie.mkv"), false); // File doesn't exist
/// assert_eq!(is_valid_mkv_file("document.txt"), false); // Wrong extension
/// ```
pub fn is_valid_mkv_file<P: AsRef<Path>>(file_path: P) -> bool {
    let path = file_path.as_ref();
    
    // Check if file exists and is a file
    if !path.exists() || !path.is_file() {
        return false;
    }
    
    // Check file extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if !["mkv", "mka", "mks"].contains(&ext_str.as_str()) {
            return false;
        }
    } else {
        return false;
    }
    
    // Check if file is readable
    std::fs::File::open(path).is_ok()
}

/// Validate that the file is a valid MKV file
pub fn validate_mkv_file<P: AsRef<Path>>(file_path: P) -> Result<()> {
    let path = file_path.as_ref();
    
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }
    
    if !path.is_file() {
        anyhow::bail!("Not a file: {}", path.display());
    }
    
    // Check file extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if !["mkv", "mka", "mks"].contains(&ext_str.as_str()) {
            anyhow::bail!("Not an MKV file: {}", path.display());
        }
    } else {
        anyhow::bail!("File has no extension: {}", path.display());
    }
    
    // Check file is readable
    std::fs::File::open(path)
        .with_context(|| format!("Cannot read file: {}", path.display()))?;
    
    // Check for EBML header (MKV signature)
    let mut file = std::fs::File::open(path)?;
    let mut header = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut header)
        .with_context(|| format!("Cannot read MKV header from: {}", path.display()))?;
    
    if header != [0x1a, 0x45, 0xdf, 0xa3] {
        anyhow::bail!("Invalid MKV file format: {}", path.display());
    }
    
    Ok(())
}

/// Validates that source and target paths are safe for batch processing
///
/// This function prevents dangerous directory relationships that could cause
/// infinite loops or data corruption during recursive batch processing:
/// - Same directory (source == target)  
/// - Target nested in source (/movies → /movies/output)
/// - Source nested in target (/movies/season1 → /movies)
///
/// Uses canonical paths to resolve symlinks and relative paths properly.
///
/// # Arguments
/// * `source_path` - The source directory or file path
/// * `target_path` - The target directory or file path  
///
/// # Returns
/// `Ok(())` if paths are safe, `Err` describing the problem if not
///
/// # Examples
/// ```rust
/// use mkv_slimmer::utils::validate_source_target_paths;
/// use std::path::Path;
///
/// // This would fail - same directory
/// let result = validate_source_target_paths(
///     Path::new("/movies"), 
///     Path::new("/movies")
/// );
/// assert!(result.is_err());
/// ```
///
/// This prevents scenarios like:
/// - Source: /movies/season1, Target: /movies/season1/processed
/// - Source: /movies/season1/episode1.mkv, Target: /movies
/// - Source: /movies, Target: /movies/processed
pub fn validate_source_target_paths(source_path: &Path, target_path: &Path) -> Result<()> {
    // Canonicalize source path to resolve symlinks and relative paths
    let source_canonical = source_path.canonicalize()
        .with_context(|| format!("Could not resolve source path: {}", source_path.display()))?;
    
    // For target path, handle the case where it might not exist yet
    let target_canonical = if target_path.exists() {
        target_path.canonicalize()
            .with_context(|| format!("Could not resolve target path: {}", target_path.display()))?
    } else {
        // If target doesn't exist, canonicalize its parent directory
        let parent = target_path.parent()
            .context("Target path has no parent directory")?;
        let parent_canonical = parent.canonicalize()
            .with_context(|| format!("Could not resolve target parent path: {}", parent.display()))?;
        parent_canonical.join(target_path.file_name().unwrap_or_default())
    };
    
    // Check if paths are exactly the same
    if source_canonical == target_canonical {
        anyhow::bail!(
            "Source and target paths cannot be the same.\nSource: {}\nTarget: {}", 
            source_path.display(), 
            target_path.display()
        );
    }
    
    // Check if target is nested within source
    if target_canonical.starts_with(&source_canonical) {
        anyhow::bail!(
            "Target path cannot be nested within the source path.\nSource: {}\nTarget: {}\nThis would cause the output to be processed as input in recursive mode.", 
            source_path.display(), 
            target_path.display()
        );
    }
    
    // Check if source is nested within target
    if source_canonical.starts_with(&target_canonical) {
        anyhow::bail!(
            "Source path cannot be nested within the target path.\nSource: {}\nTarget: {}\nThis would overwrite source files during processing.", 
            source_path.display(), 
            target_path.display()
        );
    }
    
    Ok(())
}