pub mod analyzer;
pub mod batch;
pub mod processor;

pub use analyzer::MkvAnalyzer;
pub use batch::{BatchProcessor, BatchResult};
pub use processor::analyze_and_process_mkv_file;