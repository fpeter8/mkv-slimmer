/// Formats a byte count into a human-readable size string
///
/// Converts bytes into appropriate units (B, KB, MB, GB, TB) with one decimal place.
/// Uses binary (1024) conversion rather than decimal (1000).
///
/// # Arguments
/// * `size_bytes` - The size in bytes to format
///
/// # Returns
/// A formatted string with the size and appropriate unit
///
/// # Examples
/// ```rust
/// use mkv_slimmer::utils::format_size;
///
/// assert_eq!(format_size(0), "0.0 B");
/// assert_eq!(format_size(1024), "1.0 KB"); 
/// assert_eq!(format_size(1536), "1.5 KB");
/// assert_eq!(format_size(1048576), "1.0 MB");
/// ```
pub fn format_size(size_bytes: u64) -> String {
    const SIZE_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_value = size_bytes as f64;
    let mut current_unit_index = 0;
    
    while size_value >= 1024.0 && current_unit_index < SIZE_UNITS.len() - 1 {
        size_value /= 1024.0;
        current_unit_index += 1;
    }
    
    format!("{:.1} {}", size_value, SIZE_UNITS[current_unit_index])
}